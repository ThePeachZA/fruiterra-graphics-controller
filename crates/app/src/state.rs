//! Application state ownership, persistence and mutation entry points.
//!
//! Every UI edit goes through [`AppStore`], which validates the domain model,
//! rebuilds the asset catalogue, marks the state dirty and projects the new
//! payloads to the overlay server. The saver loop persists dirty state every
//! 500 ms and once more when the root component is dropped.
//!
//! Asset imports, exports and garbage collection run on blocking tasks so
//! hashing and disk I/O never stall the UI thread.

use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use fgc_core::{
    AppState, Asset as CoreAsset, AssetCategory, AssetId, AssetOrigin, AssetStore, JsonStore,
    MemoryStore, Store, TimerAlignment, TimerMode, UnusedAssets, referenced_hashes,
    validate_and_repair,
};
use fgc_protocol::{AssetCatalog, data_url_for};
use fgc_server::{AssetMode, Outbound, ServerConfig};
use tracing::{debug, info, warn};

use crate::assets_io::format_bytes;
use crate::server::{ServerController, base_url_for, project_outbound, server_config};

/// How often dirty state is written to disk.
const SAVE_INTERVAL: Duration = Duration::from_millis(500);

/// UI-facing owner of the domain state and the overlay server.
#[derive(Clone, Copy)]
pub struct AppStore {
    state: Signal<AppState>,
    catalog: Signal<AssetCatalog>,
    store: Signal<Arc<dyn Store>>,
    asset_store: Signal<Option<AssetStore>>,
    dirty: Signal<bool>,
    saving: Signal<bool>,
    asset_busy: Signal<bool>,
    unused: Signal<UnusedAssets>,
    last_error: Signal<Option<String>>,
    notice: Signal<Option<String>>,
    server: ServerController,
}

#[derive(Clone)]
struct Bootstrap {
    state: AppState,
    store: Arc<dyn Store>,
    asset_store: Option<AssetStore>,
    loaded: bool,
    error: Option<String>,
}

impl AppStore {
    /// Creates the store, loads persisted state and starts the background
    /// saver, garbage collector and server-status tasks.
    ///
    /// Must be called from a component hook.
    pub fn use_store() -> Self {
        let server = ServerController::use_controller();
        let Bootstrap {
            state: initial_state,
            store,
            asset_store,
            loaded,
            error,
        } = use_hook(bootstrap);

        let state = use_signal(|| initial_state.clone());
        let catalog = use_signal(|| build_catalog(&initial_state));
        let store = use_signal(|| store);
        let asset_store = use_signal(|| asset_store);
        let dirty = use_signal(|| false);
        let saving = use_signal(|| false);
        let asset_busy = use_signal(|| false);
        let unused = use_signal(UnusedAssets::default);
        let last_error = use_signal(|| error);
        let notice = use_signal(|| None::<String>);

        let app = Self {
            state,
            catalog,
            store,
            asset_store,
            dirty,
            saving,
            asset_busy,
            unused,
            last_error,
            notice,
            server,
        };
        app.spawn_saver();
        if loaded {
            app.spawn_startup_gc();
        }
        app.refresh_unused();
        app
    }

    /// The current domain state as a signal for rendering.
    #[must_use]
    pub fn state(&self) -> Signal<AppState> {
        self.state
    }

    /// The overlay server controller.
    #[must_use]
    pub fn server(&self) -> ServerController {
        self.server
    }

