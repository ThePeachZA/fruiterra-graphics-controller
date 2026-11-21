//! Pure helpers for the Remote access settings panel.
//!
//! Local network and Tailscale are independent switches. `allow_lan` binds the
//! listener on all interfaces; Tailscale adds Serve handlers for `/fgc` and `/`
//! (see [`crate::tailscale`]). Funnel remains a later item.

use std::net::Ipv4Addr;

use fgc_core::{AppState, RemoteAccessMode};

use crate::server::{base_url_for, base_url_for_host, lan_ipv4};
use crate::tailscale::{SERVE_PATH, ServeState, WS_SERVE_PATH};

/// Whether another device on the LAN can open the overlays.
#[must_use]
pub fn is_local_enabled(state: &AppState) -> bool {
    state.settings.overlay.allow_lan
}

/// Whether this app installs Tailscale Serve handlers for the overlays.
#[must_use]
pub fn is_tailscale_enabled(state: &AppState) -> bool {
    state.settings.overlay.remote_access == RemoteAccessMode::Tailscale
}

fn scope_label(state: &AppState) -> &'static str {
    match (is_local_enabled(state), is_tailscale_enabled(state)) {
        (true, true) => "Local network + Tailscale",
        (true, false) => "Local network",
        (false, true) => "Tailscale",
        (false, false) => "This PC",
    }
}

/// Plain-language note for a persisted scope that is not wired up yet.
#[must_use]
pub fn pending_notice(mode: RemoteAccessMode) -> Option<&'static str> {
    match mode {
        RemoteAccessMode::Funnel => Some(
            "Funnel is not available yet. It needs Tailscale, HTTPS and an explicit public-exposure opt-in.",
        ),
        RemoteAccessMode::Off | RemoteAccessMode::Lan | RemoteAccessMode::Tailscale => None,
    }
}

/// Serve status line rendered whenever Tailscale is enabled.
#[must_use]
pub fn serve_text(serve: &ServeState, running: bool) -> String {
    if !running {
        return "Tailscale Serve is off because the overlay server is stopped.".to_owned();
    }
    match serve {
        ServeState::Off => "Tailscale Serve is preparing…".to_owned(),
        ServeState::Active => format!(
            "Tailscale Serve is active at {SERVE_PATH} and {WS_SERVE_PATH} (WebSocket) on your tailnet."
        ),
        ServeState::Failed(error) => format!("Tailscale Serve failed: {error}"),
    }
}

/// Base URL for the enabled access paths.
///
/// Tailscale uses this device's MagicDNS name and the `/fgc` prefix. Before
/// detection returns a name, LAN uses its IP and This PC uses loopback.
#[must_use]
pub fn base_url(state: &AppState, dns_name: Option<&str>) -> String {
    let overlay = &state.settings.overlay;
    if is_tailscale_enabled(state)
        && let Some(name) = dns_name.map(str::trim).filter(|name| !name.is_empty())
    {
        return format!("https://{name}{SERVE_PATH}");
    }
    base_url_for(overlay.allow_lan, overlay.port)
}

/// Whether a real Tailscale URL can be shown for the selected scope.
#[must_use]
pub fn has_remote_url(state: &AppState, dns_name: Option<&str>) -> bool {
    is_tailscale_enabled(state) && dns_name.is_some_and(|name| !name.trim().is_empty())
}

/// One copyable Browser Source URL set for an enabled access path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayUrlGroup {
    /// Stable identifier used for test selectors and layout hooks.
    pub key: &'static str,
    /// Short name for the access path.
    pub label: &'static str,
    /// Where this URL can be opened.
    pub description: &'static str,
    /// Base URL for the seven overlay files.
    pub base_url: String,
}

/// Every Browser Source URL set currently available.
///
/// Local network and Tailscale are independent, so both sets are returned when
/// both switches are on. Local network is listed only when a non-loopback LAN
/// address is detected; the loopback set is a fallback when neither enabled
/// path has a usable address.
#[must_use]
pub fn browser_source_groups(state: &AppState, dns_name: Option<&str>) -> Vec<OverlayUrlGroup> {
    let lan_ip = if is_local_enabled(state) {
        lan_ipv4()
    } else {
        None
    };
    browser_source_groups_for(state, dns_name, lan_ip)
}

fn browser_source_groups_for(
    state: &AppState,
    dns_name: Option<&str>,
    lan_ip: Option<Ipv4Addr>,
) -> Vec<OverlayUrlGroup> {
    let mut groups = Vec::new();
    let overlay = &state.settings.overlay;

    if let Some(ip) = lan_ip {
        groups.push(OverlayUrlGroup {
            key: "lan",
            label: "Local network",
            description: "Other devices on this network.",
            base_url: base_url_for_host(ip, overlay.port),
        });
    }

    if has_remote_url(state, dns_name) {
        let name = dns_name.unwrap_or_default().trim();
        groups.push(OverlayUrlGroup {
            key: "tailscale",
            label: "Tailscale",
            description: "Devices signed in to your tailnet.",
            base_url: format!("https://{name}{SERVE_PATH}"),
        });
    }

    if groups.is_empty() {
        groups.push(OverlayUrlGroup {
            key: "this-pc",
            label: "This PC",
            description: "Only this computer.",
            base_url: base_url_for(false, overlay.port),
        });
    }

    groups
}

