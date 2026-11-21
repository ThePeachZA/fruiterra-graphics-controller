//! Pure helpers for the Remote access settings panel.
//!
//! Tailscale detection, serve lifecycle and Funnel are separate backlog items.
//! This module keeps the scope labels, status text and per-scope URLs together
//! so D8-2 can render an honest UI before that plumbing exists.

use fgc_core::{AppState, RemoteAccessMode};

use crate::server::base_url_for;

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
        RemoteAccessMode::Tailscale => Some(
            "Tailscale is detected, but remote serving is not wired up yet. Overlays still use the local server until the next item lands.",
        ),
        RemoteAccessMode::Funnel => Some(
            "Funnel is not available yet. It needs Tailscale, HTTPS and an explicit public-exposure opt-in.",
        ),
        RemoteAccessMode::Off | RemoteAccessMode::Lan => None,
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
    fn tailscale_and_funnel_statuses_explain_that_they_are_pending() {
        let mut state = AppState::default();

        state.settings.overlay.remote_access = RemoteAccessMode::Tailscale;
        assert!(status_text(&state, true, true).contains("not wired up yet"));

        state.settings.overlay.remote_access = RemoteAccessMode::Funnel;
        assert!(status_text(&state, true, false).contains("not available yet"));
    }
}
