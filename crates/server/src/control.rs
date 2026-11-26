//! Authenticated local control endpoint.
//!
//! The control surface is deliberately separate from the public overlay
//! routes. It is loopback-only, bearer-token authenticated, and protected from
//! replay by a bounded nonce cache. Validated actions are forwarded to the
//! application task through an unbounded channel.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::runtime::SharedState;

/// Path of the authenticated control endpoint.
pub const CONTROL_PATH: &str = "/control";

const MAX_TIMESTAMP_SKEW_SECONDS: u64 = 60;
const NONCE_TTL: Duration = Duration::from_secs(300);
const RATE_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT: u32 = 30;
const NONCE_MIN_LEN: usize = 8;
const NONCE_MAX_LEN: usize = 128;

/// Show actions accepted from a local hardware controller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    /// Show all overlays.
    Show,
    /// Hide all overlays.
    Hide,
    /// Recalculate and resend current overlays.
    ForceUpdate,
    /// Select an auto-overlay scene by its configured label.
    SetScene(String),
    /// Advance to the next enabled auto-overlay scene.
    NextScene,
    /// Return to the previous enabled auto-overlay scene.
    PreviousScene,
}

/// Body accepted by `POST /control`.
#[derive(Debug, Deserialize)]
#[serde(try_from = "RawControlRequest")]
pub struct ControlRequest {
    /// Show action to apply.
    pub action: ControlAction,
    /// Unix timestamp in seconds; requests outside a 60-second window are
    /// rejected.
    pub timestamp: u64,
    /// Unique request identifier, retained for five minutes to stop replay.
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
struct RawControlRequest {
    action: String,
    timestamp: u64,
    nonce: String,
    #[serde(default)]
    scene: Option<String>,
}

impl TryFrom<RawControlRequest> for ControlRequest {
    type Error = String;

    fn try_from(raw: RawControlRequest) -> Result<Self, Self::Error> {
        let scene = raw.scene.filter(|scene| {
            let bytes = scene.as_bytes();
            (1..=128).contains(&bytes.len())
                && bytes
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        });
        let action = match raw.action.as_str() {
            "show" => ControlAction::Show,
            "hide" => ControlAction::Hide,
            "force_update" => ControlAction::ForceUpdate,
            "set_scene" => ControlAction::SetScene(
                scene.ok_or_else(|| "set_scene requires a valid scene file".to_owned())?,
            ),
            "next_scene" => ControlAction::NextScene,
            "previous_scene" => ControlAction::PreviousScene,
            other => return Err(format!("unknown action: {other}")),
        };
        Ok(Self {
            action,
            timestamp: raw.timestamp,
            nonce: raw.nonce,
        })
    }
}

impl ControlAction {
    /// Stable wire name used by the control endpoint and its documentation.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Hide => "hide",
            Self::ForceUpdate => "force_update",
            Self::SetScene(_) => "set_scene",
            Self::NextScene => "next_scene",
            Self::PreviousScene => "previous_scene",
        }
    }
}

/// Generates a high-entropy bearer token for the app to persist beside
/// `app.json`.
#[must_use]
pub fn generate_control_token() -> String {
    Uuid::new_v4().to_string()
}

/// Validates the bearer token and forwards accepted actions to the UI task.
#[derive(Clone)]
pub struct ControlBridge {
    token_digest: [u8; 32],
    actions: mpsc::UnboundedSender<ControlAction>,
}

impl ControlBridge {
    /// Creates a bridge that stores only a SHA-256 digest of `token`.
    #[must_use]
    pub fn new(token: &str, actions: mpsc::UnboundedSender<ControlAction>) -> Self {
        Self {
            token_digest: digest(token),
            actions,
        }
    }

    /// Returns whether `token` matches the bridge digest in constant time.
    #[must_use]
    pub fn accepts_token(&self, token: &str) -> bool {
        constant_time_eq(&self.token_digest, &digest(token))
    }

    fn authorises(&self, headers: &HeaderMap) -> bool {
        let Some(value) = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return false;
        };
        let mut parts = value.splitn(2, ' ');
        let Some(scheme) = parts.next() else {
            return false;
        };
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return false;
        }
        let Some(token) = parts.next().filter(|token| !token.is_empty()) else {
            return false;
        };
        self.accepts_token(token)
    }

    fn forward(&self, action: ControlAction) -> Result<(), ControlError> {
        self.actions
            .send(action)
            .map_err(|_| ControlError::BridgeClosed)
    }
}

/// Replay and abuse guards shared by all requests to the control endpoint.
#[derive(Default)]
pub(crate) struct ControlGuard {
    nonces: Mutex<HashMap<String, Instant>>,
    rates: Mutex<HashMap<IpAddr, RateWindow>>,
}

#[derive(Clone, Copy)]
struct RateWindow {
    started: Instant,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlError {
    BridgeMissing,
    Unauthorised,
    NonLoopback,
    StaleTimestamp,
    InvalidNonce,
    ReplayedNonce,
    RateLimited,
    BridgeClosed,
}

impl ControlError {
    fn status(self) -> StatusCode {
        match self {
            Self::BridgeMissing | Self::BridgeClosed => StatusCode::SERVICE_UNAVAILABLE,
            Self::Unauthorised => StatusCode::UNAUTHORIZED,
            Self::NonLoopback => StatusCode::FORBIDDEN,
            Self::StaleTimestamp | Self::InvalidNonce => StatusCode::BAD_REQUEST,
            Self::ReplayedNonce => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::BridgeMissing => "control_unavailable",
            Self::Unauthorised => "unauthorised",
            Self::NonLoopback => "loopback_only",
            Self::StaleTimestamp => "stale_timestamp",
            Self::InvalidNonce => "invalid_nonce",
            Self::ReplayedNonce => "replayed_nonce",
            Self::RateLimited => "rate_limited",
            Self::BridgeClosed => "control_unavailable",
        }
    }
}

impl IntoResponse for ControlError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.code() });
        (
            self.status(),
            [(header::CACHE_CONTROL, "no-store")],
            Json(body),
        )
            .into_response()
    }
}

