//! Application state ownership, persistence and mutation entry points.
//!
//! Every UI edit goes through [`AppStore`], which validates the domain model,
//! rebuilds the asset catalogue, marks the state dirty and projects the new
//! payloads to the overlay server. The saver loop persists dirty state every
//! 500 ms and once more when the root component is dropped.
//!
//! Asset imports, exports and garbage collection run on blocking tasks so
//! hashing and disk I/O never stall the UI thread.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;
use fgc_core::{
    AppState, Asset as CoreAsset, AssetAttachment, AssetAttachmentKind, AssetCategory, AssetId,
    AssetOrigin, AssetStore, BracketFormat, BracketMatchStatus, BracketSection, BracketSlot,
    DEFAULT_TIEBREAKERS, GroupId, IntervieweeRef, JsonStore, LinkError, LinkKind, MapSlot, Match,
    MatchId, MatchLink, MatchStatus, MemoryStore, Player, PlayerId, Position, RemoteAccessMode,
    Role, Scoring, Side, SlotError, Store, Talent, TalentId, TalentRef, TalentSlot, TalentType,
    Team, TeamId, TeamReference, TeamSide, Tiebreaker, TimerAlignment, TimerMode, Tournament,
    TournamentId, TournamentStage, TournamentStatus, UnusedAssets, add_entrant as core_add_entrant,
    add_group_fixture as core_add_group_fixture, add_match as core_add_match,
    add_round as core_add_round, clear_talent_references, derived_scores, insert_map,
    match_mut as core_match_mut, move_map as move_map_slot, move_map_pool as move_map_pool_slot,
    new_match, new_tournament, recalculate_loser_picks, referenced_hashes,
    refresh as core_refresh_tournament, remove_entrant as core_remove_entrant,
    remove_group as core_remove_group, remove_group_fixture as core_remove_group_fixture,
    remove_map, remove_match as core_remove_match, remove_round as core_remove_round,
    remove_team_references, rename_group as core_rename_group, rename_round as core_rename_round,
    reset_bracket_match as core_reset_bracket_match, reset_hero_bans, reset_maps,
    reset_stage as core_reset_stage, resize_maps_for_first_to,
    set_bracket_result as core_set_bracket_result, set_bracket_winner as core_set_bracket_winner,
    set_first_ban_team, set_group_first_to as core_set_group_first_to,
    set_group_result as core_set_group_result, set_group_teams as core_set_group_teams,
    set_group_winner as core_set_group_winner, set_hero_ban, set_link as core_set_link,
    set_map_mode, set_map_name, set_map_scores, set_slot as core_set_slot, toggle_map_pool,
    validate_and_repair,
};
use fgc_protocol::{AssetCatalog, data_url_for};
use fgc_server::{AssetMode, Outbound, ServerConfig};
use tracing::{debug, info, warn};

use crate::assets_io::format_bytes;
use crate::remote_access;
use crate::server::{ServerController, project_outbound, server_config};
use crate::tailscale::{ServeState, TailscaleState, TailscaleStatus};

/// How often dirty state is written to disk.
const SAVE_INTERVAL: Duration = Duration::from_millis(500);