    /// True when there are unsaved changes.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        *self.dirty.read()
    }

    /// True while a save is in flight.
    #[must_use]
    pub fn is_saving(&self) -> bool {
        *self.saving.read()
    }

    /// True while an asset import, export or cleanup is in flight.
    #[must_use]
    pub fn asset_busy(&self) -> bool {
        *self.asset_busy.read()
    }

    /// Totals for blobs that no current asset metadata references.
    #[must_use]
    pub fn unused(&self) -> UnusedAssets {
        *self.unused.read()
    }

    /// The last user-facing error, if any.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.cloned()
    }

    /// The last success message, if any.
    #[must_use]
    pub fn notice(&self) -> Option<String> {
        self.notice.cloned()
    }

    /// Number of bundled and user assets in the current catalogue.
    #[must_use]
    pub fn asset_count(&self) -> usize {
        self.catalog.read().len()
    }

    /// Clears the error banner.
    pub fn dismiss_error(&self) {
        let mut last_error = self.last_error;
        last_error.set(None);
    }

    /// Clears the success message.
    pub fn dismiss_notice(&self) {
        let mut notice = self.notice;
        notice.set(None);
    }

    /// Surfaces a user-facing error through the banner and a toast.
    pub fn report_error(&self, message: impl Into<String>) {
        let mut last_error = self.last_error;
        last_error.set(Some(message.into()));
    }

    /// Surfaces a user-facing success message through a toast.
    pub fn notify(&self, message: impl Into<String>) {
        let mut notice = self.notice;
        notice.set(Some(message.into()));
    }

    /// Applies a mutation, validates the result and reprojects it.
    ///
    /// Validation repairs are logged rather than shown as errors because the
    /// persisted model is expected to heal itself after malformed edits.
    pub fn mutate(&self, mutate: impl FnOnce(&mut AppState)) {
        let mut state_signal = self.state;
        let repairs = {
            let mut state = state_signal.write();
            mutate(&mut state);
            validate_and_repair(&mut state)
        };
        for repair in &repairs {
            warn!(?repair, "repaired invalid state");
        }

        self.rebuild_catalog();
        let mut dirty = self.dirty;
        dirty.set(true);
        self.reproject();
    }

    /// Changes the overlay server port and restarts a running listener.
    pub fn set_port(&self, port: u16) {
        let port = port.clamp(1, u16::MAX);
        if self.state.peek().settings.overlay.port == port {
            return;
        }
        self.mutate(|state| state.settings.overlay.port = port);
        self.apply_network_change();
    }

    /// Enables or disables binding on the LAN interface.
    pub fn set_allow_lan(&self, allow_lan: bool) {
        if self.state.peek().settings.overlay.allow_lan == allow_lan {
            return;
        }
        self.mutate(|state| state.settings.overlay.allow_lan = allow_lan);
        self.apply_network_change();
    }

    /// Switches between served asset URLs and inline data URLs.
    pub fn set_asset_mode(&self, mode: AssetMode) {
        if self.server.asset_mode() == mode {
            return;
        }
        self.server.set_asset_mode(mode);
        self.reproject();
    }

    /// Changes the timer counting mode.
    pub fn set_timer_mode(&self, mode: TimerMode) {
        if self.state.peek().settings.timer.mode == mode {
            return;
        }
        self.mutate(|state| state.settings.timer.mode = mode);
    }

    /// Changes the timer text alignment.
    pub fn set_timer_alignment(&self, alignment: TimerAlignment) {
        if self.state.peek().settings.timer.alignment == alignment {
            return;
        }
        self.mutate(|state| state.settings.timer.alignment = alignment);
    }

    /// Toggles zero-padding for single-digit timer values.
    pub fn set_timer_leading_zeroes(&self, leading_zeroes: bool) {
        if self.state.peek().settings.timer.leading_zeroes == leading_zeroes {
            return;
        }
        self.mutate(|state| state.settings.timer.leading_zeroes = leading_zeroes);
    }

    /// Toggles the standalone timer layout.
    pub fn set_timer_standalone(&self, standalone: bool) {
        if self.state.peek().settings.timer.standalone == standalone {
            return;
        }
        self.mutate(|state| state.settings.timer.standalone = standalone);
    }

    /// Renames a user asset. Bundled assets are read-only.
    pub fn rename_asset(&self, id: AssetId, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            self.report_error("Asset names cannot be empty.");
            return;
        }
        if !self.state.peek().assets.iter().any(|asset| asset.id == id) {
            self.report_error("That asset is bundled and cannot be renamed.");
            return;
        }

        self.mutate(|state| {
            if let Some(asset) = state.assets.iter_mut().find(|asset| asset.id == id) {
                asset.name = trimmed.to_owned();
            }
        });
    }

    /// Deletes a user asset and clears every reference to it.
    ///
    /// The blob itself becomes eligible for garbage collection once the
    /// deletion is saved; the asset's previous bytes are never mutated.
    pub fn delete_asset(&self, id: AssetId) {
        if !self.state.peek().assets.iter().any(|asset| asset.id == id) {
            return;
        }

        self.mutate(|state| {
            state.assets.retain(|asset| asset.id != id);
            clear_asset_references(state, &id);
        });
        self.refresh_unused();
        self.notify("Asset deleted. Its data is cleaned up after the next save.");
    }

    /// Opens the file picker and imports the chosen image into `category`.
    pub fn import_asset_from_file(&self, category: AssetCategory) {
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let picked = tokio::task::spawn_blocking(crate::assets_io::pick_image_file).await;
            let path = match picked {
                Ok(Some(path)) => path,
                Ok(None) => {
                    this.set_asset_busy(false);
                    return;
                }
                Err(error) => {
                    this.report_error(format!("The file picker failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned);
            let display_name = display_name_from_filename(filename.as_deref());

            let read = tokio::task::spawn_blocking(move || std::fs::read(&path)).await;
            let bytes = match read {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(error)) => {
                    this.report_error(format!("Could not read the image: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
                Err(error) => {
                    this.report_error(format!("The file read task failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            finish_import(this, asset_store, category, display_name, bytes, filename).await;
        });
    }

    /// Fetches an image from `raw_url` and imports it into `category`.
    pub fn import_asset_from_url(&self, category: AssetCategory, raw_url: &str) {
        if self.asset_busy() {
            return;
        }
        let url = match crate::assets_io::validate_import_url(raw_url) {
            Ok(url) => url,
            Err(error) => {
                self.report_error(error);
                return;
            }
        };
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let fetched = crate::assets_io::fetch_image(url.clone()).await;
            let (bytes, filename) = match fetched {
                Ok(result) => result,
                Err(error) => {
                    this.report_error(error);
                    this.set_asset_busy(false);
                    return;
                }
            };

            let display_name = display_name_from_filename(filename.as_deref());
            finish_import(
                this,
                asset_store,
                category,
                display_name,
                bytes,
                Some(url.to_string()),
            )
            .await;
        });
    }

    /// Exports the current project into a new timestamped folder.
    pub fn export_bundle(&self) {
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };
        let state = self.state.peek().clone();

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let picked = tokio::task::spawn_blocking(|| {
                crate::assets_io::pick_bundle_folder("Choose where to create the project bundle")
            })
            .await;
            let parent = match picked {
                Ok(Some(parent)) => parent,
                Ok(None) => {
                    this.set_asset_busy(false);
                    return;
                }
                Err(error) => {
                    this.report_error(format!("The folder picker failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            let directory = crate::assets_io::unique_bundle_directory(&parent);
            let export_directory = directory.clone();
            let exported = tokio::task::spawn_blocking(move || {
                asset_store.export_bundle(&state, &export_directory)
            })
            .await;

            match exported {
                Ok(Ok(report)) => this.notify(format!(
                    "Exported {} file(s) ({}) to {}.",
                    report.files,
                    format_bytes(report.bytes),
                    directory.display()
                )),
                Ok(Err(error)) => {
                    this.report_error(format!("Could not export the project: {error}"))
                }
                Err(error) => this.report_error(format!("The export task failed: {error}")),
            }
            this.set_asset_busy(false);
        });
    }

    /// Imports a project bundle, replacing the current state.
    pub fn import_bundle(&self) {
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let picked = tokio::task::spawn_blocking(|| {
                crate::assets_io::pick_bundle_folder("Choose a project bundle folder")
            })
            .await;
            let directory = match picked {
                Ok(Some(directory)) => directory,
                Ok(None) => {
                    this.set_asset_busy(false);
                    return;
                }
                Err(error) => {
                    this.report_error(format!("The folder picker failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            let imported =
                tokio::task::spawn_blocking(move || asset_store.import_bundle(&directory)).await;
            match imported {
                Ok(Ok((imported, report))) => {
                    this.mutate(move |state| *state = imported);
                    this.refresh_unused();
                    this.notify(format!(
                        "Imported project ({} file(s), {}).",
                        report.files,
                        format_bytes(report.bytes)
                    ));
                }
                Ok(Err(error)) => {
                    this.report_error(format!("Could not import the project: {error}"))
                }
                Err(error) => this.report_error(format!("The import task failed: {error}")),
            }
            this.set_asset_busy(false);
        });
    }

    /// Deletes unreferenced blobs, keeping anything the saved or current state
    /// still points at.
    pub fn clean_up_unused(&self) {
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };
        let state = self.state.peek().clone();
        let store = self.store.peek().clone();

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let collected = tokio::task::spawn_blocking(move || {
                let saved = store.load().ok();
                let mut keep = referenced_hashes(&state);
                if let Some(saved) = saved {
                    keep.extend(referenced_hashes(&saved));
                }
                asset_store.collect_garbage_hashes(&keep)
            })
            .await;

            match collected {
                Ok(Ok(report)) => this.notify(format!(
                    "Removed {} unused file(s) ({}).",
                    report.files,
                    format_bytes(report.bytes)
                )),
                Ok(Err(error)) => this.report_error(format!("Could not clean up assets: {error}")),
                Err(error) => this.report_error(format!("The cleanup task failed: {error}")),
            }
            this.refresh_unused();
            this.set_asset_busy(false);
        });
    }

    /// Recomputes the unused-asset totals using the same keep set as cleanup:
    /// the union of the saved snapshot and the current in-memory state.
    pub fn refresh_unused(&self) {
        let Some(asset_store) = self.asset_store.peek().clone() else {
            return;
        };
        let state = self.state.peek().clone();
        let store = self.store.peek().clone();
        let mut unused = self.unused;
        spawn(async move {
            let totals = tokio::task::spawn_blocking(move || {
                let mut keep = referenced_hashes(&state);
                if let Ok(saved) = store.load() {
                    keep.extend(referenced_hashes(&saved));
                }
                asset_store.unused_hashes(&keep)
            })
            .await;
            match totals {
                Ok(Ok(totals)) => unused.set(totals),
                Ok(Err(error)) => warn!(%error, "could not count unused assets"),
                Err(error) => warn!(%error, "unused asset count task failed"),
            }
        });
    }

    /// Loads an asset's bytes and returns a data URL for previews and thumbnails.
    pub async fn asset_data_url(self, asset: CoreAsset) -> Option<String> {
        let store = self.asset_store.peek().clone();
        let bytes = match asset.origin {
            AssetOrigin::Bundled => {
                fgc_server::bundled_asset_bytes_by_hash(&asset.hash, &asset.ext)
                    .map(std::borrow::Cow::into_owned)
            }
            AssetOrigin::User => {
                let store = store?;
                let hash = asset.hash.clone();
                let ext = asset.ext.clone();
                Some(
                    tokio::task::spawn_blocking(move || store.read_blob(&hash, &ext).ok())
                        .await
                        .ok()??,
                )
            }
        }?;
        Some(data_url_for(&asset, &bytes))
    }

    /// Starts the overlay server with the current settings.
    pub fn start_server(&self) {
        let (config, base_url, outbound) = self.server_plan();
        self.server.set_base_url(base_url.clone());
        self.server.start(config, base_url, outbound);
    }

    /// Stops the overlay server.
    pub fn stop_server(&self) {
        self.server.stop();
    }

    /// Restarts the overlay server with the current settings.
    pub fn restart_server(&self) {
        let (config, base_url, outbound) = self.server_plan();
        self.server.set_base_url(base_url.clone());
        self.server.restart(config, base_url, outbound);
    }

    fn apply_network_change(&self) {
        let (allow_lan, port) = {
            let state = self.state.peek();
            (
                state.settings.overlay.allow_lan,
                state.settings.overlay.port,
            )
        };
        let base_url = base_url_for(allow_lan, port);
        self.server.set_base_url(base_url);

        if self.server.is_running() {
            self.restart_server();
        }
    }

    fn server_plan(&self) -> (ServerConfig, String, Vec<Outbound>) {
        let state = self.state.peek();
        let overlay = &state.settings.overlay;
        let mut config = server_config(overlay.port, overlay.allow_lan, self.server.asset_mode());

        if let Ok(dir) = fgc_core::overlays_override_dir() {
            config = config.with_overlay_override_dir(dir);
        }
        let blob_root = fgc_core::assets_root().ok();
        if let Some(root) = &blob_root {
            config = config.with_blob_dir(root);
        }

        let base_url = base_url_for(overlay.allow_lan, overlay.port);
        let outbound = project_outbound(
            &state,
            &self.catalog.peek(),
            &base_url,
            self.server.asset_mode(),
            blob_root.as_deref(),
        );
        (config, base_url, outbound)
    }

    fn rebuild_catalog(&self) {
        let catalog = build_catalog(&self.state.peek());
        let mut catalog_signal = self.catalog;
        catalog_signal.set(catalog);
    }

    fn reproject(&self) {
        self.server
            .project(&self.state.peek(), &self.catalog.peek());
    }

    fn set_asset_busy(&self, busy: bool) {
        let mut asset_busy = self.asset_busy;
        asset_busy.set(busy);
    }

    fn spawn_saver(&self) {
        let mut dirty = self.dirty;
        let mut saving = self.saving;
        let state = self.state;
        let store = self.store;
        let asset_store = self.asset_store;
        let mut unused = self.unused;
        let mut last_error = self.last_error;

        use_future(move || async move {
            loop {
                tokio::time::sleep(SAVE_INTERVAL).await;
                if !*dirty.peek() || *saving.peek() {
                    continue;
                }

                let store = store.peek().clone();
                let snapshot = state.cloned();
                let snapshot_for_gc = snapshot.clone();
                dirty.set(false);
                saving.set(true);

                match persist(store, snapshot).await {
                    Ok(()) => {
                        debug!("state saved");
                        if let Some(asset_store) = asset_store.peek().clone() {
                            let current = state.peek().clone();
                            let collected = tokio::task::spawn_blocking(move || {
                                let mut keep = referenced_hashes(&current);
                                keep.extend(referenced_hashes(&snapshot_for_gc));
                                asset_store.collect_garbage_hashes(&keep)
                            })
                            .await;
                            match collected {
                                Ok(Ok(report)) => {
                                    if report.files > 0 {
                                        debug!(
                                            files = report.files,
                                            bytes = report.bytes,
                                            "collected unused asset blobs"
                                        );
                                    }
                                    unused.set(UnusedAssets::default());
                                }
                                Ok(Err(error)) => {
                                    warn!(%error, "asset cleanup after save failed");
                                }
                                Err(error) => {
                                    warn!(%error, "asset cleanup task after save failed");
                                }
                            }
                        }
                    }
                    Err(error) => {
                        dirty.set(true);
                        last_error.set(Some(format!("Could not save: {error}")));
                    }
                }
                saving.set(false);
            }
        });

        let state = self.state;
        let store = self.store;
        let asset_store = self.asset_store;
        let dirty = self.dirty;
        use_drop(move || {
            if *dirty.peek() {
                let store = store.peek().clone();
                let snapshot = state.cloned();
                match store.save(&snapshot) {
                    Ok(()) => {
                        if let Some(asset_store) = asset_store.peek().clone() {
                            let mut keep = referenced_hashes(&snapshot);
                            keep.extend(referenced_hashes(&state.peek()));
                            if let Err(error) = asset_store.collect_garbage_hashes(&keep) {
                                warn!(%error, "asset cleanup on exit failed");
                            }
                        }
                    }
                    Err(error) => warn!(%error, "failed to save state on exit"),
                }
            }
        });
    }

    fn spawn_startup_gc(&self) {
        let Some(asset_store) = self.asset_store.peek().clone() else {
            return;
        };
        let state = self.state.peek().clone();
        let mut unused = self.unused;

        spawn(async move {
            let collected = tokio::task::spawn_blocking(move || {
                let report = asset_store.collect_garbage(&state)?;
                let totals = asset_store.unused(&state)?;
                Ok::<_, fgc_core::CoreError>((report, totals))
            })
            .await;

            match collected {
                Ok(Ok((report, totals))) => {
                    unused.set(totals);
                    if report.files > 0 {
                        info!(
                            files = report.files,
                            bytes = report.bytes,
                            "collected unreferenced asset blobs on startup"
                        );
                    }
                }
                Ok(Err(error)) => warn!(%error, "startup asset cleanup failed"),
                Err(error) => warn!(%error, "startup asset cleanup task failed"),
            }
        });
    }
}

/// Removes every reference to `id` from teams, players, talent, asset links
/// and display settings.
pub fn clear_asset_references(state: &mut AppState, id: &AssetId) {
    for team in &mut state.teams {
        if team.logo.as_ref() == Some(id) {
            team.logo = None;
        }
        for player in &mut team.players {
            if player.portrait.as_ref() == Some(id) {
                player.portrait = None;
            }
            if player.featured_hero.as_ref() == Some(id) {
                player.featured_hero = None;
            }
        }
    }

    for talent in &mut state.talents {
        if talent.portrait.as_ref() == Some(id) {
            talent.portrait = None;
        }
    }

    for asset in &mut state.assets {
        if asset.icon.as_ref() == Some(id) {
            asset.icon = None;
        }
        if asset.portrait.as_ref() == Some(id) {
            asset.portrait = None;
        }
        if asset.game_mode.as_ref() == Some(id) {
            asset.game_mode = None;
        }
    }

    if state.settings.display.tournament_logo.as_ref() == Some(id) {
        state.settings.display.tournament_logo = None;
    }
    if state.settings.display.watermark_logo.as_ref() == Some(id) {
        state.settings.display.watermark_logo = None;
    }
}

/// Derives a display name from a file name, falling back to a neutral label.
#[must_use]
pub fn display_name_from_filename(filename: Option<&str>) -> String {
    filename
        .and_then(|name| name.rsplit_once('.').map(|(stem, _)| stem))
        .map(str::trim)
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Imported asset")
        .to_owned()
}

/// Loads state from a store, returning defaults and an error message on failure.
#[must_use]
pub fn load_state(store: &dyn Store) -> (AppState, Option<String>) {
    match store.load() {
        Ok(state) => (state, None),
        Err(error) => (
            AppState::default(),
            Some(format!("Could not load saved state: {error}")),
        ),
    }
}

/// Merges the embedded bundled catalogue with user assets.
#[must_use]
pub fn build_catalog(state: &AppState) -> AssetCatalog {
    let mut assets = fgc_server::bundled_catalog();
    assets.extend(state.assets.iter().cloned());
    AssetCatalog::new(assets)
}

/// Returns the default state file and data directories, falling back to an
/// in-memory store when the platform directories cannot be resolved.
fn bootstrap() -> Bootstrap {
    let directory_error = fgc_core::ensure_app_dirs()
        .err()
        .map(|error| format!("Could not create the data directories: {error}"));
    let asset_store = fgc_core::assets_root().ok().map(AssetStore::new);

    let store: Arc<dyn Store> = match fgc_core::state_file() {
        Ok(path) => Arc::new(JsonStore::new(path)),
        Err(error) => {
            let state = AppState::default();
            return Bootstrap {
                state: state.clone(),
                store: Arc::new(MemoryStore::new(state)),
                asset_store,
                loaded: false,
                error: Some(format!("Could not resolve the state file: {error}")),
            };
        }
    };

    let (state, load_error) = load_state(store.as_ref());
    let loaded = load_error.is_none();
    Bootstrap {
        state,
        store,
        asset_store,
        loaded,
        error: directory_error.or(load_error),
    }
}

async fn persist(store: Arc<dyn Store>, state: AppState) -> Result<(), String> {
    match tokio::task::spawn_blocking(move || store.save(&state)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(error.to_string()),
        Err(error) => Err(format!("save task failed: {error}")),
    }
}

async fn finish_import(
    this: AppStore,
    asset_store: AssetStore,
    category: AssetCategory,
    name: String,
    bytes: Vec<u8>,
    original_filename: Option<String>,
) {
    let inserted = tokio::task::spawn_blocking(move || {
        asset_store.insert(name, category, &bytes, original_filename)
    })
    .await;

    match inserted {
        Ok(Ok(outcome)) => {
            let asset = outcome.asset;
            let for_state = asset.clone();
            this.mutate(move |state| state.assets.push(for_state));
            this.refresh_unused();
            this.notify(format!("Imported “{}”.", asset.name));
        }
        Ok(Err(error)) => this.report_error(format!("Could not import the image: {error}")),
        Err(error) => this.report_error(format!("The import task failed: {error}")),
    }
    this.set_asset_busy(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fgc_core::{
        Asset, AssetCategory, CoreError, Player, PlayerId, Role, Talent, TalentId, Team, TeamId,
    };

    fn team(name: &str) -> Team {
        Team {
            id: TeamId::new(),
            name: name.to_owned(),
            tag: "PCH".to_owned(),
            record: None,
            color: "#000000".to_owned(),
            text_color: "#ffffff".to_owned(),
            aux_color: "#090909".to_owned(),
            logo: None,
            players: Vec::new(),
        }
    }

    fn user_asset(name: &str) -> Asset {
        Asset::new_user(name, AssetCategory::Logo, "0".repeat(64), "png", None)
            .expect("valid asset")
    }

    #[test]
    fn load_state_returns_the_stored_state() {
        let mut state = AppState::default();
        state.teams.push(team("Peaches"));
        let store = MemoryStore::new(state.clone());
        let (loaded, error) = load_state(&store);
        assert!(error.is_none());
        assert_eq!(loaded, state);
    }

    #[test]
    fn load_state_falls_back_to_defaults_on_error() {
        struct Failing;
        impl Store for Failing {
            fn load(&self) -> Result<AppState, CoreError> {
                Err(CoreError::StorePoisoned)
            }

            fn save(&self, _state: &AppState) -> Result<(), CoreError> {
                Ok(())
            }
        }

        let (loaded, error) = load_state(&Failing);
        assert_eq!(loaded, AppState::default());
        assert!(error.is_some());
    }

    #[test]
    fn catalogue_contains_bundled_and_user_assets() {
        let mut state = AppState::default();
        let user = user_asset("Custom logo");
        state.assets.push(user.clone());

        let catalog = build_catalog(&state);
        assert!(catalog.len() > 1);
        assert!(catalog.get(&user.id).is_some());
    }

    #[test]
    fn validation_repairs_duplicate_team_ids() {
        let mut state = AppState::default();
        let first = team("Peaches");
        let mut second = team("Mangoes");
        second.id = first.id.clone();
        state.teams.push(first);
        state.teams.push(second);

        let repairs = validate_and_repair(&mut state);
        assert!(!repairs.is_empty());
    }

    #[test]
    fn blob_paths_are_not_required_for_served_projection() {
        // Served projection must not touch the filesystem; an empty catalogue
        // is enough to prove the helper is pure.
        let state = AppState::default();
        let catalog = AssetCatalog::default();
        let outbound = project_outbound(
            &state,
            &catalog,
            "http://127.0.0.1:3001",
            AssetMode::Served,
            None,
        );
        assert!(outbound.is_empty());
    }

    #[test]
    fn server_plan_uses_the_configured_port() {
        let config = server_config(4123, false, AssetMode::Served);
        assert_eq!(config.port, 4123);
    }

    #[test]
    fn deleting_an_asset_clears_every_reference() {
        let mut state = AppState::default();
        let asset = user_asset("Referenced");
        let id = asset.id.clone();
        state.assets.push(asset);

        let mut linked = user_asset("Linked");
        linked.icon = Some(id.clone());
        linked.portrait = Some(id.clone());
        linked.game_mode = Some(id.clone());
        state.assets.push(linked);

        let mut peach = team("Peaches");
        peach.logo = Some(id.clone());
        peach.players.push(Player {
            id: PlayerId::new(),
            name: "Player".to_owned(),
            role: Role::default(),
            position: Default::default(),
            portrait: Some(id.clone()),
            featured_hero: Some(id.clone()),
            pronouns: None,
        });
        state.teams.push(peach);

        state.talents.push(Talent {
            id: TalentId::new(),
            display_name: "Caster".to_owned(),
            real_name: None,
            pronouns: None,
            social: None,
            portrait: Some(id.clone()),
        });
        state.settings.display.tournament_logo = Some(id.clone());
        state.settings.display.watermark_logo = Some(id.clone());

        clear_asset_references(&mut state, &id);

        assert!(state.teams[0].logo.is_none());
        assert!(state.teams[0].players[0].portrait.is_none());
        assert!(state.teams[0].players[0].featured_hero.is_none());
        assert!(state.talents[0].portrait.is_none());
        assert!(state.assets[1].icon.is_none());
        assert!(state.assets[1].portrait.is_none());
        assert!(state.assets[1].game_mode.is_none());
        assert!(state.settings.display.tournament_logo.is_none());
        assert!(state.settings.display.watermark_logo.is_none());
    }

    #[test]
    fn display_names_drop_the_extension_and_fall_back() {
        assert_eq!(
            display_name_from_filename(Some("team-logo.png")),
            "team-logo"
        );
        assert_eq!(
            display_name_from_filename(Some(".hidden")),
            "Imported asset"
        );
        assert_eq!(display_name_from_filename(None), "Imported asset");
    }
}
