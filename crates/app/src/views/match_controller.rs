//! Match Controller destination: the tabs that drive the live overlays.
//!
//! All persistent edits go through [`AppStore`], which validates and reprojects
//! them. Timer run state is deliberately ephemeral and owned above this view
//! (see [`crate::timer`]), so visiting another destination neither stops the
//! clock nor resets it; the persisted timer settings stay in
//! `AppState::settings`.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use fgc_core::{
    AssetCategory, AssetId, Match, MatchStatus, PlayerId, Scoring, Side, Talent, TalentId,
    TalentSlot, TalentType, Team, TeamId, TeamSide, ThemePreset, TournamentId, default_map_slots,
    empty_hero_bans, team_record as core_team_record,
};
use fgc_protocol::AssetCatalog;
use fgc_server::ServerStatus;

use crate::components::asset_picker::AssetPicker;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::components::{CollapsiblePanel, Disclosure, Field, Panel};
use crate::state::AppStore;
use crate::timer::{TimerController, broadcast_tick};
use crate::views::overlay::OverlayUrls;

/// The Match Controller tabs, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchTab {
    /// Teams, title, format, status, sides and scores.
    Setup,
    /// Map slots, scores and hero bans in one place.
    Maps,
    /// Caster, panel and interview assignments.
    Talent,
    /// Ordered map pool for the map-pool overlay.
    MapPool,
    /// Overlay appearance, branding and Browser Source URLs.
    Config,
}

impl MatchTab {
    const ALL: [Self; 5] = [
        Self::Setup,
        Self::Maps,
        Self::Talent,
        Self::MapPool,
        Self::Config,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Setup => "Match",
            Self::Maps => "Maps & Bans",
            Self::Talent => "Talent",
            Self::MapPool => "Map Pool",
            Self::Config => "Config",
        }
    }
}

/// The current match controller.
#[component]
pub fn MatchControllerView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let server = store.server();
    let match_state = state.read().current_match.clone();
    let status = server.status();
    let running = matches!(status, ServerStatus::Running { .. });
    let mut tab = use_signal(|| MatchTab::Setup);
    let timer = use_context::<TimerController>();

    use_effect(move || {
        let settings = state.read().settings.timer.clone();
        let current = (timer.state)();
        if !current.running {
            broadcast_tick(&store, &settings, current.total_seconds);
        }
    });

    let Some(match_state) = match_state else {
        return rsx! {
            div { class: "fgc-view",
                Panel {
                    title: "No current match".to_owned(),
                    description: "Create a match to start preparing the overlays.".to_owned(),
                    div { class: "fgc-empty",
                        p { "The match controller edits one current match at a time." }
                    }
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| store.create_match(),
                            "Create match"
                        }
                    }
                }
            }
        };
    };

    let teams = state.read().teams.clone();
    let talents = state.read().talents.clone();

    rsx! {
        div { class: "fgc-view fgc-match",
            div { class: "fgc-match__toolbar",
                div { class: "fgc-match__identity",
                    p { class: "fgc-match__title",
                        if match_state.title.is_empty() {
                            "Untitled match"
                        } else {
                            "{match_state.title}"
                        }
                    }
                    p { class: "fgc-match__subtitle",
                        if match_state.subtitle.is_empty() {
                            "No subtitle"
                        } else {
                            "{match_state.subtitle}"
                        }
                    }
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        disabled: !running,
                        onclick: move |_| store.force_update(),
                        "Force update"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.create_match(),
                        "New match"
                    }
                }
            }

            div { class: "fgc-tabs fgc-match__tabs",
                for item in MatchTab::ALL {
                    button {
                        key: "{item.label()}",
                        r#type: "button",
                        aria_pressed: tab() == item,
                        class: if tab() == item {
                            "fgc-tab fgc-tab--active"
                        } else {
                            "fgc-tab"
                        },
                        onclick: move |_| tab.set(item),
                        "{item.label()}"
                    }
                }
            }

            match tab() {
                MatchTab::Setup => rsx! {
                    MatchSetupTab { match_state: match_state.clone(), teams }
                },
                MatchTab::Maps => rsx! {
                    MapSelectionTab { match_state: match_state.clone(), teams: teams.clone() }
                },
                MatchTab::Talent => rsx! {
                    TalentTab { match_state: match_state.clone(), talents, teams }
                },
                MatchTab::MapPool => rsx! {
                    MapPoolTab { match_state: match_state.clone() }
                },
                MatchTab::Config => rsx! {
                    ConfigTab { match_state: match_state.clone() }
                },
            }
        }
    }
}

