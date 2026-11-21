//! Overlay server lifecycle for the Dioxus UI.
//!
//! The controller owns the [`ServerHandle`] in a signal, polls its status for
//! display, and projects domain state into broadcast frames. Starting the
//! server happens on a blocking task because [`ServerHandle::start`] waits for
//! the listener to bind.

use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

use dioxus::prelude::*;
use fgc_core::{AppState, Asset, AssetOrigin, Match};
use fgc_protocol::{AssetCatalog, DataUrlResolver, Projector, SectionCache, ServedAssetResolver};
use fgc_server::{AssetMode, Outbound, ServerConfig, ServerHandle, ServerStatus};
use tracing::{debug, warn};

/// How often the UI refreshes the displayed server status.
const STATUS_POLL: Duration = Duration::from_millis(500);

/// The default port, re-exported for the views.
pub use fgc_protocol::DEFAULT_PORT;

/// Owns the overlay server handle and the signals the UI renders.
#[derive(Clone, Copy)]
pub struct ServerController {
    handle: Signal<Option<ServerHandle>>,
    status: Signal<ServerStatus>,
    base_url: Signal<String>,
    asset_mode: Signal<AssetMode>,
    busy: Signal<bool>,
    blob_dir: Signal<Option<PathBuf>>,
    differ: Signal<ProjectionDiffer>,
}

impl ServerController {
    /// Creates the controller and starts the status-polling task.
    ///
    /// Must be called from a component hook.
    pub fn use_controller() -> Self {
        let handle = use_signal(|| None::<ServerHandle>);
        let status = use_signal(|| ServerStatus::Stopped);
        let base_url = use_signal(|| base_url_for(false, DEFAULT_PORT));
        let asset_mode = use_signal(|| AssetMode::Served);
        let busy = use_signal(|| false);
        let blob_dir = use_signal(|| fgc_core::assets_root().ok());
        let differ = use_signal(ProjectionDiffer::new);

        use_future(move || async move {
            let mut status = status;
            loop {
                tokio::time::sleep(STATUS_POLL).await;
                let next = handle.peek().as_ref().map(ServerHandle::status);
                if let Some(next) = next
                    && *status.peek() != next
                {
                    status.set(next);
                }
            }
        });

        Self {
            handle,
            status,
            base_url,
            asset_mode,
            busy,
            blob_dir,
            differ,
        }
    }

    /// Builds a controller for unit tests without starting the status task.
    #[cfg(test)]
    pub(crate) fn test_controller() -> Self {
        Self {
            handle: Signal::new(None::<ServerHandle>),
            status: Signal::new(ServerStatus::Stopped),
            base_url: Signal::new(base_url_for(false, DEFAULT_PORT)),
            asset_mode: Signal::new(AssetMode::Served),
            busy: Signal::new(false),
            blob_dir: Signal::new(None::<PathBuf>),
            differ: Signal::new(ProjectionDiffer::new()),
        }
    }

    /// Current lifecycle state.
    #[must_use]
    pub fn status(&self) -> ServerStatus {
        self.status.cloned()
    }

    /// Base URL used by served asset URLs and the Overlay view.
    #[must_use]
    pub fn base_url(&self) -> String {
        self.base_url.cloned()
    }

    /// Current asset URL mode.
    #[must_use]
    pub fn asset_mode(&self) -> AssetMode {
        *self.asset_mode.read()
    }

    /// True while a start or stop transition is in flight.
    #[must_use]
    pub fn busy(&self) -> bool {
        *self.busy.read()
    }

