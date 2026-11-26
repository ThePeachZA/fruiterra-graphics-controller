//! Authenticated local and remote control endpoint.
//!
//! The control surface is deliberately separate from the public overlay
//! routes. Loopback clients use a bearer token; remote clients sign the exact
//! request with HMAC-SHA256 so the token is never transmitted. Both modes share
//! one replay guard and forward validated actions to the application task.

use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::runtime::SharedState;

/// Path of the authenticated control endpoint.
pub const CONTROL_PATH: &str = "/control";

const DOMAIN_SEPARATOR: &str = "FGC-CONTROL-V1";
const HMAC_SCHEME: &str = "HMAC-SHA256";
const MAX_TIMESTAMP_SKEW_SECONDS: u64 = 60;
const NONCE_TTL: Duration = Duration::from_secs(300);
const RATE_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT: u32 = 30;
const NONCE_MIN_LEN: usize = 8;
const NONCE_MAX_LEN: usize = 128;

type HmacSha256 = Hmac<Sha256>;

/// Show actions accepted from a hardware controller or browser page.
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

/// Raw token bytes kept in memory for HMAC verification. Its `Debug` output is
/// redacted so a log or diagnostic cannot leak the write credential.
#[derive(Clone)]
struct RemoteSecret(Arc<[u8]>);

impl fmt::Debug for RemoteSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RemoteSecret([REDACTED])")
    }
}

/// Validates the bearer token or HMAC signature and forwards accepted actions
/// to the UI task.
#[derive(Clone)]
pub struct ControlBridge {
    token_digest: [u8; 32],
    token: RemoteSecret,
    actions: mpsc::UnboundedSender<ControlAction>,
    remote_enabled: bool,
}

impl ControlBridge {
    /// Creates a bridge for `token`. `remote_enabled` allows HMAC requests and
    /// remote peers; bearer requests remain restricted to loopback.
    #[must_use]
    pub fn new(
        token: &str,
        actions: mpsc::UnboundedSender<ControlAction>,
        remote_enabled: bool,
    ) -> Self {
        Self {
            token_digest: digest(token),
            token: RemoteSecret(Arc::from(token.as_bytes())),
            actions,
            remote_enabled,
        }
    }

    /// Whether remote access is currently enabled for this bridge.
    #[must_use]
    pub fn remote_enabled(&self) -> bool {
        self.remote_enabled
    }

    /// Returns whether `token` matches the bridge digest in constant time.
    #[must_use]
    pub fn accepts_token(&self, token: &str) -> bool {
        constant_time_eq(&self.token_digest, &digest(token))
    }

    fn authorises(
        &self,
        headers: &HeaderMap,
        peer: IpAddr,
        request: &ControlRequest,
        body: &[u8],
    ) -> Result<(), ControlError> {
        let Some(value) = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Err(ControlError::Unauthorised);
        };
        let mut parts = value.splitn(2, ' ');
        let scheme = parts.next().unwrap_or_default();
        let credential = parts
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or_default();

        if scheme.eq_ignore_ascii_case("Bearer") {
            if !peer.is_loopback() || !self.accepts_token(credential) {
                return Err(ControlError::Unauthorised);
            }
            return Ok(());
        }

        if scheme.eq_ignore_ascii_case(HMAC_SCHEME) {
            if !self.remote_enabled {
                return Err(ControlError::Unauthorised);
            }
            let provided = decode_hex_32(credential).ok_or(ControlError::Unauthorised)?;
            let mut mac = HmacSha256::new_from_slice(&self.token.0)
                .map_err(|_| ControlError::Unauthorised)?;
            mac.update(&canonical_message(request, body));
            let expected = mac.finalize().into_bytes();
            let mut expected_bytes = [0_u8; 32];
            expected_bytes.copy_from_slice(&expected);
            if !constant_time_eq(&provided, &expected_bytes) {
                return Err(ControlError::Unauthorised);
            }
            return Ok(());
        }