/// Match identity, teams, format, sides and scores.
#[component]
fn MatchSetupTab(match_state: Match, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let saved_matches = state.read().matches.clone();
    let tournaments = state.read().tournaments.clone();
    let (score_a, score_b) = fgc_protocol::scores(&match_state);
    let manual = matches!(match_state.scoring, Scoring::Manual { .. });
    let (manual_a, manual_b) = match &match_state.scoring {
        Scoring::Manual { a, b } => (*a, *b),
        Scoring::Derived => (score_a, score_b),
    };
    let team_a = match_state
        .team_a
        .as_ref()
        .and_then(|id| teams.iter().find(|team| &team.id == id))
        .cloned();
    let team_b = match_state
        .team_b
        .as_ref()
        .and_then(|id| teams.iter().find(|team| &team.id == id))
        .cloned();
    let tournament_value = match_state
        .tournament_id
        .as_ref()
        .map_or_else(String::new, |id| id.as_str().to_owned());
    let selected_tournament = match_state
        .tournament_id
        .as_ref()
        .and_then(|id| tournaments.iter().find(|tournament| &tournament.id == id));
    let auto_record = |team: Option<&Team>| -> String {
        selected_tournament
            .and_then(|tournament| team.and_then(|team| core_team_record(tournament, &team.id)))
            .unwrap_or_else(|| "0-0".to_owned())
    };
    let record_a_auto = auto_record(team_a.as_ref());
    let record_b_auto = auto_record(team_b.as_ref());
    let record_a_value = match_state.team_a_record.clone().unwrap_or_default();
    let record_b_value = match_state.team_b_record.clone().unwrap_or_default();
    let has_manual_record = [&record_a_value, &record_b_value]
        .iter()
        .any(|value| !value.trim().is_empty());
    let record_note = match selected_tournament {
        Some(tournament) if !has_manual_record => {
            format!(
                "From {} — placeholders show group win-loss where played, otherwise seed. Clear a record to use it.",
                tournament.name
            )
        }
        Some(_) => "Manual records override the selected tournament.".to_owned(),
        None => {
            "Optional. Choose a tournament to fill records from its standings or seed.".to_owned()
        }
    };
    let team_a_tag = team_a
        .as_ref()
        .map_or_else(|| "Team A".to_owned(), |team| team.tag.clone());
    let team_b_tag = team_b
        .as_ref()
        .map_or_else(|| "Team B".to_owned(), |team| team.tag.clone());
    let team_a_value = match_state
        .team_a
        .as_ref()
        .map_or_else(String::new, |id| id.as_str().to_owned());
    let team_b_value = match_state
        .team_b
        .as_ref()
        .map_or_else(String::new, |id| id.as_str().to_owned());

    rsx! {
        div { class: "fgc-grid fgc-grid--two",
            Panel {
                title: "Match info".to_owned(),
                div { class: "fgc-form-row",
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Title" }
                        Input {
                            value: "{match_state.title}",
                            placeholder: "e.g. Grand Finals",
                            oninput: move |event: FormEvent| store.update_match(|current| current.title = event.value()),
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Subtitle" }
                        Input {
                            value: "{match_state.subtitle}",
                            placeholder: "e.g. Week 4",
                            oninput: move |event: FormEvent| store.update_match(|current| current.subtitle = event.value()),
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Scheduled start" }
                        Input {
                            r#type: "datetime-local",
                            value: "{match_state.scheduled_at.clone().unwrap_or_default()}",
                            aria_label: "Scheduled start",
                            oninput: move |event: FormEvent| {
                                store.set_match_scheduled_at(event.value());
                            },
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Format" }
                        select {
                            class: "fgc-native-select",
                            value: "{match_state.first_to}",
                            onchange: move |event| {
                                if let Ok(value) = event.value().parse::<u8>() {
                                    store.set_match_first_to(value);
                                }
                            },
                            for value in 1..=4u8 {
                                option {
                                    value: "{value}",
                                    selected: value == match_state.first_to,
                                    "First to {value}"
                                }
                            }
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Status" }
                        select {
                            class: "fgc-native-select",
                            value: match match_state.status {
                                MatchStatus::Pre => "pre",
                                MatchStatus::Live => "live",
                                MatchStatus::Post => "post",
                            },
                            onchange: move |event| {
                                let status = match event.value().as_str() {
                                    "live" => MatchStatus::Live,
                                    "post" => MatchStatus::Post,
                                    _ => MatchStatus::Pre,
                                };
                                store.set_match_status(status);
                            },
                            option { value: "pre", "Upcoming" }
                            option { value: "live", "Live" }
                            option { value: "post", "Complete" }
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Side indicator" }
                        select {
                            class: "fgc-native-select",
                            value: "{match_state.side.as_u8()}",
                            onchange: move |event| {
                                let side = match event.value().as_str() {
                                    "1" => Side::HomeAttacks,
                                    "2" => Side::HomeDefends,
                                    _ => Side::Neutral,
                                };
                                store.set_match_side(side);
                            },
                            option { value: "0", "Neutral" }
                            option { value: "1", "Attack / Defend" }
                            option { value: "2", "Defend / Attack" }
                        }
                    }
                }
            }

            Panel {
                title: "Teams".to_owned(),
                label { class: "fgc-field",
                    span { class: "fgc-field__label", "Tournament" }
                    select {
                        class: "fgc-native-select",
                        value: "{tournament_value}",
                        onchange: move |event| {
                            let value = event.value();
                            store.update_match(|current| {
                                current.tournament_id = if value.is_empty() {
                                    None
                                } else {
                                    Some(TournamentId::from_raw(value))
                                };
                            });
                        },
                        option { value: "", "No tournament" }
                        for tournament in tournaments.iter() {
                            option {
                                key: "{tournament.id}",
                                value: "{tournament.id}",
                                selected: tournament.id.as_str() == tournament_value.as_str(),
                                "{tournament.name}"
                            }
                        }
                    }
                }
                p { class: "fgc-note", "{record_note}" }
                div { class: "fgc-match-teams",
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Team A" }
                        select {
                            class: "fgc-native-select",
                            value: "{team_a_value}",
                            onchange: move |event| {
                                let value = event.value();
                                store.set_match_team(
                                    TeamSide::A,
                                    if value.is_empty() { None } else { Some(TeamId::from_raw(value)) },
                                );
                            },
                            option { value: "", "Select team" }
                            for team in teams.iter() {
                                option {
                                    key: "{team.id}",
                                    value: "{team.id}",
                                    selected: team.id.as_str() == team_a_value.as_str(),
                                    "{team.tag} — {team.name}"
                                }
                            }
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Team A record" }
                        Input {
                            value: "{record_a_value}",
                            placeholder: "{record_a_auto}",
                            maxlength: "24",
                            aria_label: "Team A record",
                            oninput: move |event: FormEvent| {
                                let value = event.value();
                                store.update_match(move |current| {
                                    current.team_a_record = (!value.trim().is_empty()).then_some(value);
                                });
                            },
                        }
                    }
                    div { class: "fgc-match-teams__swap",
                        span { "VS" }
                        Switch {
                            checked: match_state.swap_sides,
                            aria_label: "Swap sides",
                            on_checked_change: move |checked| {
                                store.update_match(|current| current.swap_sides = checked);
                            },
                        }
                        span { class: "fgc-note", "Swap sides" }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Team B" }
                        select {
                            class: "fgc-native-select",
                            value: "{team_b_value}",
                            onchange: move |event| {
                                let value = event.value();
                                store.set_match_team(
                                    TeamSide::B,
                                    if value.is_empty() { None } else { Some(TeamId::from_raw(value)) },
                                );
                            },
                            option { value: "", "Select team" }
                            for team in teams.iter() {
                                option {
                                    key: "{team.id}",
                                    value: "{team.id}",
                                    selected: team.id.as_str() == team_b_value.as_str(),
                                    "{team.tag} — {team.name}"
                                }
                            }
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Team B record" }
                        Input {
                            value: "{record_b_value}",
                            placeholder: "{record_b_auto}",
                            maxlength: "24",
                            aria_label: "Team B record",
                            oninput: move |event: FormEvent| {
                                let value = event.value();
                                store.update_match(move |current| {
                                    current.team_b_record = (!value.trim().is_empty()).then_some(value);
                                });
                            },
                        }
                    }
                }
            }

            Panel {
                title: "Score".to_owned(),
                div { class: "fgc-match-score",
                    div { class: "fgc-match-score__team",
                        span { class: "fgc-match-score__tag", "{team_a_tag}" }
                        span { class: "fgc-match-score__value", "{score_a}" }
                    }
                    span { class: "fgc-match-score__dash", "–" }
                    div { class: "fgc-match-score__team",
                        span { class: "fgc-match-score__tag", "{team_b_tag}" }
                        span { class: "fgc-match-score__value", "{score_b}" }
                    }
                }
                p { class: "fgc-note",
                    if manual {
                        "Manual score override."
                    } else {
                        "Derived from completed map wins; draws are ignored."
                    }
                }
                div { class: "fgc-toggle-row",
                    div { class: "fgc-toggle-row__text",
                        p { class: "fgc-toggle-row__title", "Manual score override" }
                        p { class: "fgc-toggle-row__hint", "Enter the match score directly." }
                    }
                    Switch {
                        checked: manual,
                        aria_label: "Manual score override",
                        on_checked_change: move |checked| store.set_manual_scoring(checked),
                    }
                }
                if manual {
                    div { class: "fgc-form-row",
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "{team_a_tag}" }
                            Input {
                                r#type: "number",
                                value: "{manual_a}",
                                oninput: move |event: FormEvent| {
                                    store.set_manual_score(TeamSide::A, parse_score(&event.value()));
                                },
                            }
                        }
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "{team_b_tag}" }
                            Input {
                                r#type: "number",
                                value: "{manual_b}",
                                oninput: move |event: FormEvent| {
                                    store.set_manual_score(TeamSide::B, parse_score(&event.value()));
                                },
                            }
                        }
                    }
                }
            }

            MatchLibraryPanel { saved: saved_matches, teams }
        }
    }
}

/// Saved matches with their map pools, ready to load or move to another
/// machine. The current match stays live while this list is used.
#[component]
fn MatchLibraryPanel(saved: Vec<Match>, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();

    rsx! {
        CollapsiblePanel {
            title: format!("Match library ({})", saved.len()),
            description: "Save a snapshot of the current match, load one here, or export the whole library.".to_owned(),
            div { class: "fgc-match-library",
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.save_current_match(),
                        "Save current"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.export_matches(),
                        "Export…"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.import_matches(),
                        "Import…"
                    }
                }
                Disclosure {
                    id: "fgc-match-library-list",
                    title: format!("Saved matches ({})", saved.len()),
                    if saved.is_empty() {
                            p { class: "fgc-note", "No saved matches yet. Use Save current to add one." }
                        } else {
                            div { class: "fgc-match-library__list",
                                for entry in saved.iter() {
                                    div { class: "fgc-match-library__item", key: "{entry.id}",
                                        div { class: "fgc-match-library__info",
                                            p { class: "fgc-match-library__title",
                                                if entry.title.trim().is_empty() {
                                                    "Untitled match"
                                                } else {
                                                    "{entry.title}"
                                                }
                                            }
                                            p { class: "fgc-match-library__meta",
                                                "{saved_match_teams(entry, &teams)}"
                                                if let Some(scheduled) = &entry.scheduled_at {
                                                    " · {scheduled}"
                                                }
                                            }
                                        }
                                        div { class: "fgc-match-library__actions",
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                onclick: {
                                                    let id = entry.id.clone();
                                                    move |_| store.load_saved_match(id.clone())
                                                },
                                                "Load"
                                            }
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                onclick: {
                                                    let id = entry.id.clone();
                                                    move |_| store.delete_saved_match(id.clone())
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        p { class: "fgc-note",
                            "Map pools travel with each match. Import adds new matches without replacing teams, talent or assets."
                        }
                }
            }
        }
    }
}

fn saved_match_teams(entry: &Match, teams: &[Team]) -> String {
    let tag = |id: &Option<TeamId>| -> String {
        id.as_ref()
            .and_then(|id| teams.iter().find(|team| &team.id == id))
            .map_or_else(|| "TBD".to_owned(), |team| team.tag.clone())
    };
    format!("{} vs {}", tag(&entry.team_a), tag(&entry.team_b))
}

/// Full name of the team selected for a side, with a stable label when it is unset.
fn selected_team_label(id: &Option<TeamId>, teams: &[Team], fallback: &str) -> String {
    id.as_ref()
        .and_then(|id| teams.iter().find(|team| &team.id == id))
        .and_then(|team| {
            let name = team.name.trim();
            if !name.is_empty() {
                return Some(name.to_owned());
            }
            let tag = team.tag.trim();
            (!tag.is_empty()).then(|| tag.to_owned())
        })
        .unwrap_or_else(|| fallback.to_owned())
}

/// Map slots, map names, modes, scores and per-map hero bans.
#[component]
fn MapSelectionTab(match_state: Match, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let show_bans = state.read().settings.display.show_hero_bans;
    let catalog = store.catalog();
    let maps = match_state.maps.clone();
    let bans = match_state.hero_bans.clone();
    let map_count = maps.len();
    let expected_maps = default_map_slots(match_state.first_to);
    let mut custom_names = use_signal(BTreeMap::<usize, String>::new);
    let mut custom_modes = use_signal(BTreeMap::<usize, String>::new);
    let mut custom_hero_a = use_signal(BTreeMap::<usize, String>::new);
    let mut custom_hero_b = use_signal(BTreeMap::<usize, String>::new);
    let map_names: Vec<String> = catalog
        .in_category(AssetCategory::Map)
        .map(|asset| asset.name.clone())
        .collect();
    let mode_names: Vec<String> = catalog
        .in_category(AssetCategory::GameMode)
        .map(|asset| asset.name.clone())
        .collect();
    let heroes: Vec<String> = catalog
        .in_category(AssetCategory::Hero)
        .filter(|asset| catalog.is_selectable_hero(asset))
        .map(|asset| asset.name.clone())
        .collect();
    let team_a_label = selected_team_label(&match_state.team_a, &teams, "Team A");
    let team_b_label = selected_team_label(&match_state.team_b, &teams, "Team B");

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-match__section-header",
                p { class: "fgc-note",
                    "{map_count} map slot(s); first to {match_state.first_to} defaults to {expected_maps}. The first map is picked by {team_a_label}; later maps follow the loser of the previous map when loser picks is enabled. A map's mode follows its game mode in Assets; choose Custom to override one slot."
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.insert_map(map_count),
                        "+ Add map"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| store.reset_maps(),
                        "Reset all maps"
                    }
                    if show_bans {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| store.reset_hero_bans(),
                            "Reset all bans"
                        }
                    }
                }
            }

            div { class: "fgc-map-list",
                for (index, map) in maps.into_iter().enumerate() {
                    {
                        let catalog_for_name = catalog.clone();
                        let catalog_for_custom_name = catalog.clone();
                        let catalog_for_mode = catalog.clone();
                        let ban = bans.get(index).cloned().unwrap_or_else(empty_hero_bans);
                        let ban_team_a = ban.team_a.first().cloned().unwrap_or_default();
                        let ban_team_b = ban.team_b.first().cloned().unwrap_or_default();
                        let custom_a = custom_hero_a.peek().contains_key(&index);
                        let custom_b = custom_hero_b.peek().contains_key(&index);
                        let hero_a_draft = custom_hero_a
                            .peek()
                            .get(&index)
                            .cloned()
                            .unwrap_or_else(|| ban_team_a.clone());
                        let hero_b_draft = custom_hero_b
                            .peek()
                            .get(&index)
                            .cloned()
                            .unwrap_or_else(|| ban_team_b.clone());
                        let hero_a_value = if custom_a {
                            "__custom__".to_owned()
                        } else {
                            ban_team_a.clone()
                        };
                        let hero_b_value = if custom_b {
                            "__custom__".to_owned()
                        } else {
                            ban_team_b.clone()
                        };
                        let is_current = match_state.current_map == Some(index);
                        let chosen_by = map.chosen_by;
                        let loser_picks = match_state.loser_picks;
                        let name_is_custom = custom_names.peek().contains_key(&index);
                        let name_draft = custom_names
                            .peek()
                            .get(&index)
                            .cloned()
                            .unwrap_or_else(|| map.name.clone());
                        let name_value = if name_is_custom {
                            "__custom__".to_owned()
                        } else {
                            map.name.clone()
                        };
                        let mode_is_custom = custom_modes.peek().contains_key(&index);
                        let mode_draft = custom_modes
                            .peek()
                            .get(&index)
                            .cloned()
                            .unwrap_or_else(|| map.game_mode_name.clone());
                        let mode_value = if mode_is_custom {
                            "__custom__".to_owned()
                        } else {
                            map.game_mode_name.clone()
                        };
                        let label = format!("Map {}", index + 1);
                        rsx! {
                            Panel {
                                key: "{index}",
                                title: label,
                                div { class: "fgc-map-slot",
                                    div { class: "fgc-form-row",
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "Map name" }
                                            select {
                                                class: "fgc-native-select",
                                                aria_label: "Map name",
                                                value: name_value,
                                                onchange: move |event| {
                                                    let value = event.value();
                                                    if value == "__custom__" {
                                                        let draft = custom_names
                                                            .peek()
                                                            .get(&index)
                                                            .cloned()
                                                            .unwrap_or_default();
                                                        custom_names.write().insert(index, draft.clone());
                                                        let mode = catalog_for_name.game_mode_for_map_name(&draft);
                                                        store.set_map_name(index, &draft, &mode);
                                                    } else {
                                                        custom_names.write().remove(&index);
                                                        let mode = catalog_for_name.game_mode_for_map_name(&value);
                                                        store.set_map_name(index, &value, &mode);
                                                    }
                                                },
                                                option { value: "", "Select a map" }
                                                for name in map_names.iter() {
                                                    option { value: "{name}", selected: *name == map.name, "{name}" }
                                                }
                                                if !map.name.is_empty() && !map_names.contains(&map.name) {
                                                    option {
                                                        value: "{map.name}",
                                                        selected: name_value == map.name,
                                                        "{map.name} (imported)"
                                                    }
                                                }
                                                option {
                                                    value: "__custom__",
                                                    selected: name_is_custom,
                                                    "Custom…"
                                                }
                                            }
                                            if name_is_custom {
                                                Input {
                                                    value: name_draft,
                                                    aria_label: "Custom map name",
                                                    placeholder: "Type a map name",
                                                    oninput: move |event: FormEvent| {
                                                        let value = event.value();
                                                        custom_names.write().insert(index, value.clone());
                                                        let mode = catalog_for_custom_name.game_mode_for_map_name(&value);
                                                        store.set_map_name(index, &value, &mode);
                                                    },
                                                }
                                            }
                                        }
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "Game mode" }
                                            select {
                                                class: "fgc-native-select",
                                                aria_label: "Game mode",
                                                value: mode_value,
                                                onchange: move |event| {
                                                    let value = event.value();
                                                    if value == "__custom__" {
                                                        let draft = custom_modes
                                                            .peek()
                                                            .get(&index)
                                                            .cloned()
                                                            .unwrap_or_default();
                                                        custom_modes.write().insert(index, draft.clone());
                                                        store.set_map_mode(index, &draft);
                                                    } else {
                                                        custom_modes.write().remove(&index);
                                                        if value.is_empty() {
                                                            let derived = catalog_for_mode
                                                                .game_mode_for_map_name(&map.name);
                                                            store.set_map_name(index, &map.name, &derived);
                                                        } else {
                                                            store.set_map_mode(index, &value);
                                                        }
                                                    }
                                                },
                                                option { value: "", "From map" }
                                                for name in mode_names.iter() {
                                                    option { value: "{name}", selected: *name == map.game_mode_name, "{name}" }
                                                }
                                                if !map.game_mode_name.is_empty()
                                                    && !mode_names.contains(&map.game_mode_name)
                                                {
                                                    option {
                                                        value: "{map.game_mode_name}",
                                                        selected: mode_value == map.game_mode_name,
                                                        "{map.game_mode_name} (imported)"
                                                    }
                                                }
                                                option {
                                                    value: "__custom__",
                                                    selected: mode_is_custom,
                                                    "Custom…"
                                                }
                                            }
                                            if mode_is_custom {
                                                Input {
                                                    value: mode_draft,
                                                    aria_label: "Custom game mode",
                                                    placeholder: "Type a game mode",
                                                    oninput: move |event: FormEvent| {
                                                        let value = event.value();
                                                        custom_modes.write().insert(index, value.clone());
                                                        store.set_map_mode(index, &value);
                                                    },
                                                }
                                            }
                                        }
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "Chosen by" }
                                            select {
                                                class: "fgc-native-select",
                                                disabled: loser_picks,
                                                value: match chosen_by {
                                                    Some(TeamSide::A) => "A",
                                                    Some(TeamSide::B) => "B",
                                                    None => "",
                                                },
                                                onchange: move |event| {
                                                    let side = match event.value().as_str() {
                                                        "A" => Some(TeamSide::A),
                                                        "B" => Some(TeamSide::B),
                                                        _ => None,
                                                    };
                                                    store.set_map_chosen_by(index, side);
                                                },
                                                option { value: "", "None" }
                                                option { value: "A", "{team_a_label}" }
                                                option { value: "B", "{team_b_label}" }
                                            }
                                        }
                                    }

                                    div { class: "fgc-map-slot__row",
                                        div { class: "fgc-toggle-row fgc-toggle-row--inline",
                                            div { class: "fgc-toggle-row__text",
                                                p { class: "fgc-toggle-row__title", "Current map" }
                                            }
                                            Switch {
                                                checked: is_current,
                                                aria_label: "Current map",
                                                on_checked_change: move |checked: bool| {
                                                    store.set_current_map(checked.then_some(index));
                                                },
                                            }
                                        }
                                        div { class: "fgc-toggle-row fgc-toggle-row--inline",
                                            div { class: "fgc-toggle-row__text",
                                                p { class: "fgc-toggle-row__title", "Complete" }
                                            }
                                            Switch {
                                                checked: map.completed,
                                                aria_label: "Complete",
                                                on_checked_change: move |checked| {
                                                    store.set_map_completed(index, checked);
                                                },
                                            }
                                        }
                                        div { class: "fgc-actions",
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                disabled: index == 0,
                                                onclick: move |_| store.move_map(index, -1),
                                                "↑"
                                            }
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                disabled: index + 1 >= map_count,
                                                onclick: move |_| store.move_map(index, 1),
                                                "↓"
                                            }
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                size: ButtonSize::Sm,
                                                onclick: move |_| store.remove_map(index),
                                                "Remove"
                                            }
                                        }
                                    }

                                    div { class: "fgc-map-slot__scores",
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "{team_a_label} score" }
                                            Input {
                                                r#type: "number",
                                                value: "{map.score_a}",
                                                oninput: move |event: FormEvent| {
                                                    store.set_map_scores(
                                                        index,
                                                        parse_score(&event.value()),
                                                        map.score_b,
                                                    );
                                                },
                                            }
                                        }
                                        span { class: "fgc-map-slot__vs", "vs" }
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "{team_b_label} score" }
                                            Input {
                                                r#type: "number",
                                                value: "{map.score_b}",
                                                oninput: move |event: FormEvent| {
                                                    store.set_map_scores(
                                                        index,
                                                        map.score_a,
                                                        parse_score(&event.value()),
                                                    );
                                                },
                                            }
                                        }
                                    }

                                    if show_bans {
                                        Disclosure {
                                            class: "fgc-disclosure--compact fgc-map-slot__bans",
                                            title: "Hero bans".to_owned(),
                                                div { class: "fgc-form-row",
                                                    label { class: "fgc-field",
                                                        span { class: "fgc-field__label", "First ban" }
                                                        select {
                                                            class: "fgc-native-select",
                                                            aria_label: "First ban team",
                                                            value: match ban.first_ban_team {
                                                                Some(TeamSide::A) => "A",
                                                                Some(TeamSide::B) => "B",
                                                                None => "",
                                                            },
                                                            onchange: move |event| {
                                                                let side = match event.value().as_str() {
                                                                    "A" => Some(TeamSide::A),
                                                                    "B" => Some(TeamSide::B),
                                                                    _ => None,
                                                                };
                                                                store.set_first_ban_team(index, side);
                                                            },
                                                            option { value: "", "—" }
                                                            option { value: "A", "{team_a_label}" }
                                                            option { value: "B", "{team_b_label}" }
                                                        }
                                                    }
                                                    label { class: "fgc-field",
                                                        span { class: "fgc-field__label", "{team_a_label} ban" }
                                                        select {
                                                            class: "fgc-native-select",
                                                            aria_label: "{team_a_label} ban",
                                                            value: hero_a_value,
                                                            onchange: move |event| {
                                                                let value = event.value();
                                                                if value == "__custom__" {
                                                                    let draft = custom_hero_a
                                                                        .peek()
                                                                        .get(&index)
                                                                        .cloned()
                                                                        .unwrap_or_default();
                                                                    custom_hero_a.write().insert(index, draft.clone());
                                                                    store.set_hero_ban(index, TeamSide::A, &draft);
                                                                } else {
                                                                    custom_hero_a.write().remove(&index);
                                                                    store.set_hero_ban(index, TeamSide::A, &value);
                                                                }
                                                            },
                                                            option { value: "", "—" }
                                                            for name in heroes.iter() {
                                                                option { value: "{name}", selected: *name == ban_team_a, "{name}" }
                                                            }
                                                            if !ban_team_a.is_empty() && !heroes.contains(&ban_team_a) {
                                                                option {
                                                                    value: "{ban_team_a}",
                                                                    selected: hero_a_value == ban_team_a,
                                                                    "{ban_team_a} (imported)"
                                                                }
                                                            }
                                                            option {
                                                                value: "__custom__",
                                                                selected: custom_a,
                                                                "Custom…"
                                                            }
                                                        }
                                                        if custom_a {
                                                            Input {
                                                                value: hero_a_draft,
                                                                aria_label: "Custom {team_a_label} ban",
                                                                placeholder: "Type a hero name",
                                                                oninput: move |event: FormEvent| {
                                                                    let value = event.value();
                                                                    custom_hero_a.write().insert(index, value.clone());
                                                                    store.set_hero_ban(index, TeamSide::A, &value);
                                                                },
                                                            }
                                                        }
                                                    }
                                                    label { class: "fgc-field",
                                                        span { class: "fgc-field__label", "{team_b_label} ban" }
                                                        select {
                                                            class: "fgc-native-select",
                                                            aria_label: "{team_b_label} ban",
                                                            value: hero_b_value,
                                                            onchange: move |event| {
                                                                let value = event.value();
                                                                if value == "__custom__" {
                                                                    let draft = custom_hero_b
                                                                        .peek()
                                                                        .get(&index)
                                                                        .cloned()
                                                                        .unwrap_or_default();
                                                                    custom_hero_b.write().insert(index, draft.clone());
                                                                    store.set_hero_ban(index, TeamSide::B, &draft);
                                                                } else {
                                                                    custom_hero_b.write().remove(&index);
                                                                    store.set_hero_ban(index, TeamSide::B, &value);
                                                                }
                                                            },
                                                            option { value: "", "—" }
                                                            for name in heroes.iter() {
                                                                option { value: "{name}", selected: *name == ban_team_b, "{name}" }
                                                            }
                                                            if !ban_team_b.is_empty() && !heroes.contains(&ban_team_b) {
                                                                option {
                                                                    value: "{ban_team_b}",
                                                                    selected: hero_b_value == ban_team_b,
                                                                    "{ban_team_b} (imported)"
                                                                }
                                                            }
                                                            option {
                                                                value: "__custom__",
                                                                selected: custom_b,
                                                                "Custom…"
                                                            }
                                                        }
                                                        if custom_b {
                                                            Input {
                                                                value: hero_b_draft,
                                                                aria_label: "Custom {team_b_label} ban",
                                                                placeholder: "Type a hero name",
                                                                oninput: move |event: FormEvent| {
                                                                    let value = event.value();
                                                                    custom_hero_b.write().insert(index, value.clone());
                                                                    store.set_hero_ban(index, TeamSide::B, &value);
                                                                },
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

            if !show_bans {
                p { class: "fgc-note",
                    "Hero bans are hidden. Turn them on in Config → Overlay appearance."
                }
            }
        }
    }
}

/// Caster, panel and interview assignments.
#[component]
fn TalentTab(match_state: Match, talents: Vec<Talent>, teams: Vec<Team>) -> Element {
    let store = use_context::<AppStore>();
    let interviewee = match_state.talent.interviewee.clone().unwrap_or_default();
    let interviewee_source = if let Some(id) = interviewee.talent_id.as_ref() {
        format!("talent:{}", id.as_str())
    } else if let Some(id) = interviewee.player_id.as_ref() {
        format!("player:{}", id.as_str())
    } else {
        String::new()
    };
    let custom_name = interviewee.custom_name.clone().unwrap_or_default();
    let custom_subtext = interviewee.custom_subtext.clone().unwrap_or_default();
    let custom_name_for_subtext = custom_name.clone();
    let custom_subtext_for_name = custom_subtext.clone();
    let custom_name_for_image = custom_name.clone();
    let custom_subtext_for_image = custom_subtext.clone();
    let custom_portrait = interviewee.custom_portrait;
    let custom_portrait_for_name = custom_portrait.clone();
    let custom_portrait_for_subtext = custom_portrait.clone();
    let match_team_ids = [match_state.team_a.clone(), match_state.team_b.clone()];
    let player_options: Vec<(TeamId, PlayerId, String, String)> = teams
        .iter()
        .filter(|team| match_team_ids.iter().flatten().any(|id| id == &team.id))
        .flat_map(|team| {
            let team_label = if !team.name.trim().is_empty() {
                team.name.clone()
            } else if !team.tag.trim().is_empty() {
                team.tag.clone()
            } else {
                team.id.as_str().to_owned()
            };
            team.players
                .iter()
                .map(move |player| {
                    (
                        team.id.clone(),
                        player.id.clone(),
                        team_label.clone(),
                        player.name.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let player_options_for_change = player_options.clone();

    rsx! {
        div { class: "fgc-view",
            Panel {
                title: "Talent assignments".to_owned(),
                description: "Assign casters, panel members and the interview desk.".to_owned(),
                div { class: "fgc-talent-grid",
                    for slot in TalentSlot::ALL {
                        {
                            let reference = match_state.talent.slot(slot);
                            let current = reference
                                .map_or_else(String::new, |reference| reference.talent_id.as_str().to_owned());
                            let current_id = reference.map(|reference| reference.talent_id.clone());
                            let role = reference.map_or_else(|| slot.default_role(), |reference| reference.role);
                            rsx! {
                                div {
                                    key: "{slot.label()}",
                                    class: "fgc-talent-row",
                                    role: "group",
                                    aria_label: "{slot.label()} assignment",
                                    select {
                                        class: "fgc-native-select",
                                        aria_label: "{slot.label()} talent",
                                        value: "{current}",
                                        onchange: move |event| {
                                            let value = event.value();
                                            store.set_talent_slot(
                                                slot,
                                                if value.is_empty() {
                                                    None
                                                } else {
                                                    Some(TalentId::from_raw(value))
                                                },
                                                role,
                                            );
                                        },
                                        option { value: "", "— Select —" }
                                        for talent in talents.iter() {
                                            option {
                                                key: "{talent.id}",
                                                value: "{talent.id}",
                                                selected: talent.id.as_str() == current.as_str(),
                                                "{talent.display_name}"
                                            }
                                        }
                                    }
                                    select {
                                        class: "fgc-native-select fgc-talent-role",
                                        aria_label: "{slot.label()} role",
                                        disabled: current_id.is_none(),
                                        value: role.slug(),
                                        onchange: move |event| {
                                            let Some(next) = TalentType::from_slug(&event.value()) else {
                                                return;
                                            };
                                            if let Some(id) = current_id.clone() {
                                                store.set_talent_slot(slot, Some(id), next);
                                            }
                                        },
                                        for option in TalentType::ALL {
                                            option {
                                                key: "{option.slug()}",
                                                value: option.slug(),
                                                selected: option == role,
                                                "{option.label()}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            CollapsiblePanel {
                title: "Interview".to_owned(),
                div { class: "fgc-form-row",
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Interviewee" }
                        select {
                            class: "fgc-native-select",
                            value: "{interviewee_source}",
                            onchange: move |event| {
                                let value = event.value();
                                if let Some(raw) = value.strip_prefix("player:") {
                                    if let Some((team_id, player_id, _, _)) = player_options_for_change
                                        .iter()
                                        .find(|(_, id, _, _)| id.as_str() == raw)
                                    {
                                        store.set_interviewee_player(team_id.clone(), player_id.clone());
                                    }
                                } else if let Some(raw) = value.strip_prefix("talent:") {
                                    store.set_interviewee_talent(Some(TalentId::from_raw(raw.to_owned())));
                                } else {
                                    store.set_interviewee_talent(None);
                                }
                            },
                            option { value: "", selected: interviewee_source.is_empty(), "Custom…" }
                            for talent in talents.iter() {
                                option {
                                    key: "talent:{talent.id}",
                                    value: "talent:{talent.id}",
                                    selected: interviewee_source == format!("talent:{}", talent.id),
                                    "Talent · {talent.display_name}"
                                }
                            }
                            for (_team_id, player_id, team_label, player_name) in player_options.iter() {
                                option {
                                    key: "player:{player_id}",
                                    value: "player:{player_id}",
                                    selected: interviewee_source == format!("player:{player_id}"),
                                    "{team_label} · {player_name}"
                                }
                            }
                        }
                        if player_options.is_empty() {
                            span { class: "fgc-field__hint",
                                "No players on the match teams yet. Choose Custom… to enter a name."
                            }
                        }
                    }
                    if interviewee_source.is_empty() {
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Custom name" }
                            Input {
                                value: "{custom_name}",
                                placeholder: "Enter name",
                                oninput: move |event: FormEvent| {
                                    store.set_interviewee_custom(
                                        &event.value(),
                                        &custom_subtext_for_name,
                                        custom_portrait_for_name.clone(),
                                    );
                                },
                            }
                        }
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Custom subtext" }
                            Input {
                                value: "{custom_subtext}",
                                placeholder: "e.g. Team, Role",
                                oninput: move |event: FormEvent| {
                                    store.set_interviewee_custom(
                                        &custom_name_for_subtext,
                                        &event.value(),
                                        custom_portrait_for_subtext.clone(),
                                    );
                                },
                            }
                        }
                        div { class: "fgc-field",
                            AssetPicker {
                                label: "Guest image".to_owned(),
                                category: AssetCategory::Portrait,
                                value: custom_portrait,
                                on_change: move |portrait| {
                                    store.set_interviewee_custom(
                                        &custom_name_for_image,
                                        &custom_subtext_for_image,
                                        portrait,
                                    );
                                },
                            }
                        }
                    } else if interviewee_source.starts_with("player:") {
                        span { class: "fgc-field__hint",
                            "Portrait, team, role, position and pronouns come from the roster."
                        }
                    }
                }
            }
        }
    }
}

/// Ordered map pool used by the map-pool overlay.
#[component]
fn MapPoolTab(match_state: Match) -> Element {
    let store = use_context::<AppStore>();
    let catalog = store.catalog();
    let mut custom = use_signal(String::new);
    let pool = match_state.map_pool.clone();
    let groups = map_groups(&catalog);

    rsx! {
        div { class: "fgc-grid fgc-grid--two",
            Panel {
                title: "Available maps".to_owned(),
                description: "Select maps to include. The map-pool overlay only shows maps that have a map asset.".to_owned(),
                if groups.is_empty() {
                    p { class: "fgc-note",
                        "No map assets are bundled yet. Add map images in Assets; the overlay only shows maps that have an asset."
                    }
                } else {
                    for (mode, names) in groups.iter() {
                        div { key: "{mode}", class: "fgc-map-pool__group",
                            p { class: "fgc-map-pool__mode", "{mode}" }
                            for name in names.iter() {
                                {
                                    let included = pool.iter().any(|item| item == name);
                                    let name_for_change = name.clone();
                                    rsx! {
                                        label { key: "{name}", class: "fgc-map-pool__item",
                                            input {
                                                r#type: "checkbox",
                                                checked: included,
                                                onchange: move |event| {
                                                    store.set_map_pool_included(
                                                        &name_for_change,
                                                        event.checked(),
                                                    );
                                                },
                                            }
                                            span { "{name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "fgc-map-pool__custom",
                    Input {
                        value: "{custom}",
                        placeholder: "Custom map name",
                        oninput: move |event: FormEvent| custom.set(event.value()),
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| {
                            let name = custom.read().trim().to_owned();
                            if !name.is_empty() {
                                store.set_map_pool_included(&name, true);
                                custom.set(String::new());
                            }
                        },
                        "Add"
                    }
                }
            }

            Panel {
                title: "Pool order".to_owned(),
                description: "The overlay shows maps in this order.".to_owned(),
                if pool.is_empty() {
                    p { class: "fgc-note", "The pool is empty." }
                } else {
                    div { class: "fgc-map-pool__order",
                        for (index, name) in pool.iter().enumerate() {
                            {
                                let name_for_remove = name.clone();
                                let count = pool.len();
                                rsx! {
                                    div { key: "{name}", class: "fgc-map-pool__order-row",
                                        span { class: "fgc-map-pool__order-name", "{name}" }
                                        div { class: "fgc-actions",
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                disabled: index == 0,
                                                onclick: move |_| store.move_map_pool(index, -1),
                                                "↑"
                                            }
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                disabled: index + 1 >= count,
                                                onclick: move |_| store.move_map_pool(index, 1),
                                                "↓"
                                            }
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                size: ButtonSize::Sm,
                                                onclick: move |_| store.set_map_pool_included(&name_for_remove, false),
                                                "Remove"
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

/// Overlay appearance, branding and Browser Source URLs.
#[component]
fn ConfigTab(match_state: Match) -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let settings = state.read().settings.clone();
    let team_colors = settings.display.team_colors_enabled;

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-grid fgc-grid--two",
                Panel {
                    title: "Overlay appearance".to_owned(),
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Team colours" }
                            p { class: "fgc-toggle-row__hint",
                                if team_colors {
                                    "Team colours are used on the overlays."
                                } else {
                                    "Default white background with black text."
                                }
                            }
                        }
                        Switch {
                            checked: team_colors,
                            aria_label: "Team colours",
                            on_checked_change: move |checked| {
                                store.mutate(|state| {
                                    state.settings.display.team_colors_enabled = checked;
                                });
                            },
                        }
                    }
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Team abbreviations" }
                            p { class: "fgc-toggle-row__hint",
                                "Show the short team tag instead of the full name."
                            }
                        }
                        Switch {
                            checked: settings.display.team_abbreviations,
                            aria_label: "Team abbreviations",
                            on_checked_change: move |checked| {
                                store.mutate(|state| {
                                    state.settings.display.team_abbreviations = checked;
                                });
                            },
                        }
                    }
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Player portraits" }
                            p { class: "fgc-toggle-row__hint",
                                "Show portraits on the team roster overlays."
                            }
                        }
                        Switch {
                            checked: match_state.show_player_portraits,
                            aria_label: "Player portraits",
                            on_checked_change: move |checked| {
                                store.update_match(|current| {
                                    current.show_player_portraits = checked;
                                });
                            },
                        }
                    }
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Loser picks" }
                            p { class: "fgc-toggle-row__hint",
                                "Each map after the first is picked by the previous map's loser."
                            }
                        }
                        Switch {
                            checked: match_state.loser_picks,
                            aria_label: "Loser picks",
                            on_checked_change: move |checked: bool| store.set_match_loser_picks(checked),
                        }
                    }
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Hero bans" }
                            p { class: "fgc-toggle-row__hint",
                                "Show hero bans inside Map Selection."
                            }
                        }
                        Switch {
                            checked: settings.display.show_hero_bans,
                            aria_label: "Hero bans",
                            on_checked_change: move |checked| {
                                store.mutate(|state| {
                                    state.settings.display.show_hero_bans = checked;
                                });
                            },
                        }
                    }
                }

                CollapsiblePanel {
                    title: "Branding".to_owned(),
                    description: "Tournament logo and watermark used by the overlays.".to_owned(),
                    AssetPicker {
                        label: "Tournament logo".to_owned(),
                        category: AssetCategory::Logo,
                        value: settings.display.tournament_logo.clone(),
                        on_change: move |value: Option<AssetId>| {
                            store.mutate(|state| state.settings.display.tournament_logo = value);
                        },
                    }
                    AssetPicker {
                        label: "Watermark".to_owned(),
                        category: AssetCategory::Logo,
                        value: settings.display.watermark_logo.clone(),
                        on_change: move |value: Option<AssetId>| {
                            store.mutate(|state| state.settings.display.watermark_logo = value);
                        },
                    }
                }

                Panel {
                    title: "Overlay theme".to_owned(),
                    Field {
                        label: "Theme".to_owned(),
                        hint: Some("Applied to every bundled overlay straight away.".to_owned()),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Overlay theme",
                            value: settings.display.theme.as_str(),
                            onchange: move |event| {
                                let next = match event.value().as_str() {
                                    "light" => ThemePreset::Light,
                                    _ => ThemePreset::NightBroadcast,
                                };
                                store.set_theme(next);
                            },
                            for preset in ThemePreset::all() {
                                option {
                                    value: preset.as_str(),
                                    selected: preset == settings.display.theme,
                                    "{preset.label()}"
                                }
                            }
                        }
                    }
                    div { class: "fgc-toggle-row",
                        div { class: "fgc-toggle-row__text",
                            p { class: "fgc-toggle-row__title", "Imported theme" }
                            p { class: "fgc-toggle-row__hint",
                                match &settings.display.theme_tokens {
                                    Some(tokens) => format!(
                                        "{} tokens are layered over the selected theme.",
                                        tokens.len()
                                    ),
                                    None => "Import a JSON theme file to override individual tokens."
                                        .to_owned(),
                                }
                            }
                        }
                        div { class: "fgc-actions",
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| store.import_theme(),
                                "Import theme file"
                            }
                            if settings.display.theme_tokens.is_some() {
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| store.set_theme_tokens(None),
                                    "Remove"
                                }
                            }
                        }
                    }
                }
            }

            OverlayUrls {}
        }
    }
}

/// Groups map assets by their linked game mode.
fn map_groups(catalog: &AssetCatalog) -> Vec<(String, Vec<String>)> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for asset in catalog.in_category(AssetCategory::Map) {
        let mode = asset
            .game_mode
            .as_ref()
            .and_then(|id| catalog.get(id))
            .map_or_else(|| "Other".to_owned(), |asset| asset.name.clone());
        groups.entry(mode).or_default().push(asset.name.clone());
    }
    groups.into_iter().collect()
}

/// Parses a non-negative score, treating blank or invalid input as zero.
fn parse_score(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_bans_share_one_tab() {
        let labels: Vec<&str> = MatchTab::ALL.iter().map(|tab| tab.label()).collect();
        assert_eq!(
            labels,
            ["Match", "Maps & Bans", "Talent", "Map Pool", "Config"]
        );
    }

    #[test]
    fn scores_treat_invalid_input_as_zero() {
        assert_eq!(parse_score("12"), 12);
        assert_eq!(parse_score(""), 0);
        assert_eq!(parse_score("-1"), 0);
    }

    #[test]
    fn maps_and_bans_label_the_selected_teams() {
        let mut team_a = crate::state::new_team();
        team_a.name = "Pink Salamanders".to_owned();
        team_a.tag = "PS".to_owned();
        let mut team_b = crate::state::new_team();
        team_b.name = "Inglorious Blasters".to_owned();
        team_b.tag = "IB".to_owned();
        let teams = vec![team_a.clone(), team_b.clone()];

        assert_eq!(
            selected_team_label(&Some(team_a.id.clone()), &teams, "Team A"),
            "Pink Salamanders"
        );
        assert_eq!(
            selected_team_label(&Some(team_b.id.clone()), &teams, "Team B"),
            "Inglorious Blasters"
        );
        assert_eq!(selected_team_label(&None, &teams, "Team A"), "Team A");

        team_a.name.clear();
        let teams = vec![team_a, team_b];
        assert_eq!(
            selected_team_label(&Some(teams[0].id.clone()), &teams, "Team A"),
            "PS"
        );
    }

    #[test]
    fn hero_ban_pickers_keep_every_option_visible() {
        let source = include_str!("match_controller.rs");
        assert!(
            !source.contains("datalist { id: \"fgc-hero-names\""),
            "hero bans must not use a filtering datalist"
        );
        assert!(
            source.contains("aria_label: \"{team_a_label} ban\"")
                && source.contains("aria_label: \"{team_b_label} ban\"")
                && source.contains("for name in heroes.iter() {")
                && source.contains("\"{ban_team_a} (imported)\"")
                && source.contains("\"{ban_team_b} (imported)\""),
            "both hero pickers should render every hero and any imported value"
        );
        assert!(
            source.contains("aria_label: \"Custom {team_a_label} ban\"")
                && source.contains("aria_label: \"Custom {team_b_label} ban\"")
                && source.contains("custom_hero_a.write().insert(index, value.clone())")
                && source.contains("custom_hero_b.write().insert(index, value.clone())"),
            "Custom… should keep the old free-text path for either team"
        );
    }
}
