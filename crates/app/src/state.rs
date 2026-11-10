//! Application state ownership, persistence and mutation entry points.
//!
//! Every UI edit goes through [`AppStore`], which validates the domain model,
//! rebuilds the asset catalogue, marks the state dirty and projects the new
//! payloads to the overlay server. The saver loop persists dirty state every
//! 500 ms and once more when the root component is dropped.

use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use fgc_core::{
    AppState, JsonStore, MemoryStore, Store, TimerAlignment, TimerMode, validate_and_repair,
};
use fgc_protocol::AssetCatalog;
use fgc_server::{AssetMode, Outbound, ServerConfig};
use tracing::{debug, warn};

use crate::server::{ServerController, base_url_for, project_outbound, server_config};

/// How often dirty state is written to disk.
const SAVE_INTERVAL: Duration = Duration::from_millis(500);

/// UI-facing owner of the domain state and the overlay server.
#[derive(Clone, Copy)]
pub struct AppStore {
    state: Signal<AppState>,
    catalog: Signal<AssetCatalog>,
    store: Signal<Arc<dyn Store>>,
    dirty: Signal<bool>,
    saving: Signal<bool>,
    last_error: Signal<Option<String>>,
    server: ServerController,
}

#[derive(Clone)]
struct Bootstrap {
    state: AppState,
    store: Arc<dyn Store>,
    error: Option<String>,
}

impl AppStore {
    /// Creates the store, loads persisted state and starts the background
    /// saver and server-status tasks.
    ///
    /// Must be called from a component hook.
    pub fn use_store() -> Self {
        let server = ServerController::use_controller();
        let Bootstrap {
            state: initial_state,
            store,
            error,
        } = use_hook(bootstrap);

        let state = use_signal(|| initial_state.clone());
        let catalog = use_signal(|| build_catalog(&initial_state));
        let store = use_signal(|| store);
        let dirty = use_signal(|| false);
        let saving = use_signal(|| false);
        let last_error = use_signal(|| error);

        let app = Self {
            state,
            catalog,
            store,
            dirty,
            saving,
            last_error,
            server,
        };
        app.spawn_saver();
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

    /// The last user-facing error, if any.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.cloned()
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

    fn spawn_saver(&self) {
        let mut dirty = self.dirty;
        let mut saving = self.saving;
        let state = self.state;
        let store = self.store;
        let mut last_error = self.last_error;

        use_future(move || async move {
            loop {
                tokio::time::sleep(SAVE_INTERVAL).await;
                if !*dirty.peek() || *saving.peek() {
                    continue;
                }

                let store = store.peek().clone();
                let snapshot = state.cloned();
                dirty.set(false);
                saving.set(true);

                match persist(store, snapshot).await {
                    Ok(()) => debug!("state saved"),
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
        let dirty = self.dirty;
        use_drop(move || {
            if *dirty.peek() {
                let store = store.peek().clone();
                let snapshot = state.cloned();
                if let Err(error) = store.save(&snapshot) {
                    warn!(%error, "failed to save state on exit");
                }
            }
        });
    }
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

    let store: Arc<dyn Store> = match fgc_core::state_file() {
        Ok(path) => Arc::new(JsonStore::new(path)),
        Err(error) => {
            let state = AppState::default();
            return Bootstrap {
                state: state.clone(),
                store: Arc::new(MemoryStore::new(state)),
                error: Some(format!("Could not resolve the state file: {error}")),
            };
        }
    };

    let (state, load_error) = load_state(store.as_ref());
    Bootstrap {
        state,
        store,
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

#[cfg(test)]
mod tests {
    use super::*;
    use fgc_core::{Asset, AssetCategory, AssetId, CoreError, Team, TeamId};

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
        let user = Asset {
            id: AssetId::from_raw("user-1"),
            name: "Custom logo".to_owned(),
            category: AssetCategory::Logo,
            hash: "0".repeat(64),
            ext: "png".to_owned(),
            origin: fgc_core::AssetOrigin::User,
            original_filename: None,
            added_at: None,
            icon: None,
            portrait: None,
            game_mode: None,
        };
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
}
