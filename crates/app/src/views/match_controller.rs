//! Match Controller destination: seven tabs that drive the live overlays.
//!
//! All persistent edits go through [`AppStore`], which validates and reprojects
//! them. Timer run state is deliberately ephemeral: it lives in this view so
//! switching tabs does not reset a running clock, while the persisted timer
//! settings stay in `AppState::settings`.

use std::collections::BTreeMap;
use std::time::Duration;

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use fgc_core::{
    AssetCategory, AssetId, Match, MatchStatus, Scoring, Side, Talent, TalentId, TalentSlot, Team,
    TeamId, TeamSide, TimerMode, TimerSettings, empty_hero_bans,
};
use fgc_protocol::{AssetCatalog, TimerActionDto, timer_action, timer_tick};
use fgc_server::{Outbound, ServerStatus};

use crate::components::Panel;
use crate::components::asset_picker::AssetPicker;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::server::{OVERLAY_FILES, overlay_url};
use crate::state::AppStore;
use crate::views::overlay::copy_to_clipboard;

/// The seven Match Controller tabs, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchTab {
    /// Teams, title, format, status, sides and scores.
    Setup,
    /// Map slots, scores, completion and picker.
    Maps,
    /// One hero ban per team per map.
    Bans,
    /// Caster, panel and interview assignments.
    Talent,
    /// Ordered map pool for the map-pool overlay.
    MapPool,
    /// Timer settings, controls and live preview.
    Timer,
    /// Overlay appearance, branding and Browser Source URLs.
    Config,
}

impl MatchTab {
    const ALL: [Self; 7] = [
        Self::Setup,
        Self::Maps,
        Self::Bans,
        Self::Talent,
        Self::MapPool,
        Self::Timer,
        Self::Config,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Setup => "Match",
            Self::Maps => "Map Selection",
            Self::Bans => "Hero Bans",
            Self::Talent => "Talent",
            Self::MapPool => "Map Pool",
            Self::Timer => "Timer",
            Self::Config => "Config",
        }
    }
}

/// Ephemeral timer state shared by the tab and the tick loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TimerState {
    running: bool,
    total_seconds: u32,
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
    let timer = use_signal(TimerState::default);
    let target = use_signal(String::new);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if !timer.peek().running {
                continue;
            }

            let (mode, settings) = {
                let state = state.peek();
                (state.settings.timer.mode, state.settings.timer.clone())
            };
            let current = *timer.peek();
            let next = match mode {
                TimerMode::CountUp => current.total_seconds.saturating_add(1),
                TimerMode::CountDown => current.total_seconds.saturating_sub(1),
                TimerMode::Target => {
                    let target = target.peek().clone();
                    if target.is_empty() {
                        0
                    } else {
                        match document::eval(&target_countdown_script(&target))
                            .join::<i64>()
                            .await
                        {
                            Ok(seconds) => u32::try_from(seconds).unwrap_or(0),
                            Err(err) => {
                                tracing::warn!(target = %target, %err, "timer target eval failed");
                                0
                            }
                        }
                    }
                }
            };

            if matches!(mode, TimerMode::CountDown | TimerMode::Target) && next == 0 {
                let mut timer_signal = timer;
                timer_signal.set(TimerState {
                    running: false,
                    total_seconds: 0,
                });
                broadcast_tick(&store, &settings, 0);
                continue;
            }

            let mut timer_signal = timer;
            timer_signal.set(TimerState {
                running: true,
                total_seconds: next,
            });
            broadcast_tick(&store, &settings, next);
        }
    });

    use_effect(move || {
        let settings = state.read().settings.timer.clone();
        let current = timer();
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
                    MapSelectionTab { match_state: match_state.clone() }
                },
                MatchTab::Bans => rsx! {
                    HeroBansTab { match_state: match_state.clone() }
                },
                MatchTab::Talent => rsx! {
                    TalentTab { match_state: match_state.clone(), talents }
                },
                MatchTab::MapPool => rsx! {
                    MapPoolTab { match_state: match_state.clone() }
                },
                MatchTab::Timer => rsx! {
                    TimerTab { timer, target }
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
            }

            Panel {
                title: "Teams".to_owned(),
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
        }
    }
}