impl ControlGuard {
    pub(crate) fn check_peer(&self, peer: IpAddr) -> Result<(), ControlError> {
        if peer.is_loopback() {
            Ok(())
        } else {
            Err(ControlError::NonLoopback)
        }
    }

    pub(crate) fn rate_check(&self, peer: IpAddr) -> Result<(), ControlError> {
        let mut rates = lock(&self.rates);
        let now = Instant::now();
        rates.retain(|_, window| now.duration_since(window.started) < RATE_WINDOW);
        let window = rates.entry(peer).or_insert_with(|| RateWindow {
            started: now,
            count: 0,
        });
        if now.duration_since(window.started) >= RATE_WINDOW {
            *window = RateWindow {
                started: now,
                count: 0,
            };
        }
        window.count += 1;
        if window.count > RATE_LIMIT {
            Err(ControlError::RateLimited)
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_replay(&self, timestamp: u64, nonce: &str) -> Result<(), ControlError> {
        if !(NONCE_MIN_LEN..=NONCE_MAX_LEN).contains(&nonce.len())
            || !nonce
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(ControlError::InvalidNonce);
        }
        let now = unix_seconds()?;
        if now.abs_diff(timestamp) > MAX_TIMESTAMP_SKEW_SECONDS {
            return Err(ControlError::StaleTimestamp);
        }

        let mut nonces = lock(&self.nonces);
        let seen_at = Instant::now();
        nonces.retain(|_, seen| seen_at.duration_since(*seen) < NONCE_TTL);
        if nonces.contains_key(nonce) {
            return Err(ControlError::ReplayedNonce);
        }
        nonces.insert(nonce.to_owned(), seen_at);
        Ok(())
    }

    pub(crate) fn clear(&self) {
        lock(&self.nonces).clear();
        lock(&self.rates).clear();
    }
}

/// Handler for `POST /control`.
pub(crate) async fn control(
    State(shared): State<Arc<SharedState>>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<ControlRequest>,
) -> Response {
    let result = (|| {
        let bridge = shared.control_bridge().ok_or(ControlError::BridgeMissing)?;
        shared.control_guard().check_peer(peer.ip())?;
        shared.control_guard().rate_check(peer.ip())?;
        if !bridge.authorises(&headers) {
            return Err(ControlError::Unauthorised);
        }
        shared
            .control_guard()
            .check_replay(request.timestamp, &request.nonce)?;
        bridge.forward(request.action.clone())?;
        Ok(request.action)
    })();

    match result {
        Ok(action) => {
            let body = match &action {
                ControlAction::SetScene(scene) => serde_json::json!({
                    "ok": true,
                    "action": action.name(),
                    "scene": scene,
                }),
                _ => serde_json::json!({ "ok": true, "action": action.name() }),
            };
            (
                StatusCode::OK,
                [(header::CACHE_CONTROL, "no-store")],
                Json(body),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

fn unix_seconds() -> Result<u64, ControlError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| ControlError::StaleTimestamp)
}

fn digest(token: &str) -> [u8; 32] {
    let value = Sha256::digest(token.as_bytes());
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&value);
    bytes
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut difference = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        difference |= left_byte ^ right_byte;
    }
    difference == 0
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_version_four_uuids() {
        let token = generate_control_token();
        assert_eq!(token.len(), 36);
        assert_eq!(token.as_bytes()[14], b'4');
        assert!(matches!(token.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
    }

    #[test]
    fn bridge_accepts_only_the_configured_token() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx);
        assert!(bridge.accepts_token("secret"));
        assert!(!bridge.accepts_token("other"));
    }

    #[test]
    fn guard_accepts_loopback_peers() {
        let guard = ControlGuard::default();
        assert!(
            guard
                .check_peer("127.0.0.1".parse().expect("loopback"))
                .is_ok()
        );
    }

    #[test]
    fn guard_rejects_remote_peers() {
        let guard = ControlGuard::default();
        assert_eq!(
            guard.check_peer("192.0.2.10".parse().expect("remote")),
            Err(ControlError::NonLoopback)
        );
    }

    #[test]
    fn guard_rejects_stale_timestamps() {
        let guard = ControlGuard::default();
        let stale = unix_seconds().expect("clock").saturating_sub(120);
        assert_eq!(
            guard.check_replay(stale, "nonce-12345678"),
            Err(ControlError::StaleTimestamp)
        );
    }

    #[test]
    fn guard_rejects_malformed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert_eq!(
            guard.check_replay(now, "bad nonce"),
            Err(ControlError::InvalidNonce)
        );
    }

    #[test]
    fn guard_rejects_replayed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert!(guard.check_replay(now, "nonce-abcdefgh").is_ok());
        assert_eq!(
            guard.check_replay(now, "nonce-abcdefgh"),
            Err(ControlError::ReplayedNonce)
        );
    }

    #[test]
    fn guard_limits_requests_per_peer() {
        let guard = ControlGuard::default();
        let peer = "127.0.0.1".parse().expect("loopback");
        for _ in 0..RATE_LIMIT {
            assert!(guard.rate_check(peer).is_ok());
        }
        assert_eq!(guard.rate_check(peer), Err(ControlError::RateLimited));
    }
}