    /// True when the listener is accepting connections.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(*self.status.peek(), ServerStatus::Running { .. })
    }

    /// Replaces the displayed base URL, for example after a settings edit
    /// while the server is stopped.
    pub fn set_base_url(&self, url: impl Into<String>) {
        let mut base_url = self.base_url;
        base_url.set(url.into());
    }

    /// Changes the asset URL mode. The caller must reproject afterwards.
    pub fn set_asset_mode(&self, mode: AssetMode) {
        let mut asset_mode = self.asset_mode;
        asset_mode.set(mode);
    }

    /// Starts the listener. Ignored when already running or busy.
    pub fn start(&self, config: ServerConfig, base_url: String, initial: Vec<Outbound>) {
        if self.busy() || self.is_running() {
            return;
        }
        let mut differ = self.differ;
        differ.write().seed(&initial);
        let mut busy = self.busy;
        let mut status = self.status;
        busy.set(true);
        status.set(ServerStatus::Starting);
        self.set_base_url(base_url.clone());

        let mut handle_signal = self.handle;
        let mut status_signal = self.status;
        let mut busy_signal = self.busy;

        spawn(async move {
            let started = tokio::task::spawn_blocking(move || ServerHandle::start(config)).await;
            match started {
                Ok(Ok(handle)) => {
                    for message in initial {
                        handle.broadcast(message);
                    }
                    debug!(port = ?handle.port(), "overlay server started");
                    status_signal.set(handle.status());
                    handle_signal.set(Some(handle));
                }
                Ok(Err(error)) => {
                    warn!(%error, "overlay server failed to start");
                    status_signal.set(ServerStatus::Failed(error.to_string()));
                }
                Err(error) => {
                    warn!(%error, "overlay server start task failed");
                    status_signal.set(ServerStatus::Failed(format!("server task failed: {error}")));
                }
            }
            busy_signal.set(false);
        });
    }

    /// Stops the listener. Ignored when already stopped or busy.
    pub fn stop(&self) {
        if self.busy() {
            return;
        }
        let mut handle_signal = self.handle;
        let handle = handle_signal.write().take();
        let mut status = self.status;
        let Some(handle) = handle else {
            status.set(ServerStatus::Stopped);
            return;
        };
        let mut differ = self.differ;
        differ.write().reset();

        let mut busy = self.busy;
        busy.set(true);
        status.set(ServerStatus::Stopping);
        let mut status_signal = self.status;
        let mut busy_signal = self.busy;

        spawn(async move {
            match handle.stop().await {
                Ok(()) => {
                    debug!("overlay server stopped");
                    status_signal.set(ServerStatus::Stopped);
                }
                Err(error) => {
                    warn!(%error, "overlay server failed to stop cleanly");
                    status_signal.set(ServerStatus::Failed(error.to_string()));
                }
            }
            busy_signal.set(false);
        });
    }

    /// Stops any current listener and starts a new one with `config`.
    pub fn restart(&self, config: ServerConfig, base_url: String, initial: Vec<Outbound>) {
        if self.busy() {
            return;
        }
        let mut differ = self.differ;
        differ.write().seed(&initial);
        let mut handle_signal = self.handle;
        let previous = handle_signal.write().take();
        let mut busy = self.busy;
        let mut status = self.status;
        busy.set(true);
        status.set(ServerStatus::Starting);
        self.set_base_url(base_url.clone());

        let mut status_signal = self.status;
        let mut busy_signal = self.busy;

        spawn(async move {
            if let Some(previous) = previous
                && let Err(error) = previous.stop().await
            {
                warn!(%error, "overlay server failed to stop before restart");
            }

            let started = tokio::task::spawn_blocking(move || ServerHandle::start(config)).await;
            match started {
                Ok(Ok(handle)) => {
                    for message in initial {
                        handle.broadcast(message);
                    }
                    debug!(port = ?handle.port(), "overlay server restarted");
                    status_signal.set(handle.status());
                    handle_signal.set(Some(handle));
                }
                Ok(Err(error)) => {
                    warn!(%error, "overlay server failed to restart");
                    status_signal.set(ServerStatus::Failed(error.to_string()));
                }
                Err(error) => {
                    warn!(%error, "overlay server restart task failed");
                    status_signal.set(ServerStatus::Failed(format!("server task failed: {error}")));
                }
            }
            busy_signal.set(false);
        });
    }

    /// Sends every projected section to the running server.
    ///
    /// In served mode this performs no I/O. In data-URL mode the resolver reads
    /// bundled bytes from the binary and user blobs from the asset store.
    pub fn project(&self, state: &AppState, catalog: &AssetCatalog) {
        self.project_with(state, catalog, false);
    }

    /// Resends every projected section, even when it is unchanged.
    pub fn project_force(&self, state: &AppState, catalog: &AssetCatalog) {
        self.project_with(state, catalog, true);
    }

    /// Sends one outbound frame without touching the projection cache.
    pub fn broadcast(&self, message: Outbound) {
        let handle = self.handle.peek().as_ref().cloned();
        if let Some(handle) = handle {
            handle.broadcast(message);
        }
    }

    fn project_with(&self, state: &AppState, catalog: &AssetCatalog, force: bool) {
        let handle = self.handle.peek().as_ref().cloned();
        let Some(handle) = handle else {
            return;
        };

        let blob_dir = self.blob_dir.cloned();
        let outbound = project_outbound(
            state,
            catalog,
            &self.base_url(),
            self.asset_mode(),
            blob_dir.as_deref(),
        );

        let mut differ = self.differ;
        let frames = differ.write().take(outbound, force);
        for message in frames {
            handle.broadcast(message);
        }
    }
}

