//! HTTP routes.
//!
//! The listener serves overlays, bundled and user assets, fonts, `/health` and
//! `/state`, plus the WebSocket endpoint at `/`. Every response is small and
//! cache headers are explicit so OBS reloads overlays after an override edit
//! while content-addressed assets stay immutable.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tower_http::cors::CorsLayer;
use tracing::warn;

use crate::embed;
use crate::overrides;
use crate::range::{RangeOutcome, parse_range};
use crate::runtime::SharedState;
use crate::ws;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const NO_STORE: &str = "no-store";

/// Builds the Axum router for one server instance.
pub(crate) fn router(shared: Arc<SharedState>) -> Router {
    Router::new()
        .route("/", get(ws::upgrade))
        .route("/health", get(health))
        .route("/state", get(state))
        .route("/overlays/{file}", get(overlay))
        .route("/assets/fonts/{name}", get(font))
        .route("/assets/{file}", get(asset))
        .layer(CorsLayer::permissive())
        .with_state(shared)
}

async fn health(State(shared): State<Arc<SharedState>>) -> Response {
    let body = json!({
        "status": "ok",
        "port": shared.port.load(Ordering::SeqCst),
        "overlays": shared.clients.load(Ordering::SeqCst),
        "version": env!("CARGO_PKG_VERSION"),
    });
    ([(header::CACHE_CONTROL, NO_STORE)], Json(body)).into_response()
}

async fn state(State(shared): State<Arc<SharedState>>) -> Response {
    match shared.state_frame() {
        Ok(body) => text_response(body, "application/json", NO_STORE),
        Err(error) => {
            warn!(%error, "failed to serialise the state map");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn overlay(State(shared): State<Arc<SharedState>>, Path(file): Path<String>) -> Response {
    let Some(name) = overrides::validate_overlay_name(&file) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if let Some(bytes) = overrides::read_override(shared.overlay_override_dir.as_deref(), name) {
        return text_response(bytes, "text/html; charset=utf-8", "no-cache");
    }

    match embed::embedded_overlay(name) {
        Some(bytes) => text_response(bytes, "text/html; charset=utf-8", "no-cache"),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn font(Path(name): Path<String>, headers: HeaderMap) -> Response {
    let Some(name) = overrides::validate_font_name(&name) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(bytes) = embed::embedded_font(name) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = format!("\"{name}\"");
    if not_modified(&headers, &etag) {
        return not_modified_response(&etag);
    }
    file_response(bytes, "font/woff2", &etag, &headers)
}

async fn asset(
    State(shared): State<Arc<SharedState>>,
    Path(file): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some((hash, ext)) = overrides::split_asset_file(&file) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if fgc_core::validate_asset_identity(hash, ext).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let etag = format!("\"{hash}.{ext}\"");
    if not_modified(&headers, &etag) {
        return not_modified_response(&etag);
    }

    let Some(bytes) = load_asset(&shared, hash, ext).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let content_type = mime_guess::from_ext(ext)
        .first_or_octet_stream()
        .to_string();
    file_response(bytes, &content_type, &etag, &headers)
}

async fn load_asset(shared: &SharedState, hash: &str, ext: &str) -> Option<Vec<u8>> {
    if let Some(bytes) = embed::bundled_asset_bytes_by_hash(hash, ext) {
        return Some(bytes.into_owned());
    }

    let root = shared.blob_dir.as_ref()?;
    let path = fgc_core::blob_path(root, hash, ext).ok()?;
    match tokio::fs::read(&path).await {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            warn!(path = %path.display(), %error, "failed to read a user asset");
            None
        }
    }
}

fn text_response(body: impl Into<Vec<u8>>, content_type: &str, cache_control: &str) -> Response {
    let body = body.into();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_control)
        .header(header::CONTENT_LENGTH, body.len())
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn file_response(
    bytes: impl Into<Vec<u8>>,
    content_type: &str,
    etag: &str,
    headers: &HeaderMap,
) -> Response {
    let bytes = bytes.into();
    let len = bytes.len() as u64;

    if let Some(range_header) = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
    {
        match parse_range(range_header, len) {
            RangeOutcome::Partial(range) => {
                let (Ok(start), Ok(end)) =
                    (usize::try_from(range.start), usize::try_from(range.end))
                else {
                    return range_not_satisfiable(len);
                };
                let Some(slice) = bytes.get(start..=end) else {
                    return range_not_satisfiable(len);
                };
                return Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, IMMUTABLE)
                    .header(header::ACCEPT_RANGES, "bytes")
                    .header(header::ETAG, etag)
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", range.start, range.end, len),
                    )
                    .header(header::CONTENT_LENGTH, slice.len())
                    .body(Body::from(slice.to_vec()))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            RangeOutcome::Unsatisfiable => return range_not_satisfiable(len),
            RangeOutcome::Full => {}
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, IMMUTABLE)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, etag)
        .header(header::CONTENT_LENGTH, len)
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn range_not_satisfiable(len: u64) -> Response {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(header::CONTENT_RANGE, format!("bytes */{len}"))
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn not_modified(headers: &HeaderMap, etag: &str) -> bool {
    let Some(value) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    value
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate == etag)
}

fn not_modified_response(etag: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(header::CACHE_CONTROL, IMMUTABLE)
        .header(header::ETAG, etag)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