/// One-line status for the enabled access paths.
///
/// The base URL follows the app-wide reveal setting so Settings does not leak
/// it when URLs are hidden elsewhere.
#[must_use]
pub fn status_text(
    state: &AppState,
    running: bool,
    show_url: bool,
    dns_name: Option<&str>,
) -> String {
    if let Some(notice) = pending_notice(state.settings.overlay.remote_access) {
        return notice.to_owned();
    }

    let lifecycle = if running { "running" } else { "stopped" };
    if is_tailscale_enabled(state) && !has_remote_url(state, dns_name) && !is_local_enabled(state) {
        return format!("Tailscale, server {lifecycle}, waiting for Tailscale address");
    }

    let label = scope_label(state);
    if show_url {
        format!("{label}, server {lifecycle}, {}", base_url(state, dns_name))
    } else {
        format!("{label}, server {lifecycle}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(port: u16, local: bool, tailscale: bool) -> AppState {
        let mut state = AppState::default();
        state.settings.overlay.port = port;
        state.settings.overlay.allow_lan = local;
        if tailscale {
            state.settings.overlay.remote_access = RemoteAccessMode::Tailscale;
        }
        state
    }

    #[test]
    fn this_pc_status_uses_loopback() {
        let state = state_with(4123, false, false);
        let hidden = status_text(&state, false, false, None);
        assert!(hidden.contains("This PC"));
        assert!(hidden.contains("stopped"));
        assert!(!hidden.contains("127.0.0.1"));
        assert!(status_text(&state, false, true, None).contains("127.0.0.1:4123"));
    }

    #[test]
    fn local_network_status_uses_a_lan_address() {
        let state = state_with(4123, true, false);
        let status = status_text(&state, true, true, None);
        assert!(status.contains("Local network"));
        assert!(status.contains("running"));
        assert!(status.contains(":4123"));
        assert!(!status.contains("127.0.0.1"));
    }

    #[test]
    fn tailscale_urls_use_the_magic_dns_name_and_fgc_prefix() {
        let state = state_with(4123, false, true);

        assert_eq!(
            base_url(&state, Some("sekhmet.example.ts.net")),
            "https://sekhmet.example.ts.net/fgc"
        );
        assert!(!has_remote_url(&state, None));
        assert!(status_text(&state, true, true, None).contains("waiting for Tailscale address"));
        assert_eq!(base_url(&state, None), "http://127.0.0.1:4123");
    }

    #[test]
    fn local_and_tailscale_can_be_enabled_together() {
        let state = state_with(4123, true, true);

        assert!(is_local_enabled(&state));
        assert!(is_tailscale_enabled(&state));
        assert_eq!(
            base_url(&state, Some("sekhmet.example.ts.net")),
            "https://sekhmet.example.ts.net/fgc"
        );
        let fallback = base_url(&state, None);
        assert!(fallback.starts_with("http://"));
        assert!(fallback.contains(":4123"));
        assert!(!fallback.contains("127.0.0.1"));
        assert!(status_text(&state, true, true, None).contains("Local network + Tailscale"));
    }

    #[test]
    fn browser_source_groups_include_both_enabled_access_paths() {
        let state = state_with(4123, true, true);
        let groups = browser_source_groups_for(
            &state,
            Some("sekhmet.example.ts.net"),
            Some(Ipv4Addr::new(192, 168, 50, 121)),
        );

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].key, "lan");
        assert_eq!(groups[0].label, "Local network");
        assert_eq!(groups[0].base_url, "http://192.168.50.121:4123");
        assert_eq!(groups[1].key, "tailscale");
        assert_eq!(groups[1].label, "Tailscale");
        assert_eq!(groups[1].base_url, "https://sekhmet.example.ts.net/fgc");
    }

    #[test]
    fn browser_source_groups_fall_back_until_an_address_is_known() {
        let tailscale_only = state_with(4123, false, true);
        let waiting = browser_source_groups_for(&tailscale_only, None, None);
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].key, "this-pc");
        assert_eq!(waiting[0].base_url, "http://127.0.0.1:4123");

        let local_only = state_with(4123, true, false);
        let lan = browser_source_groups_for(
            &local_only,
            Some("sekhmet.example.ts.net"),
            Some(Ipv4Addr::new(192, 168, 50, 121)),
        );
        assert_eq!(lan.len(), 1);
        assert_eq!(lan[0].key, "lan");
    }

    #[test]
    fn browser_source_groups_never_label_loopback_as_lan() {
        let state = state_with(4123, true, true);
        let groups = browser_source_groups_for(&state, None, None);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].key, "this-pc");
        assert_eq!(groups[0].base_url, "http://127.0.0.1:4123");
        assert!(!groups.iter().any(|group| group.key == "lan"));
    }

    #[test]
    fn funnel_still_explains_that_it_is_pending() {
        let mut state = state_with(4123, false, false);
        state.settings.overlay.remote_access = RemoteAccessMode::Funnel;
        assert!(status_text(&state, true, false, None).contains("not available yet"));
        assert_eq!(
            base_url(&state, Some("sekhmet.example.ts.net")),
            "http://127.0.0.1:4123"
        );
        assert!(!has_remote_url(&state, Some("sekhmet.example.ts.net")));
    }

    #[test]
    fn tailscale_serve_status_covers_every_state() {
        assert!(serve_text(&ServeState::Active, true).contains("/fgc"));
        assert!(serve_text(&ServeState::Off, true).contains("preparing"));
        assert!(serve_text(&ServeState::Off, false).contains("stopped"));
        assert!(serve_text(&ServeState::Failed("denied".to_owned()), true).contains("denied"));
    }
}