/// Filters projected frames down to sections whose content changed.
///
/// The server always caches and sends every frame it receives, so the UI layer
/// owns content deduplication. Seeding records what the server already has
/// without emitting anything; `take` then returns only changed sections unless
/// a forced update was requested.
#[derive(Clone, Debug, Default)]
pub struct ProjectionDiffer {
    cache: SectionCache,
}

impl ProjectionDiffer {
    /// Creates an empty differ.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every previously projected section.
    pub fn reset(&mut self) {
        self.cache.clear();
    }

    /// Records frames that were sent without passing through the differ.
    pub fn seed(&mut self, outbound: &[Outbound]) {
        for frame in outbound {
            if let Outbound::Message { kind, data } = frame {
                self.cache.set_value(kind, data.clone());
            }
        }
    }

    /// Returns the frames that should be sent and records their content.
    ///
    /// `show` frames always pass through because they are not cached by the
    /// server. When `force` is true every frame passes through.
    pub fn take(&mut self, outbound: Vec<Outbound>, force: bool) -> Vec<Outbound> {
        let mut frames = Vec::with_capacity(outbound.len());
        for frame in outbound {
            match frame {
                Outbound::Message { kind, data } => {
                    if !force && self.cache.get(&kind) == Some(&data) {
                        continue;
                    }
                    self.cache.set_value(&kind, data.clone());
                    frames.push(Outbound::Message { kind, data });
                }
                Outbound::Show { visible } => frames.push(Outbound::Show { visible }),
            }
        }
        frames
    }
}

/// Builds the server configuration for the current settings.
#[must_use]
pub fn server_config(port: u16, allow_lan: bool, asset_mode: AssetMode) -> ServerConfig {
    ServerConfig::new(port.clamp(1, u16::MAX))
        .with_lan(allow_lan)
        .with_asset_mode(asset_mode)
}

/// Builds the base URL used by overlays and served asset URLs.
#[must_use]
pub fn base_url_for(allow_lan: bool, port: u16) -> String {
    let host = if allow_lan {
        lan_ipv4().unwrap_or(Ipv4Addr::LOCALHOST)
    } else {
        Ipv4Addr::LOCALHOST
    };
    base_url_for_host(host, port)
}

/// Builds a base URL for an explicit host. Kept pure for tests.
#[must_use]
pub fn base_url_for_host(host: Ipv4Addr, port: u16) -> String {
    format!("http://{host}:{port}")
}

/// The overlay files served by the embedded router, in display order.
pub const OVERLAY_FILES: [&str; 7] = [
    "game-overlay.html",
    "map-select.html",
    "hero-bans.html",
    "map-pool.html",
    "team-roster-home.html",
    "team-roster-away.html",
    "row-casters.html",
];

/// Builds the full Browser Source URL for one overlay file.
#[must_use]
pub fn overlay_url(base_url: &str, file: &str) -> String {
    format!("{}/overlays/{file}", base_url.trim_end_matches('/'))
}

/// Projects every overlay section into broadcast frames.
///
/// Served mode is pure. Data-URL mode reads bytes through the same loader used
/// by the running server, so a missing blob yields `null` URLs instead of an
/// error.
#[must_use]
pub fn project_outbound(
    state: &AppState,
    catalog: &AssetCatalog,
    base_url: &str,
    mode: AssetMode,
    blob_root: Option<&Path>,
) -> Vec<Outbound> {
    let Some(match_state) = state.current_match.as_ref() else {
        return Vec::new();
    };

    match mode {
        AssetMode::Served => {
            let resolver = ServedAssetResolver::new(base_url);
            collect(&Projector::new(state, catalog, &resolver), match_state)
        }
        AssetMode::DataUrl => {
            let blob_root = blob_root.map(Path::to_path_buf);
            let resolver = DataUrlResolver::new(move |asset: &Asset| {
                load_asset_bytes(asset, blob_root.as_deref())
            });
            collect(&Projector::new(state, catalog, &resolver), match_state)
        }
    }
}

fn collect(projector: &Projector<'_>, match_state: &Match) -> Vec<Outbound> {
    let mut outbound = Vec::with_capacity(6);
    for message in projector.scoreboard_messages(match_state) {
        push(&mut outbound, Outbound::from_message(&message));
    }
    push(
        &mut outbound,
        Outbound::from_message(&projector.casters_message(match_state)),
    );
    outbound
}

fn push(outbound: &mut Vec<Outbound>, message: Result<Outbound, fgc_protocol::ProtocolError>) {
    match message {
        Ok(message) => outbound.push(message),
        Err(error) => warn!(%error, "failed to project an overlay message"),
    }
}

