//! Pure helpers for the Remote access settings panel.
//!
//! This module keeps the scope labels, status text and per-scope URLs together.
//! Off and LAN bind the local listener; Tailscale adds a Serve handler for
//! `/fgc` (see [`crate::tailscale`]) and Funnel remains a later item.

use fgc_core::{AppState, RemoteAccessMode};

use crate::server::base_url_for;
use crate::tailscale::{SERVE_PATH, ServeState};

/// HTML `select` value for a scope.
#[must_use]
pub fn mode_value(mode: RemoteAccessMode) -> &'static str {
    match mode {
        RemoteAccessMode::Off => "off",
        RemoteAccessMode::Lan => "lan",
        RemoteAccessMode::Tailscale => "tailscale",
        RemoteAccessMode::Funnel => "funnel",
    }
}

/// Parses a scope selected in the UI. Unknown values fall back to Off.
#[must_use]
pub fn mode_from_value(value: &str) -> RemoteAccessMode {
    match value {
        "lan" => RemoteAccessMode::Lan,
        "tailscale" => RemoteAccessMode::Tailscale,
        "funnel" => RemoteAccessMode::Funnel,
        _ => RemoteAccessMode::Off,
    }
}

/// Short label used in the selector and status line.
#[must_use]
pub fn mode_label(mode: RemoteAccessMode) -> &'static str {
    match mode {
        RemoteAccessMode::Off => "This PC",
        RemoteAccessMode::Lan => "Local network",
        RemoteAccessMode::Tailscale => "Tailscale",
        RemoteAccessMode::Funnel => "Funnel",
    }
}

/// Plain-language note for scopes that are not wired up yet.
#[must_use]
pub fn pending_notice(mode: RemoteAccessMode) -> Option<&'static str> {
    match mode {
        RemoteAccessMode::Funnel => Some(
            "Funnel is not available yet. It needs Tailscale, HTTPS and an explicit public-exposure opt-in.",
        ),
        RemoteAccessMode::Off | RemoteAccessMode::Lan | RemoteAccessMode::Tailscale => None,
    }
}

/// Serve status line rendered only in Tailscale scope.
#[must_use]
pub fn serve_text(serve: &ServeState, running: bool) -> String {
    if !running {
        return "Tailscale Serve is off because the overlay server is stopped.".to_owned();
    }
    match serve {
        ServeState::Off => "Tailscale Serve is preparing…".to_owned(),
        ServeState::Active => {
            format!("Tailscale Serve is active at {SERVE_PATH} on your tailnet.")
        }
        ServeState::Failed(error) => format!("Tailscale Serve failed: {error}"),
    }
}

/// Base URL for the scope that is currently selected.
#[must_use]
pub fn base_url(state: &AppState) -> String {
    let overlay = &state.settings.overlay;
    base_url_for(overlay.allow_lan, overlay.port)
}

/// One-line status for the active scope.
///
/// The base URL follows the app-wide reveal setting so Settings does not leak
/// it when URLs are hidden elsewhere.
#[must_use]
pub fn status_text(state: &AppState, running: bool, show_url: bool) -> String {
    let mode = state.settings.overlay.remote_access;
    if let Some(notice) = pending_notice(mode) {
        return notice.to_owned();
    }

    let lifecycle = if running { "running" } else { "stopped" };
    if show_url {
        format!(
            "{} — server {} — {}",
            mode_label(mode),
            lifecycle,
            base_url(state)
        )
    } else {
        format!("{} — server {}", mode_label(mode), lifecycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_values_round_trip() {
        for mode in [
            RemoteAccessMode::Off,
            RemoteAccessMode::Lan,
            RemoteAccessMode::Tailscale,
            RemoteAccessMode::Funnel,
        ] {
            assert_eq!(mode_from_value(mode_value(mode)), mode);
        }
        assert_eq!(mode_from_value("unknown"), RemoteAccessMode::Off);
    }

    #[test]
    fn local_status_uses_loopback_and_lan_status_uses_a_lan_address() {
        let mut state = AppState::default();
        state.settings.overlay.port = 4123;
        let hidden = status_text(&state, false, false);
        assert!(hidden.contains("This PC"));
        assert!(hidden.contains("stopped"));
        assert!(!hidden.contains("127.0.0.1"));
        assert!(status_text(&state, false, true).contains("127.0.0.1:4123"));

        state.settings.overlay.remote_access = RemoteAccessMode::Lan;
        state.settings.overlay.allow_lan = true;
        let status = status_text(&state, true, true);
        assert!(status.contains("Local network"));
        assert!(status.contains("running"));
        assert!(status.contains(":4123"));
        assert!(!status.contains("127.0.0.1"));
    }

    #[test]
    fn funnel_still_explains_that_it_is_pending() {
        let mut state = AppState::default();
        state.settings.overlay.remote_access = RemoteAccessMode::Funnel;
        assert!(status_text(&state, true, false).contains("not available yet"));
    }

    #[test]
    fn tailscale_serve_status_covers_every_state() {
        assert!(serve_text(&ServeState::Active, true).contains("/fgc"));
        assert!(serve_text(&ServeState::Off, true).contains("preparing"));
        assert!(serve_text(&ServeState::Off, false).contains("stopped"));
        assert!(serve_text(&ServeState::Failed("denied".to_owned()), true).contains("denied"));
    }
}