/// Map slots, map names, modes, scores and per-map state.
#[component]
fn MapSelectionTab(match_state: Match) -> Element {
    let store = use_context::<AppStore>();
    let catalog = store.catalog();
    let maps = match_state.maps.clone();
    let map_count = maps.len();
    let map_names: Vec<String> = catalog
        .in_category(AssetCategory::Map)
        .map(|asset| asset.name.clone())
        .collect();
    let mode_names: Vec<String> = catalog
        .in_category(AssetCategory::GameMode)
        .map(|asset| asset.name.clone())
        .collect();

    rsx! {
        datalist { id: "fgc-map-names",
            for name in map_names.iter() {
                option { value: "{name}" }
            }
        }
        datalist { id: "fgc-mode-names",
            for name in mode_names.iter() {
                option { value: "{name}" }
            }
        }

        div { class: "fgc-view",
            div { class: "fgc-match__section-header",
                p { class: "fgc-note",
                    "{map_count} map slot(s). The first map is picked by team A; later maps follow the loser of the previous map when loser picks is enabled."
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
                }
            }

            div { class: "fgc-map-list",
                for (index, map) in maps.into_iter().enumerate() {
                    {
                        let catalog_for_name = catalog.clone();
                        let is_current = match_state.current_map == Some(index);
                        let chosen_by = map.chosen_by;
                        let loser_picks = match_state.loser_picks;
                        let label = if index == 7 {
                            "Tiebreaker".to_owned()
                        } else {
                            format!("Map {}", index + 1)
                        };
                        rsx! {
                            Panel {
                                key: "{index}",
                                title: label,
                                div { class: "fgc-map-slot",
                                    div { class: "fgc-form-row",
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "Map name" }
                                            Input {
                                                list: "fgc-map-names",
                                                value: "{map.name}",
                                                placeholder: "Select or type a map",
                                                oninput: move |event: FormEvent| {
                                                    let name = event.value();
                                                    let mode = mode_for_map(&catalog_for_name, &name);
                                                    store.set_map_name(index, &name, &mode);
                                                },
                                            }
                                        }
                                        label { class: "fgc-field",
                                            span { class: "fgc-field__label", "Game mode" }
                                            Input {
                                                list: "fgc-mode-names",
                                                value: "{map.game_mode_name}",
                                                placeholder: "TBC",
                                                oninput: move |event: FormEvent| {
                                                    store.set_map_mode(index, &event.value());
                                                },
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
                                                option { value: "A", "Team A" }
                                                option { value: "B", "Team B" }
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
                                            span { class: "fgc-field__label", "Team A score" }
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
                                            span { class: "fgc-field__label", "Team B score" }
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
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One hero ban per team per map.
#[component]
fn HeroBansTab(match_state: Match) -> Element {
    let store = use_context::<AppStore>();
    let catalog = store.catalog();
    let maps = match_state.maps.clone();
    let bans = match_state.hero_bans.clone();
    let heroes: Vec<String> = catalog
        .in_category(AssetCategory::Hero)
        .filter(|asset| !catalog.is_linked_icon(&asset.id))
        .map(|asset| asset.name.clone())
        .collect();

    rsx! {
        datalist { id: "fgc-hero-names",
            for name in heroes.iter() {
                option { value: "{name}" }
            }
        }

        div { class: "fgc-view",
            div { class: "fgc-match__section-header",
                p { class: "fgc-note",
                    "The overlays show one ban per team. Choose the first-ban team and the hero each team banned."
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Sm,
                    onclick: move |_| store.reset_hero_bans(),
                    "Reset all bans"
                }
            }

            Panel {
                div { class: "fgc-bans-table",
                    div { class: "fgc-bans-table__head",
                        span { "Map" }
                        span { "First ban" }
                        span { "Team A" }
                        span { "Team B" }
                    }
                    for (index, map) in maps.into_iter().enumerate() {
                        {
                            let ban = bans.get(index).cloned().unwrap_or_else(empty_hero_bans);
                            let team_a = ban.team_a.first().cloned().unwrap_or_default();
                            let team_b = ban.team_b.first().cloned().unwrap_or_default();
                            rsx! {
                                div { key: "{index}", class: "fgc-bans-table__row",
                                    span { class: "fgc-bans-table__map",
                                        if map.name.is_empty() {
                                            "Map {index + 1}"
                                        } else {
                                            "{map.name}"
                                        }
                                    }
                                    select {
                                        class: "fgc-native-select",
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
                                        option { value: "A", "Team A" }
                                        option { value: "B", "Team B" }
                                    }
                                    Input {
                                        list: "fgc-hero-names",
                                        value: "{team_a}",
                                        placeholder: "Hero",
                                        oninput: move |event: FormEvent| {
                                            store.set_hero_ban(index, TeamSide::A, &event.value());
                                        },
                                    }
                                    Input {
                                        list: "fgc-hero-names",
                                        value: "{team_b}",
                                        placeholder: "Hero",
                                        oninput: move |event: FormEvent| {
                                            store.set_hero_ban(index, TeamSide::B, &event.value());
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

/// Caster, panel and interview assignments.
#[component]
fn TalentTab(match_state: Match, talents: Vec<Talent>) -> Element {
    let store = use_context::<AppStore>();
    let interviewee = match_state.talent.interviewee.clone().unwrap_or_default();
    let interviewee_talent = interviewee
        .talent_id
        .as_ref()
        .map_or_else(String::new, |id| id.as_str().to_owned());
    let custom_name = interviewee.custom_name.clone().unwrap_or_default();
    let custom_subtext = interviewee.custom_subtext.clone().unwrap_or_default();
    let custom_name_for_subtext = custom_name.clone();
    let custom_subtext_for_name = custom_subtext.clone();

    rsx! {
        div { class: "fgc-view",
            Panel {
                title: "Talent assignments".to_owned(),
                description: "Assign casters, panel members and the interview desk.".to_owned(),
                div { class: "fgc-talent-grid",
                    for slot in TalentSlot::ALL {
                        {
                            let current = match_state
                                .talent
                                .slot(slot)
                                .map_or_else(String::new, |reference| reference.talent_id.as_str().to_owned());
                            rsx! {
                                label { key: "{slot.label()}", class: "fgc-field",
                                    span { class: "fgc-field__label", "{slot.label()}" }
                                    select {
                                        class: "fgc-native-select",
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
                                }
                            }
                        }
                    }
                }
            }

            Panel {
                title: "Interview".to_owned(),
                div { class: "fgc-form-row",
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Interviewee" }
                        select {
                            class: "fgc-native-select",
                            value: "{interviewee_talent}",
                            onchange: move |event| {
                                let value = event.value();
                                store.set_interviewee_talent(
                                    if value.is_empty() {
                                        None
                                    } else {
                                        Some(TalentId::from_raw(value))
                                    },
                                );
                            },
                            option { value: "", "Custom…" }
                            for talent in talents.iter() {
                                option {
                                    key: "{talent.id}",
                                    value: "{talent.id}",
                                    selected: talent.id.as_str() == interviewee_talent.as_str(),
                                    "{talent.display_name}"
                                }
                            }
                        }
                    }
                    if interviewee.talent_id.is_none() {
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Custom name" }
                            Input {
                                value: "{custom_name}",
                                placeholder: "Enter name",
                                oninput: move |event: FormEvent| {
                                    store.set_interviewee_custom(
                                        &event.value(),
                                        &custom_subtext_for_name,
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
                                    );
                                },
                            }
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
                description: "Select maps to include. Grouped by game mode when map assets exist.".to_owned(),
                if groups.is_empty() {
                    p { class: "fgc-note",
                        "No map assets are bundled yet. Add map images in Assets, or type a custom map name below."
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

/// Timer settings, controls and a live preview.
#[component]
fn TimerTab(timer: Signal<TimerState>, target: Signal<String>) -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let settings = state.read().settings.timer.clone();
    let current = timer();
    let (hours, minutes, seconds) = split_time(current.total_seconds);
    let zeroes = settings.leading_zeroes;
    let mode = settings.mode;
    let alignment = settings.alignment;

    rsx! {
        div { class: "fgc-grid fgc-grid--two",
            Panel {
                title: "Timer settings".to_owned(),
                div { class: "fgc-form-row",
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Mode" }
                        select {
                            class: "fgc-native-select",
                            value: match mode {
                                TimerMode::CountUp => "countup",
                                TimerMode::CountDown => "countdown",
                                TimerMode::Target => "target",
                            },
                            onchange: move |event| {
                                let mode = match event.value().as_str() {
                                    "countdown" => TimerMode::CountDown,
                                    "target" => TimerMode::Target,
                                    _ => TimerMode::CountUp,
                                };
                                store.set_timer_mode(mode);
                                let mut timer = timer;
                                timer.set(TimerState::default());
                            },
                            option { value: "countup", "Count up" }
                            option { value: "countdown", "Count down" }
                            option { value: "target", "Count to target" }
                        }
                    }
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Alignment" }
                        select {
                            class: "fgc-native-select",
                            value: match alignment {
                                fgc_core::TimerAlignment::Left => "left",
                                fgc_core::TimerAlignment::Center => "center",
                                fgc_core::TimerAlignment::Right => "right",
                            },
                            onchange: move |event| {
                                let alignment = match event.value().as_str() {
                                    "left" => fgc_core::TimerAlignment::Left,
                                    "right" => fgc_core::TimerAlignment::Right,
                                    _ => fgc_core::TimerAlignment::Center,
                                };
                                store.set_timer_alignment(alignment);
                            },
                            option { value: "left", "Left" }
                            option { value: "center", "Centre" }
                            option { value: "right", "Right" }
                        }
                    }
                }

                if mode == TimerMode::CountDown {
                    div { class: "fgc-form-row",
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Hours" }
                            Input {
                                r#type: "number",
                                value: "{hours}",
                                oninput: move |event: FormEvent| {
                                    let mut timer = timer;
                                    let (_, minutes, seconds) = split_time(timer.peek().total_seconds);
                                    timer.set(TimerState {
                                        running: false,
                                        total_seconds: join_time(
                                            parse_u32(&event.value()),
                                            minutes,
                                            seconds,
                                        ),
                                    });
                                },
                            }
                        }
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Minutes" }
                            Input {
                                r#type: "number",
                                value: "{minutes}",
                                oninput: move |event: FormEvent| {
                                    let mut timer = timer;
                                    let (hours, _, seconds) = split_time(timer.peek().total_seconds);
                                    timer.set(TimerState {
                                        running: false,
                                        total_seconds: join_time(
                                            hours,
                                            parse_u32(&event.value()),
                                            seconds,
                                        ),
                                    });
                                },
                            }
                        }
                        label { class: "fgc-field",
                            span { class: "fgc-field__label", "Seconds" }
                            Input {
                                r#type: "number",
                                value: "{seconds}",
                                oninput: move |event: FormEvent| {
                                    let mut timer = timer;
                                    let (hours, minutes, _) = split_time(timer.peek().total_seconds);
                                    timer.set(TimerState {
                                        running: false,
                                        total_seconds: join_time(
                                            hours,
                                            minutes,
                                            parse_u32(&event.value()),
                                        ),
                                    });
                                },
                            }
                        }
                    }
                }

                if mode == TimerMode::Target {
                    label { class: "fgc-field",
                        span { class: "fgc-field__label", "Target date and time" }
                        Input {
                            r#type: "datetime-local",
                            value: "{target}",
                            oninput: move |event: FormEvent| target.set(event.value()),
                        }
                    }
                    p { class: "fgc-note",
                        "The timer counts down to the target. Set a target before starting."
                    }
                }

                div { class: "fgc-toggle-row",
                    div { class: "fgc-toggle-row__text",
                        p { class: "fgc-toggle-row__title", "Leading zeroes" }
                        p { class: "fgc-toggle-row__hint", "Pad single-digit values on the overlay." }
                    }
                    Switch {
                        checked: zeroes,
                        aria_label: "Leading zeroes",
                        on_checked_change: move |checked| store.set_timer_leading_zeroes(checked),
                    }
                }
                div { class: "fgc-toggle-row",
                    div { class: "fgc-toggle-row__text",
                        p { class: "fgc-toggle-row__title", "Standalone timer" }
                        p { class: "fgc-toggle-row__hint", "Hide the timer surround." }
                    }
                    Switch {
                        checked: settings.standalone,
                        aria_label: "Standalone timer",
                        on_checked_change: move |checked| store.set_timer_standalone(checked),
                    }
                }
            }

            Panel {
                title: "Timer".to_owned(),
                div { class: "fgc-timer",
                    span { class: "fgc-timer__value", "{format_clock(current.total_seconds)}" }
                    p { class: "fgc-note",
                        if current.running {
                            "Running"
                        } else {
                            "Paused"
                        }
                    }
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: current.running
                            || (mode == TimerMode::Target && target.is_empty()),
                        onclick: move |_| {
                            let mut timer = timer;
                            let total_seconds = timer.peek().total_seconds;
                            timer.set(TimerState {
                                running: true,
                                total_seconds,
                            });
                            send_timer_action(&store, TimerActionDto::start());
                        },
                        "Start"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        disabled: !current.running,
                        onclick: move |_| {
                            let mut timer = timer;
                            let total_seconds = timer.peek().total_seconds;
                            timer.set(TimerState {
                                running: false,
                                total_seconds,
                            });
                            send_timer_action(&store, TimerActionDto::pause());
                        },
                        "Pause"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| {
                            let (hours, minutes, seconds) = if mode == TimerMode::CountDown {
                                split_time(timer.peek().total_seconds)
                            } else {
                                (0, 0, 0)
                            };
                            let mut timer = timer;
                            timer.set(TimerState {
                                running: false,
                                total_seconds: join_time(hours, minutes, seconds),
                            });
                            send_timer_action(
                                &store,
                                TimerActionDto::reset(hours, minutes, seconds),
                            );
                        },
                        "Reset"
                    }
                }
                p { class: "fgc-note",
                    "Timer run state is not saved; timer settings are."
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
    let toast = use_toast();
    let settings = state.read().settings.clone();
    let server = store.server();
    let base_url = server.base_url();
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
                }

                Panel {
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
            }

            Panel {
                title: "Overlay URLs".to_owned(),
                description: "Paste these into OBS Browser Sources at 1920×1080.".to_owned(),
                div { class: "fgc-source-list",
                    for file in OVERLAY_FILES {
                        {
                            let url = overlay_url(&base_url, file);
                            let url_for_code = url.clone();
                            let url_for_click = url.clone();
                            let file_for_toast = file.to_owned();
                            rsx! {
                                div { key: "{file}", class: "fgc-source",
                                    div { class: "fgc-source__meta",
                                        p { class: "fgc-source__file", "{file}" }
                                        code { class: "fgc-code", "{url_for_code}" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            copy_to_clipboard(&url_for_click);
                                            toast.success(
                                                "Browser source copied".to_owned(),
                                                ToastOptions::new()
                                                    .description(file_for_toast.clone()),
                                            );
                                        },
                                        "Copy URL"
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

/// Returns the game-mode name linked to a map asset, if any.
fn mode_for_map(catalog: &AssetCatalog, name: &str) -> String {
    catalog
        .find_by_name(AssetCategory::Map, name)
        .and_then(|asset| asset.game_mode.as_ref())
        .and_then(|id| catalog.get(id))
        .map_or_else(String::new, |asset| asset.name.clone())
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

/// Splits a total number of seconds into hours, minutes and seconds.
fn split_time(total: u32) -> (u32, u32, u32) {
    (total / 3600, (total % 3600) / 60, total % 60)
}

/// Combines hours, minutes and seconds into a total.
fn join_time(hours: u32, minutes: u32, seconds: u32) -> u32 {
    hours
        .saturating_mul(3600)
        .saturating_add(minutes.saturating_mul(60))
        .saturating_add(seconds)
}

/// Formats the preview clock as `HH:MM:SS`.
fn format_clock(total: u32) -> String {
    let (hours, minutes, seconds) = split_time(total);
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// Builds the JavaScript that returns the whole seconds remaining until `target`.
///
/// Dioxus evaluates a script as an async function body, so the value must be
/// returned explicitly: a trailing expression would yield `undefined` and the
/// eval result would fail to deserialize.
fn target_countdown_script(target: &str) -> String {
    format!("return Math.max(0, Math.floor((Date.parse({target:?}) - Date.now()) / 1000));")
}

/// Parses a non-negative score, treating blank or invalid input as zero.
fn parse_score(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(0)
}

/// Parses a non-negative number, treating blank or invalid input as zero.
fn parse_u32(value: &str) -> u32 {
    value.trim().parse::<u32>().unwrap_or(0)
}

/// Broadcasts one timer tick when the server is running.
fn broadcast_tick(store: &AppStore, settings: &TimerSettings, total: u32) {
    if !store.server().is_running() {
        return;
    }
    let (hours, minutes, seconds) = split_time(total);
    let message = timer_tick(settings, hours, minutes, seconds);
    if let Ok(outbound) = Outbound::from_message(&message) {
        store.broadcast_timer(outbound);
    }
}

/// Broadcasts one timer control action when the server is running.
fn send_timer_action(store: &AppStore, action: TimerActionDto) {
    if !store.server().is_running() {
        return;
    }
    let message = timer_action(action);
    if let Ok(outbound) = Outbound::from_message(&message) {
        store.broadcast_timer(outbound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_have_the_original_order() {
        let labels: Vec<&str> = MatchTab::ALL.iter().map(|tab| tab.label()).collect();
        assert_eq!(
            labels,
            [
                "Match",
                "Map Selection",
                "Hero Bans",
                "Talent",
                "Map Pool",
                "Timer",
                "Config"
            ]
        );
    }

    #[test]
    fn clock_helpers_round_trip() {
        assert_eq!(split_time(3661), (1, 1, 1));
        assert_eq!(join_time(1, 1, 1), 3661);
        assert_eq!(format_clock(3661), "01:01:01");
    }

    #[test]
    fn scores_treat_invalid_input_as_zero() {
        assert_eq!(parse_score("12"), 12);
        assert_eq!(parse_score(""), 0);
        assert_eq!(parse_score("-1"), 0);
        assert_eq!(parse_u32("90"), 90);
    }

    #[test]
    fn target_script_returns_a_number() {
        // The desktop eval wrapper runs the script as an async function body, so
        // the result must be returned; a trailing expression evaluates to
        // `undefined` and the timer silently stops.
        let script = target_countdown_script("2026-11-15T18:12");
        assert!(script.starts_with("return "), "{script}");
        assert!(script.contains("\"2026-11-15T18:12\""), "{script}");
        assert!(script.contains("Date.parse"), "{script}");
    }
}
