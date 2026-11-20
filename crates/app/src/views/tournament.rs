//! Tournament destination: setup, generated bracket board, groups and settings.
//!
//! Generation is previewed before it is committed. Once a tournament has a
//! group stage or bracket, entrants and fixtures are fixed: results and match
//! details remain editable, but structural additions happen through a fresh
//! preview and generation.

use std::collections::{HashMap, HashSet};

use dioxus::html::InteractionLocation;
use dioxus::prelude::*;
use fgc_core::{
    BracketFormat, BracketMatch, BracketMatchStatus, BracketSection, BracketSlot, Group, GroupId,
    GroupMatch, LinkKind, MatchId, MatchLink, Round, Standing, Team, TeamId, Tiebreaker,
    Tournament, TournamentId, TournamentStage, TournamentStatus, entrants_in_seed_order,
    find_match, generate_double_elimination as core_generate_double_elimination,
    generate_elimination_from_teams as core_generate_elimination_from_teams,
    generate_multistage as core_generate_multistage, generate_playoff as core_generate_playoff,
    generate_round_robin as core_generate_round_robin,
    generate_single_elimination as core_generate_single_elimination, group_stage_complete,
    match_address, project_playoff as core_project_playoff, refresh as core_refresh_tournament,
    set_entrants as core_set_entrants, swap_slots as core_swap_slots,
    sync_all_group_fixtures as core_sync_all_group_fixtures,
};

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::focus_element_later;
use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::components::{Field, Panel, ToggleRow};
use crate::state::AppStore;

// ---------------------------------------------------------------------------
// Local view state
// ---------------------------------------------------------------------------

/// The four sections of a tournament.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TournamentTab {
    /// Entrants, format and generation.
    Setup,
    /// Editable bracket board.
    Board,
    /// Groups, fixtures and standings.
    Groups,
    /// Tiebreakers, playoff and champion.
    Settings,
}

impl TournamentTab {
    /// Tabs shown for a bracket format. A single stage shows either the board
    /// or the groups; a multi-stage tournament shows both.
    #[must_use]
    fn for_format(format: BracketFormat) -> Vec<Self> {
        match format {
            BracketFormat::RoundRobin => vec![Self::Setup, Self::Groups, Self::Settings],
            BracketFormat::SingleElim | BracketFormat::DoubleElim => {
                vec![Self::Setup, Self::Board, Self::Settings]
            }
            BracketFormat::MultiStage => {
                vec![Self::Setup, Self::Groups, Self::Board, Self::Settings]
            }
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Setup => "Setup",
            Self::Board => "Bracket",
            Self::Groups => "Groups",
            Self::Settings => "Settings",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Board => "board",
            Self::Groups => "groups",
            Self::Settings => "settings",
        }
    }
}

/// Ephemeral board zoom state. Participant placement is fixed at generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BoardState {
    /// Board zoom percentage.
    zoom: u16,
}

impl Default for BoardState {
    fn default() -> Self {
        Self { zoom: 100 }
    }
}

/// A generation action that must be previewed before it is committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationKind {
    /// Split entrants into round-robin groups.
    RoundRobin(usize),
    /// Generate groups and the later playoff.
    MultiStage(usize),
    /// Generate the bracket for the tournament's current format.
    Bracket,
    /// Seed the playoff from completed groups.
    Playoff,
}

impl GenerationKind {
    fn label(self) -> &'static str {
        match self {
            Self::RoundRobin(_) => "round robin",
            Self::MultiStage(_) => "multi-stage tournament",
            Self::Bracket => "bracket",
            Self::Playoff => "playoff",
        }
    }

    /// Group-stage generations preview the split only. Bracket generations
    /// preview placements only. Showing both at once is misleading: the
    /// playoff does not exist before the group stage finishes.
    const fn shows_groups(self) -> bool {
        matches!(self, Self::RoundRobin(_) | Self::MultiStage(_))
    }

    const fn shows_bracket(self) -> bool {
        matches!(self, Self::Bracket | Self::Playoff)
    }

    fn hint(self) -> &'static str {
        if self.shows_groups() {
            "Drag teams between groups in the preview only. Nothing is saved until Generate."
        } else {
            "Edit seeds or drag participants in the preview only. Nothing is saved until Generate."
        }
    }
}

/// A preview request and the control that opened it.
#[derive(Clone)]
struct GenerationRequest {
    kind: GenerationKind,
    trigger_id: String,
}

/// An uncommitted generation result with preview-local drag edits.
#[derive(Clone, PartialEq)]
struct GenerationPreview {
    kind: GenerationKind,
    target: Tournament,
    bracket_preview: Tournament,
    trigger_id: String,
    projected: bool,
}

/// Fixture awaiting confirmation before it replaces the current match.
#[derive(Clone)]
struct PushTarget {
    title: String,
    subtitle: String,
    team_a: Option<TeamId>,
    team_b: Option<TeamId>,
    score1: u16,
    score2: u16,
    status: BracketMatchStatus,
    first_to: u8,
    trigger_id: String,
}

/// A confirmed stage reset, including the control that opened the prompt.
#[derive(Clone)]
struct StageResetTarget {
    tournament_id: TournamentId,
    stage: TournamentStage,
    trigger_id: String,
}

// ---------------------------------------------------------------------------
// Top-level view
// ---------------------------------------------------------------------------

