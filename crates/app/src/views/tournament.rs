//! Tournament destination: setup, editable bracket board, groups and settings.
//!
//! The board is the source of truth. Every slot, match, round and link can be
//! changed by hand; the core helpers keep propagation, byes, standings and the
//! champion in sync after each edit.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use fgc_core::{
    BracketFormat, BracketMatch, BracketMatchStatus, BracketSection, BracketSlot, Group,
    GroupMatch, LinkKind, MatchId, MatchLink, Round, Standing, Team, TeamId, Tiebreaker,
    Tournament, TournamentId, TournamentStatus, entrants_in_seed_order, find_match, match_address,
    rounds as bracket_rounds,
};

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
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
    const ALL: [Self; 4] = [Self::Setup, Self::Board, Self::Groups, Self::Settings];

    const fn label(self) -> &'static str {
        match self {
            Self::Setup => "Setup",
            Self::Board => "Board",
            Self::Groups => "Groups",
            Self::Settings => "Settings",
        }
    }
}

/// Ephemeral board interaction state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BoardState {
    /// Slot picked up by click or keyboard.
    selected: Option<BracketSlot>,
    /// Board zoom percentage.
    zoom: u16,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            selected: None,
            zoom: 100,
        }
    }
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
    let mut delete_target = use_signal(|| None::<TournamentId>);

    let tournaments = state.read().tournaments.clone();
    let teams = state.read().teams.clone();
    let selected = selected_id().and_then(|id| {
        tournaments
            .iter()
            .find(|tournament| tournament.id == id)
            .cloned()
    });

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-tournaments",
                Panel {
                    title: format!("Tournaments ({})", tournaments.len()),
                    description: "Select a tournament to set it up and run its board.".to_owned(),
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let id = store.create_tournament("New Tournament", BracketFormat::SingleElim);
                                selected_id.set(Some(id));
                                tab.set(TournamentTab::Setup);
                                board.set(BoardState::default());
                                match_dialog.set(None);
                            },
                            "+ New tournament"
                        }
                    }
                    if tournaments.is_empty() {
                        div { class: "fgc-asset-empty",
                            p { class: "fgc-asset-empty__title", "No tournaments yet." }
                            p { class: "fgc-note", "Create one to add entrants and generate a bracket." }
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

                if let Some(tournament) = selected.clone() {
                    TournamentDetail {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                        tab: tab(),
                        on_tab: move |next| tab.set(next),
                        board: board(),
                        on_board: move |next| board.set(next),
                        on_open_match: move |id| match_dialog.set(Some(id)),
                        on_delete: move |id| delete_target.set(Some(id)),
                    }
                } else {
                    Panel {
                        title: "No tournament selected".to_owned(),
                        description: "Choose a tournament from the list, or create a new one.".to_owned(),
                        div { class: "fgc-empty", "Select a tournament to edit its setup and board." }
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
                on_close: move |_| match_dialog.set(None),
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
                                delete_target.set(None);
                            },
                            "Delete tournament"
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
    on_delete: EventHandler<TournamentId>,
) -> Element {
    let tournament_id = tournament.id.clone();
    let champion = tournament
        .champion
        .as_ref()
        .and_then(|id| teams.iter().find(|team| &team.id == id))
        .map(|team| team.name.clone());

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
                for item in TournamentTab::ALL {
                    button {
                        key: "{item.label()}",
                        r#type: "button",
                        aria_pressed: item == tab,
                        class: if item == tab {
                            "fgc-tab fgc-tab--active"
                        } else {
                            "fgc-tab"
                        },
                        onclick: move |_| on_tab.call(item),
                        "{item.label()}"
                    }
                }
            }

            {match tab {
                TournamentTab::Setup => rsx! {
                    SetupTab { tournament: tournament.clone(), teams: teams.clone() }
                },
                TournamentTab::Board => rsx! {
                    BoardTab {
                        tournament: tournament.clone(),
                        teams: teams.clone(),
                        board,
                        on_board,
                        on_open_match,
                    }
                },
                TournamentTab::Groups => rsx! {
                    GroupsTab { tournament: tournament.clone(), teams: teams.clone() }
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
    let mut group_count = use_signal(|| 2usize);
    let max_groups = entrants.len().clamp(1, 8);

    let name_id = tournament_id.clone();
    let format_id = tournament_id.clone();
    let first_to_id = tournament_id.clone();
    let reset_id = tournament_id.clone();
    let status_id = tournament_id.clone();
    let add_id = tournament_id.clone();
    let generate_id = tournament_id.clone();
    let round_robin_id = tournament_id.clone();

    rsx! {
        div { class: "fgc-tournament-stack",
            Panel {
                title: "Tournament details".to_owned(),
                description: "Name, format and the defaults used by new matches.".to_owned(),
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
                            for format in [BracketFormat::SingleElim, BracketFormat::DoubleElim, BracketFormat::RoundRobin] {
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
                    hint: Some("Play a second grand final when the losers-bracket team wins the first.".to_owned()),
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
                description: "Seeds run top to bottom. Generation places byes against the highest seeds.".to_owned(),
                if available.is_empty() {
                    p { class: "fgc-note", "Every team is already entered, or no teams exist yet." }
                } else {
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
                    p { class: "fgc-note", "Seeds are renumbered after every add, remove or reorder." }
                }
            }

            Panel {
                title: "Generate".to_owned(),
                description: "Generation replaces the current board. Everything it creates can still be edited.".to_owned(),
                if entrants.len() < 2 {
                    p { class: "fgc-note", "At least two entrants are required." }
                } else if tournament.format == BracketFormat::RoundRobin {
                    div { class: "fgc-tournament-add",
                        Field { label: "Groups".to_owned(),
                            select {
                                class: "fgc-native-select",
                                aria_label: "Groups",
                                value: "{group_count()}",
                                onchange: move |event| {
                                    if let Ok(value) = event.value().parse::<usize>() {
                                        group_count.set(value);
                                    }
                                },
                                for value in 1..=max_groups {
                                    option {
                                        value: "{value}",
                                        selected: value == group_count(),
                                        "{value}"
                                    }
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                store.generate_tournament_round_robin(&round_robin_id, group_count());
                            },
                            "Generate round robin"
                        }
                    }
                } else {
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| store.generate_tournament_bracket(&generate_id),
                            "Generate {format_label(tournament.format)} bracket"
                        }
                    }
                }
                if !tournament.winners.is_empty() || !tournament.losers.is_empty() {
                    p { class: "fgc-note",
                        "A board already exists. Regenerating discards its rounds, links and results."
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
) -> Element {
    let store = use_context::<AppStore>();
    let tournament_id = tournament.id.clone();
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

    let click_id = tournament_id.clone();
    let on_slot_click = EventHandler::new(move |address: BracketSlot| match board.selected {
        None => on_board.call(BoardState {
            selected: Some(address),
            ..board
        }),
        Some(selected) if selected == address => on_board.call(BoardState {
            selected: None,
            ..board
        }),
        Some(selected) => {
            if let Err(error) = store.swap_tournament_slots(&click_id, selected, address) {
                store.report_error(error.to_string());
            }
            on_board.call(BoardState {
                selected: None,
                ..board
            });
        }
    });

    let key_tournament = tournament.clone();
    let on_slot_key = EventHandler::new(move |(address, key): (BracketSlot, Key)| match key {
        Key::Escape => on_board.call(BoardState {
            selected: None,
            ..board
        }),
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            if let Some(target) = adjacent_slot(&key_tournament, address, key) {
                focus_slot(target);
            }
        }
        // Enter and Space activate the native button, which calls `onclick`.
        _ => {}
    });

    rsx! {
        div { class: "fgc-board",
            div { class: "fgc-board__toolbar",
                div { class: "fgc-board__hint",
                    p { class: "fgc-board__title", "Bracket board" }
                    p { class: "fgc-note",
                        "Click a slot, then click another to swap. Enter picks up, Escape cancels, arrows move focus."
                    }
                }
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
                                on_board.call(BoardState { zoom: value, ..board });
                            }
                        }
                    }
                    span { class: "fgc-board__zoom-value", "{board.zoom}%" }
                }
            }

            if !has_board {
                Panel {
                    title: "No board yet".to_owned(),
                    description: "Generate a bracket from the Setup tab, then edit it here.".to_owned(),
                    div { class: "fgc-empty", "The board appears once the tournament has rounds." }
                }
            } else {
                div { class: "fgc-board__canvas", style: "zoom: {board.zoom}%;",
                    for (section, rounds) in sections {
                        if !rounds.is_empty() {
                            BoardSection {
                                key: "{section_key(section)}",
                                tournament_id: tournament_id.clone(),
                                section,
                                rounds,
                                teams: teams.clone(),
                                seeds: seeds.clone(),
                                selected: board.selected,
                                on_slot_click,
                                on_slot_key,
                                on_open_match,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoardSection(
    tournament_id: TournamentId,
    section: BracketSection,
    rounds: Vec<Round>,
    teams: Vec<Team>,
    seeds: HashMap<TeamId, u32>,
    selected: Option<BracketSlot>,
    on_slot_click: EventHandler<BracketSlot>,
    on_slot_key: EventHandler<(BracketSlot, Key)>,
    on_open_match: EventHandler<MatchId>,
) -> Element {
    let store = use_context::<AppStore>();
    let add_round_id = tournament_id.clone();
    let round_count = rounds.len();

    rsx! {
        section { class: "fgc-bracket-section",
            div { class: "fgc-bracket-section__header",
                h3 { class: "fgc-bracket-section__title", "{section_label(section)}" }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        store.add_tournament_round(&add_round_id, section);
                    },
                    "Add round"
                }
            }
            div { class: "fgc-bracket-scroll",
                div { class: "fgc-bracket-grid",
                    for (round_index, round) in rounds.iter().enumerate() {
                        {
                            let rename_id = tournament_id.clone();
                            let remove_round_id = tournament_id.clone();
                            let add_match_id = tournament_id.clone();
                            let round_name = round.name.clone();
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
                                    for (match_index, game) in round.matches.iter().enumerate() {
                                        MatchCard {
                                            key: "{game.id}",
                                            tournament_id: tournament_id.clone(),
                                            game: game.clone(),
                                            section,
                                            round_index,
                                            match_index,
                                            round_len: round.matches.len(),
                                            teams: teams.clone(),
                                            seeds: seeds.clone(),
                                            selected,
                                            on_slot_click,
                                            on_slot_key,
                                            on_open_match,
                                        }
                                    }
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

#[component]
fn MatchCard(
    tournament_id: TournamentId,
    game: BracketMatch,
    section: BracketSection,
    round_index: usize,
    match_index: usize,
    round_len: usize,
    teams: Vec<Team>,
    seeds: HashMap<TeamId, u32>,
    selected: Option<BracketSlot>,
    on_slot_click: EventHandler<BracketSlot>,
    on_slot_key: EventHandler<(BracketSlot, Key)>,
    on_open_match: EventHandler<MatchId>,
) -> Element {
    let store = use_context::<AppStore>();
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

    let open_id = game.id.clone();
    let up_id = tournament_id.clone();
    let down_id = tournament_id.clone();
    let can_up = match_index > 0;
    let can_down = match_index + 1 < round_len;

    rsx! {
        div {
            class: "fgc-bracket-match fgc-bracket-match--{match_status_key(game.status)}",
            div { class: "fgc-bracket-match__header",
                span { class: "fgc-bracket-match__number", "M{game.number}" }
                span { class: "fgc-bracket-match__status", "{match_status_label(game.status)}" }
                div { class: "fgc-bracket-match__tools",
                    button {
                        r#type: "button",
                        class: "fgc-icon-button",
                        title: "Move match up",
                        aria_label: "Move match up",
                        disabled: !can_up,
                        onclick: move |_| {
                            if can_up {
                                store.move_tournament_match(
                                    &up_id,
                                    section,
                                    round_index,
                                    match_index,
                                    match_index - 1,
                                );
                            }
                        },
                        "↑"
                    }
                    button {
                        r#type: "button",
                        class: "fgc-icon-button",
                        title: "Move match down",
                        aria_label: "Move match down",
                        disabled: !can_down,
                        onclick: move |_| {
                            if can_down {
                                store.move_tournament_match(
                                    &down_id,
                                    section,
                                    round_index,
                                    match_index,
                                    match_index + 1,
                                );
                            }
                        },
                        "↓"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_open_match.call(open_id.clone()),
                        "Edit"
                    }
                }
            }
            BracketSlotButton {
                address: slot1_address,
                label: name1,
                seed: seed1,
                is_selected: selected == Some(slot1_address),
                is_winner: winner1,
                on_click: on_slot_click,
                on_key: on_slot_key,
            }
            BracketSlotButton {
                address: slot2_address,
                label: name2,
                seed: seed2,
                is_selected: selected == Some(slot2_address),
                is_winner: winner2,
                on_click: on_slot_click,
                on_key: on_slot_key,
            }
            div { class: "fgc-bracket-match__footer",
                span { class: "fgc-bracket-match__score", "{game.score1} – {game.score2}" }
                if game.winner_manual {
                    span { class: "fgc-badge fgc-badge--manual", "Manual winner" }
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
    is_selected: bool,
    is_winner: bool,
    on_click: EventHandler<BracketSlot>,
    on_key: EventHandler<(BracketSlot, Key)>,
) -> Element {
    let dom_id = slot_dom_id(address);
    let class = if is_selected && is_winner {
        "fgc-bracket-slot fgc-bracket-slot--selected fgc-bracket-slot--winner"
    } else if is_selected {
        "fgc-bracket-slot fgc-bracket-slot--selected"
    } else if is_winner {
        "fgc-bracket-slot fgc-bracket-slot--winner"
    } else {
        "fgc-bracket-slot"
    };

    rsx! {
        button {
            id: "{dom_id}",
            r#type: "button",
            class: "{class}",
            aria_pressed: is_selected,
            onclick: move |_| on_click.call(address),
            onkeydown: move |event: KeyboardEvent| on_key.call((address, event.key())),
            if let Some(seed) = seed {
                span { class: "fgc-bracket-slot__seed", "{seed}" }
            }
            span { class: "fgc-bracket-slot__name", "{label}" }
        }
    }
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[component]
fn GroupsTab(tournament: Tournament, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let tournament_id = tournament.id.clone();
    let mut group_count = use_signal(|| 2usize);
    let max_groups = tournament.entrants.len().clamp(1, 8);
    let generate_id = tournament_id.clone();
    let add_id = tournament_id.clone();
    let playoff_id = tournament_id.clone();
    let mut playoff_error = use_signal(String::new);
    let groups = tournament.groups.clone();
    let next_group_name = format!("Group {}", groups.len() + 1);
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

    rsx! {
        div { class: "fgc-tournament-stack",
            Panel {
                title: "Group stage".to_owned(),
                description: "Split entrants into round-robin groups, then edit teams and fixtures freely.".to_owned(),
                if tournament.entrants.len() < 2 {
                    p { class: "fgc-note", "Add at least two entrants in Setup first." }
                } else {
                    div { class: "fgc-tournament-add",
                        Field { label: "Groups".to_owned(),
                            select {
                                class: "fgc-native-select",
                                aria_label: "Groups",
                                value: "{group_count()}",
                                onchange: move |event| {
                                    if let Ok(value) = event.value().parse::<usize>() {
                                        group_count.set(value);
                                    }
                                },
                                for value in 1..=max_groups {
                                    option {
                                        value: "{value}",
                                        selected: value == group_count(),
                                        "{value}"
                                    }
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                store.generate_tournament_round_robin(&generate_id, group_count());
                            },
                            "Generate round robin"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                store.create_tournament_group(&add_id, &next_group_name, Vec::new());
                            },
                            "+ Group"
                        }
                    }
                }
                p { class: "fgc-note",
                    "Generating replaces the bracket board. Adding a team regenerates only its missing fixtures."
                }
            }

            if groups.is_empty() {
                Panel {
                    title: "No groups".to_owned(),
                    description: "Generate round robin, or add an empty group and pick its teams.".to_owned(),
                    div { class: "fgc-empty", "Groups and fixtures appear here." }
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
                                group: group.clone(),
                                teams: teams.clone(),
                                entrant_names: entrant_names.clone(),
                                standings,
                                fixtures,
                            }
                        }
                    }
                }

                Panel {
                    title: "Playoff".to_owned(),
                    description: "Seed the qualifiers from every group into a new bracket.".to_owned(),
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
                        Field { label: "Playoff format".to_owned(),
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
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                if let Err(error) = store.generate_tournament_playoff(&playoff_id) {
                                    playoff_error.set(error);
                                } else {
                                    playoff_error.set(String::new());
                                }
                            },
                            "Generate playoff"
                        }
                    }
                    if !playoff_error().is_empty() {
                        p { class: "fgc-field__error", role: "alert", "{playoff_error()}" }
                    }
                    p { class: "fgc-note",
                        "Playoff generation replaces the current bracket with the group qualifiers in seed order."
                    }
                }
            }
        }
    }
}

#[component]
fn GroupPanel(
    tournament_id: TournamentId,
    group: Group,
    teams: Vec<Team>,
    entrant_names: HashMap<TeamId, String>,
    standings: Vec<Standing>,
    fixtures: Vec<GroupMatch>,
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

    rsx! {
        Panel {
            title: group.name.clone(),
            description: "Round-robin group. Teams, fixtures and results can be changed at any time.".to_owned(),
            div { class: "fgc-group-header",
                Input {
                    value: "{group.name}",
                    aria_label: "Group name",
                    onchange: move |event: FormEvent| {
                        store.rename_tournament_group(&rename_id, &rename_group_id, &event.value());
                    },
                }
                if confirming_delete() {
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| confirming_delete.set(false),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            store.remove_tournament_group(&remove_id, &delete_group_id);
                        },
                        "Delete group"
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| confirming_delete.set(true),
                        "Delete group"
                    }
                }
            }
            if confirming_delete() {
                p { class: "fgc-note",
                    "Deleting this group also removes its {fixtures.len()} fixture(s) and results. This cannot be undone."
                }
            }

            div { class: "fgc-group-teams",
                for (team_id, name) in member_names.iter() {
                    {
                        let remove_team_id = tournament_id.clone();
                        let group_id = group.id.clone();
                        let team_id_for_remove = team_id.clone();
                        let team_label = name.clone();
                        let remaining: Vec<TeamId> = group
                            .team_ids
                            .iter()
                            .filter(|id| **id != team_id_for_remove)
                            .cloned()
                            .collect();
                        rsx! {
                            span { key: "{team_id}", class: "fgc-chip",
                                span { "{team_label}" }
                                button {
                                    r#type: "button",
                                    class: "fgc-chip__remove",
                                    title: "Remove from group",
                                    aria_label: "Remove from group",
                                    onclick: move |_| {
                                        store.set_tournament_group_teams(
                                            &remove_team_id,
                                            &group_id,
                                            remaining.clone(),
                                        );
                                    },
                                    "×"
                                }
                            }
                        }
                    }
                }
            }

            if !available.is_empty() {
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
                            tr { key: "{row.team_id}",
                                td { "{entrant_names.get(&row.team_id).cloned().unwrap_or_else(|| row.team_id.to_string())}" }
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

            h4 { class: "fgc-group-subtitle", "Fixtures" }
            if fixtures.is_empty() {
                p { class: "fgc-note", "No fixtures yet." }
            } else {
                div { class: "fgc-fixture-list",
                    for fixture in fixtures {
                        GroupFixtureRow {
                            key: "{fixture.id}",
                            tournament_id: tournament_id.clone(),
                            fixture: fixture.clone(),
                            teams: teams.clone(),
                        }
                    }
                }
            }

            if group.team_ids.len() >= 2 {
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
    }
}

#[component]
fn GroupFixtureRow(tournament_id: TournamentId, fixture: GroupMatch, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let mut score1 = use_signal(|| fixture.score1.to_string());
    let mut score2 = use_signal(|| fixture.score2.to_string());
    let mut status = use_signal(|| fixture.status);
    let name1 = team_option_name(&teams, fixture.team1.as_ref());
    let name2 = team_option_name(&teams, fixture.team2.as_ref());
    let remove_id = fixture.id.clone();
    let save_id = fixture.id.clone();
    let save_tournament_id = tournament_id.clone();

    rsx! {
        div { class: "fgc-fixture",
            span { class: "fgc-fixture__team", "{name1}" }
            input {
                class: "fgc-fixture__score",
                r#type: "number",
                min: "0",
                max: "99",
                value: "{score1()}",
                aria_label: "Score for {name1}",
                oninput: move |event: FormEvent| score1.set(event.value()),
            }
            span { class: "fgc-fixture__dash", "–" }
            input {
                class: "fgc-fixture__score",
                r#type: "number",
                min: "0",
                max: "99",
                value: "{score2()}",
                aria_label: "Score for {name2}",
                oninput: move |event: FormEvent| score2.set(event.value()),
            }
            span { class: "fgc-fixture__team", "{name2}" }
            select {
                class: "fgc-native-select fgc-fixture__status",
                aria_label: "Match status",
                value: "{match_status_key(status())}",
                onchange: move |event| status.set(match_status_from_key(&event.value())),
                for item in [BracketMatchStatus::Pending, BracketMatchStatus::Live, BracketMatchStatus::Completed] {
                    option {
                        value: "{match_status_key(item)}",
                        selected: item == status(),
                        "{match_status_label(item)}"
                    }
                }
            }
            Button {
                variant: ButtonVariant::Secondary,
                size: ButtonSize::Sm,
                onclick: move |_| {
                    store.set_tournament_group_result(
                        &save_tournament_id,
                        &save_id,
                        parse_score(&score1()),
                        parse_score(&score2()),
                        status(),
                    );
                },
                "Save"
            }
            button {
                r#type: "button",
                class: "fgc-icon-button fgc-icon-button--danger",
                title: "Remove fixture",
                aria_label: "Remove fixture",
                onclick: move |_| {
                    store.remove_tournament_group_fixture(&tournament_id, &remove_id);
                },
                "×"
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
                description: "Applied in order to teams level on league points. Head-to-head uses a mini-table.".to_owned(),
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
                description: "How many teams advance from each group and what bracket they feed.".to_owned(),
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
                p { class: "fgc-note", "Generate the playoff from the Groups tab once standings are final." }
            }

            Panel {
                title: "Champion".to_owned(),
                description: "Auto-set from the final unless overridden here.".to_owned(),
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
    let mut status = use_signal(|| game.status);
    let mut first_to = use_signal(|| game.first_to);
    let mut next_target = use_signal(|| game.next.as_ref().map(|link| link.match_id.clone()));
    let mut next_slot = use_signal(|| game.next.as_ref().map_or(1, |link| link.slot));
    let mut loser_target =
        use_signal(|| game.loser_next.as_ref().map(|link| link.match_id.clone()));
    let mut loser_slot = use_signal(|| game.loser_next.as_ref().map_or(1, |link| link.slot));
    let mut error = use_signal(String::new);
    let mut confirming_delete = use_signal(|| false);

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
                    select {
                        class: "fgc-native-select",
                        aria_label: "Match status",
                        value: "{match_status_key(status())}",
                        onchange: move |event| status.set(match_status_from_key(&event.value())),
                        for item in [BracketMatchStatus::Pending, BracketMatchStatus::Live, BracketMatchStatus::Completed] {
                            option {
                                value: "{match_status_key(item)}",
                                selected: item == status(),
                                "{match_status_label(item)}"
                            }
                        }
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
                    p { class: "fgc-note", "This winner is a manual override. Clearing it falls back to the scores." }
                }
            }

            div { class: "fgc-form-row",
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

            div { class: "fgc-dialog-danger",
                if confirming_delete() {
                    p { class: "fgc-note",
                        "Delete this match and clear {dependent_links} dependent link(s)?"
                    }
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| confirming_delete.set(false),
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
                                }
                            },
                            "Delete match"
                        }
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| confirming_delete.set(true),
                        "Delete match"
                    }
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
    }
}

fn format_key(format: BracketFormat) -> &'static str {
    match format {
        BracketFormat::SingleElim => "single-elim",
        BracketFormat::DoubleElim => "double-elim",
        BracketFormat::RoundRobin => "round-robin",
    }
}

fn format_from_key(value: &str) -> BracketFormat {
    match value {
        "double-elim" => BracketFormat::DoubleElim,
        "round-robin" => BracketFormat::RoundRobin,
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

fn match_status_label(status: BracketMatchStatus) -> &'static str {
    match status {
        BracketMatchStatus::Pending => "Pending",
        BracketMatchStatus::Live => "Live",
        BracketMatchStatus::Completed => "Completed",
    }
}

fn match_status_key(status: BracketMatchStatus) -> &'static str {
    match status {
        BracketMatchStatus::Pending => "pending",
        BracketMatchStatus::Live => "live",
        BracketMatchStatus::Completed => "completed",
    }
}

fn match_status_from_key(value: &str) -> BracketMatchStatus {
    match value {
        "live" => BracketMatchStatus::Live,
        "completed" => BracketMatchStatus::Completed,
        _ => BracketMatchStatus::Pending,
    }
}

fn section_label(section: BracketSection) -> &'static str {
    match section {
        BracketSection::Winners => "Winners bracket",
        BracketSection::Losers => "Losers bracket",
        BracketSection::GrandFinal => "Grand final",
    }
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

fn focus_slot(address: BracketSlot) {
    let id = slot_dom_id(address);
    let script = format!("document.getElementById('{id}')?.focus();");
    let _ = document::eval(&script);
}

fn adjacent_slot(tournament: &Tournament, address: BracketSlot, key: Key) -> Option<BracketSlot> {
    let rounds = bracket_rounds(tournament, address.section);
    match key {
        Key::ArrowUp | Key::ArrowDown => {
            let round = rounds.get(address.round)?;
            let delta: isize = if key == Key::ArrowUp { -1 } else { 1 };
            let index = isize::try_from(address.match_index).ok()? + delta;
            if index < 0 || usize::try_from(index).ok()? >= round.matches.len() {
                return None;
            }
            Some(BracketSlot {
                match_index: usize::try_from(index).ok()?,
                ..address
            })
        }
        Key::ArrowLeft | Key::ArrowRight => {
            let delta: isize = if key == Key::ArrowLeft { -1 } else { 1 };
            let index = isize::try_from(address.round).ok()? + delta;
            if index < 0 || usize::try_from(index).ok()? >= rounds.len() {
                return None;
            }
            let round_index = usize::try_from(index).ok()?;
            let round = rounds.get(round_index)?;
            if round.matches.is_empty() {
                return None;
            }
            Some(BracketSlot {
                round: round_index,
                match_index: address.match_index.min(round.matches.len() - 1),
                ..address
            })
        }
        _ => None,
    }
}

fn parse_score(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(0).min(99)
}
