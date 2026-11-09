//! HTTP route tests: health, state, overlays, assets, fonts and overrides.

mod common;

use std::fs;

use common::{start, wait_for_running};
use fgc_server::{ServerConfig, bundled_assets};
use reqwest::StatusCode;
use sha2::{Digest, Sha256};

fn client() -> reqwest::Client {
    reqwest::Client::builder().build().expect("http client")
}

fn peach() -> &'static fgc_server::BundledAsset {
    bundled_assets()
        .iter()
        .find(|asset| asset.name == "Peach")
        .expect("Peach logo is bundled")
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[tokio::test]
async fn health_and_state_are_available_before_any_broadcast() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    let health: serde_json::Value = client
        .get(format!("{base_url}/health"))
        .send()
        .await
        .expect("health request")
        .json()
        .await
        .expect("health json");
    assert_eq!(health["status"], "ok");
    assert_eq!(health["port"], u64::from(wait_for_running(&handle)));
    assert_eq!(health["overlays"], 0);
    assert_eq!(health["version"], env!("CARGO_PKG_VERSION"));

    let response = client
        .get(format!("{base_url}/state"))
        .send()
        .await
        .expect("state request");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let state: serde_json::Value = response.json().await.expect("state json");
    assert_eq!(state, serde_json::json!({}));

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn serves_embedded_overlays_and_rejects_unsafe_names() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    let response = client
        .get(format!("{base_url}/overlays/game-overlay.html"))
        .send()
        .await
        .expect("overlay request");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    let body = response.text().await.expect("overlay body");
    assert!(body.contains("<!DOCTYPE html>"), "unexpected overlay body");

    assert_eq!(
        client
            .get(format!("{base_url}/overlays/missing.html"))
            .send()
            .await
            .expect("missing overlay")
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        client
            .get(format!("{base_url}/overlays/a..b.html"))
            .send()
            .await
            .expect("unsafe overlay")
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .get(format!("{base_url}/overlays/game-overlay.txt"))
            .send()
            .await
            .expect("non-html overlay")
            .status(),
        StatusCode::BAD_REQUEST
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn serves_bundled_assets_with_immutable_headers_etag_and_ranges() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();
    let asset = peach();
    let url = format!("{base_url}/assets/{}.{}", asset.hash, asset.ext);

    let response = client.get(&url).send().await.expect("asset request");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("public, max-age=31536000, immutable")
    );
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::ACCEPT_RANGES)
            .and_then(|value| value.to_str().ok()),
        Some("bytes")
    );
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .expect("etag")
        .to_owned();
    assert_eq!(etag, format!("\"{}.{}\"", asset.hash, asset.ext));

    let bytes = response.bytes().await.expect("asset bytes");
    let expected = fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(asset.path),
    )
    .expect("bundled file");
    assert_eq!(bytes.as_ref(), expected.as_slice());

    let response = client
        .get(&url)
        .header(reqwest::header::IF_NONE_MATCH, &etag)
        .send()
        .await
        .expect("conditional request");
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|value| value.to_str().ok()),
        Some(etag.as_str())
    );

    let response = client
        .get(&url)
        .header(reqwest::header::RANGE, "bytes=0-9")
        .send()
        .await
        .expect("range request");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok()),
        Some(format!("bytes 0-9/{}", expected.len()).as_str())
    );
    assert_eq!(response.bytes().await.expect("range bytes").len(), 10);

    let response = client
        .get(&url)
        .header(reqwest::header::RANGE, format!("bytes={}-", expected.len()))
        .send()
        .await
        .expect("unsatisfiable range");
    assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn serves_user_blobs_and_rejects_bad_asset_identities() {
    let assets_root = tempfile::tempdir().expect("tempdir");
    let bytes = b"user asset bytes";
    let hash = sha256(bytes);
    let directory = assets_root.path().join("blobs").join(&hash[..2]);
    fs::create_dir_all(&directory).expect("blob directory");
    fs::write(directory.join(format!("{hash}.png")), bytes).expect("blob");

    let config = ServerConfig::new(0).with_blob_dir(assets_root.path());
    let (handle, base_url) = start(config);
    let client = client();

    let response = client
        .get(format!("{base_url}/assets/{hash}.png"))
        .send()
        .await
        .expect("user asset");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.bytes().await.expect("bytes").as_ref(), bytes);

    assert_eq!(
        client
            .get(format!("{base_url}/assets/not-a-hash.png"))
            .send()
            .await
            .expect("bad hash")
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .get(format!("{base_url}/assets/{hash}.exe"))
            .send()
            .await
            .expect("bad extension")
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .get(format!("{base_url}/assets/{}.png", "a".repeat(64)))
            .send()
            .await
            .expect("unknown asset")
            .status(),
        StatusCode::NOT_FOUND
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn font_route_rejects_unsafe_names_and_missing_files() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    assert_eq!(
        client
            .get(format!("{base_url}/assets/fonts/missing.woff2"))
            .send()
            .await
            .expect("missing font")
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        client
            .get(format!("{base_url}/assets/fonts/a..b.woff2"))
            .send()
            .await
            .expect("unsafe font")
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .get(format!("{base_url}/assets/fonts/font.woff"))
            .send()
            .await
            .expect("non-woff2 font")
            .status(),
        StatusCode::BAD_REQUEST
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn per_file_overrides_change_only_that_overlay() {
    let overrides = tempfile::tempdir().expect("tempdir");
    fs::write(overrides.path().join("game-overlay.html"), b"OVERRIDE").expect("override");
    let config = ServerConfig::new(0).with_overlay_override_dir(overrides.path());
    let (handle, base_url) = start(config);
    let client = client();

    let overridden = client
        .get(format!("{base_url}/overlays/game-overlay.html"))
        .send()
        .await
        .expect("overridden overlay")
        .text()
        .await
        .expect("override body");
    assert_eq!(overridden, "OVERRIDE");

    let embedded = client
        .get(format!("{base_url}/overlays/map-pool.html"))
        .send()
        .await
        .expect("embedded overlay")
        .text()
        .await
        .expect("embedded body");
    assert!(embedded.contains("<!DOCTYPE html>"));
    assert_ne!(embedded, "OVERRIDE");

    fs::write(overrides.path().join("map-pool.html"), b"LIVE EDIT").expect("live override");
    let live = client
        .get(format!("{base_url}/overlays/map-pool.html"))
        .send()
        .await
        .expect("live override")
        .text()
        .await
        .expect("live body");
    assert_eq!(live, "LIVE EDIT");

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn responses_carry_permissive_cors_headers() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    let response = client
        .get(format!("{base_url}/health"))
        .header(reqwest::header::ORIGIN, "http://example.com")
        .send()
        .await
        .expect("health request");
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );

    handle.stop().await.expect("stop");
}