/// Notice shown when cleanup is refused because state cannot be trusted.
const CLEANUP_PAUSED_NOTICE: &str = "Asset cleanup is paused because the saved state could not be read. Restore app.json or import a project before cleaning up.";

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
    cleanup_allowed: Signal<bool>,
    unused: Signal<UnusedAssets>,
    last_error: Signal<Option<String>>,
    notice: Signal<Option<String>>,
    /// Cached Tailscale client detection; `None` before the first result.
    tailscale: Signal<Option<TailscaleStatus>>,
    /// State of this app's Tailscale Serve `/fgc` handler.
    serve: Signal<ServeState>,
    /// Ephemeral UI state for revealing Browser Source URLs.
    reveal_urls: Signal<bool>,
    /// Cached data URLs for thumbnails, keyed by `hash:ext`.
    thumb_urls: Signal<HashMap<String, String>>,
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
        let cleanup_allowed = use_signal(|| loaded);
        let unused = use_signal(UnusedAssets::default);
        let last_error = use_signal(|| error);
        let notice = use_signal(|| None::<String>);
        let tailscale = use_signal(|| None);
        let serve = use_signal(ServeState::default);
        let reveal_urls = use_signal(|| false);
        let thumb_urls = use_signal(HashMap::<String, String>::new);

        let app = Self {
            state,
            catalog,
            store,
            asset_store,
            dirty,
            saving,
            asset_busy,
            cleanup_allowed,
            unused,
            last_error,
            notice,
            tailscale,
            serve,
            reveal_urls,
            thumb_urls,
            server,
        };
        app.spawn_saver();
        if loaded {
            app.spawn_startup_gc();
        }
        app.refresh_unused();
        app.refresh_tailscale();
        if app.state.peek().settings.overlay.auto_start
            && !app.server.is_running()
            && !app.needs_tailscale_address()
        {
            app.start_server();
        }
        app
    }

    /// Builds a store for unit tests without starting background tasks.
    #[cfg(test)]
    pub(crate) fn test_store(initial: AppState) -> Self {
        let catalog = build_catalog(&initial);
        let store: Arc<dyn Store> = Arc::new(MemoryStore::new(AppState::default()));
        Self {
            state: Signal::new(initial),
            catalog: Signal::new(catalog),
            store: Signal::new(store),
            asset_store: Signal::new(None),
            dirty: Signal::new(false),
            saving: Signal::new(false),
            asset_busy: Signal::new(false),
            cleanup_allowed: Signal::new(true),
            unused: Signal::new(UnusedAssets::default()),
            last_error: Signal::new(None),
            notice: Signal::new(None),
            tailscale: Signal::new(None),
            serve: Signal::new(ServeState::Off),
            reveal_urls: Signal::new(false),
            thumb_urls: Signal::new(HashMap::new()),
            server: ServerController::test_controller(),
        }
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

    /// True while saved state is trustworthy enough to delete unused blobs.
    #[must_use]
    pub fn cleanup_allowed(&self) -> bool {
        *self.cleanup_allowed.read()
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

    /// Cached Tailscale detection; `None` while the first check is in flight.
    #[must_use]
    pub fn tailscale_status(&self) -> Option<TailscaleStatus> {
        self.tailscale.cloned()
    }

    /// State of this app's Tailscale Serve handler.
    #[must_use]
    pub fn serve_state(&self) -> ServeState {
        self.serve.cloned()
    }

    /// MagicDNS name of this device, once Tailscale detection has returned.
    #[must_use]
    pub fn tailscale_dns_name(&self) -> Option<String> {
        match self.tailscale_status()?.state {
            TailscaleState::Ready { dns_name, .. } if !dns_name.trim().is_empty() => {
                Some(dns_name.trim().to_owned())
            }
            _ => None,
        }
    }

    /// Base URL for the selected scope, including the Tailscale prefix.
    fn current_base_url(&self) -> String {
        let dns_name = self.tailscale_dns_name();
        remote_access::base_url(&self.state.peek(), dns_name.as_deref())
    }

    /// True while Tailscale-only auto-start is waiting for the first address.
    fn needs_tailscale_address(&self) -> bool {
        let (tailscale, local) = {
            let state = self.state.peek();
            (
                remote_access::is_tailscale_enabled(&state),
                remote_access::is_local_enabled(&state),
            )
        };
        tailscale && !local && self.tailscale_status().is_none()
    }

    /// True when a Tailscale auto-start can run now that detection has returned.
    fn should_start_after_detection(&self) -> bool {
        let state = self.state.peek();
        let overlay = &state.settings.overlay;
        overlay.auto_start
            && remote_access::is_tailscale_enabled(&state)
            && !self.server.is_running()
            && !self.server.busy()
    }

    /// Rebuilds projected asset URLs after a Tailscale address becomes known.
    fn refresh_base_url(&self) {
        let base_url = self.current_base_url();
        if self.server.base_url() != base_url {
            self.server.set_base_url(base_url);
            if self.server.is_running() {
                self.server
                    .project_force(&self.state.peek(), &self.catalog.peek());
            }
        }
    }

    /// Reads the installed Tailscale client off the UI thread.
    ///
    /// The last known result stays visible while a refresh runs, so revisiting
    /// Settings does not briefly hide the generated URLs.
    pub fn refresh_tailscale(&self) {
        let app = *self;
        let mut slot = self.tailscale;
        spawn(async move {
            let detected = tokio::task::spawn_blocking(crate::tailscale::detect).await;
            let status = detected.unwrap_or_else(|error| TailscaleStatus {
                executable: None,
                version: None,
                state: TailscaleState::Unavailable {
                    reason: format!("detection task failed: {error}"),
                },
            });
            slot.set(Some(status));
            app.refresh_base_url();
            if app.should_start_after_detection() {
                app.start_server();
            }
        });
    }

    /// Whether Browser Source URLs are currently revealed in the UI.
    #[must_use]
    pub fn urls_visible(&self) -> bool {
        *self.reveal_urls.read()
    }

    /// Shows or hides Browser Source URLs across the app.
    pub fn set_urls_visible(&self, visible: bool) {
        let mut reveal_urls = self.reveal_urls;
        reveal_urls.set(visible);
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

    /// Enables or disables local network access without changing Tailscale.
    pub fn set_local_network(&self, enabled: bool) {
        let current = self.state.peek().settings.overlay.allow_lan;
        if current == enabled {
            return;
        }
        self.mutate(move |state| {
            let overlay = &mut state.settings.overlay;
            overlay.allow_lan = enabled;
            if !enabled && overlay.remote_access == RemoteAccessMode::Lan {
                overlay.remote_access = RemoteAccessMode::Off;
            } else if enabled && overlay.remote_access == RemoteAccessMode::Off {
                overlay.remote_access = RemoteAccessMode::Lan;
            }
        });
        self.apply_network_change();
    }

    /// Enables or disables Tailscale Serve without changing local network access.
    pub fn set_tailscale(&self, enabled: bool) {
        let active =
            self.state.peek().settings.overlay.remote_access == RemoteAccessMode::Tailscale;
        if active == enabled {
            return;
        }
        self.mutate(move |state| {
            let overlay = &mut state.settings.overlay;
            overlay.remote_access = if enabled {
                RemoteAccessMode::Tailscale
            } else if overlay.allow_lan {
                RemoteAccessMode::Lan
            } else {
                RemoteAccessMode::Off
            };
        });
        self.apply_network_change();
    }

    /// Starts the overlay server automatically on the next application launch.
    pub fn set_auto_start(&self, auto_start: bool) {
        if self.state.peek().settings.overlay.auto_start == auto_start {
            return;
        }
        self.mutate(|state| state.settings.overlay.auto_start = auto_start);
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

    /// Returns the merged bundled and user asset catalogue.
    #[must_use]
    pub fn catalog(&self) -> AssetCatalog {
        self.catalog.peek().clone()
    }

    /// Creates the default current match with eight empty map slots.
    pub fn create_match(&self) {
        let match_state = new_match();
        self.mutate(move |state| state.current_match = Some(match_state));
    }

    /// Sets the scheduled start of the current match, as typed into a
    /// `datetime-local` field. An empty value clears it.
    pub fn set_match_scheduled_at(&self, value: String) {
        let scheduled = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
        self.update_match(|current| current.scheduled_at = scheduled);
    }

    /// Saves a snapshot of the current match into the saved-match library.
    ///
    /// A match with the same id is replaced, so saving twice does not create
    /// duplicates. The current match itself is left unchanged and stays live.
    pub fn save_current_match(&self) {
        let snapshot = self.state.peek().current_match.clone();
        let Some(snapshot) = snapshot else {
            self.report_error("There is no current match to save.");
            return;
        };
        let title = snapshot.title.clone();
        self.mutate(move |state| {
            if let Some(existing) = state
                .matches
                .iter_mut()
                .find(|entry| entry.id == snapshot.id)
            {
                *existing = snapshot;
            } else {
                state.matches.push(snapshot);
            }
        });
        self.notify(format!("Saved “{title}” to the match library."));
    }

    /// Loads a saved match as the current match for live control.
    pub fn load_saved_match(&self, id: MatchId) {
        let saved = self
            .state
            .peek()
            .matches
            .iter()
            .find(|entry| entry.id == id)
            .cloned();
        let Some(saved) = saved else { return };
        let title = saved.title.clone();
        self.mutate(move |state| state.current_match = Some(saved));
        self.notify(format!("Loaded “{title}” into the controller."));
    }

    /// Removes a saved match without touching the current match.
    pub fn delete_saved_match(&self, id: MatchId) {
        self.mutate(move |state| state.matches.retain(|entry| entry.id != id));
    }

    /// Exports the saved matches, including their map pools, to a JSON file.
    pub fn export_matches(&self) {
        let matches = self.state.peek().matches.clone();
        if matches.is_empty() {
            self.report_error("Save a match to the library before exporting.");
            return;
        }

        let this = *self;
        let count = matches.len();
        spawn(async move {
            let picked =
                tokio::task::spawn_blocking(crate::assets_io::pick_match_library_save).await;
            let path = match picked {
                Ok(Some(path)) => path,
                Ok(None) => return,
                Err(error) => {
                    this.report_error(format!("The save dialog failed: {error}"));
                    return;
                }
            };
            let display = path.display().to_string();
            let write_path = path.clone();
            let saved = tokio::task::spawn_blocking(move || {
                fgc_core::write_match_library(&write_path, &matches)
            })
            .await;
            match saved {
                Ok(Ok(())) => this.notify(format!("Exported {count} match(es) to {display}.")),
                Ok(Err(error)) => this.report_error(format!("Could not export matches: {error}")),
                Err(error) => this.report_error(format!("The export task failed: {error}")),
            }
        });
    }

    /// Imports saved matches from a JSON file, adding any new ids to the
    /// library without replacing the current project.
    pub fn import_matches(&self) {
        let this = *self;
        let current_ids = self
            .state
            .peek()
            .matches
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        spawn(async move {
            let picked =
                tokio::task::spawn_blocking(crate::assets_io::pick_match_library_open).await;
            let path = match picked {
                Ok(Some(path)) => path,
                Ok(None) => return,
                Err(error) => {
                    this.report_error(format!("The open dialog failed: {error}"));
                    return;
                }
            };
            let imported =
                tokio::task::spawn_blocking(move || fgc_core::read_match_library(&path)).await;
            match imported {
                Ok(Ok(matches)) => {
                    let added = matches
                        .iter()
                        .filter(|entry| !current_ids.contains(&entry.id))
                        .count();
                    this.mutate(move |state| {
                        for entry in matches {
                            if state.matches.iter().any(|existing| existing.id == entry.id) {
                                continue;
                            }
                            state.matches.push(entry);
                        }
                    });
                    if added == 0 {
                        this.notify("No new matches to import.");
                    } else {
                        this.notify(format!("Imported {added} match(es)."));
                    }
                }
                Ok(Err(error)) => this.report_error(format!("Could not import matches: {error}")),
                Err(error) => this.report_error(format!("The import task failed: {error}")),
            }
        });
    }

    /// Applies an edit to the current match, if one exists.
    pub fn update_match(&self, update: impl FnOnce(&mut Match)) {
        self.mutate(|state| {
            if let Some(current) = state.current_match.as_mut() {
                update(current);
            }
        });
    }

    /// Assigns or clears one side of the current match.
    pub fn set_match_team(&self, side: TeamSide, team: Option<TeamId>) {
        self.update_match(|current| match side {
            TeamSide::A => current.team_a = team,
            TeamSide::B => current.team_b = team,
        });
    }

    /// Sets the match status.
    pub fn set_match_status(&self, status: MatchStatus) {
        self.update_match(|current| current.status = status);
    }

    /// Sets the side indicator shown by the game overlay.
    pub fn set_match_side(&self, side: Side) {
        self.update_match(|current| current.side = side);
    }

    /// Sets the first-to target and aligns the default map slots.
    ///
    /// Entered maps and user-added tiebreakers are preserved; only trailing
    /// empty slots are removed when the format needs fewer maps.
    pub fn set_match_first_to(&self, first_to: u8) {
        let first_to = first_to.clamp(1, 4);
        self.update_match(|current| {
            current.first_to = first_to;
            resize_maps_for_first_to(current);
        });
    }

    /// Enables loser picks and recalculates every map's picker.
    pub fn set_match_loser_picks(&self, enabled: bool) {
        self.update_match(|current| {
            current.loser_picks = enabled;
            if enabled {
                recalculate_loser_picks(&mut current.maps);
            }
        });
    }

    /// Switches between derived and manual match scores.
    pub fn set_manual_scoring(&self, enabled: bool) {
        self.update_match(|current| {
            current.scoring = if enabled {
                match &current.scoring {
                    Scoring::Manual { a, b } => Scoring::Manual { a: *a, b: *b },
                    Scoring::Derived => {
                        let (a, b) = derived_scores(current);
                        Scoring::Manual { a, b }
                    }
                }
            } else {
                Scoring::Derived
            };
        });
    }

    /// Sets one side of a manual match score.
    pub fn set_manual_score(&self, side: TeamSide, score: u16) {
        self.update_match(|current| {
            let (a, b) = match &current.scoring {
                Scoring::Manual { a, b } => (*a, *b),
                Scoring::Derived => derived_scores(current),
            };
            current.scoring = Scoring::Manual {
                a: if side == TeamSide::A { score } else { a },
                b: if side == TeamSide::B { score } else { b },
            };
        });
    }

    /// Updates a map name and its free-text mode.
    pub fn set_map_name(&self, index: usize, name: &str, mode: &str) {
        self.update_match(|current| set_map_name(current, index, name, mode));
    }

    /// Updates only a map's free-text mode.
    pub fn set_map_mode(&self, index: usize, mode: &str) {
        self.update_match(|current| set_map_mode(current, index, mode));
    }

    /// Updates one map's score.
    pub fn set_map_scores(&self, index: usize, score_a: u16, score_b: u16) {
        self.update_match(|current| set_map_scores(current, index, score_a, score_b));
    }

    /// Marks one map as complete or incomplete.
    pub fn set_map_completed(&self, index: usize, completed: bool) {
        self.update_match(|current| {
            if let Some(map) = current.maps.get_mut(index) {
                map.completed = completed;
            }
        });
    }

    /// Marks one map as the current map, or clears the current map.
    pub fn set_current_map(&self, index: Option<usize>) {
        self.update_match(|current| {
            current.current_map = index.filter(|index| *index < current.maps.len());
        });
    }

    /// Overrides one map's picker.
    pub fn set_map_chosen_by(&self, index: usize, side: Option<TeamSide>) {
        self.update_match(|current| {
            if let Some(map) = current.maps.get_mut(index) {
                map.chosen_by = side;
            }
        });
    }

    /// Inserts an empty map slot at `index`.
    pub fn insert_map(&self, index: usize) {
        self.update_match(|current| insert_map(current, index));
    }

    /// Removes one map slot.
    pub fn remove_map(&self, index: usize) {
        self.update_match(|current| remove_map(current, index));
    }

    /// Moves one map slot by `delta` positions.
    pub fn move_map(&self, index: usize, delta: isize) {
        self.update_match(|current| {
            move_map_slot(current, index, delta);
        });
    }

    /// Resets every map slot and hero-ban row.
    pub fn reset_maps(&self) {
        self.update_match(reset_maps);
    }

    /// Clears every hero ban.
    pub fn reset_hero_bans(&self) {
        self.update_match(reset_hero_bans);
    }

    /// Sets the team that placed the first ban for one map.
    pub fn set_first_ban_team(&self, index: usize, team: Option<TeamSide>) {
        self.update_match(|current| set_first_ban_team(current, index, team));
    }

    /// Sets one team's hero ban for one map.
    pub fn set_hero_ban(&self, index: usize, side: TeamSide, hero: &str) {
        self.update_match(|current| set_hero_ban(current, index, side, hero));
    }

    /// Assigns or clears one talent slot with its on-air role.
    pub fn set_talent_slot(&self, slot: TalentSlot, talent: Option<TalentId>, role: TalentType) {
        self.update_match(|current| {
            current
                .talent
                .set_slot(slot, talent.map(|talent_id| TalentRef { talent_id, role }));
        });
    }

    /// Assigns an interviewee from the talent library, or switches to a custom
    /// interviewee when `talent` is `None`.
    pub fn set_interviewee_talent(&self, talent: Option<TalentId>) {
        self.update_match(|current| {
            let interviewee = current
                .talent
                .interviewee
                .get_or_insert_with(IntervieweeRef::default);
            let has_talent = talent.is_some();
            interviewee.talent_id = talent;
            if has_talent {
                interviewee.custom_name = None;
                interviewee.custom_subtext = None;
            }
        });
    }

    /// Sets the custom interviewee name and subtext.
    pub fn set_interviewee_custom(&self, name: &str, subtext: &str) {
        self.update_match(|current| {
            let interviewee = current
                .talent
                .interviewee
                .get_or_insert_with(IntervieweeRef::default);
            interviewee.custom_name = non_empty(name);
            interviewee.custom_subtext = non_empty(subtext);
        });
    }

    /// Adds or removes a map name from the map pool.
    pub fn set_map_pool_included(&self, name: &str, included: bool) {
        self.update_match(|current| toggle_map_pool(current, name, included));
    }

    /// Moves a map-pool entry by `delta` positions.
    pub fn move_map_pool(&self, index: usize, delta: isize) {
        self.update_match(|current| {
            move_map_pool_slot(current, index, delta);
        });
    }

    /// Resends every projected overlay section.
    pub fn force_update(&self) {
        self.server
            .project_force(&self.state.peek(), &self.catalog.peek());
    }

    /// Sends one timer frame to the overlays.
    pub fn broadcast_timer(&self, message: Outbound) {
        self.server.broadcast(message);
    }

    /// Creates a team with the original application defaults.
    pub fn create_team(&self) -> TeamId {
        let team = new_team();
        let id = team.id.clone();
        self.mutate(move |state| state.teams.push(team));
        id
    }

    /// Applies an edit to one team.
    pub fn update_team(&self, id: &TeamId, update: impl FnOnce(&mut Team)) {
        self.mutate(|state| {
            if let Some(team) = state.teams.iter_mut().find(|team| &team.id == id) {
                update(team);
            }
        });
    }

    /// Reports every tournament or current-match reference to `id`.
    #[must_use]
    pub fn team_references(&self, id: &TeamId) -> Vec<TeamReference> {
        let state = self.state.peek();
        fgc_core::team_references(&state, id)
    }

    /// Deletes `id` when nothing references it.
    ///
    /// # Errors
    ///
    /// Returns the reference list when the team is still in use.
    pub fn delete_team(&self, id: &TeamId) -> Result<(), Vec<TeamReference>> {
        let references = self.team_references(id);
        if !references.is_empty() {
            return Err(references);
        }
        self.mutate(|state| state.teams.retain(|team| &team.id != id));
        Ok(())
    }

    /// Removes every tournament and current-match reference, then deletes the
    /// team.
    pub fn delete_team_with_cleanup(&self, id: &TeamId) {
        self.mutate(|state| {
            remove_team_references(state, id);
            state.teams.retain(|team| &team.id != id);
        });
    }

    /// Adds a player to `team_id` using the original defaults.
    pub fn create_player(&self, team_id: &TeamId) -> Option<PlayerId> {
        let player = new_player();
        let id = player.id.clone();
        let mut created = false;
        self.mutate(|state| {
            if let Some(team) = state.teams.iter_mut().find(|team| &team.id == team_id) {
                team.players.push(player);
                created = true;
            }
        });
        created.then_some(id)
    }

    /// Applies an edit to one player on a team.
    pub fn update_player(
        &self,
        team_id: &TeamId,
        player_id: &PlayerId,
        update: impl FnOnce(&mut Player),
    ) {
        self.mutate(|state| {
            if let Some(team) = state.teams.iter_mut().find(|team| &team.id == team_id)
                && let Some(player) = team
                    .players
                    .iter_mut()
                    .find(|player| &player.id == player_id)
            {
                update(player);
            }
        });
    }

    /// Removes one player from a team.
    pub fn delete_player(&self, team_id: &TeamId, player_id: &PlayerId) {
        self.mutate(|state| {
            if let Some(team) = state.teams.iter_mut().find(|team| &team.id == team_id) {
                team.players.retain(|player| &player.id != player_id);
            }
        });
    }

    /// Creates a talent member with the original defaults.
    pub fn create_talent(&self) -> TalentId {
        let talent = new_talent();
        let id = talent.id.clone();
        self.mutate(move |state| state.talents.push(talent));
        id
    }

    /// Applies an edit to one talent member.
    pub fn update_talent(&self, id: &TalentId, update: impl FnOnce(&mut Talent)) {
        self.mutate(|state| {
            if let Some(talent) = state.talents.iter_mut().find(|talent| &talent.id == id) {
                update(talent);
            }
        });
    }

    /// Deletes a talent member and clears match assignments that used them.
    pub fn delete_talent(&self, id: &TalentId) {
        self.mutate(|state| {
            clear_talent_references(state, id);
            state.talents.retain(|talent| &talent.id != id);
        });
    }

    // -----------------------------------------------------------------------
    // Tournaments
    // -----------------------------------------------------------------------

    /// Applies an edit to one tournament.
    fn with_tournament(&self, id: &TournamentId, update: impl FnOnce(&mut Tournament)) {
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                update(tournament);
            }
        });
    }

    /// Creates a tournament with the documented defaults.
    pub fn create_tournament(
        &self,
        name: impl Into<String>,
        format: BracketFormat,
    ) -> TournamentId {
        let tournament = new_tournament(name, format);
        let id = tournament.id.clone();
        self.mutate(move |state| state.tournaments.push(tournament));
        id
    }

    /// Deletes a tournament.
    pub fn delete_tournament(&self, id: &TournamentId) {
        self.mutate(|state| state.tournaments.retain(|tournament| &tournament.id != id));
    }

    /// Replaces a tournament with a preview that the operator has confirmed.
    pub fn commit_tournament(&self, tournament: Tournament) {
        self.mutate(|state| {
            if let Some(existing) = state
                .tournaments
                .iter_mut()
                .find(|existing| existing.id == tournament.id)
            {
                *existing = tournament;
            }
        });
    }

    /// Adds a team to the entrant list.
    pub fn add_tournament_entrant(&self, id: &TournamentId, team_id: TeamId) {
        self.with_tournament(id, |tournament| {
            core_add_entrant(tournament, team_id);
            core_refresh_tournament(tournament);
        });
    }

    /// Removes a team from the entrant list and normalises the seeds.
    pub fn remove_tournament_entrant(&self, id: &TournamentId, team_id: &TeamId) {
        self.with_tournament(id, |tournament| {
            core_remove_entrant(tournament, team_id);
            core_refresh_tournament(tournament);
        });
    }

    /// Moves an entrant to another seed position.
    pub fn move_tournament_entrant(&self, id: &TournamentId, from: usize, to: usize) {
        self.with_tournament(id, |tournament| {
            if from >= tournament.entrants.len() || to >= tournament.entrants.len() || from == to {
                return;
            }
            let entrant = tournament.entrants.remove(from);
            tournament.entrants.insert(to, entrant);
            for (index, entrant) in tournament.entrants.iter_mut().enumerate() {
                entrant.seed = u32::try_from(index + 1).unwrap_or(u32::MAX);
            }
            core_refresh_tournament(tournament);
        });
    }

    /// Loads a tournament fixture into the current match and refreshes overlays.
    #[allow(clippy::too_many_arguments)]
    pub fn push_tournament_match_to_overlay(
        &self,
        title: String,
        subtitle: String,
        team_a: Option<TeamId>,
        team_b: Option<TeamId>,
        score1: u16,
        score2: u16,
        status: BracketMatchStatus,
        first_to: u8,
    ) {
        let match_status = match status {
            BracketMatchStatus::Pending => MatchStatus::Pre,
            BracketMatchStatus::Live => MatchStatus::Live,
            BracketMatchStatus::Completed => MatchStatus::Post,
        };
        let mut match_state = new_match();
        match_state.title = title;
        match_state.subtitle = subtitle;
        match_state.team_a = team_a;
        match_state.team_b = team_b;
        match_state.first_to = first_to;
        match_state.status = match_status;
        match_state.scoring = Scoring::Manual {
            a: score1,
            b: score2,
        };

        self.mutate(move |state| state.current_match = Some(match_state));
        self.force_update();
        self.notify("Pushed fixture to the overlay controller.");
    }

    /// Renames a tournament.
    pub fn set_tournament_name(&self, id: &TournamentId, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return;
        }
        let name = trimmed.to_owned();
        self.with_tournament(id, |tournament| tournament.name = name);
    }

    /// Changes the selected bracket format.
    pub fn set_tournament_format(&self, id: &TournamentId, format: BracketFormat) {
        self.with_tournament(id, |tournament| tournament.format = format);
    }

    /// Changes the default first-to target for new matches.
    pub fn set_tournament_default_first_to(&self, id: &TournamentId, first_to: u8) {
        self.with_tournament(id, |tournament| {
            tournament.default_first_to = first_to.clamp(1, 4);
        });
    }

    /// Enables or disables the grand-final reset.
    pub fn set_tournament_grand_finals_reset(&self, id: &TournamentId, enabled: bool) {
        self.with_tournament(id, |tournament| {
            tournament.grand_finals_reset = enabled;
            core_refresh_tournament(tournament);
        });
    }

    /// Replaces the ordered tiebreaker list.
    pub fn set_tournament_tiebreakers(&self, id: &TournamentId, tiebreakers: Vec<Tiebreaker>) {
        let tiebreakers = if tiebreakers.is_empty() {
            DEFAULT_TIEBREAKERS.to_vec()
        } else {
            tiebreakers
        };
        self.with_tournament(id, |tournament| {
            tournament.tiebreakers = tiebreakers;
            core_refresh_tournament(tournament);
        });
    }

    /// Restores the default tiebreaker order.
    pub fn reset_tournament_tiebreakers(&self, id: &TournamentId) {
        self.with_tournament(id, |tournament| {
            tournament.tiebreakers = DEFAULT_TIEBREAKERS.to_vec();
            core_refresh_tournament(tournament);
        });
    }

    /// Sets how many teams advance from each group.
    pub fn set_tournament_playoff_qualifiers(&self, id: &TournamentId, qualifiers: u8) {
        self.with_tournament(id, |tournament| {
            tournament.playoff_qualifiers = qualifiers.max(1);
        });
    }

    /// Changes the playoff bracket format.
    pub fn set_tournament_playoff_format(&self, id: &TournamentId, format: BracketFormat) {
        self.with_tournament(id, |tournament| tournament.playoff_format = format);
    }

    /// Clears one generated stage and its derived results.
    pub fn reset_tournament_stage(&self, id: &TournamentId, stage: TournamentStage) {
        self.with_tournament(id, |tournament| core_reset_stage(tournament, stage));
    }

    /// Manually overrides the tournament lifecycle state.
    pub fn set_tournament_status(&self, id: &TournamentId, status: TournamentStatus) {
        self.with_tournament(id, |tournament| tournament.status = status);
    }

    /// Sets or clears a manual champion override.
    ///
    /// A champion that is not an entrant is ignored; clearing the override
    /// falls back to the derived champion.
    pub fn set_tournament_champion(&self, id: &TournamentId, champion: Option<TeamId>) {
        self.with_tournament(id, |tournament| {
            match champion {
                Some(team)
                    if tournament
                        .entrants
                        .iter()
                        .any(|entrant| entrant.team_id == team) =>
                {
                    tournament.champion = Some(team);
                    tournament.champion_manual = true;
                }
                Some(_) => {}
                None => {
                    tournament.champion = None;
                    tournament.champion_manual = false;
                }
            }
            core_refresh_tournament(tournament);
        });
    }

    /// Assigns or clears one bracket slot by hand.
    ///
    /// # Errors
    ///
    /// Returns a [`SlotError`] when the match is missing, the slot number is
    /// invalid or the same team would occupy both sides.
    pub fn set_tournament_slot(
        &self,
        id: &TournamentId,
        address: BracketSlot,
        team: Option<TeamId>,
    ) -> Result<(), SlotError> {
        let mut result = Err(SlotError::NotFound);
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_slot(tournament, address, team);
            }
        });
        result
    }

    /// Appends an empty match to a round.
    pub fn add_tournament_match(
        &self,
        id: &TournamentId,
        section: BracketSection,
        round: usize,
    ) -> Option<MatchId> {
        let mut result = None;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_add_match(tournament, section, round);
            }
        });
        result
    }

    /// Removes a match and clears every link that pointed at it.
    pub fn remove_tournament_match(&self, id: &TournamentId, match_id: &MatchId) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_remove_match(tournament, match_id);
            }
        });
        result
    }

    /// Appends an empty round to a section and returns its index.
    pub fn add_tournament_round(&self, id: &TournamentId, section: BracketSection) -> usize {
        let mut result = 0;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_add_round(tournament, section);
            }
        });
        result
    }

    /// Removes a round and its matches.
    pub fn remove_tournament_round(
        &self,
        id: &TournamentId,
        section: BracketSection,
        round: usize,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_remove_round(tournament, section, round);
            }
        });
        result
    }

    /// Renames a round.
    pub fn rename_tournament_round(
        &self,
        id: &TournamentId,
        section: BracketSection,
        round: usize,
        name: &str,
    ) -> bool {
        let name = name.to_owned();
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_rename_round(tournament, section, round, name);
            }
        });
        result
    }

    /// Sets or clears a winner or loser link.
    ///
    /// # Errors
    ///
    /// Returns a [`LinkError`] when the link is dangling, points at the match
    /// itself or would create a cycle.
    pub fn set_tournament_link(
        &self,
        id: &TournamentId,
        match_id: &MatchId,
        kind: LinkKind,
        link: Option<MatchLink>,
    ) -> Result<(), LinkError> {
        let mut result = Err(LinkError::NotFound);
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_link(tournament, match_id, kind, link);
            }
        });
        result
    }

    /// Sets a bracket match's scores and status, deriving the winner.
    pub fn set_tournament_result(
        &self,
        id: &TournamentId,
        match_id: &MatchId,
        score1: u16,
        score2: u16,
        status: BracketMatchStatus,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_bracket_result(tournament, match_id, score1, score2, status);
            }
        });
        result
    }

    /// Sets or clears a manual winner override.
    pub fn set_tournament_winner(
        &self,
        id: &TournamentId,
        match_id: &MatchId,
        winner: Option<TeamId>,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_bracket_winner(tournament, match_id, winner);
            }
        });
        result
    }

    /// Clears a bracket match's result and its downstream auto-fills.
    pub fn reset_tournament_match(&self, id: &TournamentId, match_id: &MatchId) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_reset_bracket_match(tournament, match_id);
            }
        });
        result
    }

    /// Changes one match's first-to override.
    pub fn set_tournament_match_first_to(
        &self,
        id: &TournamentId,
        match_id: &MatchId,
        first_to: u8,
    ) {
        self.with_tournament(id, |tournament| {
            if let Some(game) = core_match_mut(tournament, match_id) {
                game.first_to = first_to.clamp(1, 4);
            }
            core_refresh_tournament(tournament);
        });
    }

    /// Removes a group and all of its fixtures.
    pub fn remove_tournament_group(&self, id: &TournamentId, group_id: &GroupId) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_remove_group(tournament, group_id);
            }
        });
        result
    }

    /// Renames a group.
    pub fn rename_tournament_group(
        &self,
        id: &TournamentId,
        group_id: &GroupId,
        name: &str,
    ) -> bool {
        let name = name.to_owned();
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_rename_group(tournament, group_id, name);
            }
        });
        result
    }

    /// Replaces a group's team list and regenerates its fixtures.
    pub fn set_tournament_group_teams(
        &self,
        id: &TournamentId,
        group_id: &GroupId,
        teams: Vec<TeamId>,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_group_teams(tournament, group_id, teams);
            }
        });
        result
    }

    /// Adds a group fixture by hand.
    pub fn add_tournament_group_fixture(
        &self,
        id: &TournamentId,
        group_id: &GroupId,
        team1: TeamId,
        team2: TeamId,
    ) -> Option<MatchId> {
        let mut result = None;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_add_group_fixture(tournament, group_id, team1, team2);
            }
        });
        result
    }

    /// Removes a group fixture by hand.
    pub fn remove_tournament_group_fixture(&self, id: &TournamentId, fixture_id: &MatchId) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_remove_group_fixture(tournament, fixture_id);
            }
        });
        result
    }

    /// Sets a group fixture's scores and status.
    ///
    /// Playoff-seeding failures are intentionally silent here: the result is
    /// still saved, while **Bracket** reports why a playoff cannot be generated.
    ///
    /// # Errors
    ///
    /// Returns a not-found message when the fixture no longer exists.
    pub fn set_tournament_group_result(
        &self,
        id: &TournamentId,
        fixture_id: &MatchId,
        score1: u16,
        score2: u16,
        status: BracketMatchStatus,
    ) -> Result<(), String> {
        let mut result = Err("Tournament not found.".to_owned());
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = if core_set_group_result(tournament, fixture_id, score1, score2, status) {
                    Ok(())
                } else {
                    Err("Group fixture not found.".to_owned())
                };
            }
        });
        result
    }

    /// Sets or clears a group fixture's manual winner override.
    pub fn set_tournament_group_winner(
        &self,
        id: &TournamentId,
        fixture_id: &MatchId,
        winner: Option<TeamId>,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_group_winner(tournament, fixture_id, winner);
            }
        });
        result
    }

    /// Changes one group fixture's first-to override.
    pub fn set_tournament_group_first_to(
        &self,
        id: &TournamentId,
        fixture_id: &MatchId,
        first_to: u8,
    ) -> bool {
        let mut result = false;
        self.mutate(|state| {
            if let Some(tournament) = state.tournaments.iter_mut().find(|t| &t.id == id) {
                result = core_set_group_first_to(tournament, fixture_id, first_to);
            }
        });
        result
    }

    /// Renames a user asset. Bundled assets are read-only.
    pub fn rename_asset(&self, id: AssetId, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            self.report_error("Asset names cannot be empty.");
            return;
        }
        let asset = self
            .state
            .peek()
            .assets
            .iter()
            .find(|asset| asset.id == id)
            .cloned();
        match asset {
            None => {
                self.report_error("That asset is bundled and cannot be renamed.");
                return;
            }
            Some(asset) if asset.is_attachment() => {
                self.report_error("Attached images are edited from their parent asset.");
                return;
            }
            Some(_) => {}
        }

        self.mutate(|state| {
            if let Some(asset) = state.assets.iter_mut().find(|asset| asset.id == id) {
                asset.name = trimmed.to_owned();
            }
        });
    }

    /// Uploads a ban icon owned by one hero.
    ///
    /// The file is stored as an attached Hero asset, so it never appears in the
    /// library or in another hero's picker. Any previous attached icon is
    /// removed; a legacy library icon is only unlinked, never deleted.
    pub fn upload_hero_ban_icon(&self, hero_id: AssetId) {
        let Some(hero) = self
            .state
            .peek()
            .assets
            .iter()
            .find(|asset| asset.id == hero_id)
            .cloned()
        else {
            self.report_error("That hero is unavailable.");
            return;
        };
        if hero.is_attachment() || hero.origin == AssetOrigin::Bundled {
            self.report_error("Bundled heroes cannot be edited.");
            return;
        }
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };

        let previous_icon = hero.icon.clone();
        let icon_name = format!("{} ban icon", hero.name);
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

            let read = tokio::task::spawn_blocking(move || read_image_file(&path)).await;
            let (_, bytes, filename) = match read {
                Ok(Ok(value)) => value,
                Ok(Err(error)) => {
                    this.report_error(error);
                    this.set_asset_busy(false);
                    return;
                }
                Err(error) => {
                    this.report_error(format!("The file read task failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            let inserted = tokio::task::spawn_blocking(move || {
                asset_store.insert(icon_name, AssetCategory::Hero, &bytes, filename)
            })
            .await;
            match inserted {
                Ok(Ok(mut outcome)) => {
                    outcome.asset.attachment =
                        Some(AssetAttachment::hero_ban_icon(hero_id.clone()));
                    let new_id = outcome.asset.id.clone();
                    this.mutate(move |state| {
                        if let Some(previous) = previous_icon.as_ref() {
                            remove_attached_asset(
                                state,
                                previous,
                                &hero_id,
                                AssetAttachmentKind::HeroBanIcon,
                            );
                        }
                        if let Some(hero) =
                            state.assets.iter_mut().find(|asset| asset.id == hero_id)
                        {
                            hero.icon = Some(new_id);
                        }
                        state.assets.push(outcome.asset);
                    });
                    this.refresh_unused();
                    this.notify("Ban icon updated for this hero.");
                }
                Ok(Err(error)) => this.report_error(format!("Could not upload the image: {error}")),
                Err(error) => this.report_error(format!("The upload task failed: {error}")),
            }
            this.set_asset_busy(false);
        });
    }

    /// Removes a hero's ban icon and deletes it when it is an attached asset.
    pub fn clear_hero_ban_icon(&self, hero_id: AssetId) {
        let previous_icon = self
            .state
            .peek()
            .assets
            .iter()
            .find(|asset| asset.id == hero_id)
            .and_then(|hero| hero.icon.clone());
        let Some(previous_icon) = previous_icon else {
            return;
        };

        self.mutate(move |state| {
            remove_attached_asset(
                state,
                &previous_icon,
                &hero_id,
                AssetAttachmentKind::HeroBanIcon,
            );
            if let Some(hero) = state.assets.iter_mut().find(|asset| asset.id == hero_id) {
                hero.icon = None;
            }
        });
        self.refresh_unused();
        self.notify("Ban icon removed.");
    }

    /// Sets the game mode on a map asset.
    ///
    /// Passing `None` clears the tag and moves the map back to **Other**. Every
    /// map slot using that map follows the new tag unless its mode was typed by
    /// hand. Saved matches are refreshed too, so a loaded match stays correct.
    pub fn set_asset_game_mode(&self, id: AssetId, game_mode: Option<AssetId>) {
        let map_name = {
            let catalog = self.catalog.peek();
            catalog
                .in_category(AssetCategory::Map)
                .find(|asset| asset.id == id)
                .map(|asset| asset.name.clone())
        };

        self.set_asset_link(id, |asset, value| asset.game_mode = value, game_mode);

        if let Some(map_name) = map_name {
            let catalog = self.catalog.peek().clone();
            self.mutate(|state| {
                if let Some(current) = state.current_match.as_mut() {
                    refresh_slot_modes(&mut current.maps, &catalog, &map_name);
                }
                for saved in &mut state.matches {
                    refresh_slot_modes(&mut saved.maps, &catalog, &map_name);
                }
            });
        }
    }

    /// Applies one optional asset link to a user asset.
    fn set_asset_link(
        &self,
        id: AssetId,
        set: impl FnOnce(&mut CoreAsset, Option<AssetId>),
        value: Option<AssetId>,
    ) {
        let (editable, attached) = {
            let assets = &self.state.peek().assets;
            (
                assets.iter().any(|asset| asset.id == id),
                assets.iter().any(|asset| {
                    asset.is_attachment() && (asset.id == id || value.as_ref() == Some(&asset.id))
                }),
            )
        };
        if !editable {
            self.report_error("Bundled assets cannot be edited.");
            return;
        }
        if attached {
            self.report_error("Attached images are edited from their parent asset.");
            return;
        }
        if value.as_ref() == Some(&id) {
            self.report_error("An asset cannot link to itself.");
            return;
        }

        self.mutate(|state| {
            if let Some(asset) = state.assets.iter_mut().find(|asset| asset.id == id) {
                set(asset, value);
            }
        });
    }

    /// Deletes a user asset and clears every reference to it.
    ///
    /// The blob itself becomes eligible for garbage collection once the
    /// deletion is saved; the asset's previous bytes are never mutated.
    pub fn delete_asset(&self, id: AssetId) {
        let asset = self
            .state
            .peek()
            .assets
            .iter()
            .find(|asset| asset.id == id)
            .cloned();
        let Some(asset) = asset else {
            return;
        };
        if asset.is_attachment() {
            self.report_error("Attached images are deleted with their parent asset.");
            return;
        }
        let attached = attached_asset_ids(&self.state.peek(), &id);

        self.mutate(move |state| {
            state
                .assets
                .retain(|asset| asset.id != id && !attached.contains(&asset.id));
            clear_asset_references(state, &id);
            for child in &attached {
                clear_asset_references(state, child);
            }
        });
        self.refresh_unused();
        self.notify("Asset deleted. Its data is cleaned up after the next save.");
    }

    /// Opens the file picker and imports every chosen image into `category`.
    ///
    /// Reads and hashing stay on blocking tasks. One success toast summarises
    /// the batch; individual failures are reported without stopping the rest.
    pub fn import_assets_from_files(&self, category: AssetCategory) {
        self.import_assets_from_files_then(category, |_| {});
    }

    /// Like [`Self::import_assets_from_files`], but reports the new asset ids
    /// once the batch is stored so a picker can select the first upload.
    pub fn import_assets_from_files_then(
        &self,
        category: AssetCategory,
        on_imported: impl FnMut(Vec<AssetId>) + 'static,
    ) {
        if self.asset_busy() {
            return;
        }
        let Some(asset_store) = self.asset_store.peek().clone() else {
            self.report_error("Asset storage is unavailable.");
            return;
        };

        self.set_asset_busy(true);
        let this = *self;
        let mut on_imported = on_imported;
        spawn(async move {
            let picked = tokio::task::spawn_blocking(crate::assets_io::pick_image_files).await;
            let paths = match picked {
                Ok(paths) => paths,
                Err(error) => {
                    this.report_error(format!("The file picker failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };
            if paths.is_empty() {
                this.set_asset_busy(false);
                return;
            }

            let reads = tokio::task::spawn_blocking(move || {
                paths
                    .into_iter()
                    .map(|path| {
                        let filename = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .map(ToOwned::to_owned);
                        let name = display_name_from_filename(filename.as_deref());
                        match std::fs::read(&path) {
                            Ok(bytes) => Ok((name, bytes, filename)),
                            Err(error) => Err(format!(
                                "Could not read “{}”: {error}",
                                filename.as_deref().unwrap_or("image")
                            )),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .await;

            let reads = match reads {
                Ok(reads) => reads,
                Err(error) => {
                    this.report_error(format!("The file read task failed: {error}"));
                    this.set_asset_busy(false);
                    return;
                }
            };

            let mut imported = Vec::new();
            let mut errors = Vec::new();
            for read in reads {
                let (name, bytes, filename) = match read {
                    Ok(value) => value,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let store = asset_store.clone();
                let inserted = tokio::task::spawn_blocking(move || {
                    store.insert(name, category, &bytes, filename)
                })
                .await;
                match inserted {
                    Ok(Ok(outcome)) => imported.push(outcome.asset),
                    Ok(Err(error)) => errors.push(format!("Could not import the image: {error}")),
                    Err(error) => errors.push(format!("The import task failed: {error}")),
                }
            }

            if !imported.is_empty() {
                let count = imported.len();
                let names = imported
                    .iter()
                    .take(3)
                    .map(|asset| asset.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if count > 3 { "…" } else { "" };
                let ids = imported
                    .iter()
                    .map(|asset| asset.id.clone())
                    .collect::<Vec<_>>();
                this.mutate(move |state| state.assets.extend(imported));
                this.refresh_unused();
                this.notify(format!("Imported {count} image(s): {names}{suffix}"));
                on_imported(ids);
            }
            for error in errors {
                this.report_error(error);
            }
            this.set_asset_busy(false);
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
                    this.allow_cleanup();
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
        let cleanup_allowed = self.cleanup_allowed();

        self.set_asset_busy(true);
        let this = *self;
        spawn(async move {
            let collected = tokio::task::spawn_blocking(move || {
                let Some(keep) = cleanup_keep_hashes(cleanup_allowed, &state, store.as_ref())?
                else {
                    return Ok(None);
                };
                asset_store
                    .collect_garbage_hashes(&keep)
                    .map(Some)
                    .map_err(|error| error.to_string())
            })
            .await;

            match collected {
                Ok(Ok(Some(report))) => this.notify(format!(
                    "Removed {} unused file(s) ({}).",
                    report.files,
                    format_bytes(report.bytes)
                )),
                Ok(Ok(None)) => this.notify(CLEANUP_PAUSED_NOTICE),
                Ok(Err(error)) => this.report_error(error),
                Err(error) => this.report_error(format!("The cleanup task failed: {error}")),
            }
            this.refresh_unused();
            this.set_asset_busy(false);
        });
    }

    /// Recomputes the unused-asset totals using the same keep set as cleanup.
    pub fn refresh_unused(&self) {
        let Some(asset_store) = self.asset_store.peek().clone() else {
            return;
        };
        let state = self.state.peek().clone();
        let store = self.store.peek().clone();
        let cleanup_allowed = self.cleanup_allowed();
        let mut unused = self.unused;
        spawn(async move {
            let totals = tokio::task::spawn_blocking(move || {
                let Some(keep) = cleanup_keep_hashes(cleanup_allowed, &state, store.as_ref())?
                else {
                    return Ok(UnusedAssets::default());
                };
                asset_store
                    .unused_hashes(&keep)
                    .map_err(|error| error.to_string())
            })
            .await;
            match totals {
                Ok(Ok(totals)) => unused.set(totals),
                Ok(Err(error)) => {
                    unused.set(UnusedAssets::default());
                    warn!(%error, "could not count unused assets");
                }
                Err(error) => {
                    unused.set(UnusedAssets::default());
                    warn!(%error, "unused asset count task failed");
                }
            }
        });
    }

    /// Loads an asset's bytes and returns a data URL for previews and thumbnails.
    ///
    /// Results are cached by content hash so repeated thumbnail renders do not
    /// re-read and re-encode the same image.
    pub async fn asset_data_url(self, asset: CoreAsset) -> Option<String> {
        let key = format!("{}:{}", asset.hash, asset.ext);
        if let Some(url) = self.thumb_urls.peek().get(&key).cloned() {
            return Some(url);
        }

        let store = self.asset_store.peek().clone();
        let hash = asset.hash.clone();
        let ext = asset.ext.clone();
        let origin = asset.origin;
        let url = tokio::task::spawn_blocking(move || {
            let bytes = match origin {
                AssetOrigin::Bundled => fgc_server::bundled_asset_bytes_by_hash(&hash, &ext)
                    .map(std::borrow::Cow::into_owned),
                AssetOrigin::User => {
                    let store = store?;
                    store.read_blob(&hash, &ext).ok()
                }
            }?;
            Some(data_url_for(&asset, &bytes))
        })
        .await
        .ok()??;

        let mut cache = self.thumb_urls;
        cache.write().insert(key, url.clone());
        Some(url)
    }

    /// Starts the overlay server with the current settings.
    pub fn start_server(&self) {
        let (config, base_url, outbound) = self.server_plan();
        self.server.set_base_url(base_url.clone());
        self.server.start(config, base_url, outbound);
        self.sync_tailscale_serve(true);
    }

    /// Stops the overlay server and removes this app's Tailscale handler.
    pub fn stop_server(&self) {
        self.server.stop();
        self.sync_tailscale_serve(false);
    }

    /// Restarts the overlay server with the current settings.
    pub fn restart_server(&self) {
        let (config, base_url, outbound) = self.server_plan();
        self.server.set_base_url(base_url.clone());
        self.server.restart(config, base_url, outbound);
        self.sync_tailscale_serve(true);
    }

    fn apply_network_change(&self) {
        self.server.set_base_url(self.current_base_url());

        if self.server.is_running() {
            self.restart_server();
        } else {
            self.sync_tailscale_serve(false);
        }
    }

    /// Installs or removes this app's Serve handlers to match the current
    /// switches. `server_will_run` is the desired listener state; handlers are
    /// only installed when the server is running and Tailscale is enabled.
    fn sync_tailscale_serve(&self, server_will_run: bool) {
        let (tailscale, port) = {
            let state = self.state.peek();
            (
                remote_access::is_tailscale_enabled(&state),
                state.settings.overlay.port,
            )
        };
        if tailscale && server_will_run {
            let mut serve = self.serve;
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    match crate::tailscale::find_executable() {
                        Some(executable) => crate::tailscale::start_serve(&executable, port),
                        None => Err("Tailscale CLI not found".to_owned()),
                    }
                })
                .await;
                match result {
                    Ok(Ok(())) => serve.set(ServeState::Active),
                    Ok(Err(error)) => serve.set(ServeState::Failed(error)),
                    Err(error) => {
                        serve.set(ServeState::Failed(format!("Serve task failed: {error}")))
                    }
                }
            });
        } else {
            self.remove_tailscale_serve();
        }
    }

    /// Removes this app's handler on a blocking task.
    fn remove_tailscale_serve(&self) {
        let mut serve = self.serve;
        spawn(async move {
            let result =
                tokio::task::spawn_blocking(|| match crate::tailscale::find_executable() {
                    Some(executable) => crate::tailscale::stop_serve(&executable),
                    None => Ok(()),
                })
                .await;
            match result {
                Ok(Ok(())) => serve.set(ServeState::Off),
                Ok(Err(error)) => serve.set(ServeState::Failed(error)),
                Err(error) => {
                    serve.set(ServeState::Failed(format!("Teardown task failed: {error}")))
                }
            }
        });
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

        let base_url = self.current_base_url();
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

    fn allow_cleanup(&self) {
        let mut cleanup_allowed = self.cleanup_allowed;
        cleanup_allowed.set(true);
    }

    fn spawn_saver(&self) {
        let mut dirty = self.dirty;
        let mut saving = self.saving;
        let state = self.state;
        let store = self.store;
        let asset_store = self.asset_store;
        let mut unused = self.unused;
        let mut last_error = self.last_error;
        let cleanup_allowed = self.cleanup_allowed;

        use_future(move || async move {
            loop {
                tokio::time::sleep(SAVE_INTERVAL).await;
                if !*dirty.peek() || *saving.peek() {
                    continue;
                }

                let store = store.peek().clone();
                let gc_store = store.clone();
                let snapshot = state.cloned();
                dirty.set(false);
                saving.set(true);

                match persist(store, snapshot).await {
                    Ok(()) => {
                        debug!("state saved");
                        let allowed = *cleanup_allowed.peek();
                        if !allowed {
                            unused.set(UnusedAssets::default());
                            debug!("asset cleanup paused; saved state was not trusted");
                        } else if let Some(asset_store) = asset_store.peek().clone() {
                            let current = state.peek().clone();
                            let collected = tokio::task::spawn_blocking(move || {
                                let Some(keep) =
                                    cleanup_keep_hashes(allowed, &current, gc_store.as_ref())?
                                else {
                                    return Ok(None);
                                };
                                asset_store
                                    .collect_garbage_hashes(&keep)
                                    .map(Some)
                                    .map_err(|error| error.to_string())
                            })
                            .await;
                            match collected {
                                Ok(Ok(Some(report))) => {
                                    if report.files > 0 {
                                        debug!(
                                            files = report.files,
                                            bytes = report.bytes,
                                            "collected unused asset blobs"
                                        );
                                    }
                                    unused.set(UnusedAssets::default());
                                }
                                Ok(Ok(None)) => {
                                    unused.set(UnusedAssets::default());
                                    debug!("asset cleanup paused; saved state was not trusted");
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
        let cleanup_allowed = self.cleanup_allowed;
        use_drop(move || {
            // Released before the runtime shuts down so a Tailscale handler
            // cannot outlive the app. Only this app's `/fgc` and `/` paths are
            // touched.
            crate::tailscale::find_executable()
                .and_then(|executable| crate::tailscale::stop_serve(&executable).err())
                .inspect(|error| warn!(%error, "failed to tear down Tailscale Serve on exit"));
            if *dirty.peek() {
                let store = store.peek().clone();
                let snapshot = state.cloned();
                match store.save(&snapshot) {
                    Ok(()) => {
                        let allowed = *cleanup_allowed.peek();
                        if !allowed {
                            warn!("asset cleanup on exit paused; saved state was not trusted");
                        } else if let Some(asset_store) = asset_store.peek().clone() {
                            match cleanup_keep_hashes(allowed, &snapshot, store.as_ref()) {
                                Ok(Some(keep)) => {
                                    if let Err(error) = asset_store.collect_garbage_hashes(&keep) {
                                        warn!(%error, "asset cleanup on exit failed");
                                    }
                                }
                                Ok(None) => {
                                    warn!(
                                        "asset cleanup on exit paused; saved state was not trusted"
                                    );
                                }
                                Err(error) => {
                                    warn!(%error, "asset cleanup on exit could not read saved state");
                                }
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

/// Reads one picked image into the form used by [`AssetStore::insert`].
fn read_image_file(path: &Path) -> Result<(String, Vec<u8>, Option<String>), String> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    let name = display_name_from_filename(filename.as_deref());
    let bytes = std::fs::read(path).map_err(|source| {
        format!(
            "Could not read “{}”: {source}",
            filename.as_deref().unwrap_or("image")
        )
    })?;
    Ok((name, bytes, filename))
}

/// Removes an attached asset only when it belongs to `owner` with `kind`.
fn remove_attached_asset(
    state: &mut AppState,
    id: &AssetId,
    owner: &AssetId,
    kind: AssetAttachmentKind,
) -> bool {
    let is_attached = state.assets.iter().any(|asset| {
        &asset.id == id
            && asset
                .attachment
                .as_ref()
                .is_some_and(|attachment| &attachment.owner == owner && attachment.kind == kind)
    });
    if !is_attached {
        return false;
    }

    state.assets.retain(|asset| &asset.id != id);
    clear_asset_references(state, id);
    true
}

/// Finds every asset attached to `owner`, including nested attachments.
fn attached_asset_ids(state: &AppState, owner: &AssetId) -> Vec<AssetId> {
    let mut found = Vec::new();
    let mut pending = vec![owner.clone()];
    while let Some(current) = pending.pop() {
        for asset in &state.assets {
            let is_child = asset
                .attachment
                .as_ref()
                .is_some_and(|attachment| attachment.owner == current);
            if is_child && !found.contains(&asset.id) {
                found.push(asset.id.clone());
                pending.push(asset.id.clone());
            }
        }
    }
    found
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

/// Refreshes the derived mode of every slot using `map_name`.
///
/// A slot typed by the owner keeps its own value; all other slots follow the
/// map's current game-mode tag. This is the single refresh path for both the
/// current match and saved matches.
fn refresh_slot_modes(slots: &mut [MapSlot], catalog: &AssetCatalog, map_name: &str) {
    let new_mode = catalog.game_mode_for_map_name(map_name);
    for slot in slots {
        if !slot.game_mode_manual && slot.name.eq_ignore_ascii_case(map_name) {
            slot.game_mode_name = new_mode.clone();
        }
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

/// Builds the keep set for asset cleanup, or returns `None` when cleanup must
/// be paused because the saved state cannot be trusted.
fn cleanup_keep_hashes(
    cleanup_allowed: bool,
    state: &AppState,
    store: &dyn Store,
) -> Result<Option<BTreeSet<String>>, String> {
    if !cleanup_allowed {
        return Ok(None);
    }

    let saved = store
        .load()
        .map_err(|error| format!("Could not load saved state: {error}"))?;
    let mut keep = referenced_hashes(state);
    keep.extend(referenced_hashes(&saved));
    Ok(Some(keep))
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

/// Returns `Some(value)` when the trimmed text is not empty.
fn non_empty(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_owned())
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

/// Deletes a team when nothing references it.
///
/// # Errors
///
/// Returns the reference list when the team is still in use.
#[cfg(test)]
pub fn delete_team_checked(state: &mut AppState, id: &TeamId) -> Result<(), Vec<TeamReference>> {
    let references = fgc_core::team_references(state, id);
    if !references.is_empty() {
        return Err(references);
    }
    state.teams.retain(|team| &team.id != id);
    Ok(())
}

/// Builds a team with the original application defaults.
#[must_use]
pub fn new_team() -> Team {
    Team {
        id: TeamId::new(),
        name: "New Team".to_owned(),
        tag: "TAG".to_owned(),
        record: None,
        color: "#7C3AED".to_owned(),
        text_color: "#FFFFFF".to_owned(),
        aux_color: "#1A1A24".to_owned(),
        logo: None,
        logo_background: false,
        players: Vec::new(),
    }
}

/// Builds a player with the original application defaults.
#[must_use]
pub fn new_player() -> Player {
    Player {
        id: PlayerId::new(),
        name: "New Player".to_owned(),
        role: Role::Damage,
        position: Position::Core,
        portrait: None,
        featured_hero: None,
        pronouns: None,
    }
}

/// Builds a talent member with the original application defaults.
#[must_use]
pub fn new_talent() -> Talent {
    Talent {
        id: TalentId::new(),
        display_name: String::new(),
        real_name: None,
        pronouns: None,
        social: None,
        portrait: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fgc_core::{Asset, CoreError, SocialLink, SocialPlatform};

    struct FailingStore;

    impl Store for FailingStore {
        fn load(&self) -> Result<AppState, CoreError> {
            Err(CoreError::StorePoisoned)
        }

        fn save(&self, _state: &AppState) -> Result<(), CoreError> {
            Ok(())
        }
    }

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
            logo_background: false,
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
        let (loaded, error) = load_state(&FailingStore);
        assert_eq!(loaded, AppState::default());
        assert!(error.is_some());
    }

    #[test]
    fn cleanup_keep_hashes_pauses_when_cleanup_is_not_allowed() {
        let keep = cleanup_keep_hashes(false, &AppState::default(), &FailingStore)
            .expect("paused cleanup should not report an error");

        assert!(keep.is_none());
    }

    #[test]
    fn cleanup_keep_hashes_refuses_unreadable_saved_state() {
        let error = cleanup_keep_hashes(true, &AppState::default(), &FailingStore)
            .expect_err("unreadable saved state should refuse cleanup");

        assert!(error.contains("Could not load saved state"), "{error}");
    }

    #[test]
    fn cleanup_keep_hashes_unions_saved_and_current_references() {
        let mut saved = AppState::default();
        saved.assets.push(
            Asset::new_user("Saved", AssetCategory::Logo, "1".repeat(64), "png", None)
                .expect("saved asset"),
        );
        let store = MemoryStore::new(saved);
        let mut current = AppState::default();
        current.assets.push(
            Asset::new_user("Current", AssetCategory::Logo, "2".repeat(64), "png", None)
                .expect("current asset"),
        );

        let keep = cleanup_keep_hashes(true, &current, &store)
            .expect("saved state should load")
            .expect("cleanup should be allowed");

        assert!(keep.contains(&"1".repeat(64)));
        assert!(keep.contains(&"2".repeat(64)));
    }

    #[test]
    fn changing_a_game_mode_tag_refreshes_derived_map_slots_only() {
        let mut map = Asset::new_user("Ilios", AssetCategory::Map, "1".repeat(64), "png", None)
            .expect("map asset");
        let control = Asset::new_user(
            "Control",
            AssetCategory::GameMode,
            "2".repeat(64),
            "png",
            None,
        )
        .expect("control asset");
        let escort = Asset::new_user(
            "Escort",
            AssetCategory::GameMode,
            "3".repeat(64),
            "png",
            None,
        )
        .expect("escort asset");
        map.game_mode = Some(escort.id.clone());
        let catalog = AssetCatalog::new(vec![map, control, escort]);

        let mut current = new_match();
        {
            let slots = &mut current.maps;
            slots[0].name = "Ilios".to_owned();
            slots[0].game_mode_name = "Control".to_owned();
            slots[1].name = "Ilios".to_owned();
            slots[1].game_mode_name = "Hybrid".to_owned();
            slots[1].game_mode_manual = true;
            slots[2].name = "Nepal".to_owned();
            slots[2].game_mode_name = "Control".to_owned();
        }

        refresh_slot_modes(&mut current.maps, &catalog, "Ilios");

        assert_eq!(current.maps[0].game_mode_name, "Escort");
        assert_eq!(current.maps[1].game_mode_name, "Hybrid");
        assert_eq!(current.maps[2].game_mode_name, "Control");
    }

    #[test]
    fn setting_a_map_game_mode_through_the_store_refreshes_slots() {
        fn scenario() -> Element {
            let mut map = Asset::new_user("Ilios", AssetCategory::Map, "1".repeat(64), "png", None)
                .expect("map asset");
            let control = Asset::new_user(
                "Control",
                AssetCategory::GameMode,
                "2".repeat(64),
                "png",
                None,
            )
            .expect("control asset");
            let escort = Asset::new_user(
                "Escort",
                AssetCategory::GameMode,
                "3".repeat(64),
                "png",
                None,
            )
            .expect("escort asset");
            map.game_mode = Some(control.id.clone());
            let map_id = map.id.clone();
            let escort_id = escort.id.clone();

            let mut initial = AppState::default();
            initial.assets.extend([map, control, escort]);
            let mut current = new_match();
            current.maps[0].name = "Ilios".to_owned();
            current.maps[0].game_mode_name = "Control".to_owned();
            initial.current_match = Some(current);

            let store = AppStore::test_store(initial);
            store.set_asset_game_mode(map_id, Some(escort_id));

            let state = store.state.peek();
            let match_state = state.current_match.as_ref().expect("current match");
            assert_eq!(match_state.maps[0].game_mode_name, "Escort");
            VNode::empty()
        }

        let mut dom = VirtualDom::new(scenario);
        dom.rebuild_in_place();
    }

    #[test]
    fn a_manual_map_mode_survives_tag_changes_until_the_map_name_changes() {
        let mut current = new_match();
        set_map_name(&mut current, 0, "Ilios", "Control");
        assert!(!current.maps[0].game_mode_manual);
        set_map_mode(&mut current, 0, "Custom");
        assert!(current.maps[0].game_mode_manual);
        set_map_name(&mut current, 0, "Ilios", "Escort");
        assert!(!current.maps[0].game_mode_manual);
        assert_eq!(current.maps[0].game_mode_name, "Escort");
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
    fn attached_asset_ids_follows_every_generation() {
        let hero = user_asset("Sojourn");
        let icon = user_asset("Sojourn ban icon");
        let nested = user_asset("Sojourn animation");
        let other = user_asset("Unrelated");

        let mut state = AppState::default();
        let hero_id = hero.id.clone();
        let icon_id = icon.id.clone();
        let nested_id = nested.id.clone();
        state.assets.extend([hero, other]);
        state.assets.push(icon.clone());
        state.assets.push(nested.clone());
        state.assets[2].attachment = Some(AssetAttachment::hero_ban_icon(hero_id.clone()));
        state.assets[3].attachment = Some(AssetAttachment::hero_ban_icon(icon_id.clone()));

        let attached = attached_asset_ids(&state, &hero_id);

        assert_eq!(attached, vec![icon_id, nested_id]);
    }

    #[test]
    fn deleting_a_hero_removes_its_attached_icon() {
        fn scenario() -> Element {
            let hero = user_asset("Sojourn");
            let icon = user_asset("Sojourn ban icon");
            let hero_id = hero.id.clone();
            let icon_id = icon.id.clone();
            let mut initial = AppState::default();
            initial.assets.extend([hero, icon]);
            initial.assets[1].attachment = Some(AssetAttachment::hero_ban_icon(hero_id.clone()));

            let store = AppStore::test_store(initial);
            store.delete_asset(hero_id.clone());

            let assets = &store.state.peek().assets;
            assert!(!assets.iter().any(|asset| asset.id == hero_id));
            assert!(!assets.iter().any(|asset| asset.id == icon_id));
            VNode::empty()
        }

        let mut dom = VirtualDom::new(scenario);
        dom.rebuild_in_place();
    }

    #[test]
    fn remove_attached_asset_ignores_a_different_owner() {
        let mut state = AppState::default();
        let hero = user_asset("Sojourn");
        let other_hero = user_asset("Kiriko");
        let icon = user_asset("Ban icon");
        let hero_id = hero.id.clone();
        let other_id = other_hero.id.clone();
        let icon_id = icon.id.clone();
        state.assets.extend([hero, other_hero, icon]);
        state.assets[2].attachment = Some(AssetAttachment::hero_ban_icon(other_id));

        let removed = remove_attached_asset(
            &mut state,
            &icon_id,
            &hero_id,
            AssetAttachmentKind::HeroBanIcon,
        );

        assert!(!removed);
        assert!(state.assets.iter().any(|asset| asset.id == icon_id));
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

    #[test]
    fn new_records_match_the_original_defaults() {
        let team = new_team();
        assert_eq!(team.name, "New Team");
        assert_eq!(team.tag, "TAG");
        assert_eq!(team.color, "#7C3AED");
        assert_eq!(team.text_color, "#FFFFFF");
        assert_eq!(team.aux_color, "#1A1A24");
        assert!(team.players.is_empty());

        let player = new_player();
        assert_eq!(player.name, "New Player");
        assert_eq!(player.role, Role::Damage);
        assert_eq!(player.position, Position::Core);

        let talent = new_talent();
        assert!(talent.display_name.is_empty());
        assert!(talent.social.is_none());
    }

    #[test]
    fn roster_order_and_enum_values_survive_round_trip() {
        let mut state = AppState::default();
        let mut peaches = new_team();
        peaches.name = "Peaches".to_owned();

        let mut first = new_player();
        first.name = "First".to_owned();
        first.role = Role::Tank;
        first.position = Position::Core;

        let mut second = new_player();
        second.name = "Second".to_owned();
        second.role = Role::Support;
        second.position = Position::Coach;

        peaches.players.push(first);
        peaches.players.push(second);
        state.teams.push(peaches);

        let json = serde_json::to_string(&state).expect("serialise state");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse state");
        assert_eq!(value["teams"][0]["players"][0]["role"], "tank");
        assert_eq!(value["teams"][0]["players"][1]["position"], "coach");

        let loaded: AppState = serde_json::from_str(&json).expect("deserialise state");
        let names: Vec<&str> = loaded.teams[0]
            .players
            .iter()
            .map(|player| player.name.as_str())
            .collect();
        assert_eq!(names, vec!["First", "Second"]);
        assert_eq!(loaded.teams[0].players[0].role, Role::Tank);
        assert_eq!(loaded.teams[0].players[0].position, Position::Core);
        assert_eq!(loaded.teams[0].players[1].role, Role::Support);
        assert_eq!(loaded.teams[0].players[1].position, Position::Coach);
    }

    #[test]
    fn full_team_roster_and_talent_survive_save_and_reload() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("app.json");
        let store = JsonStore::new(path);

        let mut state = AppState::default();
        let mut team = new_team();
        team.name = "Peaches".to_owned();
        team.tag = "PCH".to_owned();
        team.record = Some("12-4".to_owned());

        let mut player = new_player();
        player.name = "Ripe".to_owned();
        player.role = Role::Support;
        player.position = Position::Sub;
        player.pronouns = Some("she/her".to_owned());
        team.players.push(player);

        let mut talent = new_talent();
        talent.display_name = "Caster".to_owned();
        talent.real_name = Some("Real Name".to_owned());
        talent.pronouns = Some("they/them".to_owned());
        talent.social = Some(SocialLink {
            platform: SocialPlatform::YouTube,
            handle: "@caster".to_owned(),
        });

        state.teams.push(team);
        state.talents.push(talent);
        store.save(&state).expect("save state");

        let loaded = store.load().expect("load state");
        let loaded_team = &loaded.teams[0];
        assert_eq!(loaded_team.name, "Peaches");
        assert_eq!(loaded_team.tag, "PCH");
        assert_eq!(loaded_team.record.as_deref(), Some("12-4"));
        assert_eq!(loaded_team.players.len(), 1);
        assert_eq!(loaded_team.players[0].name, "Ripe");
        assert_eq!(loaded_team.players[0].role, Role::Support);
        assert_eq!(loaded_team.players[0].position, Position::Sub);
        assert_eq!(loaded_team.players[0].pronouns.as_deref(), Some("she/her"));

        let loaded_talent = &loaded.talents[0];
        assert_eq!(loaded_talent.display_name, "Caster");
        assert_eq!(loaded_talent.real_name.as_deref(), Some("Real Name"));
        assert_eq!(loaded_talent.pronouns.as_deref(), Some("they/them"));
        let social = loaded_talent.social.as_ref().expect("social link");
        assert_eq!(social.platform, SocialPlatform::YouTube);
        assert_eq!(social.handle, "@caster");
    }

    #[test]
    fn team_deletion_is_blocked_while_referenced_and_cleanup_removes_references() {
        let mut state = AppState::default();
        let team = new_team();
        let id = team.id.clone();
        state.teams.push(team);
        state.tournaments.push(
            serde_json::from_value(serde_json::json!({
                "id": "tournament-1",
                "name": "Test Cup",
                "entrants": [{ "teamId": id.as_str(), "seed": 1 }]
            }))
            .expect("valid tournament"),
        );

        let references = delete_team_checked(&mut state, &id).expect_err("deletion is blocked");
        assert_eq!(references.len(), 1);
        assert!(state.teams.iter().any(|team| team.id == id));

        remove_team_references(&mut state, &id);
        assert!(delete_team_checked(&mut state, &id).is_ok());
        assert!(!state.teams.iter().any(|team| team.id == id));
    }

    #[test]
    fn talent_crud_round_trip() {
        let mut state = AppState::default();
        let mut talent = new_talent();
        talent.display_name = "Caster".to_owned();
        talent.real_name = Some("Real Name".to_owned());
        talent.pronouns = Some("they/them".to_owned());
        talent.social = Some(SocialLink {
            platform: SocialPlatform::Twitch,
            handle: "@caster".to_owned(),
        });
        state.talents.push(talent);

        let json = serde_json::to_string(&state).expect("serialise state");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse state");
        assert_eq!(value["talents"][0]["social"]["platform"], "twitch");
        assert_eq!(value["talents"][0]["social"]["handle"], "@caster");

        let loaded: AppState = serde_json::from_str(&json).expect("deserialise state");
        let loaded_talent = &loaded.talents[0];
        assert_eq!(loaded_talent.display_name, "Caster");
        assert_eq!(loaded_talent.real_name.as_deref(), Some("Real Name"));
        assert_eq!(loaded_talent.pronouns.as_deref(), Some("they/them"));
        let social = loaded_talent.social.as_ref().expect("social link");
        assert_eq!(social.platform, SocialPlatform::Twitch);
        assert_eq!(social.handle, "@caster");
    }

    #[test]
    fn player_updates_apply_only_to_the_selected_team() {
        let mut state = AppState::default();
        let mut peaches = new_team();
        let player = new_player();
        let player_id = player.id.clone();
        peaches.players.push(player);
        let peaches_id = peaches.id.clone();
        let mangoes_id = new_team().id;
        state.teams.push(peaches);
        state.teams.push(new_team());

        let team = state
            .teams
            .iter_mut()
            .find(|team| team.id == peaches_id)
            .expect("selected team");
        let player = team
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
            .expect("selected player");
        player.name = "Updated".to_owned();
        player.role = Role::Support;

        assert_eq!(state.teams[0].players[0].name, "Updated");
        assert_eq!(state.teams[0].players[0].role, Role::Support);
        assert!(state.teams[1].players.is_empty());
        assert_ne!(peaches_id, mangoes_id);
    }
}