/// Tournament list, setup, board, groups and settings.
#[component]
pub fn TournamentView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let mut selected_id = use_signal(|| None::<TournamentId>);
    let mut tab = use_signal(|| TournamentTab::Setup);
    let mut board = use_signal(BoardState::default);
    let mut match_dialog = use_signal(|| None::<MatchId>);
    let mut group_match_dialog = use_signal(|| None::<MatchId>);
    let mut delete_target = use_signal(|| None::<TournamentId>);
    let mut push_target = use_signal(|| None::<PushTarget>);
    let mut generation_error = use_signal(String::new);
    let mut preview = use_signal(|| None::<GenerationPreview>);
    let mut stage_reset_target = use_signal(|| None::<StageResetTarget>);
    let mut tournaments_open = use_signal(|| true);

    let tournaments = state.read().tournaments.clone();
    let teams = state.read().teams.clone();
    let selected = selected_id().and_then(|id| {
        tournaments
            .iter()
            .find(|tournament| tournament.id == id)
            .cloned()
    });

    let on_preview = EventHandler::new(move |request: GenerationRequest| {
        let source = state
            .read()
            .tournaments
            .iter()
            .find(|tournament| selected_id().as_ref() == Some(&tournament.id))
            .cloned();
        let Some(source) = source else {
            generation_error.set("Tournament not found.".to_owned());
            return;
        };
        match build_generation_preview(&source, request.kind, request.trigger_id.clone()) {
            Ok(next) => {
                generation_error.set(String::new());
                preview.set(Some(next));
            }
            Err(error) => {
                preview.set(None);
                generation_error.set(error);
                focus_element_later(&request.trigger_id);
            }
        }
    });

    let on_commit_preview = EventHandler::new(move |tournament: Tournament| {
        let trigger_id = preview()
            .map(|preview| preview.trigger_id.clone())
            .unwrap_or_default();
        store.commit_tournament(tournament);
        preview.set(None);
        generation_error.set(String::new());
        if !trigger_id.is_empty() {
            focus_element_later(&trigger_id);
        }
    });

    let on_cancel_preview = EventHandler::new(move |_| {
        let trigger_id = preview()
            .map(|preview| preview.trigger_id.clone())
            .unwrap_or_default();
        preview.set(None);
        if !trigger_id.is_empty() {
            focus_element_later(&trigger_id);
        }
    });

    let on_reset_stage = EventHandler::new(move |target: StageResetTarget| {
        stage_reset_target.set(Some(target));
    });

    let preview_key = preview()
        .map(|preview| format!("{}-{:?}", preview.target.id, preview.kind))
        .unwrap_or_default();

    rsx! {
        div { class: "fgc-view",
            div { class: if tournaments_open() { "fgc-tournaments" } else { "fgc-tournaments fgc-tournaments--collapsed" },
                Panel {
                    title: format!("Tournaments ({})", tournaments.len()),
                    div { class: "fgc-tournaments__header",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let id = store.create_tournament("New Tournament", BracketFormat::SingleElim);
                                selected_id.set(Some(id));
                                tab.set(TournamentTab::Setup);
                                board.set(BoardState::default());
                                match_dialog.set(None);
                                group_match_dialog.set(None);
                                push_target.set(None);
                                generation_error.set(String::new());
                                preview.set(None);
                            },
                            "+ New tournament"
                        }
                        Button {
                            id: "fgc-tournaments-toggle",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: if tournaments_open() { "Hide tournament list" } else { "Show tournament list" },
                            aria_expanded: tournaments_open(),
                            aria_controls: "fgc-tournaments-list",
                            onclick: move |_| {
                                let next = !tournaments_open();
                                tournaments_open.set(next);
                                if !next {
                                    focus_element_later("fgc-tournaments-toggle");
                                }
                            },
                            if tournaments_open() { "▴" } else { "▾" }
                        }
                    }
                    div { id: "fgc-tournaments-list", class: "fgc-tournaments__region", hidden: !tournaments_open(),
                        if tournaments.is_empty() {
                            div { class: "fgc-asset-empty",
                                p { class: "fgc-asset-empty__title", "No tournaments yet." }
                                p { class: "fgc-note", "Create a tournament to begin." }
                            }
                        } else {
                            div { class: "fgc-tournament-list",
                                for tournament in tournaments.clone() {
                                    {
                                        let item_id = tournament.id.clone();
                                        let is_selected = selected_id().as_ref() == Some(&item_id);
                                        rsx! {
                                            button {
                                                key: "{tournament.id}",
                                                id: "fgc-tournament-item-{tournament.id}",
                                                r#type: "button",
                                                aria_pressed: is_selected,
                                                class: if is_selected {
                                                    "fgc-tournament-list__item fgc-tournament-list__item--active"
                                                } else {
                                                    "fgc-tournament-list__item"
                                                },
                                                onclick: move |_| {
                                                    selected_id.set(Some(item_id.clone()));
                                                    tab.set(TournamentTab::Setup);
                                                    board.set(BoardState::default());
                                                    match_dialog.set(None);
                                                    group_match_dialog.set(None);
                                                    push_target.set(None);
                                                    generation_error.set(String::new());
                                                    preview.set(None);
                                                },
                                                span { class: "fgc-tournament-list__name", "{tournament.name}" }
                                                span { class: "fgc-tournament-list__meta",
                                                    "{format_label(tournament.format)} · {tournament.entrants.len()} teams"
                                                }
                                                span {
                                                    class: "fgc-tournament-list__status fgc-tournament-list__status--{status_key(tournament.status)}",
                                                    "{status_label(tournament.status)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(tournament) = selected.clone() {
                    TournamentDetail {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                        tab: tab(),
                        on_tab: move |next| tab.set(next),
                        board: board(),
                        on_board: move |next| board.set(next),
                        on_open_match: move |id| match_dialog.set(Some(id)),
                        on_open_group_match: move |id| group_match_dialog.set(Some(id)),
                        on_delete: move |id| delete_target.set(Some(id)),
                        on_push: move |target| push_target.set(Some(target)),
                        on_reset_stage,
                        on_preview,
                        generation_error,
                    }
                } else {
                    Panel {
                        title: "No tournament selected".to_owned(),
                        div { class: "fgc-empty", "Select or create a tournament." }
                    }
                }
            }
        }

        if let (Some(match_id), Some(tournament)) = (match_dialog(), selected.clone()) {
            MatchDialog {
                key: "{match_id}",
                tournament: tournament.clone(),
                match_id: match_id.clone(),
                teams: teams.clone(),
                on_close: {
                    let closed_id = match_id.clone();
                    move |_| {
                        match_dialog.set(None);
                        focus_element_later(&format!("fgc-edit-match-{closed_id}"));
                    }
                },
            }
        }

        if let (Some(fixture_id), Some(tournament)) = (group_match_dialog(), selected.clone()) {
            GroupFixtureDialog {
                key: "{fixture_id}",
                tournament: tournament.clone(),
                fixture_id: fixture_id.clone(),
                teams: teams.clone(),
                on_close: {
                    let closed_id = fixture_id.clone();
                    move |_| {
                        group_match_dialog.set(None);
                        focus_element_later(&format!("fgc-edit-group-fixture-{closed_id}"));
                    }
                },
            }
        }

        Dialog {
            open: delete_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { delete_target.set(None) },
            if let Some(id) = delete_target() {
                if let Some(tournament) = tournaments.iter().find(|item| item.id == id) {
                    DialogTitle { "Delete {tournament.name}?" }
                    DialogDescription {
                        "The tournament, its bracket, groups and results are removed. This cannot be undone."
                    }
                    div { class: "fgc-dialog-actions",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| delete_target.set(None),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            onclick: move |_| {
                                let Some(id) = delete_target() else {
                                    return;
                                };
                                store.delete_tournament(&id);
                                selected_id.set(None);
                                match_dialog.set(None);
                                group_match_dialog.set(None);
                                delete_target.set(None);
                            },
                            "Delete tournament"
                        }
                    }
                }
            }
        }

        Dialog {
            open: stage_reset_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { stage_reset_target.set(None) },
            if let Some(target) = stage_reset_target() {
                {
                    let title = match target.stage {
                        TournamentStage::Groups => "Reset group stage?",
                        TournamentStage::Bracket => "Reset bracket?",
                    };
                    let description = match target.stage {
                        TournamentStage::Groups => {
                            "This clears the groups, fixtures, standings and any playoff seeded from them. Entrants stay unchanged."
                        }
                        TournamentStage::Bracket => {
                            "This clears every bracket round, result and the champion. Entrants stay unchanged."
                        }
                    };
                    let confirm_label = match target.stage {
                        TournamentStage::Groups => "Reset group stage",
                        TournamentStage::Bracket => "Reset bracket",
                    };
                    let cancel_trigger = target.trigger_id.clone();
                    let confirm = target.clone();
                    rsx! {
                        DialogTitle { "{title}" }
                        DialogDescription { "{description}" }
                        div { class: "fgc-dialog-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                id: "fgc-stage-reset-cancel",
                                onclick: move |_| {
                                    stage_reset_target.set(None);
                                    focus_element_later(&cancel_trigger);
                                },
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                id: "fgc-stage-reset-confirm",
                                onclick: move |_| {
                                    store.reset_tournament_stage(
                                        &confirm.tournament_id,
                                        confirm.stage,
                                    );
                                    stage_reset_target.set(None);
                                    focus_element_later(&confirm.trigger_id);
                                },
                                "{confirm_label}"
                            }
                        }
                    }
                }
            }
        }

        Dialog {
            open: push_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { push_target.set(None) },
            if let Some(target) = push_target() {
                {
                    let cancel_trigger = target.trigger_id.clone();
                    let confirm = target.clone();
                    let confirm_trigger = target.trigger_id;
                    rsx! {
                        DialogTitle { "Replace the current Match Controller match with this fixture?" }
                        DialogDescription {
                            "This loads the fixture into Match Controller, replaces the current match and refreshes the overlays. Tournament data is unchanged."
                        }
                        div { class: "fgc-dialog-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                id: "fgc-push-cancel",
                                onclick: move |_| {
                                    push_target.set(None);
                                    focus_element_later(&cancel_trigger);
                                },
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                id: "fgc-push-confirm",
                                onclick: move |_| {
                                    store.push_tournament_match_to_overlay(
                                        confirm.title.clone(),
                                        confirm.subtitle.clone(),
                                        confirm.team_a.clone(),
                                        confirm.team_b.clone(),
                                        confirm.score1,
                                        confirm.score2,
                                        confirm.status,
                                        confirm.first_to,
                                    );
                                    push_target.set(None);
                                    focus_element_later(&confirm_trigger);
                                },
                                "Push to match"
                            }
                        }
                    }
                }
            }
        }

        if let Some(preview_state) = preview() {
            GenerationPreviewDialog {
                key: "{preview_key}",
                preview: preview_state,
                teams: teams.clone(),
                on_cancel: on_cancel_preview,
                on_commit: on_commit_preview,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generation preview
// ---------------------------------------------------------------------------

/// Whether structural additions are locked for this tournament.
fn tournament_is_generated(tournament: &Tournament) -> bool {
    let groups_started = tournament
        .groups
        .iter()
        .any(|group| !group.team_ids.is_empty());
    groups_started
        || !tournament.group_matches.is_empty()
        || !tournament.winners.is_empty()
        || !tournament.losers.is_empty()
        || !tournament.grand_final.is_empty()
}

fn has_bracket_rounds(tournament: &Tournament) -> bool {
    !tournament.winners.is_empty()
        || !tournament.losers.is_empty()
        || !tournament.grand_final.is_empty()
}

fn stage_reset_target(
    tournament: &Tournament,
    stage: TournamentStage,
    trigger_id: &str,
) -> StageResetTarget {
    StageResetTarget {
        tournament_id: tournament.id.clone(),
        stage,
        trigger_id: trigger_id.to_owned(),
    }
}

/// Visual meaning of a compact stage action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StageActionTone {
    Preview,
    Regenerate,
    Reset,
}

impl StageActionTone {
    const fn class(self) -> &'static str {
        match self {
            Self::Preview => "fgc-stage-action--preview",
            Self::Regenerate => "fgc-stage-action--regenerate",
            Self::Reset => "fgc-stage-action--reset",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Self::Preview => "▷",
            Self::Regenerate => "↻",
            Self::Reset => "⌫",
        }
    }
}

/// Icon-and-label action used in a stage heading's horizontal button row.
#[component]
fn StageAction(
    id: String,
    label: String,
    tone: StageActionTone,
    on_action: EventHandler<()>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            size: ButtonSize::Sm,
            id: id.clone(),
            class: "fgc-stage-action {tone.class()}",
            aria_label: label.clone(),
            title: label.clone(),
            onclick: move |_| on_action.call(()),
            span { class: "fgc-stage-action__icon", aria_hidden: "true", "{tone.icon()}" }
            span { "{label}" }
        }
    }
}

fn generate_bracket(tournament: &mut Tournament) -> Result<(), String> {
    match tournament.format {
        BracketFormat::SingleElim => core_generate_single_elimination(tournament),
        BracketFormat::DoubleElim => core_generate_double_elimination(tournament),
        BracketFormat::RoundRobin => core_generate_round_robin(tournament, 1),
        BracketFormat::MultiStage => {
            core_generate_multistage(tournament, 2).map_err(|error| error.to_string())?
        }
    }
    Ok(())
}

fn bracket_preview_for(target: &Tournament) -> (Tournament, bool) {
    if has_bracket_rounds(target) {
        return (target.clone(), false);
    }
    if target.format == BracketFormat::MultiStage {
        let mut projected = target.clone();
        if core_project_playoff(&mut projected).is_ok() && has_bracket_rounds(&projected) {
            return (projected, true);
        }
    }
    (target.clone(), false)
}

/// Swaps two preview slots without letting a provisional multi-stage playoff
/// be cleared by the normal refresh rules.
fn swap_preview_slots(
    tournament: &mut Tournament,
    left: BracketSlot,
    right: BracketSlot,
) -> Result<(), String> {
    let format = tournament.format;
    if format == BracketFormat::MultiStage {
        tournament.format = BracketFormat::SingleElim;
    }
    let result = core_swap_slots(tournament, left, right);
    tournament.format = format;
    result.map_err(|error| error.to_string())
}

fn build_generation_preview(
    source: &Tournament,
    kind: GenerationKind,
    trigger_id: String,
) -> Result<GenerationPreview, String> {
    let mut target = source.clone();
    match kind {
        GenerationKind::RoundRobin(group_count) => {
            core_generate_round_robin(&mut target, group_count);
        }
        GenerationKind::MultiStage(group_count) => {
            core_generate_multistage(&mut target, group_count)
                .map_err(|error| error.to_string())?;
        }
        GenerationKind::Bracket => generate_bracket(&mut target)?,
        GenerationKind::Playoff => {
            core_generate_playoff(&mut target).map_err(|error| error.to_string())?;
        }
    }
    core_refresh_tournament(&mut target);
    let (bracket_preview, projected) = if kind.shows_bracket() {
        bracket_preview_for(&target)
    } else {
        (target.clone(), false)
    };
    Ok(GenerationPreview {
        kind,
        target,
        bracket_preview,
        trigger_id,
        projected,
    })
}

/// Rebuilds an elimination tournament from an explicit seed order.
fn regenerate_elimination_preview(target: &mut Tournament, ordered: &[TeamId]) {
    let format = target.format;
    core_set_entrants(target, ordered.iter().cloned());
    core_generate_elimination_from_teams(target, format, ordered);
    core_refresh_tournament(target);
}

/// Rebuilds an elimination preview from an explicit seed order.
fn apply_seed_preview(mut current: Signal<GenerationPreview>, ordered: Vec<TeamId>) {
    let previous = current();
    let mut target = previous.target.clone();
    regenerate_elimination_preview(&mut target, &ordered);
    let bracket_preview = target.clone();
    current.set(GenerationPreview {
        target,
        bracket_preview,
        projected: false,
        ..previous
    });
}

/// Returns the seed order after moving `dragged` before `before` or to `seed`.
fn reordered_seed_ids(
    order: &[TeamId],
    dragged: &TeamId,
    before: Option<&TeamId>,
    seed: Option<u32>,
) -> Vec<TeamId> {
    let mut next: Vec<TeamId> = order
        .iter()
        .filter(|team| *team != dragged)
        .cloned()
        .collect();
    let index = if let Some(seed) = seed {
        seed.saturating_sub(1) as usize
    } else {
        before
            .and_then(|team| next.iter().position(|existing| existing == team))
            .unwrap_or(next.len())
    };
    next.insert(index.min(next.len()), dragged.clone());
    next
}

/// Clears every transient signal that belongs to a preview drag.
#[allow(clippy::too_many_arguments)]
fn clear_preview_drag(
    mut group_drag: Signal<Option<(GroupId, TeamId)>>,
    mut slot_drag: Signal<Option<BracketSlot>>,
    mut seed_drag: Signal<Option<TeamId>>,
    mut group_drop: Signal<Option<(GroupId, Option<TeamId>)>>,
    mut slot_drop: Signal<Option<BracketSlot>>,
    mut seed_drop: Signal<Option<TeamId>>,
    mut drag_pointer: Signal<Option<(f64, f64)>>,
    mut drag_label: Signal<Option<String>>,
) {
    group_drag.set(None);
    slot_drag.set(None);
    seed_drag.set(None);
    group_drop.set(None);
    slot_drop.set(None);
    seed_drop.set(None);
    drag_pointer.set(None);
    drag_label.set(None);
}

/// Preview pop-up shown before any generation is saved.
#[component]
fn GenerationPreviewDialog(
    preview: GenerationPreview,
    teams: Vec<Team>,
    on_cancel: EventHandler<()>,
    on_commit: EventHandler<Tournament>,
) -> Element {
    let mut current = use_signal(|| preview);
    let mut group_drag = use_signal(|| None::<(GroupId, TeamId)>);
    let mut slot_drag = use_signal(|| None::<BracketSlot>);
    let mut seed_drag = use_signal(|| None::<TeamId>);
    let mut group_drop = use_signal(|| None::<(GroupId, Option<TeamId>)>);
    let slot_drop = use_signal(|| None::<BracketSlot>);
    let mut seed_drop = use_signal(|| None::<TeamId>);
    let mut drag_pointer = use_signal(|| None::<(f64, f64)>);
    let mut drag_label = use_signal(|| None::<String>);
    let state = current();
    let team_names: HashMap<TeamId, String> = teams
        .iter()
        .map(|team| (team.id.clone(), team.name.clone()))
        .collect();
    let team_name = |id: &TeamId| {
        team_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string())
    };
    let seeds: HashMap<TeamId, u32> = state
        .target
        .entrants
        .iter()
        .map(|entrant| (entrant.team_id.clone(), entrant.seed))
        .collect();
    let seed_order = entrants_in_seed_order(&state.target);
    let editable_seeds = matches!(state.kind, GenerationKind::Bracket)
        && matches!(
            state.target.format,
            BracketFormat::SingleElim | BracketFormat::DoubleElim
        );
    let commit_target = state.target.clone();
    let preview_hint = state.kind.hint();

    let apply_group_move =
        move |from_group: GroupId, team: TeamId, to_group: GroupId, before: Option<TeamId>| {
            let previous = current();
            let mut target = previous.target.clone();
            let mut moved = false;
            for group in target.groups.iter_mut() {
                if group.id == from_group {
                    let before_len = group.team_ids.len();
                    group.team_ids.retain(|id| id != &team);
                    moved = group.team_ids.len() < before_len;
                }
            }
            if !moved {
                return;
            }
            if let Some(destination) = target.groups.iter_mut().find(|g| g.id == to_group) {
                let index = before.and_then(|id| {
                    destination
                        .team_ids
                        .iter()
                        .position(|existing| existing == &id)
                });
                match index {
                    Some(index) => destination.team_ids.insert(index, team.clone()),
                    None => destination.team_ids.push(team.clone()),
                }
            }
            core_sync_all_group_fixtures(&mut target);
            core_refresh_tournament(&mut target);
            let (bracket_preview, projected) = if previous.kind.shows_bracket() {
                bracket_preview_for(&target)
            } else {
                (target.clone(), false)
            };
            current.set(GenerationPreview {
                target,
                bracket_preview,
                projected,
                ..previous
            });
        };

    let on_group_down =
        EventHandler::new(move |(group, team, label): (GroupId, TeamId, String)| {
            group_drag.set(Some((group, team)));
            drag_label.set(Some(format!("⠿ {label}")));
        });
    let on_group_up = {
        let mut apply = apply_group_move;
        EventHandler::new(move |(group, team): (GroupId, TeamId)| {
            let Some((from, dragged)) = group_drag() else {
                return;
            };
            if from == group || dragged == team {
                clear_preview_drag(
                    group_drag,
                    slot_drag,
                    seed_drag,
                    group_drop,
                    slot_drop,
                    seed_drop,
                    drag_pointer,
                    drag_label,
                );
                return;
            }
            apply(from, dragged, group, Some(team));
            clear_preview_drag(
                group_drag,
                slot_drag,
                seed_drag,
                group_drop,
                slot_drop,
                seed_drop,
                drag_pointer,
                drag_label,
            );
        })
    };
    let on_group_area_up = {
        let mut apply = apply_group_move;
        EventHandler::new(move |group: GroupId| {
            let Some((from, dragged)) = group_drag() else {
                return;
            };
            if from == group {
                clear_preview_drag(
                    group_drag,
                    slot_drag,
                    seed_drag,
                    group_drop,
                    slot_drop,
                    seed_drop,
                    drag_pointer,
                    drag_label,
                );
                return;
            }
            apply(from, dragged, group, None);
            clear_preview_drag(
                group_drag,
                slot_drag,
                seed_drag,
                group_drop,
                slot_drop,
                seed_drop,
                drag_pointer,
                drag_label,
            );
        })
    };

    let on_slot_down = EventHandler::new(move |(address, label): (BracketSlot, String)| {
        slot_drag.set(Some(address));
        drag_label.set(Some(format!("⠿ {label}")));
    });
    let on_slot_up = EventHandler::new(move |target_address: BracketSlot| {
        let Some(source) = slot_drag() else {
            return;
        };
        if source == target_address
            || source.section != target_address.section
            || source.round != target_address.round
        {
            clear_preview_drag(
                group_drag,
                slot_drag,
                seed_drag,
                group_drop,
                slot_drop,
                seed_drop,
                drag_pointer,
                drag_label,
            );
            return;
        }
        let previous = current();
        let mut target = previous.target.clone();
        if has_bracket_rounds(&target) {
            let _ = swap_preview_slots(&mut target, source, target_address);
        }
        let mut bracket_preview = previous.bracket_preview.clone();
        let _ = swap_preview_slots(&mut bracket_preview, source, target_address);
        current.set(GenerationPreview {
            target,
            bracket_preview,
            ..previous
        });
        clear_preview_drag(
            group_drag,
            slot_drag,
            seed_drag,
            group_drop,
            slot_drop,
            seed_drop,
            drag_pointer,
            drag_label,
        );
    });

    let on_seed_down = EventHandler::new(move |(team, label): (TeamId, String)| {
        seed_drag.set(Some(team));
        drag_label.set(Some(format!("⠿ {label}")));
    });
    let on_seed_up = EventHandler::new(move |target_team: TeamId| {
        let Some(dragged) = seed_drag() else {
            return;
        };
        if dragged == target_team {
            clear_preview_drag(
                group_drag,
                slot_drag,
                seed_drag,
                group_drop,
                slot_drop,
                seed_drop,
                drag_pointer,
                drag_label,
            );
            return;
        }
        let previous = current();
        let order: Vec<TeamId> = entrants_in_seed_order(&previous.target)
            .into_iter()
            .map(|entrant| entrant.team_id)
            .collect();
        apply_seed_preview(
            current,
            reordered_seed_ids(&order, &dragged, Some(&target_team), None),
        );
        clear_preview_drag(
            group_drag,
            slot_drag,
            seed_drag,
            group_drop,
            slot_drop,
            seed_drop,
            drag_pointer,
            drag_label,
        );
    });

    rsx! {
        Dialog {
            class: "fgc-generation-preview-dialog",
            open: true,
            on_open_change: move |is_open: bool| if !is_open { on_cancel.call(()) },
            div {
                class: "fgc-preview-drag-surface",
                onpointermove: move |event: PointerEvent| {
                    let point = event.client_coordinates();
                    drag_pointer.set(Some((point.x, point.y)));
                },
                onpointerup: move |_| {
                    clear_preview_drag(
                        group_drag,
                        slot_drag,
                        seed_drag,
                        group_drop,
                        slot_drop,
                        seed_drop,
                        drag_pointer,
                        drag_label,
                    )
                },
                onpointercancel: move |_| {
                    clear_preview_drag(
                        group_drag,
                        slot_drag,
                        seed_drag,
                        group_drop,
                        slot_drop,
                        seed_drop,
                        drag_pointer,
                        drag_label,
                    )
                },
                DialogTitle { "Preview {state.kind.label()} generation" }
                DialogDescription { "{preview_hint}" }
                div { class: if state.kind.shows_bracket() {
                "fgc-generation-preview fgc-generation-preview--bracket"
            } else {
                "fgc-generation-preview"
            },
                section { class: "fgc-generation-preview__column",
                    h3 { "Seeding" }
                    if editable_seeds {
                        p { class: "fgc-note", "Change a seed or drag a participant; placements update immediately." }
                    }
                    ol { class: if editable_seeds {
                        "fgc-generation-preview__seeds fgc-generation-preview__seeds--editable"
                    } else {
                        "fgc-generation-preview__seeds"
                    },
                        for entrant in seed_order.iter() {
                            {
                                let name = team_name(&entrant.team_id);
                                let seed_class = if seed_drag().as_ref() == Some(&entrant.team_id) {
                                    "fgc-generation-preview__seed-row fgc-generation-preview__seed-row--dragging"
                                } else if seed_drop().as_ref() == Some(&entrant.team_id) {
                                    "fgc-generation-preview__seed-row fgc-generation-preview__seed-row--drop-before"
                                } else {
                                    "fgc-generation-preview__seed-row"
                                };
                                let team_id = entrant.team_id.clone();
                                let seed = entrant.seed;
                                rsx! {
                                    li {
                                        key: "{team_id}",
                                        class: seed_class,
                                        onpointerenter: {
                                            let hovered = team_id.clone();
                                            move |_| seed_drop.set(Some(hovered.clone()))
                                        },
                                        onpointerleave: {
                                            let hovered = team_id.clone();
                                            move |_| {
                                                if seed_drop().as_ref() == Some(&hovered) {
                                                    seed_drop.set(None);
                                                }
                                            }
                                        },
                                        onpointerup: {
                                            let dropped = team_id.clone();
                                            move |event: PointerEvent| {
                                                event.stop_propagation();
                                                on_seed_up.call(dropped.clone());
                                            }
                                        },
                                        if editable_seeds {
                                            button {
                                                r#type: "button",
                                                class: "fgc-generation-preview__seed-handle",
                                                aria_label: "Drag {name} to change seed",
                                                title: "Drag to reorder seeds",
                                                onpointerdown: {
                                                    let dragged = team_id.clone();
                                                    let drag_name = name.clone();
                                                    move |event: PointerEvent| {
                                                        event.prevent_default();
                                                        on_seed_down.call((dragged.clone(), drag_name.clone()));
                                                    }
                                                },
                                                "⠿"
                                            }
                                            input {
                                                r#type: "number",
                                                min: "1",
                                                max: "{seed_order.len()}",
                                                class: "fgc-generation-preview__seed-input",
                                                id: "fgc-preview-seed-{team_id}",
                                                aria_label: "Seed number for {name}",
                                                value: "{seed}",
                                                onchange: {
                                                    let seeded_team = team_id.clone();
                                                    move |event: FormEvent| {
                                                        let Some(target_seed) = event.value().trim().parse::<u32>().ok() else {
                                                            return;
                                                        };
                                                        let previous = current();
                                                        let order: Vec<TeamId> = entrants_in_seed_order(&previous.target)
                                                            .into_iter()
                                                            .map(|entrant| entrant.team_id)
                                                            .collect();
                                                        let target_seed = target_seed.clamp(1, order.len() as u32);
                                                        if let Some(position) =
                                                            order.iter().position(|id| id == &seeded_team)
                                                            && position as u32 + 1 == target_seed
                                                        {
                                                            return;
                                                        }
                                                        apply_seed_preview(
                                                            current,
                                                            reordered_seed_ids(&order, &seeded_team, None, Some(target_seed)),
                                                        );
                                                    }
                                                },
                                            }
                                            span { class: "fgc-generation-preview__seed-name", "{name}" }
                                        } else {
                                            span { class: "fgc-generation-preview__seed", "{seed}" }
                                            span { "{name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if state.kind.shows_groups() {
                    section { class: "fgc-generation-preview__column",
                        h3 { "Group distribution" }
                        div { class: "fgc-generation-preview__groups",
                            for (group_index, group) in state.target.groups.iter().enumerate() {
                                {
                                    let group_id = group.id.clone();
                                    let group_class = if group_drop().as_ref() == Some(&(group.id.clone(), None)) {
                                        "fgc-generation-preview__group fgc-generation-preview__group--drop-target"
                                    } else {
                                        "fgc-generation-preview__group"
                                    };
                                    rsx! {
                                        div {
                                            key: "{group.id}",
                                            class: group_class,
                                            onpointerenter: {
                                                let hovered = group_id.clone();
                                                move |_| group_drop.set(Some((hovered.clone(), None)))
                                            },
                                            onpointerleave: {
                                                let hovered = group_id.clone();
                                                move |_| {
                                                    if group_drop().as_ref() == Some(&(hovered.clone(), None)) {
                                                        group_drop.set(None);
                                                    }
                                                }
                                            },
                                            onpointerup: {
                                                let dropped_group = group_id.clone();
                                                move |_| on_group_area_up.call(dropped_group.clone())
                                            },
                                            h4 { "{group.name}" }
                                            div { class: "fgc-generation-preview__chips",
                                                for (team_index, team_id) in group.team_ids.iter().enumerate() {
                                                    {
                                                        let chip_class = if group_drag().as_ref() == Some(&(group.id.clone(), team_id.clone())) {
                                                            "fgc-generation-preview__chip fgc-generation-preview__chip--dragging"
                                                        } else if group_drop().as_ref() == Some(&(group.id.clone(), Some(team_id.clone()))) {
                                                            "fgc-generation-preview__chip fgc-generation-preview__chip--drop-before"
                                                        } else {
                                                            "fgc-generation-preview__chip"
                                                        };
                                                        let name = team_name(team_id);
                                                        rsx! {
                                                            button {
                                                                key: "{team_id}",
                                                                r#type: "button",
                                                                id: "fgc-preview-group-{group_index}-{team_index}",
                                                                class: chip_class,
                                                                title: "Drag to another group",
                                                                onpointerenter: {
                                                                    let hovered_group = group.id.clone();
                                                                    let hovered_team = team_id.clone();
                                                                    move |_| group_drop.set(Some((hovered_group.clone(), Some(hovered_team.clone()))))
                                                                },
                                                                onpointerdown: {
                                                                    let source_group = group.id.clone();
                                                                    let dragged = team_id.clone();
                                                                    let drag_name = name.clone();
                                                                    move |event: PointerEvent| {
                                                                        event.prevent_default();
                                                                        on_group_down.call((source_group.clone(), dragged.clone(), drag_name.clone()));
                                                                    }
                                                                },
                                                                onpointerup: {
                                                                    let target_group = group.id.clone();
                                                                    let dropped = team_id.clone();
                                                                    move |event: PointerEvent| {
                                                                        event.stop_propagation();
                                                                        on_group_up.call((target_group.clone(), dropped.clone()));
                                                                    }
                                                                },
                                                                onpointercancel: move |_| {
                                                                    clear_preview_drag(
                                                                        group_drag,
                                                                        slot_drag,
                                                                        seed_drag,
                                                                        group_drop,
                                                                        slot_drop,
                                                                        seed_drop,
                                                                        drag_pointer,
                                                                        drag_label,
                                                                    )
                                                                },
                                                                "⠿ {name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if state.kind.shows_bracket() {
                    section { class: "fgc-generation-preview__column fgc-generation-preview__column--wide",
                        h3 { if state.projected { "Projected bracket placements" } else { "Bracket placements" } }
                        if state.projected {
                            p { class: "fgc-note", "Based on the current group order; the real playoff seeds after all group results." }
                        }
                        if !has_bracket_rounds(&state.bracket_preview) {
                            p { class: "fgc-note", "This generation does not create a bracket yet." }
                        } else {
                            GenerationPreviewBracket {
                                bracket: state.bracket_preview.clone(),
                                seeds: seeds.clone(),
                                team_names: team_names.clone(),
                                slot_drag,
                                slot_drop,
                                on_slot_down,
                                on_slot_up,
                            }
                        }
                    }
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    id: "fgc-generation-preview-cancel",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    id: "fgc-generation-preview-confirm",
                    onclick: move |_| on_commit.call(commit_target.clone()),
                    "Generate"
                }
            }
            }
        }
        if let (Some((x, y)), Some(label)) = (drag_pointer(), drag_label()) {
            {
                let ghost_style = format!("transform: translate3d({:.0}px, {:.0}px, 0);", x + 14.0, y + 12.0);
                rsx! {
                    div { class: "fgc-drag-ghost", style: ghost_style, aria_hidden: "true",
                        span { class: "fgc-drag-ghost__grip", "⠿" }
                        span { class: "fgc-drag-ghost__label", "{label}" }
                    }
                }
            }
        }
    }
}

/// Bracket preview that reuses the live Bracket card layout, centred columns
/// and connector geometry, with preview-only slot dragging.
#[component]
fn GenerationPreviewBracket(
    bracket: Tournament,
    seeds: HashMap<TeamId, u32>,
    team_names: HashMap<TeamId, String>,
    slot_drag: Signal<Option<BracketSlot>>,
    slot_drop: Signal<Option<BracketSlot>>,
    on_slot_down: EventHandler<(BracketSlot, String)>,
    on_slot_up: EventHandler<BracketSlot>,
) -> Element {
    let sections = [
        (BracketSection::Winners, bracket.winners.clone()),
        (BracketSection::Losers, bracket.losers.clone()),
        (BracketSection::GrandFinal, bracket.grand_final.clone()),
    ];

    rsx! {
        div { class: "fgc-generation-preview__bracket",
            for (section_index, (section, rounds)) in sections.into_iter().enumerate() {
                if !rounds.is_empty() {
                    {
                        let (leaf_rows, bracket_height, grid_width, links, drop_stubs) =
                            bracket_geometry(section, &rounds);
                        let view_box = format!("0 0 {grid_width} {bracket_height}");
                        let grid_style = format!(
                            "width: {grid_width}px; --fgc-bracket-card-width: {BRACKET_CARD_WIDTH}px; --fgc-bracket-column-gap: {BRACKET_COLUMN_GAP}px;"
                        );
                        rsx! {
                            section {
                                key: "{section_index}",
                                class: "fgc-bracket-section",
                                div { class: "fgc-bracket-section__header",
                                    h3 { class: "fgc-bracket-section__title", "{section_label(section)}" }
                                }
                                div { class: "fgc-bracket-scroll",
                                    div { class: "fgc-bracket-grid", style: grid_style,
                                        svg {
                                            class: "fgc-bracket-links",
                                            view_box: view_box,
                                            width: "{grid_width}",
                                            height: "{bracket_height}",
                                            role: "presentation",
                                            for path_data in links {
                                                path {
                                                    class: "fgc-bracket-link",
                                                    d: path_data,
                                                    fill: "none",
                                                    stroke: "var(--fgc-line-strong)",
                                                    stroke_width: 2,
                                                }
                                            }
                                            for path_data in drop_stubs {
                                                path {
                                                    class: "fgc-bracket-link--drop",
                                                    d: path_data,
                                                    fill: "none",
                                                    stroke: "var(--fgc-accent)",
                                                    stroke_width: 2,
                                                    stroke_dasharray: "5 4",
                                                }
                                            }
                                        }
                                        for (round_index, round) in rounds.iter().enumerate() {
                                            {
                                                let span = leaf_rows as f64 / round.matches.len().max(1) as f64;
                                                let matches_style = format!("height: {bracket_height}px;");
                                                rsx! {
                                                    div { key: "{round_index}", class: "fgc-bracket-round",
                                                        div { class: "fgc-bracket-round__header",
                                                            span { class: "fgc-bracket-round__name", "{round.name}" }
                                                        }
                                                        div { class: "fgc-bracket-round__matches", style: matches_style,
                                                            for (match_index, game) in round.matches.iter().enumerate() {
                                                                {
                                                                    let card_top = ((match_index as f64 + 0.5) * span * BRACKET_ROW_HEIGHT as f64).round() as i32;
                                                                    rsx! {
                                                                        div {
                                                                            key: "{game.id}",
                                                                            class: "fgc-bracket-match fgc-bracket-match--{match_status_key(game.status)}",
                                                                            style: "top: {card_top}px;",
                                                                            div { class: "fgc-bracket-match__header",
                                                                                span { class: "fgc-bracket-match__number", "M{game.number}" }
                                                                                span { class: "fgc-bracket-match__status", "{match_status_label(game.status)}" }
                                                                            }
                                                                            for (slot_number, team) in [game.slot1.as_ref(), game.slot2.as_ref()].into_iter().enumerate() {
                                                                                {
                                                                                    let address = BracketSlot {
                                                                                        section,
                                                                                        round: round_index,
                                                                                        match_index,
                                                                                        slot: (slot_number + 1) as u8,
                                                                                    };
                                                                                    let name = team.map_or_else(|| "TBD".to_owned(), |id| {
                                                                                        team_names.get(id).cloned().unwrap_or_else(|| id.to_string())
                                                                                    });
                                                                                    let seed = team.and_then(|id| seeds.get(id).copied());
                                                                                    let label = slot_aria_label(section, game.number, (slot_number + 1) as u8, &name);
                                                                                    let slot_class = if slot_drag() == Some(address) {
                                                                                        "fgc-bracket-slot fgc-bracket-slot--preview fgc-bracket-slot--dragging"
                                                                                    } else if slot_drop() == Some(address) {
                                                                                        "fgc-bracket-slot fgc-bracket-slot--preview fgc-bracket-slot--drop-target"
                                                                                    } else {
                                                                                        "fgc-bracket-slot fgc-bracket-slot--preview"
                                                                                    };
                                                                                    rsx! {
                                                                                        button {
                                                                                            key: "{slot_number}",
                                                                                            r#type: "button",
                                                                                            id: "fgc-preview-slot-{section_index}-{round_index}-{match_index}-{slot_number}",
                                                                                            class: slot_class,
                                                                                            aria_label: label,
                                                                                            title: "Drag to another participant in the same round",
                                                                                            onpointerenter: move |_| slot_drop.set(Some(address)),
                                                                                            onpointerleave: move |_| {
                                                                                                if slot_drop() == Some(address) {
                                                                                                    slot_drop.set(None);
                                                                                                }
                                                                                            },
                                                                                            onpointerdown: move |event: PointerEvent| {
                                                                                                event.prevent_default();
                                                                                                on_slot_down.call((address, name.clone()));
                                                                                            },
                                                                                            onpointerup: move |event: PointerEvent| {
                                                                                                event.stop_propagation();
                                                                                                on_slot_up.call(address);
                                                                                            },
                                                                                            onpointercancel: move |_| {
                                                                                                slot_drag.set(None);
                                                                                                slot_drop.set(None);
                                                                                            },
                                                                                            span { class: "fgc-bracket-slot__seed", if let Some(seed) = seed { "{seed}" } }
                                                                                            span { class: "fgc-bracket-slot__name", "{name}" }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Detail shell
// ---------------------------------------------------------------------------

#[component]
fn TournamentDetail(
    tournament: Tournament,
    teams: Vec<Team>,
    tab: TournamentTab,
    on_tab: EventHandler<TournamentTab>,
    board: BoardState,
    on_board: EventHandler<BoardState>,
    on_open_match: EventHandler<MatchId>,
    on_open_group_match: EventHandler<MatchId>,
    on_delete: EventHandler<TournamentId>,
    on_push: EventHandler<PushTarget>,
    on_reset_stage: EventHandler<StageResetTarget>,
    on_preview: EventHandler<GenerationRequest>,
    generation_error: Signal<String>,
) -> Element {
    let tournament_id = tournament.id.clone();
    let champion = tournament
        .champion
        .as_ref()
        .and_then(|id| teams.iter().find(|team| &team.id == id))
        .map(|team| team.name.clone());
    let tabs = TournamentTab::for_format(tournament.format);
    let active = if tabs.contains(&tab) {
        tab
    } else {
        TournamentTab::Setup
    };

    rsx! {
        div { class: "fgc-tournament-detail",
            div { class: "fgc-tournament__toolbar",
                div { class: "fgc-tournament__identity",
                    p { class: "fgc-tournament__name", "{tournament.name}" }
                    p { class: "fgc-tournament__meta",
                        "{format_label(tournament.format)} · {tournament.entrants.len()} teams"
                        if let Some(champion) = champion.clone() {
                            " · Champion: {champion}"
                        }
                    }
                }
                div { class: "fgc-actions",
                    span {
                        class: "fgc-tournament__status fgc-tournament__status--{status_key(tournament.status)}",
                        "{status_label(tournament.status)}"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_delete.call(tournament_id.clone()),
                        "Delete"
                    }
                }
            }

            div { class: "fgc-tabs",
                for item in tabs.clone() {
                    button {
                        key: "{item.label()}",
                        id: "fgc-tournament-tab-{item.key()}",
                        r#type: "button",
                        aria_pressed: item == active,
                        class: if item == active {
                            "fgc-tab fgc-tab--active"
                        } else {
                            "fgc-tab"
                        },
                        onclick: move |_| on_tab.call(item),
                        "{item.label()}"
                    }
                }
            }

            {match active {
                TournamentTab::Setup => rsx! {
                    SetupTab {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                    }
                },
                TournamentTab::Board => rsx! {
                    BoardTab {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                        board,
                        on_board,
                        on_open_match,
                        on_push,
                        on_reset_stage,
                        on_preview,
                        generation_error,
                    }
                },
                TournamentTab::Groups => rsx! {
                    GroupsTab {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                        on_open_group_match,
                        on_push,
                        on_reset_stage,
                        on_preview,
                        generation_error,
                    }
                },
                TournamentTab::Settings => rsx! {
                    SettingsTab { tournament: tournament.clone(), teams: teams.clone() }
                },
            }}
        }
    }
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

#[component]
fn SetupTab(tournament: Tournament, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let generated = tournament_is_generated(&tournament);
    let tournament_id = tournament.id.clone();
    let entrants = entrants_in_seed_order(&tournament);
    let entrant_ids: HashSet<TeamId> = entrants
        .iter()
        .map(|entrant| entrant.team_id.clone())
        .collect();
    let available: Vec<Team> = teams
        .iter()
        .filter(|team| !entrant_ids.contains(&team.id))
        .cloned()
        .collect();
    let mut pending_team = use_signal(|| None::<TeamId>);

    let name_id = tournament_id.clone();
    let format_id = tournament_id.clone();
    let first_to_id = tournament_id.clone();
    let reset_id = tournament_id.clone();
    let status_id = tournament_id.clone();
    let add_id = tournament_id.clone();

    rsx! {
        div { class: "fgc-tournament-stack",
            Panel {
                title: "Tournament details".to_owned(),
                div { class: "fgc-form-row",
                    Field { label: "Name".to_owned(),
                        Input {
                            value: "{tournament.name}",
                            aria_label: "Tournament name",
                            placeholder: "e.g. Autumn Invitational",
                            onchange: move |event: FormEvent| {
                                store.set_tournament_name(&name_id, &event.value());
                            },
                        }
                    }
                    Field { label: "Format".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Format",
                            value: "{format_key(tournament.format)}",
                            onchange: move |event| {
                                let format = format_from_key(&event.value());
                                store.set_tournament_format(&format_id, format);
                            },
                            for format in [
                                BracketFormat::SingleElim,
                                BracketFormat::DoubleElim,
                                BracketFormat::RoundRobin,
                                BracketFormat::MultiStage,
                            ] {
                                option {
                                    value: "{format_key(format)}",
                                    selected: format == tournament.format,
                                    "{format_label(format)}"
                                }
                            }
                        }
                    }
                    Field { label: "First to".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "First to",
                            value: "{tournament.default_first_to}",
                            onchange: move |event| {
                                if let Ok(value) = event.value().parse::<u8>() {
                                    store.set_tournament_default_first_to(&first_to_id, value);
                                }
                            },
                            for value in 1..=4u8 {
                                option {
                                    value: "{value}",
                                    selected: value == tournament.default_first_to,
                                    "First to {value}"
                                }
                            }
                        }
                    }
                    Field { label: "Status".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Tournament status",
                            value: "{status_key(tournament.status)}",
                            onchange: move |event| {
                                store.set_tournament_status(&status_id, status_from_key(&event.value()));
                            },
                            for status in [TournamentStatus::Setup, TournamentStatus::InProgress, TournamentStatus::Completed] {
                                option {
                                    value: "{status_key(status)}",
                                    selected: status == tournament.status,
                                    "{status_label(status)}"
                                }
                            }
                        }
                    }
                }
                ToggleRow {
                    title: "Grand-final reset".to_owned(),
                    hint: Some("Replay the grand final when the losers-bracket team wins.".to_owned()),
                    Switch {
                        checked: tournament.grand_finals_reset,
                        aria_label: "Grand-final reset",
                        on_checked_change: move |enabled: bool| {
                            store.set_tournament_grand_finals_reset(&reset_id, enabled);
                        },
                    }
                }
            }

            Panel {
                title: format!("Entrants ({})", entrants.len()),
                if generated {
                    p { class: "fgc-note", "Locked after generation." }
                } else if !available.is_empty() {
                    div { class: "fgc-tournament-add",
                        select {
                            class: "fgc-native-select",
                            aria_label: "Team to add",
                            value: pending_team().map(|id| id.as_str().to_owned()).unwrap_or_default(),
                            onchange: move |event| {
                                let value = event.value();
                                pending_team.set(if value.is_empty() { None } else { Some(TeamId::from_raw(value)) });
                            },
                            option { value: "", "Select a team" }
                            for team in available.iter() {
                                option {
                                    key: "{team.id}",
                                    value: "{team.id}",
                                    selected: pending_team().as_ref() == Some(&team.id),
                                    "{team.name}"
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            disabled: pending_team().is_none(),
                            onclick: move |_| {
                                let Some(team) = pending_team() else {
                                    return;
                                };
                                store.add_tournament_entrant(&add_id, team);
                                pending_team.set(None);
                            },
                            "Add entrant"
                        }
                    }
                }
                if entrants.is_empty() {
                    div { class: "fgc-empty", "Add at least two teams before generating a bracket." }
                } else {
                    div { class: "fgc-entrant-list",
                        for (index, entrant) in entrants.iter().enumerate() {
                            {
                                let team = teams.iter().find(|team| team.id == entrant.team_id);
                                let name = team.map_or_else(|| entrant.team_id.to_string(), |team| team.name.clone());
                                let up_id = tournament_id.clone();
                                let down_id = tournament_id.clone();
                                let remove_id = tournament_id.clone();
                                let team_id = entrant.team_id.clone();
                                let can_up = index > 0;
                                let can_down = index + 1 < entrants.len();
                                rsx! {
                                    div { key: "{entrant.team_id}", class: "fgc-entrant",
                                        span { class: "fgc-entrant__seed", "{entrant.seed}" }
                                        span { class: "fgc-entrant__name", "{name}" }
                                        if !generated {
                                            div { class: "fgc-entrant__actions",
                                                button {
                                                    r#type: "button",
                                                    class: "fgc-icon-button",
                                                    disabled: !can_up,
                                                    title: "Move up",
                                                    aria_label: "Move up",
                                                    onclick: move |_| {
                                                        if can_up {
                                                            store.move_tournament_entrant(&up_id, index, index - 1);
                                                        }
                                                    },
                                                    "↑"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "fgc-icon-button",
                                                    disabled: !can_down,
                                                    title: "Move down",
                                                    aria_label: "Move down",
                                                    onclick: move |_| {
                                                        if can_down {
                                                            store.move_tournament_entrant(&down_id, index, index + 1);
                                                        }
                                                    },
                                                    "↓"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "fgc-icon-button fgc-icon-button--danger",
                                                    title: "Remove entrant",
                                                    aria_label: "Remove entrant",
                                                    onclick: move |_| {
                                                        store.remove_tournament_entrant(&remove_id, &team_id);
                                                    },
                                                    "×"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

        }
    }
}

// ---------------------------------------------------------------------------
// Board
// ---------------------------------------------------------------------------

#[component]
fn BoardTab(
    tournament: Tournament,
    teams: Vec<Team>,
    board: BoardState,
    on_board: EventHandler<BoardState>,
    on_open_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
    on_reset_stage: EventHandler<StageResetTarget>,
    on_preview: EventHandler<GenerationRequest>,
    generation_error: Signal<String>,
) -> Element {
    let tournament_id = tournament.id.clone();
    let reset_bracket = stage_reset_target(
        &tournament,
        TournamentStage::Bracket,
        "fgc-reset-bracket-board",
    );
    let seeds: HashMap<TeamId, u32> = tournament
        .entrants
        .iter()
        .map(|entrant| (entrant.team_id.clone(), entrant.seed))
        .collect();
    let sections: [(BracketSection, Vec<Round>); 3] = [
        (BracketSection::Winners, tournament.winners.clone()),
        (BracketSection::Losers, tournament.losers.clone()),
        (BracketSection::GrandFinal, tournament.grand_final.clone()),
    ];
    let has_board = sections.iter().any(|(_, rounds)| !rounds.is_empty());
    let groups_complete = group_stage_complete(&tournament);
    let playoff_locked =
        tournament.format == BracketFormat::MultiStage && (!groups_complete || !has_board);

    let generated = tournament_is_generated(&tournament);
    let is_multistage = tournament.format == BracketFormat::MultiStage;
    let show_bracket_action = is_multistage || generated;
    let bracket_kind = if is_multistage {
        GenerationKind::Playoff
    } else {
        GenerationKind::Bracket
    };
    let bracket_trigger = if is_multistage && !has_board {
        "fgc-preview-playoff-bracket"
    } else {
        "fgc-regenerate-bracket-board"
    };
    let bracket_label = if is_multistage {
        if has_board {
            "Regenerate playoff"
        } else {
            "Preview playoff"
        }
    } else if has_board {
        "Regenerate bracket"
    } else {
        "Preview bracket"
    };
    let bracket_tone = if bracket_label.starts_with("Preview") {
        StageActionTone::Preview
    } else {
        StageActionTone::Regenerate
    };
    let bracket_actions = if show_bracket_action || (generated && has_board) {
        Some(rsx! {
            div { class: "fgc-stage-actions", role: "group", aria_label: "Bracket actions",
                StageAction {
                    id: bracket_trigger.to_owned(),
                    label: bracket_label.to_owned(),
                    tone: bracket_tone,
                    on_action: EventHandler::new(move |_| on_preview.call(GenerationRequest {
                        kind: bracket_kind,
                        trigger_id: bracket_trigger.to_owned(),
                    })),
                }
                if generated && has_board {
                    StageAction {
                        id: "fgc-reset-bracket-board".to_owned(),
                        label: "Reset bracket".to_owned(),
                        tone: StageActionTone::Reset,
                        on_action: EventHandler::new(move |_| {
                            on_reset_stage.call(reset_bracket.clone());
                        }),
                    }
                }
            }
        })
    } else {
        None
    };

    rsx! {
        Panel {
            title: "Bracket".to_owned(),
            actions: bracket_actions,
        div { class: "fgc-board",
            div { class: "fgc-board__toolbar",
                div { class: "fgc-board__controls",
                    label { class: "fgc-board__zoom",
                        span { class: "fgc-field__label", "Zoom" }
                        input {
                            r#type: "range",
                            min: "50",
                            max: "150",
                            step: "5",
                            value: "{board.zoom}",
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<u16>() {
                                    on_board.call(BoardState { zoom: value });
                                }
                            }
                        }
                        span { class: "fgc-board__zoom-value", "{board.zoom}%" }
                    }
                }
            }

            if !generation_error().is_empty() {
                p { class: "fgc-field__error", role: "alert", "{generation_error()}" }
            }

            if playoff_locked {
                Panel {
                    title: "Playoff locked".to_owned(),
                    description: if groups_complete {
                        "Could not seed the playoff. Check Qualifiers per group in Settings.".to_owned()
                    } else {
                        "Finish every group fixture first.".to_owned()
                    },
                    div { class: "fgc-empty", "Complete the group stage to seed the playoff." }
                }
            } else if !has_board {
                Panel {
                    title: "No bracket yet".to_owned(),
                    div { class: "fgc-empty", "Use the Preview bracket action beside the Bracket heading." }
                }
            } else {
                div { class: "fgc-board__canvas", style: "zoom: {board.zoom}%;",
                    for (section, rounds) in sections {
                        if !rounds.is_empty() {
                            BoardSection {
                                key: "{section_key(section)}",
                                tournament_id: tournament_id.clone(),
                                tournament_name: tournament.name.clone(),
                                section,
                                rounds,
                                teams: teams.clone(),
                                seeds: seeds.clone(),
                                generated,
                                on_open_match,
                                on_push,
                            }
                        }
                    }
                }
            }
        }
        }
    }
}

const BRACKET_CARD_WIDTH: i32 = 248;
const BRACKET_COLUMN_GAP: i32 = 56;
const BRACKET_ROW_HEIGHT: i32 = 108;

/// Geometry for a centred traditional bracket. Solid paths follow the stored
/// winner links, including the lower bracket's one-to-one first round, so two
/// adjacent matches feed the correct next-round card without crossing. Dotted
/// stubs mark upper-bracket losers dropping into the lower bracket, which is
/// drawn as a separate section.
fn bracket_geometry(
    section: BracketSection,
    rounds: &[Round],
) -> (usize, i32, i32, Vec<String>, Vec<String>) {
    let leaf_rows = rounds
        .iter()
        .map(|round| round.matches.len().saturating_mul(2))
        .max()
        .unwrap_or(2)
        .max(2);
    let bracket_height = (leaf_rows as i32) * BRACKET_ROW_HEIGHT;
    let grid_width = (rounds.len() as i32) * BRACKET_CARD_WIDTH
        + rounds.len().saturating_sub(1) as i32 * BRACKET_COLUMN_GAP;
    let positions: HashMap<MatchId, (usize, usize)> = rounds
        .iter()
        .enumerate()
        .flat_map(|(round_index, round)| {
            round
                .matches
                .iter()
                .enumerate()
                .map(move |(match_index, game)| (game.id.clone(), (round_index, match_index)))
        })
        .collect();
    let mut links = Vec::new();
    let mut drop_stubs = Vec::new();

    let center_y = |round_index: usize, match_index: usize| {
        let count = rounds[round_index].matches.len().max(1);
        let span = leaf_rows as f64 / count as f64;
        (match_index as f64 + 0.5) * span * BRACKET_ROW_HEIGHT as f64
    };
    let right_x = |round_index: usize| {
        (round_index as i32) * (BRACKET_CARD_WIDTH + BRACKET_COLUMN_GAP) + BRACKET_CARD_WIDTH
    };

    for (round_index, round) in rounds.iter().enumerate() {
        for (match_index, game) in round.matches.iter().enumerate() {
            let source_x = right_x(round_index);
            let bus_x = source_x + BRACKET_COLUMN_GAP / 2;
            let source_y = center_y(round_index, match_index);

            if let Some(link) = &game.next
                && let Some(&(target_round, target_match)) = positions.get(&link.match_id)
                && target_round > round_index
            {
                let target_x = target_round as i32 * (BRACKET_CARD_WIDTH + BRACKET_COLUMN_GAP);
                let target_y = center_y(target_round, target_match);
                links.push(format!(
                    "M {source_x} {source_y:.0} H {bus_x} V {target_y:.0} H {target_x}"
                ));
            }

            if section == BracketSection::Winners && game.loser_next.is_some() {
                let stub_y = (source_y + BRACKET_ROW_HEIGHT as f64 * 0.2)
                    .min(bracket_height as f64 - BRACKET_ROW_HEIGHT as f64 * 0.1);
                drop_stubs.push(format!(
                    "M {source_x} {source_y:.0} H {bus_x} V {stub_y:.0}"
                ));
            }
        }
    }

    (leaf_rows, bracket_height, grid_width, links, drop_stubs)
}

#[component]
fn BoardSection(
    tournament_id: TournamentId,
    tournament_name: String,
    section: BracketSection,
    rounds: Vec<Round>,
    teams: Vec<Team>,
    seeds: HashMap<TeamId, u32>,
    generated: bool,
    on_open_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
) -> Element {
    let store = use_context::<AppStore>();
    let add_round_id = tournament_id.clone();
    let round_count = rounds.len();
    let (leaf_rows, bracket_height, grid_width, links, drop_stubs) =
        bracket_geometry(section, &rounds);
    let view_box = format!("0 0 {grid_width} {bracket_height}");
    let grid_style = format!(
        "width: {grid_width}px; --fgc-bracket-card-width: {BRACKET_CARD_WIDTH}px; --fgc-bracket-column-gap: {BRACKET_COLUMN_GAP}px;"
    );

    rsx! {
        section { class: "fgc-bracket-section",
            div { class: "fgc-bracket-section__header",
                h3 { class: "fgc-bracket-section__title", "{section_label(section)}" }
                if !generated {
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            store.add_tournament_round(&add_round_id, section);
                        },
                        "Add round"
                    }
                }
            }
            div { class: "fgc-bracket-scroll",
                div { class: "fgc-bracket-grid", style: grid_style,
                    if !links.is_empty() || !drop_stubs.is_empty() {
                        svg {
                            class: "fgc-bracket-links",
                            view_box: view_box,
                            width: "{grid_width}",
                            height: "{bracket_height}",
                            role: "presentation",
                            for path_data in links {
                                path {
                                    class: "fgc-bracket-link",
                                    d: path_data,
                                    fill: "none",
                                    stroke: "var(--fgc-line-strong)",
                                    stroke_width: 2,
                                }
                            }
                            for path_data in drop_stubs {
                                path {
                                    class: "fgc-bracket-link--drop",
                                    d: path_data,
                                    fill: "none",
                                    stroke: "var(--fgc-accent)",
                                    stroke_width: 2,
                                    stroke_dasharray: "5 4",
                                }
                            }
                        }
                    }
                    for (round_index, round) in rounds.iter().enumerate() {
                        {
                            let rename_id = tournament_id.clone();
                            let remove_round_id = tournament_id.clone();
                            let add_match_id = tournament_id.clone();
                            let round_name = round.name.clone();
                            let span = leaf_rows as f64 / round.matches.len().max(1) as f64;
                            let matches_style = format!("height: {bracket_height}px;");
                            rsx! {
                                div { key: "{round_index}", class: "fgc-bracket-round",
                                    div { class: "fgc-bracket-round__header",
                                        input {
                                            class: "fgc-bracket-round__name",
                                            value: "{round_name}",
                                            aria_label: "Round name",
                                            onchange: move |event: FormEvent| {
                                                store.rename_tournament_round(
                                                    &rename_id,
                                                    section,
                                                    round_index,
                                                    &event.value(),
                                                );
                                            },
                                        }
                                        if !generated {
                                            button {
                                                r#type: "button",
                                                class: "fgc-icon-button fgc-icon-button--danger",
                                                title: "Remove round",
                                                aria_label: "Remove round",
                                                disabled: round_count <= 1,
                                                onclick: move |_| {
                                                    store.remove_tournament_round(
                                                        &remove_round_id,
                                                        section,
                                                        round_index,
                                                    );
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                    div { class: "fgc-bracket-round__matches", style: matches_style,
                                        for (match_index, game) in round.matches.iter().enumerate() {
                                            {
                                                let card_top = ((match_index as f64 + 0.5) * span * BRACKET_ROW_HEIGHT as f64).round() as i32;
                                                rsx! {
                                                    MatchCard {
                                                        key: "{game.id}",
                                                        tournament_name: tournament_name.clone(),
                                                        round_name: round_name.clone(),
                                                        game: game.clone(),
                                                        section,
                                                        round_index,
                                                        match_index,
                                                        card_top,
                                                        teams: teams.clone(),
                                                        seeds: seeds.clone(),
                                                        on_open_match,
                                                        on_push,
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !generated {
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            onclick: move |_| {
                                                store.add_tournament_match(&add_match_id, section, round_index);
                                            },
                                            "+ Match"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MatchCard(
    tournament_name: String,
    round_name: String,
    game: BracketMatch,
    section: BracketSection,
    round_index: usize,
    match_index: usize,
    card_top: i32,
    teams: Vec<Team>,
    seeds: HashMap<TeamId, u32>,
    on_open_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
) -> Element {
    let slot1_address = BracketSlot {
        section,
        round: round_index,
        match_index,
        slot: 1,
    };
    let slot2_address = BracketSlot {
        section,
        round: round_index,
        match_index,
        slot: 2,
    };
    let name1 = team_option_name(&teams, game.slot1.as_ref());
    let name2 = team_option_name(&teams, game.slot2.as_ref());
    let seed1 = game.slot1.as_ref().and_then(|id| seeds.get(id).copied());
    let seed2 = game.slot2.as_ref().and_then(|id| seeds.get(id).copied());
    let winner1 = game.winner.is_some() && game.winner == game.slot1;
    let winner2 = game.winner.is_some() && game.winner == game.slot2;
    let aria1 = slot_aria_label(section, game.number, 1, &name1);
    let aria2 = slot_aria_label(section, game.number, 2, &name2);

    let open_id = game.id.clone();
    let push_id = game.id.clone();
    let push_title = tournament_name;
    let push_subtitle = format!("{} · M{}", round_name, game.number);
    let push_team_a = game.slot1.clone();
    let push_team_b = game.slot2.clone();
    let push_score1 = game.score1;
    let push_score2 = game.score2;
    let push_status = game.status;
    let push_first_to = game.first_to;

    rsx! {
        div {
            style: "top: {card_top}px;",
            class: "fgc-bracket-match fgc-bracket-match--{match_status_key(game.status)}",
            div { class: "fgc-bracket-match__header",
                span { class: "fgc-bracket-match__number", "M{game.number}" }
                span { class: "fgc-bracket-match__status", "{match_status_label(game.status)}" }
                if game.winner_manual {
                    span { class: "fgc-badge fgc-badge--manual", "Manual winner" }
                }
            }
            BracketSlotButton {
                address: slot1_address,
                label: name1,
                seed: seed1,
                score: Some(game.score1),
                aria_label: aria1,
                is_winner: winner1,
            }
            BracketSlotButton {
                address: slot2_address,
                label: name2,
                seed: seed2,
                score: Some(game.score2),
                aria_label: aria2,
                is_winner: winner2,
            }
            div { class: "fgc-bracket-match__footer",
                div { class: "fgc-bracket-match__tools",
                    button {
                        r#type: "button",
                        class: "fgc-bracket-match__action",
                        id: "fgc-edit-match-{open_id}",
                        title: "Edit match details",
                        aria_label: "Edit match M{game.number} details",
                        onclick: move |_| on_open_match.call(open_id.clone()),
                        span { class: "fgc-bracket-match__action-icon", "✎" }
                        span { class: "fgc-bracket-match__action-label", "Edit" }
                    }
                    button {
                        r#type: "button",
                        class: "fgc-bracket-match__action",
                        id: "fgc-push-match-{push_id}",
                        title: "Push to match — replaces the current Match Controller match after confirmation. Tournament graphics are separate in v1.",
                        aria_label: "Push match M{game.number} to Match Controller",
                        onclick: move |_| {
                            let trigger_id = format!("fgc-push-match-{push_id}");
                            on_push.call(PushTarget {
                                title: push_title.clone(),
                                subtitle: push_subtitle.clone(),
                                team_a: push_team_a.clone(),
                                team_b: push_team_b.clone(),
                                score1: push_score1,
                                score2: push_score2,
                                status: push_status,
                                first_to: push_first_to,
                                trigger_id,
                            });
                        },
                        span { class: "fgc-bracket-match__action-icon", "↗" }
                        span { class: "fgc-bracket-match__action-label", "Push" }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Slot button
// ---------------------------------------------------------------------------

#[component]
fn BracketSlotButton(
    address: BracketSlot,
    label: String,
    seed: Option<u32>,
    score: Option<u16>,
    aria_label: String,
    is_winner: bool,
) -> Element {
    let dom_id = slot_dom_id(address);
    let class = if is_winner {
        "fgc-bracket-slot fgc-bracket-slot--winner"
    } else {
        "fgc-bracket-slot"
    };

    rsx! {
        div {
            id: "{dom_id}",
            class: "{class}",
            aria_label: "{aria_label}",
            title: "Placements are fixed at generation",
            span { class: "fgc-bracket-slot__seed", if let Some(seed) = seed { "{seed}" } }
            span { class: "fgc-bracket-slot__name", "{label}" }
            if let Some(score) = score {
                span { class: "fgc-bracket-slot__score", "{score}" }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[component]
fn GroupsTab(
    tournament: Tournament,
    teams: Vec<Team>,
    on_open_group_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
    on_reset_stage: EventHandler<StageResetTarget>,
    on_preview: EventHandler<GenerationRequest>,
    generation_error: Signal<String>,
) -> Element {
    let generated = tournament_is_generated(&tournament);
    let tournament_id = tournament.id.clone();
    let reset_groups = stage_reset_target(
        &tournament,
        TournamentStage::Groups,
        "fgc-reset-groups-groups",
    );
    let is_multistage = tournament.format == BracketFormat::MultiStage;
    let group_trigger = if is_multistage {
        "fgc-preview-multistage-groups"
    } else {
        "fgc-preview-round-robin-groups"
    };
    let group_kind = if is_multistage {
        GenerationKind::MultiStage(2)
    } else {
        GenerationKind::RoundRobin(2)
    };
    let group_label = if generated {
        if is_multistage {
            "Regenerate group stage"
        } else {
            "Regenerate round robin"
        }
    } else if is_multistage {
        "Preview group stage"
    } else {
        "Preview round robin"
    };
    let groups = tournament.groups.clone();
    let entrant_names: HashMap<TeamId, String> = tournament
        .entrants
        .iter()
        .map(|entrant| {
            let name = teams
                .iter()
                .find(|team| team.id == entrant.team_id)
                .map_or_else(|| entrant.team_id.to_string(), |team| team.name.clone());
            (entrant.team_id.clone(), name)
        })
        .collect();

    let group_tone = if group_label.starts_with("Preview") {
        StageActionTone::Preview
    } else {
        StageActionTone::Regenerate
    };
    let group_actions = rsx! {
        div { class: "fgc-stage-actions", role: "group", aria_label: "Group stage actions",
            if tournament.entrants.len() >= 2 {
                StageAction {
                    id: group_trigger.to_owned(),
                    label: group_label.to_owned(),
                    tone: group_tone,
                    on_action: EventHandler::new(move |_| on_preview.call(GenerationRequest {
                        kind: group_kind,
                        trigger_id: group_trigger.to_owned(),
                    })),
                }
            }
            if generated {
                StageAction {
                    id: "fgc-reset-groups-groups".to_owned(),
                    label: "Reset group stage".to_owned(),
                    tone: StageActionTone::Reset,
                    on_action: EventHandler::new(move |_| {
                        on_reset_stage.call(reset_groups.clone());
                    }),
                }
            }
        }
    };

    rsx! {
        div { class: "fgc-tournament-stack",
            Panel {
                title: "Group stage".to_owned(),
                actions: Some(group_actions),
                if tournament.entrants.len() < 2 {
                    p { class: "fgc-note", "Add at least two entrants in Setup first." }
                }
                if !generation_error().is_empty() {
                    p { class: "fgc-field__error", role: "alert", "{generation_error()}" }
                }
            }

            if groups.is_empty() {
                Panel {
                    title: "No groups".to_owned(),
                    div { class: "fgc-empty", "Use the Group stage heading action to generate groups." }
                }
            } else {
                for group in groups.clone() {
                    {
                        let fixtures: Vec<GroupMatch> = tournament
                            .group_matches
                            .iter()
                            .filter(|fixture| fixture.group == group.id)
                            .cloned()
                            .collect();
                        let standings = tournament
                            .standings
                            .get(&group.id)
                            .cloned()
                            .unwrap_or_default();
                        rsx! {
                            GroupPanel {
                                key: "{group.id}",
                                tournament_id: tournament_id.clone(),
                                tournament_name: tournament.name.clone(),
                                group: group.clone(),
                                teams: teams.clone(),
                                entrant_names: entrant_names.clone(),
                                standings,
                                fixtures,
                                generated,
                                focus_group_id: group_trigger.to_owned(),
                                on_open_group_match,
                                on_push,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GroupPanel(
    tournament_id: TournamentId,
    tournament_name: String,
    group: Group,
    teams: Vec<Team>,
    entrant_names: HashMap<TeamId, String>,
    standings: Vec<Standing>,
    fixtures: Vec<GroupMatch>,
    generated: bool,
    focus_group_id: String,
    on_open_group_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
) -> Element {
    let store = use_context::<AppStore>();
    let rename_id = tournament_id.clone();
    let remove_id = tournament_id.clone();
    let add_team_id = tournament_id.clone();
    let add_fixture_id = tournament_id.clone();
    let mut pending_team = use_signal(|| None::<TeamId>);
    let mut fixture_team1 = use_signal(|| None::<TeamId>);
    let mut fixture_team2 = use_signal(|| None::<TeamId>);
    let mut confirming_delete = use_signal(|| false);
    let mut team_to_remove = use_signal(|| None::<(TeamId, String)>);
    let delete_trigger_id = format!("fgc-group-delete-{}", group.id);
    let delete_cancel_id = format!("fgc-group-cancel-{}", group.id);
    let remove_team_trigger_id = format!("fgc-remove-team-trigger-{}", group.id);
    let remove_team_cancel_id = format!("fgc-remove-team-cancel-{}", group.id);
    let rename_group_id = group.id.clone();
    let delete_group_id = group.id.clone();
    let add_fixture_group_id = group.id.clone();
    let add_team_group_id = group.id.clone();
    let group_team_ids = group.team_ids.clone();
    let available: Vec<Team> = teams
        .iter()
        .filter(|team| !group.team_ids.contains(&team.id))
        .cloned()
        .collect();
    let member_names: Vec<(TeamId, String)> = group
        .team_ids
        .iter()
        .map(|id| {
            let name = entrant_names
                .get(id)
                .cloned()
                .or_else(|| {
                    teams
                        .iter()
                        .find(|team| &team.id == id)
                        .map(|team| team.name.clone())
                })
                .unwrap_or_else(|| id.to_string());
            (id.clone(), name)
        })
        .collect();
    let fixture_team1_value = fixture_team1()
        .map(|id| id.as_str().to_owned())
        .unwrap_or_default();
    let fixture_team2_value = fixture_team2()
        .map(|id| id.as_str().to_owned())
        .unwrap_or_default();
    let focus_after_delete = focus_group_id.clone();
    let focus_after_remove = focus_group_id;

    rsx! {
        Panel {
            title: group.name.clone(),
            div { class: "fgc-group-header",
                onkeydown: {
                    let trigger = delete_trigger_id.clone();
                    move |event: KeyboardEvent| {
                        if event.key() == Key::Escape && confirming_delete() {
                            event.stop_propagation();
                            confirming_delete.set(false);
                            focus_element_later(&trigger);
                        }
                    }
                },
                Input {
                    value: "{group.name}",
                    aria_label: "Group name",
                    onchange: move |event: FormEvent| {
                        store.rename_tournament_group(&rename_id, &rename_group_id, &event.value());
                    },
                }
                if !generated {
                    if confirming_delete() {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            id: "{delete_cancel_id}",
                            onclick: {
                                let trigger = delete_trigger_id.clone();
                                move |_| {
                                    confirming_delete.set(false);
                                    focus_element_later(&trigger);
                                }
                            },
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                store.remove_tournament_group(&remove_id, &delete_group_id);
                                focus_element_later(&focus_after_delete);
                            },
                            "Delete group"
                        }
                    } else {
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            id: "{delete_trigger_id}",
                            onclick: {
                                let cancel = delete_cancel_id.clone();
                                move |_| {
                                    confirming_delete.set(true);
                                    focus_element_later(&cancel);
                                }
                            },
                            "Delete group"
                        }
                    }
                }
            }
            if !generated && confirming_delete() {
                p { class: "fgc-note",
                    "Deleting this group also removes its {fixtures.len()} fixture(s) and results. This cannot be undone."
                }
            }

            if !generated && !available.is_empty() {
                div { class: "fgc-tournament-add",
                    select {
                        class: "fgc-native-select",
                        aria_label: "Team to add",
                        value: pending_team().map(|id| id.as_str().to_owned()).unwrap_or_default(),
                        onchange: move |event| {
                            let value = event.value();
                            pending_team.set(if value.is_empty() { None } else { Some(TeamId::from_raw(value)) });
                        },
                        option { value: "", "Add team to group" }
                        for team in available.iter() {
                            option {
                                key: "{team.id}",
                                value: "{team.id}",
                                selected: pending_team().as_ref() == Some(&team.id),
                                "{team.name}"
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        disabled: pending_team().is_none(),
                        onclick: move |_| {
                            let Some(team) = pending_team() else {
                                return;
                            };
                            let mut next = group_team_ids.clone();
                            next.push(team);
                            store.set_tournament_group_teams(&add_team_id, &add_team_group_id, next);
                            pending_team.set(None);
                        },
                        "Add"
                    }
                }
            }

            if standings.is_empty() {
                p { class: "fgc-note", "Standings appear once the group has teams." }
            } else {
                table { class: "fgc-standings", aria_label: "Group standings",
                    thead {
                        tr {
                            th { scope: "col", "Team" }
                            th { scope: "col", title: "Played", "P" }
                            th { scope: "col", title: "Won", "W" }
                            th { scope: "col", title: "Lost", "L" }
                            th { scope: "col", title: "Drawn", "D" }
                            th { scope: "col", title: "Points for", "PF" }
                            th { scope: "col", title: "Points against", "PA" }
                            th { scope: "col", title: "Differential", "Diff" }
                            th { scope: "col", title: "Points", "Pts" }
                        }
                    }
                    tbody {
                        for row in standings {
                            {
                                let team_label = entrant_names.get(&row.team_id).cloned()
                                    .or_else(|| teams.iter().find(|team| team.id == row.team_id).map(|team| team.name.clone()))
                                    .unwrap_or_else(|| row.team_id.to_string());
                                let remove_id = row.team_id.clone();
                                let cancel_id = remove_team_cancel_id.clone();
                                rsx! {
                                    tr { key: "{row.team_id}",
                                        td { class: "fgc-standings__team",
                                            span { class: "fgc-standings__name", "{team_label}" }
                                            if !generated {
                                                button {
                                                    r#type: "button",
                                                    class: "fgc-icon-button fgc-icon-button--danger fgc-standings__remove",
                                                    id: "{remove_team_trigger_id}-{remove_id}",
                                                    title: "Remove from group",
                                                    aria_label: "Remove {team_label} from {group.name}",
                                                    onclick: move |_| {
                                                        team_to_remove.set(Some((remove_id.clone(), team_label.clone())));
                                                        focus_element_later(&cancel_id);
                                                    },
                                                    "×"
                                                }
                                            }
                                        }
                                        td { "{row.played}" }
                                        td { "{row.wins}" }
                                        td { "{row.losses}" }
                                        td { "{row.draws}" }
                                        td { "{row.points_for}" }
                                        td { "{row.points_against}" }
                                        td { "{row.map_differential}" }
                                        td { "{row.points}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            h4 { class: "fgc-group-subtitle", "Fixtures" }
            if fixtures.is_empty() {
                p { class: "fgc-note", "No fixtures yet." }
            } else {
                div { class: "fgc-fixture-list",
                    for fixture in fixtures {
                        GroupFixtureRow {
                            key: "{fixture.id}",
                            tournament_id: tournament_id.clone(),
                            tournament_name: tournament_name.clone(),
                            group_name: group.name.clone(),
                            fixture: fixture.clone(),
                            teams: teams.clone(),
                            on_open_group_match,
                            on_push,
                        }
                    }
                }
            }

            if !generated && group.team_ids.len() >= 2 {
                div { class: "fgc-tournament-add",
                    select {
                        class: "fgc-native-select",
                        aria_label: "Fixture team 1",
                        value: "{fixture_team1_value}",
                        onchange: move |event| {
                            let value = event.value();
                            fixture_team1.set(if value.is_empty() { None } else { Some(TeamId::from_raw(value)) });
                        },
                        option { value: "", "Team 1" }
                        for (id, name) in member_names.iter() {
                            option {
                                key: "{id}",
                                value: "{id}",
                                selected: id.as_str() == fixture_team1_value.as_str(),
                                "{name}"
                            }
                        }
                    }
                    select {
                        class: "fgc-native-select",
                        aria_label: "Fixture team 2",
                        value: "{fixture_team2_value}",
                        onchange: move |event| {
                            let value = event.value();
                            fixture_team2.set(if value.is_empty() { None } else { Some(TeamId::from_raw(value)) });
                        },
                        option { value: "", "Team 2" }
                        for (id, name) in member_names.iter() {
                            option {
                                key: "{id}",
                                value: "{id}",
                                selected: id.as_str() == fixture_team2_value.as_str(),
                                "{name}"
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        disabled: fixture_team1().is_none() || fixture_team2().is_none(),
                        onclick: move |_| {
                            let (Some(team1), Some(team2)) = (fixture_team1(), fixture_team2()) else {
                                return;
                            };
                            store.add_tournament_group_fixture(
                                &add_fixture_id,
                                &add_fixture_group_id,
                                team1,
                                team2,
                            );
                            fixture_team1.set(None);
                            fixture_team2.set(None);
                        },
                        "Add fixture"
                    }
                }
            }
        }
        Dialog {
            open: team_to_remove().is_some(),
            on_open_change: move |is_open: bool| if !is_open { team_to_remove.set(None) },
            if let Some((team_id, team_name)) = team_to_remove() {
                {
                    let confirm_id = tournament_id.clone();
                    let confirm_group_id = group.id.clone();
                    let current_team_ids = group.team_ids.clone();
                    let trigger = format!("{remove_team_trigger_id}-{team_id}");
                    rsx! {
                        DialogTitle { "Remove {team_name} from {group.name}?" }
                        DialogDescription {
                            "This removes the team from the group and discards its fixtures and results."
                        }
                        div { class: "fgc-dialog-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                id: "{remove_team_cancel_id}",
                                onclick: move |_| {
                                    team_to_remove.set(None);
                                    focus_element_later(&trigger);
                                },
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    let remaining: Vec<TeamId> = current_team_ids
                                        .iter()
                                        .filter(|id| **id != team_id)
                                        .cloned()
                                        .collect();
                                    store.set_tournament_group_teams(
                                        &confirm_id,
                                        &confirm_group_id,
                                        remaining,
                                    );
                                    team_to_remove.set(None);
                                    focus_element_later(&focus_after_remove);
                                },
                                "Remove team"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GroupFixtureRow(
    tournament_id: TournamentId,
    tournament_name: String,
    group_name: String,
    fixture: GroupMatch,
    teams: Vec<Team>,
    on_open_group_match: EventHandler<MatchId>,
    on_push: EventHandler<PushTarget>,
) -> Element {
    let store = use_context::<AppStore>();
    let name1 = team_option_name(&teams, fixture.team1.as_ref());
    let name2 = team_option_name(&teams, fixture.team2.as_ref());
    let edit_id = fixture.id.clone();
    let remove_id = fixture.id.clone();
    let push_id = fixture.id.clone();
    let push_title = tournament_name;
    let push_subtitle = group_name;
    let push_team_a = fixture.team1.clone();
    let push_team_b = fixture.team2.clone();

    rsx! {
        div { class: "fgc-fixture",
            span { class: "fgc-fixture__team", "{name1}" }
            span { class: "fgc-fixture__score", "{fixture.score1}" }
            span { class: "fgc-fixture__dash", "–" }
            span { class: "fgc-fixture__score", "{fixture.score2}" }
            span { class: "fgc-fixture__team", "{name2}" }
            div { class: "fgc-fixture__actions",
                span {
                    class: "fgc-fixture__status fgc-fixture__status--{match_status_key(fixture.status)}",
                    "{match_status_label(fixture.status)}"
                }
                button {
                    r#type: "button",
                    class: "fgc-icon-button",
                    id: "fgc-edit-group-fixture-{edit_id}",
                    title: "Edit match details, score, status and winner",
                    aria_label: "Edit group match details for {name1} versus {name2}",
                    onclick: move |_| on_open_group_match.call(edit_id.clone()),
                    "✎"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Xs,
                    id: "fgc-push-fixture-{push_id}",
                    title: "Push to match — replaces the current Match Controller match after confirmation. Tournament graphics are separate in v1.",
                    aria_label: "Push group match {name1} versus {name2} to Match Controller",
                    onclick: move |_| {
                        let trigger_id = format!("fgc-push-fixture-{push_id}");
                        on_push.call(PushTarget {
                            title: push_title.clone(),
                            subtitle: push_subtitle.clone(),
                            team_a: push_team_a.clone(),
                            team_b: push_team_b.clone(),
                            score1: fixture.score1,
                            score2: fixture.score2,
                            status: fixture.status,
                            first_to: fixture.first_to,
                            trigger_id,
                        });
                    },
                    "↗"
                }
                button {
                    r#type: "button",
                    class: "fgc-icon-button fgc-icon-button--danger",
                    title: "Remove fixture",
                    aria_label: "Remove fixture {name1} versus {name2}",
                    onclick: move |_| {
                        store.remove_tournament_group_fixture(&tournament_id, &remove_id);
                    },
                    "×"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[component]
fn SettingsTab(tournament: Tournament, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let tournament_id = tournament.id.clone();
    let tiebreakers = tournament.tiebreakers.clone();
    let unused: Vec<Tiebreaker> = [
        Tiebreaker::Wins,
        Tiebreaker::HeadToHead,
        Tiebreaker::MapDifferential,
        Tiebreaker::MapsWon,
        Tiebreaker::PointsFor,
        Tiebreaker::PointsAgainst,
        Tiebreaker::Seed,
    ]
    .into_iter()
    .filter(|item| !tiebreakers.contains(item))
    .collect();
    let mut pending_tiebreaker = use_signal(String::new);
    let champion_value = if tournament.champion_manual {
        tournament
            .champion
            .as_ref()
            .map(|id| id.as_str().to_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let champion_id = tournament_id.clone();
    let reset_tiebreakers_id = tournament_id.clone();
    let add_tiebreaker_id = tournament_id.clone();

    rsx! {
        div { class: "fgc-tournament-stack",
            Panel {
                title: "Tiebreakers".to_owned(),
                description: "Applied in order to sort the standings.".to_owned(),
                ol { class: "fgc-tiebreakers",
                    for (index, item) in tiebreakers.iter().enumerate() {
                        {
                            let remove_id = tournament_id.clone();
                            let up_id = tournament_id.clone();
                            let down_id = tournament_id.clone();
                            let mut without = tiebreakers.clone();
                            without.remove(index);
                            let can_up = index > 0;
                            let can_down = index + 1 < tiebreakers.len();
                            let mut up_order = tiebreakers.clone();
                            if can_up {
                                up_order.swap(index, index - 1);
                            }
                            let mut down_order = tiebreakers.clone();
                            if can_down {
                                down_order.swap(index, index + 1);
                            }
                            rsx! {
                                li { key: "{tiebreaker_key(*item)}", class: "fgc-tiebreaker",
                                    span { class: "fgc-tiebreaker__index", "{index + 1}" }
                                    span { class: "fgc-tiebreaker__name", "{tiebreaker_label(*item)}" }
                                    div { class: "fgc-tiebreaker__actions",
                                        button {
                                            r#type: "button",
                                            class: "fgc-icon-button",
                                            title: "Move up",
                                            aria_label: "Move up",
                                            disabled: !can_up,
                                            onclick: move |_| {
                                                if can_up {
                                                    store.set_tournament_tiebreakers(&up_id, up_order.clone());
                                                }
                                            },
                                            "↑"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "fgc-icon-button",
                                            title: "Move down",
                                            aria_label: "Move down",
                                            disabled: !can_down,
                                            onclick: move |_| {
                                                if can_down {
                                                    store.set_tournament_tiebreakers(&down_id, down_order.clone());
                                                }
                                            },
                                            "↓"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "fgc-icon-button fgc-icon-button--danger",
                                            title: "Remove criterion",
                                            aria_label: "Remove criterion",
                                            onclick: move |_| {
                                                store.set_tournament_tiebreakers(&remove_id, without.clone());
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !unused.is_empty() {
                    div { class: "fgc-tournament-add",
                        select {
                            class: "fgc-native-select",
                            value: "{pending_tiebreaker()}",
                            onchange: move |event| pending_tiebreaker.set(event.value()),
                            option { value: "", "Add criterion" }
                            for item in unused.iter() {
                                option {
                                    key: "{tiebreaker_key(*item)}",
                                    value: "{tiebreaker_key(*item)}",
                                    selected: tiebreaker_key(*item) == pending_tiebreaker().as_str(),
                                    "{tiebreaker_label(*item)}"
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            disabled: pending_tiebreaker().is_empty(),
                            onclick: move |_| {
                                let value = pending_tiebreaker();
                                if value.is_empty() {
                                    return;
                                }
                                let mut next = tiebreakers.clone();
                                next.push(tiebreaker_from_key(&value));
                                store.set_tournament_tiebreakers(&add_tiebreaker_id, next);
                                pending_tiebreaker.set(String::new());
                            },
                            "Add"
                        }
                    }
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            store.reset_tournament_tiebreakers(&reset_tiebreakers_id);
                        },
                        "Reset to default"
                    }
                }
            }

            Panel {
                title: "Playoff".to_owned(),
                div { class: "fgc-form-row",
                    Field { label: "Qualifiers per group".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Qualifiers per group",
                            value: "{tournament.playoff_qualifiers}",
                            onchange: {
                                let id = tournament_id.clone();
                                move |event| {
                                    if let Ok(value) = event.value().parse::<u8>() {
                                        store.set_tournament_playoff_qualifiers(&id, value);
                                    }
                                }
                            },
                            for value in 1..=8u8 {
                                option {
                                    value: "{value}",
                                    selected: value == tournament.playoff_qualifiers,
                                    "{value}"
                                }
                            }
                        }
                    }
                    Field { label: "Format".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Playoff format",
                            value: "{format_key(tournament.playoff_format)}",
                            onchange: {
                                let id = tournament_id.clone();
                                move |event| {
                                    store.set_tournament_playoff_format(&id, format_from_key(&event.value()));
                                }
                            },
                            option { value: "single-elim", "Single elimination" }
                            option { value: "double-elim", "Double elimination" }
                        }
                    }
                }
            }

            Panel {
                title: "Champion".to_owned(),
                div { class: "fgc-form-row",
                    Field { label: "Champion".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Champion",
                            value: "{champion_value}",
                            onchange: move |event| {
                                let value = event.value();
                                let champion = if value.is_empty() {
                                    None
                                } else {
                                    Some(TeamId::from_raw(value))
                                };
                                store.set_tournament_champion(&champion_id, champion);
                            },
                            option { value: "", "Derived from the bracket" }
                            for entrant in entrants_in_seed_order(&tournament) {
                                {
                                    let name = teams
                                        .iter()
                                        .find(|team| team.id == entrant.team_id)
                                        .map_or_else(|| entrant.team_id.to_string(), |team| team.name.clone());
                                    rsx! {
                                        option {
                                            key: "{entrant.team_id}",
                                            value: "{entrant.team_id}",
                                            selected: tournament.champion.as_ref() == Some(&entrant.team_id),
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Field { label: "Current".to_owned(),
                        p { class: "fgc-champion",
                            if let Some(champion) = tournament.champion.as_ref() {
                                "{teams.iter().find(|team| &team.id == champion).map_or_else(|| champion.to_string(), |team| team.name.clone())}"
                            } else {
                                "No champion yet"
                            }
                            if tournament.champion_manual {
                                span { class: "fgc-badge fgc-badge--manual", "Manual" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Match dialog
// ---------------------------------------------------------------------------

#[component]
fn MatchDialog(
    tournament: Tournament,
    match_id: MatchId,
    teams: Vec<Team>,
    on_close: EventHandler<()>,
) -> Element {
    let store = use_context::<AppStore>();
    let generated = tournament_is_generated(&tournament);
    let tournament_id = tournament.id.clone();
    let Some(game) = find_match(&tournament, &match_id).cloned() else {
        return rsx! {};
    };
    let Some((address_section, address_round, address_match)) =
        match_address(&tournament, &match_id)
    else {
        return rsx! {};
    };
    let address = BracketSlot {
        section: address_section,
        round: address_round,
        match_index: address_match,
        slot: 1,
    };

    let mut score1 = use_signal(|| game.score1.to_string());
    let mut score2 = use_signal(|| game.score2.to_string());
    let mut status = use_signal(|| editable_match_status(game.status));
    let mut first_to = use_signal(|| game.first_to);
    let mut next_target = use_signal(|| game.next.as_ref().map(|link| link.match_id.clone()));
    let mut next_slot = use_signal(|| game.next.as_ref().map_or(1, |link| link.slot));
    let mut loser_target =
        use_signal(|| game.loser_next.as_ref().map(|link| link.match_id.clone()));
    let mut loser_slot = use_signal(|| game.loser_next.as_ref().map_or(1, |link| link.slot));
    let mut error = use_signal(String::new);
    let mut confirming_delete = use_signal(|| false);
    let delete_trigger_id = format!("fgc-match-delete-{match_id}");
    let delete_cancel_id = format!("fgc-match-delete-cancel-{match_id}");

    let mut options: Vec<(MatchId, String)> = Vec::new();
    let sections = [
        (BracketSection::Winners, &tournament.winners),
        (BracketSection::Losers, &tournament.losers),
        (BracketSection::GrandFinal, &tournament.grand_final),
    ];
    for (section, rounds) in sections {
        for round in rounds {
            for candidate in &round.matches {
                if candidate.id != match_id {
                    options.push((
                        candidate.id.clone(),
                        format!(
                            "{} · {} · M{}",
                            section_label(section),
                            round.name,
                            candidate.number
                        ),
                    ));
                }
            }
        }
    }
    let next_target_value = next_target()
        .map(|id| id.as_str().to_owned())
        .unwrap_or_default();
    let loser_target_value = loser_target()
        .map(|id| id.as_str().to_owned())
        .unwrap_or_default();
    let override_value = if game.winner_manual {
        game.winner
            .as_ref()
            .map(|id| id.as_str().to_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let original_next = game.next.clone();
    let original_loser = game.loser_next.clone();
    let slot1 = game.slot1.clone();
    let slot2 = game.slot2.clone();
    let dependent_links = tournament
        .winners
        .iter()
        .chain(tournament.losers.iter())
        .chain(tournament.grand_final.iter())
        .flat_map(|round| round.matches.iter())
        .filter(|candidate| {
            candidate
                .next
                .as_ref()
                .is_some_and(|link| link.match_id == match_id)
                || candidate
                    .loser_next
                    .as_ref()
                    .is_some_and(|link| link.match_id == match_id)
        })
        .count();

    rsx! {
        Dialog {
            open: true,
            on_open_change: move |is_open: bool| if !is_open { on_close.call(()) },
            DialogTitle { "Edit match" }
            DialogDescription {
                "{section_label(address.section)} · match M{game.number} · first to {game.first_to}"
            }

            div { class: "fgc-form-row",
                Field { label: "Slot 1".to_owned(),
                    select {
                        class: "fgc-native-select",
                        aria_label: "Slot 1",
                        value: slot1.as_ref().map(|id| id.as_str().to_owned()).unwrap_or_default(),
                        onchange: {
                            let id = tournament_id.clone();
                            move |event| {
                                let value = event.value();
                                let team = if value.is_empty() { None } else { Some(TeamId::from_raw(value)) };
                                if let Err(message) = store.set_tournament_slot(&id, address, team) {
                                    error.set(message.to_string());
                                }
                            }
                        },
                        option { value: "", "Empty" }
                        for team in teams.iter() {
                            option {
                                key: "{team.id}",
                                value: "{team.id}",
                                selected: slot1.as_ref() == Some(&team.id),
                                "{team.name}"
                            }
                        }
                    }
                }
                Field { label: "Slot 2".to_owned(),
                    select {
                        class: "fgc-native-select",
                        aria_label: "Slot 2",
                        value: slot2.as_ref().map(|id| id.as_str().to_owned()).unwrap_or_default(),
                        onchange: {
                            let id = tournament_id.clone();
                            move |event| {
                                let value = event.value();
                                let team = if value.is_empty() { None } else { Some(TeamId::from_raw(value)) };
                                if let Err(message) = store.set_tournament_slot(
                                    &id,
                                    BracketSlot { slot: 2, ..address },
                                    team,
                                ) {
                                    error.set(message.to_string());
                                }
                            }
                        },
                        option { value: "", "Empty" }
                        for team in teams.iter() {
                            option {
                                key: "{team.id}",
                                value: "{team.id}",
                                selected: slot2.as_ref() == Some(&team.id),
                                "{team.name}"
                            }
                        }
                    }
                }
            }

            div { class: "fgc-form-row",
                Field { label: "Score 1".to_owned(),
                    Input {
                        r#type: "number",
                        min: "0",
                        max: "99",
                        aria_label: "Score 1",
                        value: "{score1()}",
                        oninput: move |event: FormEvent| score1.set(event.value()),
                    }
                }
                Field { label: "Score 2".to_owned(),
                    Input {
                        r#type: "number",
                        min: "0",
                        max: "99",
                        aria_label: "Score 2",
                        value: "{score2()}",
                        oninput: move |event: FormEvent| score2.set(event.value()),
                    }
                }
                Field { label: "Status".to_owned(),
                    div { class: "fgc-status-toggle",
                        Switch {
                            checked: status() == BracketMatchStatus::Completed,
                            aria_label: "Match status — completed",
                            on_checked_change: move |completed: bool| {
                                status.set(if completed {
                                    BracketMatchStatus::Completed
                                } else {
                                    BracketMatchStatus::Pending
                                });
                            },
                        }
                        span { class: "fgc-status-toggle__label", "{match_status_label(status())}" }
                    }
                }
                Field { label: "First to".to_owned(),
                    select {
                        class: "fgc-native-select",
                        aria_label: "First to",
                        value: "{first_to()}",
                        onchange: {
                            let id = tournament_id.clone();
                            let id_for_match = match_id.clone();
                            move |event| {
                                if let Ok(value) = event.value().parse::<u8>() {
                                    first_to.set(value);
                                    store.set_tournament_match_first_to(&id, &id_for_match, value);
                                }
                            }
                        },
                        for value in 1..=4u8 {
                            option {
                                value: "{value}",
                                selected: value == first_to(),
                                "{value}"
                            }
                        }
                    }
                }
            }

            details { class: "fgc-advanced",
                summary { "Advanced settings" }
                div { class: "fgc-advanced__fields",
                    Field { label: "Winner".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Winner",
                            value: "{override_value}",
                            onchange: {
                                let id = tournament_id.clone();
                                let id_for_match = match_id.clone();
                                move |event| {
                                    let value = event.value();
                                    let winner = if value.is_empty() { None } else { Some(TeamId::from_raw(value)) };
                                    if !store.set_tournament_winner(&id, &id_for_match, winner) {
                                        error.set("That team is not in either slot.".to_owned());
                                    }
                                }
                            },
                            option { value: "", "Derived from scores" }
                            if let Some(team) = game.slot1.as_ref() {
                                {
                                    let name = team_option_name(&teams, Some(team));
                                    rsx! {
                                        option {
                                            value: "{team}",
                                            selected: team.as_str() == override_value.as_str(),
                                            "Slot 1 · {name}"
                                        }
                                    }
                                }
                            }
                            if let Some(team) = game.slot2.as_ref() {
                                {
                                    let name = team_option_name(&teams, Some(team));
                                    rsx! {
                                        option {
                                            value: "{team}",
                                            selected: team.as_str() == override_value.as_str(),
                                            "Slot 2 · {name}"
                                        }
                                    }
                                }
                            }
                        }
                        if game.winner_manual {
                            p { class: "fgc-note", "Clearing this override falls back to the scores." }
                        }
                    }
                    Field { label: "Winner advances to".to_owned(),
                        div { class: "fgc-link-row",
                            select {
                                class: "fgc-native-select",
                                aria_label: "Winner target match",
                                value: "{next_target_value}",
                                onchange: {
                                    let id = tournament_id.clone();
                                    let id_for_match = match_id.clone();
                                    let fallback = original_next.clone();
                                    move |event| {
                                        let value = event.value();
                                        let target = if value.is_empty() { None } else { Some(MatchId::from_raw(value)) };
                                        next_target.set(target.clone());
                                        let link = target.map(|target| MatchLink { match_id: target, slot: next_slot() });
                                        if let Err(message) = store.set_tournament_link(&id, &id_for_match, LinkKind::Winner, link) {
                                            error.set(message.to_string());
                                            next_target.set(fallback.as_ref().map(|link| link.match_id.clone()));
                                        }
                                    }
                                },
                                option { value: "", "None" }
                                for (id, label) in options.iter() {
                                    option {
                                        key: "{id}",
                                        value: "{id}",
                                        selected: id.as_str() == next_target_value.as_str(),
                                        "{label}"
                                    }
                                }
                            }
                            select {
                                class: "fgc-native-select fgc-link-row__slot",
                                aria_label: "Winner target slot",
                                value: "{next_slot()}",
                                onchange: {
                                    let id = tournament_id.clone();
                                    let id_for_match = match_id.clone();
                                    move |event| {
                                        let slot = event.value().parse::<u8>().unwrap_or(1);
                                        next_slot.set(slot);
                                        if let Some(target) = next_target() {
                                            let link = MatchLink { match_id: target, slot };
                                            if let Err(message) = store.set_tournament_link(&id, &id_for_match, LinkKind::Winner, Some(link)) {
                                                error.set(message.to_string());
                                            }
                                        }
                                    }
                                },
                                option { value: "1", "Slot 1" }
                                option { value: "2", "Slot 2" }
                            }
                        }
                    }
                    Field { label: "Loser drops to".to_owned(),
                        div { class: "fgc-link-row",
                            select {
                                class: "fgc-native-select",
                                aria_label: "Loser target match",
                                value: "{loser_target_value}",
                                onchange: {
                                    let id = tournament_id.clone();
                                    let id_for_match = match_id.clone();
                                    let fallback = original_loser.clone();
                                    move |event| {
                                        let value = event.value();
                                        let target = if value.is_empty() { None } else { Some(MatchId::from_raw(value)) };
                                        loser_target.set(target.clone());
                                        let link = target.map(|target| MatchLink { match_id: target, slot: loser_slot() });
                                        if let Err(message) = store.set_tournament_link(&id, &id_for_match, LinkKind::Loser, link) {
                                            error.set(message.to_string());
                                            loser_target.set(fallback.as_ref().map(|link| link.match_id.clone()));
                                        }
                                    }
                                },
                                option { value: "", "None" }
                                for (id, label) in options.iter() {
                                    option {
                                        key: "{id}",
                                        value: "{id}",
                                        selected: id.as_str() == loser_target_value.as_str(),
                                        "{label}"
                                    }
                                }
                            }
                            select {
                                class: "fgc-native-select fgc-link-row__slot",
                                aria_label: "Loser target slot",
                                value: "{loser_slot()}",
                                onchange: {
                                    let id = tournament_id.clone();
                                    let id_for_match = match_id.clone();
                                    move |event| {
                                        let slot = event.value().parse::<u8>().unwrap_or(1);
                                        loser_slot.set(slot);
                                        if let Some(target) = loser_target() {
                                            let link = MatchLink { match_id: target, slot };
                                            if let Err(message) = store.set_tournament_link(&id, &id_for_match, LinkKind::Loser, Some(link)) {
                                                error.set(message.to_string());
                                            }
                                        }
                                    }
                                },
                                option { value: "1", "Slot 1" }
                                option { value: "2", "Slot 2" }
                            }
                        }
                    }
                    p { class: "fgc-note", "A target slot replaces its previous source." }
                }
            }

            if !error().is_empty() {
                p { class: "fgc-field__error", role: "alert", "{error()}" }
            }

            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| on_close.call(()),
                    "Close"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: {
                        let id = tournament_id.clone();
                        let id_for_match = match_id.clone();
                        move |_| {
                            store.reset_tournament_match(&id, &id_for_match);
                            on_close.call(());
                        }
                    },
                    "Reset result"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: {
                        let id = tournament_id.clone();
                        let id_for_match = match_id.clone();
                        move |_| {
                            let next_status = status();
                            let next_score1 = parse_score(&score1());
                            let next_score2 = parse_score(&score2());
                            if next_status == BracketMatchStatus::Completed
                                && next_score1 == next_score2
                                && !game.winner_manual
                            {
                                error.set("A completed match needs a winner. Set the scores or override the winner.".to_owned());
                                return;
                            }
                            if store.set_tournament_result(&id, &id_for_match, next_score1, next_score2, next_status) {
                                on_close.call(());
                            } else {
                                error.set("Could not save this result.".to_owned());
                            }
                        }
                    },
                    "Save result"
                }
            }

            if !generated {
                div { class: "fgc-dialog-danger",
                    if confirming_delete() {
                        p { class: "fgc-note",
                            "Delete this match and clear {dependent_links} dependent link(s)?"
                        }
                        div { class: "fgc-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                id: "{delete_cancel_id}",
                                onclick: {
                                    let trigger = delete_trigger_id.clone();
                                    move |_| {
                                        confirming_delete.set(false);
                                        focus_element_later(&trigger);
                                    }
                                },
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                onclick: {
                                    let id = tournament_id.clone();
                                    let id_for_match = match_id.clone();
                                    move |_| {
                                        store.remove_tournament_match(&id, &id_for_match);
                                        on_close.call(());
                                        focus_element_later("fgc-tournament-tab-board");
                                    }
                                },
                                "Delete match"
                            }
                        }
                    } else {
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            id: "{delete_trigger_id}",
                            onclick: {
                                let cancel = delete_cancel_id.clone();
                                move |_| {
                                    confirming_delete.set(true);
                                    focus_element_later(&cancel);
                                }
                            },
                            "Delete match"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Group match details dialog
// ---------------------------------------------------------------------------

#[component]
fn GroupFixtureDialog(
    tournament: Tournament,
    fixture_id: MatchId,
    teams: Vec<Team>,
    on_close: EventHandler<()>,
) -> Element {
    let store = use_context::<AppStore>();
    let tournament_id = tournament.id.clone();
    let Some(fixture) = tournament
        .group_matches
        .iter()
        .find(|item| item.id == fixture_id)
        .cloned()
    else {
        return rsx! {};
    };
    let group_name = tournament
        .groups
        .iter()
        .find(|group| group.id == fixture.group)
        .map_or_else(|| "Group".to_owned(), |group| group.name.clone());
    let name1 = team_option_name(&teams, fixture.team1.as_ref());
    let name2 = team_option_name(&teams, fixture.team2.as_ref());

    let mut score1 = use_signal(|| fixture.score1);
    let mut score2 = use_signal(|| fixture.score2);
    let mut status = use_signal(|| editable_match_status(fixture.status));
    let mut first_to = use_signal(|| fixture.first_to);
    let mut winner_override = use_signal(|| {
        if fixture.winner_manual {
            fixture.winner.clone()
        } else {
            None
        }
    });
    let mut error = use_signal(String::new);

    let score_winner = if score1() > score2() {
        fixture.team1.clone()
    } else if score2() > score1() {
        fixture.team2.clone()
    } else {
        None
    };
    let effective_winner = winner_override().or(score_winner);
    let winner_text = if let Some(team) = effective_winner.as_ref() {
        format!("{} wins", team_option_name(&teams, Some(team)))
    } else if status() == BracketMatchStatus::Completed {
        "Draw — no winner".to_owned()
    } else {
        "No winner yet".to_owned()
    };

    rsx! {
        Dialog {
            open: true,
            on_open_change: move |is_open: bool| if !is_open { on_close.call(()) },
            DialogTitle { "Group match details" }
            DialogDescription {
                "{group_name} · {name1} vs {name2} · first to {first_to()}"
            }
            div { class: "fgc-form-row",
                Field { label: "Score 1".to_owned(),
                    Input {
                        r#type: "number",
                        min: "0",
                        max: "99",
                        aria_label: "Score for {name1}",
                        value: "{score1()}",
                        oninput: move |event: FormEvent| score1.set(parse_score(&event.value())),
                        onchange: {
                            let commit_id = tournament_id.clone();
                            let commit_fixture = fixture.id.clone();
                            move |event: FormEvent| {
                                let value = parse_score(&event.value());
                                score1.set(value);
                                match store.set_tournament_group_result(
                                    &commit_id,
                                    &commit_fixture,
                                    value,
                                    score2(),
                                    status(),
                                ) {
                                    Ok(()) => error.set(String::new()),
                                    Err(message) => error.set(message),
                                }
                            }
                        },
                    }
                }
                Field { label: "Score 2".to_owned(),
                    Input {
                        r#type: "number",
                        min: "0",
                        max: "99",
                        aria_label: "Score for {name2}",
                        value: "{score2()}",
                        oninput: move |event: FormEvent| score2.set(parse_score(&event.value())),
                        onchange: {
                            let commit_id = tournament_id.clone();
                            let commit_fixture = fixture.id.clone();
                            move |event: FormEvent| {
                                let value = parse_score(&event.value());
                                score2.set(value);
                                match store.set_tournament_group_result(
                                    &commit_id,
                                    &commit_fixture,
                                    score1(),
                                    value,
                                    status(),
                                ) {
                                    Ok(()) => error.set(String::new()),
                                    Err(message) => error.set(message),
                                }
                            }
                        },
                    }
                }
                Field { label: "Status".to_owned(),
                    div { class: "fgc-status-toggle",
                        Switch {
                            checked: status() == BracketMatchStatus::Completed,
                            aria_label: "Match status — completed",
                            on_checked_change: {
                                let commit_id = tournament_id.clone();
                                let commit_fixture = fixture.id.clone();
                                move |completed: bool| {
                                    let next = if completed {
                                        BracketMatchStatus::Completed
                                    } else {
                                        BracketMatchStatus::Pending
                                    };
                                    status.set(next);
                                    if !completed {
                                        winner_override.set(None);
                                    }
                                    match store.set_tournament_group_result(
                                        &commit_id,
                                        &commit_fixture,
                                        score1(),
                                        score2(),
                                        next,
                                    ) {
                                        Ok(()) => error.set(String::new()),
                                        Err(message) => error.set(message),
                                    }
                                }
                            },
                        }
                        span { class: "fgc-status-toggle__label", "{match_status_label(status())}" }
                    }
                }
                Field { label: "First to".to_owned(),
                    select {
                        class: "fgc-native-select",
                        aria_label: "First to",
                        value: "{first_to()}",
                        onchange: {
                            let commit_id = tournament_id.clone();
                            let commit_fixture = fixture.id.clone();
                            move |event| {
                                if let Ok(value) = event.value().parse::<u8>() {
                                    let value = value.clamp(1, 4);
                                    first_to.set(value);
                                    if !store.set_tournament_group_first_to(
                                        &commit_id,
                                        &commit_fixture,
                                        value,
                                    ) {
                                        error.set("Group fixture not found.".to_owned());
                                    }
                                }
                            }
                        },
                        for value in 1..=4u8 {
                            option {
                                value: "{value}",
                                selected: value == first_to(),
                                "{value}"
                            }
                        }
                    }
                }
            }
            if !error().is_empty() {
                p { class: "fgc-field__error", role: "alert", "{error()}" }
            }
            details { class: "fgc-advanced",
                summary { "Advanced settings" }
                div { class: "fgc-advanced__fields",
                    Field { label: "Winner".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Winner",
                            value: winner_override().map_or_else(String::new, |id| id.as_str().to_owned()),
                            onchange: {
                                let commit_id = tournament_id.clone();
                                let commit_fixture = fixture.id.clone();
                                move |event| {
                                    let value = event.value();
                                    let winner = if value.is_empty() {
                                        None
                                    } else {
                                        Some(TeamId::from_raw(value))
                                    };
                                    if store.set_tournament_group_winner(
                                        &commit_id,
                                        &commit_fixture,
                                        winner.clone(),
                                    ) {
                                        winner_override.set(winner);
                                        if winner_override().is_some() {
                                            status.set(BracketMatchStatus::Completed);
                                        }
                                        error.set(String::new());
                                    } else {
                                        error.set("That team is not in this fixture.".to_owned());
                                    }
                                }
                            },
                            option { value: "", "Derived from scores" }
                            if let Some(team) = fixture.team1.as_ref() {
                                option {
                                    value: "{team}",
                                    "Slot 1 · {name1}"
                                }
                            }
                            if let Some(team) = fixture.team2.as_ref() {
                                option {
                                    value: "{team}",
                                    "Slot 2 · {name2}"
                                }
                            }
                        }
                        if winner_override().is_some() {
                            p { class: "fgc-note", "Clearing this override falls back to the scores." }
                        }
                    }
                }
            }
            Field { label: "Effective winner".to_owned(),
                p { class: "fgc-group-winner", "{winner_text}" }
                p { class: "fgc-note", "Derived from scores unless overridden above. Results save automatically." }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| on_close.call(()),
                    "Done"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_label(format: BracketFormat) -> &'static str {
    match format {
        BracketFormat::SingleElim => "Single elimination",
        BracketFormat::DoubleElim => "Double elimination",
        BracketFormat::RoundRobin => "Round robin",
        BracketFormat::MultiStage => "Multi-stage",
    }
}

fn format_key(format: BracketFormat) -> &'static str {
    match format {
        BracketFormat::SingleElim => "single-elim",
        BracketFormat::DoubleElim => "double-elim",
        BracketFormat::RoundRobin => "round-robin",
        BracketFormat::MultiStage => "multi-stage",
    }
}

fn format_from_key(value: &str) -> BracketFormat {
    match value {
        "double-elim" => BracketFormat::DoubleElim,
        "round-robin" => BracketFormat::RoundRobin,
        "multi-stage" => BracketFormat::MultiStage,
        _ => BracketFormat::SingleElim,
    }
}

fn status_label(status: TournamentStatus) -> &'static str {
    match status {
        TournamentStatus::Setup => "Setup",
        TournamentStatus::InProgress => "In progress",
        TournamentStatus::Completed => "Completed",
    }
}

fn status_key(status: TournamentStatus) -> &'static str {
    match status {
        TournamentStatus::Setup => "setup",
        TournamentStatus::InProgress => "in-progress",
        TournamentStatus::Completed => "completed",
    }
}

fn status_from_key(value: &str) -> TournamentStatus {
    match value {
        "in-progress" => TournamentStatus::InProgress,
        "completed" => TournamentStatus::Completed,
        _ => TournamentStatus::Setup,
    }
}

fn editable_match_status(status: BracketMatchStatus) -> BracketMatchStatus {
    match status {
        BracketMatchStatus::Completed => BracketMatchStatus::Completed,
        BracketMatchStatus::Pending | BracketMatchStatus::Live => BracketMatchStatus::Pending,
    }
}

fn match_status_label(status: BracketMatchStatus) -> &'static str {
    match status {
        BracketMatchStatus::Pending | BracketMatchStatus::Live => "Upcoming",
        BracketMatchStatus::Completed => "Complete",
    }
}

fn match_status_key(status: BracketMatchStatus) -> &'static str {
    match status {
        BracketMatchStatus::Pending | BracketMatchStatus::Live => "upcoming",
        BracketMatchStatus::Completed => "complete",
    }
}

fn section_label(section: BracketSection) -> &'static str {
    match section {
        BracketSection::Winners => "Winners bracket",
        BracketSection::Losers => "Losers bracket",
        BracketSection::GrandFinal => "Grand final",
    }
}

/// Accessible name for a bracket slot: the visible seed and team plus the
/// section, match and slot, so the board is usable with a screen reader.
fn slot_aria_label(section: BracketSection, match_number: u32, slot: u8, label: &str) -> String {
    format!(
        "{} · M{match_number} · slot {slot}: {label}",
        section_label(section)
    )
}

fn section_key(section: BracketSection) -> &'static str {
    match section {
        BracketSection::Winners => "winners",
        BracketSection::Losers => "losers",
        BracketSection::GrandFinal => "grand-final",
    }
}

fn tiebreaker_label(tiebreaker: Tiebreaker) -> &'static str {
    match tiebreaker {
        Tiebreaker::Wins => "Wins",
        Tiebreaker::HeadToHead => "Head-to-head",
        Tiebreaker::MapDifferential => "Map differential",
        Tiebreaker::MapsWon => "Maps won",
        Tiebreaker::PointsFor => "Points for",
        Tiebreaker::PointsAgainst => "Points against",
        Tiebreaker::Seed => "Seed",
    }
}

fn tiebreaker_key(tiebreaker: Tiebreaker) -> &'static str {
    match tiebreaker {
        Tiebreaker::Wins => "wins",
        Tiebreaker::HeadToHead => "head-to-head",
        Tiebreaker::MapDifferential => "map-differential",
        Tiebreaker::MapsWon => "maps-won",
        Tiebreaker::PointsFor => "points-for",
        Tiebreaker::PointsAgainst => "points-against",
        Tiebreaker::Seed => "seed",
    }
}

fn tiebreaker_from_key(value: &str) -> Tiebreaker {
    match value {
        "head-to-head" => Tiebreaker::HeadToHead,
        "map-differential" => Tiebreaker::MapDifferential,
        "maps-won" => Tiebreaker::MapsWon,
        "points-for" => Tiebreaker::PointsFor,
        "points-against" => Tiebreaker::PointsAgainst,
        "seed" => Tiebreaker::Seed,
        _ => Tiebreaker::Wins,
    }
}

fn team_option_name(teams: &[Team], id: Option<&TeamId>) -> String {
    let Some(id) = id else {
        return "—".to_owned();
    };
    teams
        .iter()
        .find(|team| &team.id == id)
        .map_or_else(|| id.to_string(), |team| team.name.clone())
}

fn slot_dom_id(address: BracketSlot) -> String {
    format!(
        "fgc-slot-{}-{}-{}-{}",
        section_key(address.section),
        address.round,
        address.match_index,
        address.slot
    )
}

fn parse_score(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(0).min(99)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_slot_swap_keeps_provisional_multistage_bracket() {
        let mut source = fgc_core::new_tournament("Preview", BracketFormat::MultiStage);
        let teams: Vec<TeamId> = (0..8).map(|_| TeamId::new()).collect();
        fgc_core::set_entrants(&mut source, teams);
        core_generate_multistage(&mut source, 2).expect("multi-stage generation should succeed");

        let (mut projected, is_projected) = bracket_preview_for(&source);
        assert!(is_projected);
        assert!(has_bracket_rounds(&projected));

        let left = BracketSlot {
            section: BracketSection::Winners,
            round: 0,
            match_index: 0,
            slot: 1,
        };
        let right = BracketSlot {
            section: BracketSection::Winners,
            round: 0,
            match_index: 1,
            slot: 1,
        };
        let before_left = projected.winners[0].matches[0].slot1.clone();
        let before_right = projected.winners[0].matches[1].slot1.clone();

        swap_preview_slots(&mut projected, left, right).expect("preview swap should succeed");

        assert_eq!(projected.format, BracketFormat::MultiStage);
        assert!(has_bracket_rounds(&projected));
        assert_eq!(projected.winners[0].matches[0].slot1, before_right);
        assert_eq!(projected.winners[0].matches[1].slot1, before_left);
    }

    #[test]
    fn bracket_formats_show_board_and_round_robin_shows_groups() {
        assert_eq!(
            TournamentTab::for_format(BracketFormat::SingleElim),
            vec![
                TournamentTab::Setup,
                TournamentTab::Board,
                TournamentTab::Settings
            ]
        );
        assert_eq!(
            TournamentTab::for_format(BracketFormat::DoubleElim),
            vec![
                TournamentTab::Setup,
                TournamentTab::Board,
                TournamentTab::Settings
            ]
        );
        assert_eq!(
            TournamentTab::for_format(BracketFormat::RoundRobin),
            vec![
                TournamentTab::Setup,
                TournamentTab::Groups,
                TournamentTab::Settings
            ]
        );
        assert_eq!(
            TournamentTab::for_format(BracketFormat::MultiStage),
            vec![
                TournamentTab::Setup,
                TournamentTab::Groups,
                TournamentTab::Board,
                TournamentTab::Settings
            ]
        );
    }

    #[test]
    fn seed_reorder_updates_preview_generation() {
        let ids: Vec<TeamId> = (0..4).map(|_| TeamId::new()).collect();
        let mut target = fgc_core::new_tournament("Preview", BracketFormat::SingleElim);
        core_set_entrants(&mut target, ids.iter().cloned());

        let reordered = reordered_seed_ids(&ids, &ids[3], Some(&ids[1]), None);
        assert_eq!(
            reordered,
            vec![
                ids[0].clone(),
                ids[3].clone(),
                ids[1].clone(),
                ids[2].clone()
            ]
        );
        assert_eq!(reordered_seed_ids(&ids, &ids[3], None, Some(2)), reordered);

        regenerate_elimination_preview(&mut target, &reordered);

        assert_eq!(target.entrants[1].team_id, ids[3]);
        let first = &target.winners[0].matches[0];
        assert_eq!(first.slot1, Some(ids[0].clone()));
        assert_eq!(first.slot2, Some(ids[2].clone()));
    }

    #[test]
    fn double_elimination_geometry_follows_lower_links_and_marks_drops() {
        let ids: Vec<TeamId> = (0..8).map(|_| TeamId::new()).collect();
        let mut tournament = fgc_core::new_tournament("Preview", BracketFormat::DoubleElim);
        core_set_entrants(&mut tournament, ids.iter().cloned());
        core_generate_double_elimination(&mut tournament);

        let (_, _, _, links, drops) = bracket_geometry(BracketSection::Losers, &tournament.losers);
        assert_eq!(
            links,
            vec![
                "M 248 108 H 276 V 108 H 304".to_owned(),
                "M 248 324 H 276 V 324 H 304".to_owned(),
                "M 552 108 H 580 V 216 H 608".to_owned(),
                "M 552 324 H 580 V 216 H 608".to_owned(),
                "M 856 216 H 884 V 216 H 912".to_owned(),
            ]
        );
        assert!(drops.is_empty());

        let (_, _, _, winner_links, winner_drops) =
            bracket_geometry(BracketSection::Winners, &tournament.winners);
        let winner_matches: usize = tournament
            .winners
            .iter()
            .map(|round| round.matches.len())
            .sum();
        assert_eq!(winner_links.len(), 6);
        assert_eq!(winner_drops.len(), winner_matches);
        assert!(
            winner_drops
                .iter()
                .all(|path| path.starts_with("M ") && path.contains(" V "))
        );
    }
}