        Err(ControlError::Unauthorised)
    }

    fn forward(&self, action: ControlAction) -> Result<(), ControlError> {
        self.actions
            .send(action)
            .map_err(|_| ControlError::BridgeClosed)
    }
}

/// The exact signed message. The trailing newline is part of the contract.
fn canonical_message(request: &ControlRequest, body: &[u8]) -> Vec<u8> {
    format!(
        "{DOMAIN_SEPARATOR}\n{}\n{}\nPOST\n{CONTROL_PATH}\n{}\n",
        request.timestamp,
        request.nonce,
        hex_lower(Sha256::digest(body)),
    )
    .into_bytes()
}

/// Replay and abuse guards shared by all requests to the control endpoint.
#[derive(Default)]
pub(crate) struct ControlGuard {
    nonces: Mutex<HashMap<(IpAddr, String), Instant>>,
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
    InvalidRequest,
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
            Self::InvalidRequest => StatusCode::UNPROCESSABLE_ENTITY,
            Self::StaleTimestamp | Self::InvalidNonce => StatusCode::BAD_REQUEST,
            Self::ReplayedNonce => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::BridgeMissing | Self::BridgeClosed => "control_unavailable",
            Self::Unauthorised => "unauthorised",
            Self::NonLoopback => "loopback_only",
            Self::InvalidRequest => "invalid_request",
            Self::StaleTimestamp => "stale_timestamp",
            Self::InvalidNonce => "invalid_nonce",
            Self::ReplayedNonce => "replayed_nonce",
            Self::RateLimited => "rate_limited",
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
    pub(crate) fn check_peer(
        &self,
        peer: IpAddr,
        remote_enabled: bool,
    ) -> Result<(), ControlError> {
        if peer.is_loopback() || remote_enabled {
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

    pub(crate) fn check_replay(
        &self,
        peer: IpAddr,
        timestamp: u64,
        nonce: &str,
    ) -> Result<(), ControlError> {
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

        let key = (peer, nonce.to_owned());
        let mut nonces = lock(&self.nonces);
        let seen_at = Instant::now();
        nonces.retain(|_, seen| seen_at.duration_since(*seen) < NONCE_TTL);
        if nonces.contains_key(&key) {
            return Err(ControlError::ReplayedNonce);
        }
        nonces.insert(key, seen_at);
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
    body: Bytes,
) -> Response {
    let request: ControlRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            tracing::debug!(%error, "malformed control request");
            return ControlError::InvalidRequest.into_response();
        }
    };

    let result: Result<ControlAction, ControlError> = (|| {
        let bridge = shared.control_bridge().ok_or(ControlError::BridgeMissing)?;
        shared
            .control_guard()
            .check_peer(peer.ip(), bridge.remote_enabled())?;
        shared.control_guard().rate_check(peer.ip())?;
        bridge.authorises(&headers, peer.ip(), &request, &body)?;
        shared
            .control_guard()
            .check_replay(peer.ip(), request.timestamp, &request.nonce)?;
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

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        let high = hex_nibble(bytes[index * 2])?;
        let low = hex_nibble(bytes[index * 2 + 1])?;
        *byte = (high << 4) | low;
    }
    Some(output)
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

    fn peer() -> IpAddr {
        "127.0.0.1".parse().expect("loopback")
    }

    fn signed_request(token: &str, body: &[u8], timestamp: u64, nonce: &str) -> String {
        let request = ControlRequest {
            action: ControlAction::Hide,
            timestamp,
            nonce: nonce.to_owned(),
        };
        let mut mac = HmacSha256::new_from_slice(token.as_bytes()).expect("key");
        mac.update(&canonical_message(&request, body));
        format!("{HMAC_SCHEME} {}", hex_lower(mac.finalize().into_bytes()))
    }

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
        let bridge = ControlBridge::new("secret", tx, false);
        assert!(bridge.accepts_token("secret"));
        assert!(!bridge.accepts_token("other"));
    }

    #[test]
    fn canonical_message_has_a_trailing_newline_and_body_hash() {
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = ControlRequest {
            action: ControlAction::Hide,
            timestamp: 100,
            nonce: "nonce-abcdef".to_owned(),
        };
        let message = String::from_utf8(canonical_message(&request, body)).expect("utf8");
        assert_eq!(
            message,
            format!(
                "{DOMAIN_SEPARATOR}\n100\nnonce-abcdef\nPOST\n{CONTROL_PATH}\n{}\n",
                hex_lower(Sha256::digest(body))
            )
        );
    }

    #[test]
    fn hmac_authorises_exact_body_when_remote_enabled() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            signed_request("secret", body, request.timestamp, &request.nonce)
                .parse()
                .expect("header"),
        );
        assert!(bridge.authorises(&headers, peer(), &request, body).is_ok());
    }

    #[test]
    fn hmac_rejects_tampered_body_and_wrong_key() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let signed = signed_request("secret", body, request.timestamp, &request.nonce);
        let tampered = br#"{"action":"show","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, signed.parse().expect("header"));
        assert_eq!(
            bridge.authorises(&headers, peer(), &request, tampered),
            Err(ControlError::Unauthorised)
        );

        let (tx, _rx) = mpsc::unbounded_channel();
        let other = ControlBridge::new("other", tx, true);
        assert_eq!(
            other.authorises(&headers, peer(), &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn hmac_is_refused_when_remote_is_disabled() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, false);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            signed_request("secret", body, request.timestamp, &request.nonce)
                .parse()
                .expect("header"),
        );
        assert_eq!(
            bridge.authorises(&headers, peer(), &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn bearer_is_rejected_from_a_remote_peer() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer secret".parse().expect("header"),
        );
        let remote = "192.0.2.10".parse().expect("remote");
        assert_eq!(
            bridge.authorises(&headers, remote, &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn guard_accepts_loopback_peers() {
        let guard = ControlGuard::default();
        assert!(guard.check_peer(peer(), false).is_ok());
    }

    #[test]
    fn guard_rejects_remote_peers_when_remote_is_disabled() {
        let guard = ControlGuard::default();
        assert_eq!(
            guard.check_peer("192.0.2.10".parse().expect("remote"), false),
            Err(ControlError::NonLoopback)
        );
        assert!(
            guard
                .check_peer("192.0.2.10".parse().expect("remote"), true)
                .is_ok()
        );
    }

    #[test]
    fn guard_rejects_stale_timestamps() {
        let guard = ControlGuard::default();
        let stale = unix_seconds().expect("clock").saturating_sub(120);
        assert_eq!(
            guard.check_replay(peer(), stale, "nonce-12345678"),
            Err(ControlError::StaleTimestamp)
        );
    }

    #[test]
    fn guard_rejects_malformed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert_eq!(
            guard.check_replay(peer(), now, "bad nonce"),
            Err(ControlError::InvalidNonce)
        );
    }

    #[test]
    fn guard_rejects_replayed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert!(guard.check_replay(peer(), now, "nonce-abcdefgh").is_ok());
        assert_eq!(
            guard.check_replay(peer(), now, "nonce-abcdefgh"),
            Err(ControlError::ReplayedNonce)
        );
    }

    #[test]
    fn guard_scopes_nonces_to_each_peer() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        let other = "192.0.2.10".parse().expect("remote");
        assert!(guard.check_replay(peer(), now, "nonce-abcdefgh").is_ok());
        assert!(guard.check_replay(other, now, "nonce-abcdefgh").is_ok());
    }

    #[test]
    fn guard_limits_requests_per_peer() {
        let guard = ControlGuard::default();
        for _ in 0..RATE_LIMIT {
            assert!(guard.rate_check(peer()).is_ok());
        }
        assert_eq!(guard.rate_check(peer()), Err(ControlError::RateLimited));
    }
}