fn load_asset_bytes(asset: &Asset, blob_root: Option<&Path>) -> Option<Vec<u8>> {
    match asset.origin {
        AssetOrigin::Bundled => fgc_server::bundled_asset_bytes_by_hash(&asset.hash, &asset.ext)
            .map(std::borrow::Cow::into_owned),
        AssetOrigin::User => {
            let root = blob_root?;
            let path = fgc_core::blob_path(root, &asset.hash, &asset.ext).ok()?;
            std::fs::read(path).ok()
        }
    }
}

/// Best-effort LAN IPv4 address using a UDP route lookup.
///
/// The socket is never connected to a peer; connecting only asks the operating
/// system which local interface would be used. Offline machines fall back to
/// loopback in [`base_url_for`].
fn lan_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(address) if !address.is_loopback() => Some(address),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_uses_loopback_by_default() {
        assert_eq!(
            base_url_for_host(Ipv4Addr::LOCALHOST, 3001),
            "http://127.0.0.1:3001"
        );
    }

    #[test]
    fn base_url_uses_the_supplied_host() {
        assert_eq!(
            base_url_for_host(Ipv4Addr::new(192, 168, 1, 20), 4000),
            "http://192.168.1.20:4000"
        );
    }

    #[test]
    fn overlay_urls_are_absolute() {
        assert_eq!(
            overlay_url("http://127.0.0.1:3001/", "game-overlay.html"),
            "http://127.0.0.1:3001/overlays/game-overlay.html"
        );
    }

    #[test]
    fn server_config_carries_network_and_asset_settings() {
        let config = server_config(3001, true, AssetMode::DataUrl);
        assert_eq!(config.port, 3001);
        assert!(config.allow_lan);
        assert_eq!(config.asset_mode, AssetMode::DataUrl);
    }

    #[test]
    fn server_config_clamps_zero_to_a_valid_port() {
        assert_eq!(server_config(0, false, AssetMode::Served).port, 1);
    }

    #[test]
    fn projection_differ_sends_only_changed_sections() {
        let mut differ = ProjectionDiffer::new();
        let first = vec![
            Outbound::section("scoreboard", &serde_json::json!({ "a": 0 })).unwrap(),
            Outbound::section("casters", &serde_json::json!({ "casters": [] })).unwrap(),
        ];
        assert_eq!(differ.take(first.clone(), false).len(), 2);

        assert!(differ.take(first.clone(), false).is_empty());

        let changed = vec![
            Outbound::section("scoreboard", &serde_json::json!({ "a": 1 })).unwrap(),
            Outbound::section("casters", &serde_json::json!({ "casters": [] })).unwrap(),
        ];
        let frames = differ.take(changed, false);
        assert_eq!(frames.len(), 1);
        assert!(matches!(
            &frames[0],
            Outbound::Message { kind, .. } if kind == "scoreboard"
        ));
    }

    #[test]
    fn projection_differ_force_resends_unchanged_sections() {
        let mut differ = ProjectionDiffer::new();
        let frames = vec![
            Outbound::section("scoreboard", &serde_json::json!({ "a": 0 })).unwrap(),
            Outbound::section("casters", &serde_json::json!({ "casters": [] })).unwrap(),
        ];
        assert_eq!(differ.take(frames.clone(), false).len(), 2);
        assert_eq!(differ.take(frames, true).len(), 2);
    }

    #[test]
    fn projection_differ_seed_suppresses_the_initial_rebroadcast() {
        let mut differ = ProjectionDiffer::new();
        let frames = vec![
            Outbound::section("scoreboard", &serde_json::json!({ "a": 0 })).unwrap(),
            Outbound::section("casters", &serde_json::json!({ "casters": [] })).unwrap(),
        ];
        differ.seed(&frames);
        assert!(differ.take(frames, false).is_empty());
    }

    #[test]
    fn projection_differ_reset_forces_everything_through_again() {
        let mut differ = ProjectionDiffer::new();
        let frames = vec![Outbound::section("scoreboard", &serde_json::json!({ "a": 0 })).unwrap()];
        assert_eq!(differ.take(frames.clone(), false).len(), 1);
        differ.reset();
        assert_eq!(differ.take(frames, false).len(), 1);
    }

    #[test]
    fn projection_differ_always_passes_show_frames() {
        let mut differ = ProjectionDiffer::new();
        let frames = vec![Outbound::show(true), Outbound::show(true)];
        assert_eq!(differ.take(frames, false).len(), 2);
    }
}
