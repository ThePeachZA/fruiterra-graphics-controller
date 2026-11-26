//! Authenticated local control endpoint tests.

mod common;

use std::time::{SystemTime, UNIX_EPOCH};

use common::start;
use fgc_server::{
    CONTROL_PATH, ControlAction, ControlTeamSide, ServerConfig, generate_control_token,
};
use reqwest::{Client, StatusCode, header};
use tokio::sync::mpsc;

fn client() -> Client {
    Client::builder().build().expect("http client")
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_secs()
}

fn body(action: &str, nonce: &str) -> serde_json::Value {
    serde_json::json!({
        "action": action,
        "timestamp": unix_seconds(),
        "nonce": nonce,
    })
}

async fn post(
    client: &Client,
    base_url: &str,
    token: &str,
    body: &serde_json::Value,
) -> reqwest::Response {
    client
        .post(format!("{base_url}{CONTROL_PATH}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .json(body)
        .send()
        .await
        .expect("control request")
}

#[tokio::test]
async fn valid_request_delivers_action_without_cors_headers() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();

    let response = client
        .post(format!("{base_url}{CONTROL_PATH}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ORIGIN, "http://example.com")
        .json(&body("show", "nonce-valid-0001"))
        .send()
        .await
        .expect("control request");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "control responses must not inherit permissive CORS"
    );
    let json: serde_json::Value = response.json().await.expect("control response json");
    assert_eq!(json, serde_json::json!({ "ok": true, "action": "show" }));
    assert_eq!(
        rx.recv().await.expect("forwarded action"),
        ControlAction::Show
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn set_scene_uses_its_scene_field() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let request = serde_json::json!({
        "action": "set_scene",
        "scene": "game-overlay.html",
        "timestamp": unix_seconds(),
        "nonce": "nonce-scene-0001",
    });

    let response = post(&client, &base_url, &token, &request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = response.json().await.expect("control response json");
    assert_eq!(
        json,
        serde_json::json!({
            "ok": true,
            "action": "set_scene",
            "scene": "game-overlay.html",
        })
    );
    assert_eq!(
        rx.recv().await.expect("forwarded action"),
        ControlAction::SetScene("game-overlay.html".to_owned())
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn set_scene_without_a_scene_is_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let request = body("set_scene", "nonce-noscene-001");

    let response = post(&client, &base_url, &token, &request).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(rx.try_recv().is_err(), "no action should be delivered");

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn missing_and_invalid_tokens_are_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();

    let missing = client
        .post(format!("{base_url}{CONTROL_PATH}"))
        .json(&body("show", "nonce-missing-001"))
        .send()
        .await
        .expect("unauthenticated control request");
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    let json: serde_json::Value = missing.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "unauthorised" }));

    let invalid = post(
        &client,
        &base_url,
        "not-the-token",
        &body("hide", "nonce-invalid-001"),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    assert!(rx.try_recv().is_err(), "no action should be delivered");

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn stale_timestamp_is_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let stale = serde_json::json!({
        "action": "show",
        "timestamp": unix_seconds().saturating_sub(120),
        "nonce": "nonce-stale-0001",
    });

    let response = post(&client, &base_url, &token, &stale).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "stale_timestamp" }));
    assert!(rx.try_recv().is_err(), "no action should be delivered");

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn replayed_nonce_is_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let request = body("next_scene", "nonce-replay-001");

    let first = post(&client, &base_url, &token, &request).await;
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(
        rx.recv().await.expect("first action"),
        ControlAction::NextScene
    );

    let replay = post(&client, &base_url, &token, &request).await;
    assert_eq!(replay.status(), StatusCode::CONFLICT);
    let json: serde_json::Value = replay.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "replayed_nonce" }));
    assert!(
        rx.try_recv().is_err(),
        "replayed action must not be delivered"
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn wrong_method_is_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    let response = client
        .get(format!("{base_url}{CONTROL_PATH}"))
        .send()
        .await
        .expect("GET control request");

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn unconfigured_control_bridge_returns_service_unavailable() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let client = client();

    let response = post(
        &client,
        &base_url,
        &token,
        &body("force_update", "nonce-nobridge-01"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json: serde_json::Value = response.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "control_unavailable" }));
    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn public_routes_keep_permissive_cors_headers() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let client = client();

    let response = client
        .get(format!("{base_url}/health"))
        .header(header::ORIGIN, "http://example.com")
        .send()
        .await
        .expect("health request");

    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("*")
    );
    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn requests_over_the_rate_limit_are_rejected() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();

    for index in 0..30 {
        let nonce = format!("nonce-rate-{index:04}");
        let response = post(&client, &base_url, &token, &body("show", &nonce)).await;
        assert_eq!(response.status(), StatusCode::OK, "request {index}");
        assert_eq!(
            rx.recv().await.expect("forwarded action"),
            ControlAction::Show
        );
    }

    let blocked = post(&client, &base_url, &token, &body("show", "nonce-rate-0030")).await;
    assert_eq!(blocked.status(), StatusCode::TOO_MANY_REQUESTS);
    let json: serde_json::Value = blocked.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "rate_limited" }));
    assert!(
        rx.try_recv().is_err(),
        "limited action must not be delivered"
    );

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn destructive_match_edits_require_confirmation() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let mut request = body("set_map_pick", "nonce-pick-0001");
    request["index"] = serde_json::json!(0);
    request["team"] = serde_json::json!("b");

    let response = post(&client, &base_url, &token, &request).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json: serde_json::Value = response.json().await.expect("error json");
    assert_eq!(
        json,
        serde_json::json!({ "error": "confirmation_required" })
    );
    assert!(
        rx.try_recv().is_err(),
        "unconfirmed edit must not be delivered"
    );

    request["confirm"] = serde_json::json!(true);
    let accepted = post(&client, &base_url, &token, &request).await;
    assert_eq!(accepted.status(), StatusCode::OK);
    assert!(rx.recv().await.is_some(), "confirmed edit is delivered");

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn match_writes_return_a_single_use_undo_token() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let token = generate_control_token();
    let (tx, mut rx) = mpsc::unbounded_channel();
    handle.set_control(&token, tx, false);
    let client = client();
    let mut request = body("score_adjust", "nonce-score-0001");
    request["team"] = serde_json::json!("a");
    request["delta"] = serde_json::json!(1);

    let response = post(&client, &base_url, &token, &request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = response.json().await.expect("response json");
    let undo = json["undo"].as_str().expect("undo token").to_owned();
    assert!(undo.starts_with("u-"));
    assert_eq!(
        rx.recv().await.expect("forwarded action"),
        ControlAction::WithUndo {
            action: Box::new(ControlAction::ScoreAdjust {
                team: ControlTeamSide::A,
                delta: 1,
            }),
            undo: undo.clone(),
        }
    );

    let mut restore = body("undo", "nonce-undo-0001");
    restore["undo"] = serde_json::json!(undo);
    let response = post(&client, &base_url, &token, &restore).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        rx.recv().await.expect("forwarded undo"),
        ControlAction::Undo(undo.clone())
    );

    let mut again = body("undo", "nonce-undo-0002");
    again["undo"] = serde_json::json!(undo);
    let response = post(&client, &base_url, &token, &again).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let json: serde_json::Value = response.json().await.expect("error json");
    assert_eq!(json, serde_json::json!({ "error": "undo_unavailable" }));

    handle.stop().await.expect("stop");
}
