//! Teams destination: team identity, colours, logos and rosters.

use dioxus::prelude::*;
use fgc_core::{
    Asset as CoreAsset, AssetCategory, MAX_PLAYER_NAME_CHARS, MAX_TEAM_NAME_CHARS, Player,
    PlayerId, Position, Role, Team, TeamId, TeamReference,
};

use crate::components::asset_picker::{AssetPicker, AssetThumb, find_asset, library_assets};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::focus_element_later;
use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::components::{CollapsiblePanel, Field, Panel, ToggleRow};
use crate::logo_colors::{contrast_text_color, harmonize_text_color, sample_logo};
use crate::state::AppStore;

/// The two sections of the selected team.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TeamSection {
    /// Name, tag, record, logo and colours.
    Info,
    /// Ordered roster and player editor.
    Players,
}

/// Team list, team identity editor and roster editor.
#[component]
pub fn TeamsView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let mut selected_team_id = use_signal(|| None::<TeamId>);
    let mut section = use_signal(|| TeamSection::Info);
    let mut editing_player_id = use_signal(|| None::<PlayerId>);
    let mut delete_target = use_signal(|| None::<Team>);
    let mut delete_references = use_signal(Vec::<TeamReference>::new);
    let mut player_delete_target = use_signal(|| None::<PlayerId>);

    let teams = state.read().teams.clone();
    let assets = library_assets(&state.read().assets);
    let selected_team =
        selected_team_id().and_then(|id| teams.iter().find(|team| team.id == id).cloned());
    let selected_team_for_delete = selected_team.clone();

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-teams",
                Panel {
                    title: format!("Teams ({})", teams.len()),
                    description: "Select a team to edit its identity, colours and roster.".to_owned(),
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let id = store.create_team();
                                selected_team_id.set(Some(id));
                                section.set(TeamSection::Info);
                                editing_player_id.set(None);
                            },
                            "+ New team"
                        }
                    }
                    if teams.is_empty() {
                        div { class: "fgc-asset-empty",
                            p { class: "fgc-asset-empty__title", "No teams yet." }
                            p { class: "fgc-note", "Create a team to add a logo, colours and players." }
                        }
                    } else {
                        div { class: "fgc-team-list",
                            for team in teams {
                                {
                                    let logo = find_asset(&assets, team.logo.as_ref()).cloned();
                                    let is_selected = selected_team_id().as_ref() == Some(&team.id);
                                    rsx! {
                                        TeamListItem {
                                            key: "{team.id}",
                                            team: team.clone(),
                                            logo,
                                            selected: is_selected,
                                            on_select: move |id: TeamId| {
                                                selected_team_id.set(Some(id));
                                                section.set(TeamSection::Info);
                                                editing_player_id.set(None);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(team) = selected_team.clone() {
                    TeamDetail {
                        team: team.clone(),
                        assets: assets.clone(),
                        section: section(),
                        editing_player_id: editing_player_id(),
                        on_section: move |next| section.set(next),
                        on_edit_player: move |id| {
                            editing_player_id.set(Some(id));
                            section.set(TeamSection::Players);
                            focus_element_later("fgc-player-name");
                        },
                        on_add_player: move |team_id: TeamId| {
                            if let Some(id) = store.create_player(&team_id) {
                                editing_player_id.set(Some(id));
                                section.set(TeamSection::Players);
                                focus_element_later("fgc-player-name");
                            }
                        },
                        on_done_player: move |_| {
                            let card = editing_player_id()
                                .map(|id| format!("fgc-player-card-{id}"));
                            editing_player_id.set(None);
                            if let Some(card) = card {
                                focus_element_later(&card);
                            }
                        },
                        on_delete_player: move |id| player_delete_target.set(Some(id)),
                        on_delete_team: move |team: Team| {
                            delete_references.set(store.team_references(&team.id));
                            delete_target.set(Some(team));
                        },
                    }
                } else {
                    Panel {
                        title: "No team selected".to_owned(),
                        description: "Choose a team from the list, or create a new one.".to_owned(),
                        div { class: "fgc-empty", "Select a team to edit its roster." }
                    }
                }
            }
        }

        Dialog {
            open: delete_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { delete_target.set(None) },
            if let Some(team) = delete_target() {
                DialogTitle { "Delete {team.name}?" }
                if delete_references().is_empty() {
                    DialogDescription {
                        "This team is not used by any tournament or the current match. Deleting it cannot be undone."
                    }
                } else {
                    DialogDescription {
                        "This team is still used. Remove it from the following places, then delete it:"
                    }
                    ul { class: "fgc-reference-list",
                        for reference in delete_references() {
                            li { "{reference.summary()}" }
                        }
                    }
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
                            let Some(team) = delete_target() else {
                                return;
                            };
                            if delete_references().is_empty() {
                                let _ = store.delete_team(&team.id);
                            } else {
                                store.delete_team_with_cleanup(&team.id);
                            }
                            selected_team_id.set(None);
                            editing_player_id.set(None);
                            delete_target.set(None);
                            delete_references.set(Vec::new());
                        },
                        if delete_references().is_empty() {
                            "Delete team"
                        } else {
                            "Remove references and delete"
                        }
                    }
                }
            }
        }

        Dialog {
            open: player_delete_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { player_delete_target.set(None) },
            if let Some(player_id) = player_delete_target() {
                if let Some(team) = selected_team_for_delete.clone() {
                    if let Some(player) = team.players.iter().find(|player| player.id == player_id) {
                        DialogTitle { "Delete {player.name}?" }
                        DialogDescription {
                            "The player is removed from {team.name}'s roster. This cannot be undone."
                        }
                        div { class: "fgc-dialog-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| player_delete_target.set(None),
                                "Cancel"
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| {
                                    let Some(player_id) = player_delete_target() else {
                                        return;
                                    };
                                    store.delete_player(&team.id, &player_id);
                                    editing_player_id.set(None);
                                    player_delete_target.set(None);
                                    focus_element_later("fgc-add-player");
                                },
                                "Delete player"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The selected team with its header, tabs and active section.
#[component]
fn TeamDetail(
    team: Team,
    assets: Vec<CoreAsset>,
    section: TeamSection,
    editing_player_id: Option<PlayerId>,
    on_section: EventHandler<TeamSection>,
    on_edit_player: EventHandler<PlayerId>,
    on_add_player: EventHandler<TeamId>,
    on_done_player: EventHandler<()>,
    on_delete_player: EventHandler<PlayerId>,
    on_delete_team: EventHandler<Team>,
) -> Element {
    let logo = find_asset(&assets, team.logo.as_ref()).cloned();
    let team_for_delete = team.clone();
    let team_id = team.id.clone();

    rsx! {
        div { class: "fgc-team-detail",
            div { class: "fgc-team-header",
                TeamBadge {
                    team: team.clone(),
                    logo: logo.clone(),
                    class: "fgc-team-badge--lg".to_owned(),
                }
                div { class: "fgc-team-header__identity",
                    p { class: "fgc-team-header__name", "{team.name}" }
                    p { class: "fgc-team-header__tag",
                        "[{team.tag}] · {team.players.len()} players"
                    }
                }
                div { class: "fgc-team-header__actions",
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_delete_team.call(team_for_delete.clone()),
                        "Delete team"
                    }
                }
            }
            div { class: "fgc-tabs",
                button {
                    r#type: "button",
                    aria_pressed: section == TeamSection::Info,
                    class: if section == TeamSection::Info {
                        "fgc-tab fgc-tab--active"
                    } else {
                        "fgc-tab"
                    },
                    onclick: move |_| on_section.call(TeamSection::Info),
                    "Team Info"
                }
                button {
                    r#type: "button",
                    aria_pressed: section == TeamSection::Players,
                    class: if section == TeamSection::Players {
                        "fgc-tab fgc-tab--active"
                    } else {
                        "fgc-tab"
                    },
                    onclick: move |_| on_section.call(TeamSection::Players),
                    "Players ({team.players.len()})"
                }
            }
            if section == TeamSection::Info {
                TeamInfoPanel { team: team.clone(), logo }
            } else {
                RosterPanel {
                    team: team.clone(),
                    assets,
                    editing_player_id,
                    on_add_player: move |_| on_add_player.call(team_id.clone()),
                    on_edit_player,
                    on_done_player,
                    on_delete_player,
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ColorRules {
    color_from_logo: bool,
    text_color_from_logo: bool,
    computed_contrast: bool,
}

async fn sample_logo_colors(
    store: AppStore,
    team_id: TeamId,
    asset_id: fgc_core::AssetId,
    rules: ColorRules,
) {
    let asset = {
        let state = store.state();
        state
            .read()
            .assets
            .iter()
            .find(|asset| asset.id == asset_id)
            .cloned()
    };
    let Some(asset) = asset else {
        return;
    };
    let Some(url) = store.asset_data_url(asset).await else {
        return;
    };
    let Some(palette) = sample_logo(&url).await else {
        store.notify("The selected logo has no colours to sample.");
        return;
    };

    let current_background = {
        let state = store.state();
        state
            .read()
            .teams
            .iter()
            .find(|team| team.id == team_id)
            .map(|team| team.color.clone())
            .unwrap_or_default()
    };
    let background = if rules.color_from_logo {
        palette.background().to_owned()
    } else {
        current_background
    };
    let text = if rules.text_color_from_logo {
        palette.text_for_background(&background, rules.computed_contrast)
    } else if rules.computed_contrast {
        contrast_text_color(&background)
    } else {
        None
    };

    store.update_team(&team_id, move |team| {
        if rules.color_from_logo {
            team.color = palette.background().to_owned();
        }
        if let Some(text) = text {
            team.text_color = text;
        }
    });
    store.notify("Recommended colours applied from the logo.");
}

fn rules_for(team: &Team) -> ColorRules {
    ColorRules {
        color_from_logo: team.color_from_logo,
        text_color_from_logo: team.text_color_from_logo,
        computed_contrast: team.computed_contrast,
    }
}

/// Basic information, logo, colours and a live team preview.
#[component]
fn TeamInfoPanel(team: Team, logo: Option<CoreAsset>) -> Element {
    let store = use_context::<AppStore>();
    let name_id = team.id.clone();
    let tag_id = team.id.clone();
    let logo_id = team.id.clone();
    let sample_id = team.id.clone();
    let color_id = team.id.clone();
    let text_color_id = team.id.clone();
    let color_from_logo_id = team.id.clone();
    let text_from_logo_id = team.id.clone();
    let computed_contrast_id = team.id.clone();
    let aux_color_id = team.id.clone();
    let logo_background_id = team.id.clone();
    let background_value = team.color.clone();
    let text_value = team.text_color.clone();
    let logo_value = team.logo.clone();
    let logo_rules = rules_for(&team);
    let color_from_logo = team.color_from_logo;
    let text_color_from_logo = team.text_color_from_logo;
    let computed_contrast = team.computed_contrast;

    rsx! {
        div { class: "fgc-grid fgc-grid--two",
            Panel { title: "Basic info".to_owned(),
                Field { label: "Team name".to_owned(),
                    Input {
                        value: "{team.name}",
                        aria_label: "Team name",
                        maxlength: MAX_TEAM_NAME_CHARS,
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            store.update_team(&name_id, move |team| team.name = value);
                        },
                    }
                }
                Field { label: "Team tag".to_owned(), hint: Some("3–4 letters".to_owned()),
                    Input {
                        value: "{team.tag}",
                        aria_label: "Team tag",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            store.update_team(&tag_id, move |team| team.tag = value);
                        },
                    }
                }
            }
            Panel { title: "Logo".to_owned(),
                AssetPicker {
                    label: "Team logo".to_owned(),
                    category: AssetCategory::Logo,
                    value: team.logo.clone(),
                    on_change: {
                        let rules = logo_rules;
                        let team_id = sample_id.clone();
                        move |id: Option<fgc_core::AssetId>| {
                            store.update_team(&logo_id, {
                                let id = id.clone();
                                move |team| team.logo = id
                            });
                            if let Some(id) = id {
                                let store = store;
                                let team_id = team_id.clone();
                                spawn(async move {
                                    sample_logo_colors(store, team_id, id, rules).await;
                                });
                            }
                        }
                    },
                }
                p { class: "fgc-note",
                    "Logos come from the Logo category. Automatic from logo fills the selected colour field with the recommended value."
                }
            }
            Panel { title: "Colours".to_owned(),
                ColorField {
                    label: "Background".to_owned(),
                    value: team.color.clone(),
                    disabled: team.color_from_logo,
                    on_change: {
                        let color_id = color_id.clone();
                        let text_color_id = text_color_id.clone();
                        let rules = logo_rules;
                        let text_value = text_value.clone();
                        move |value: String| {
                            let background = value.clone();
                            store.update_team(&color_id, move |team| team.color = background);
                            if rules.text_color_from_logo {
                                if let Some(text) = harmonize_text_color(&text_value, &value, rules.computed_contrast) {
                                    store.update_team(&text_color_id, move |team| team.text_color = text);
                                }
                            } else if rules.computed_contrast
                                && let Some(text) = contrast_text_color(&value)
                            {
                                store.update_team(&text_color_id, move |team| team.text_color = text);
                            }
                        }
                    },
                }
                ToggleRow {
                    title: "Background from logo".to_owned(),
                    hint: Some("Fills Background with the recommended logo colour.".to_owned()),
                    Switch {
                        checked: team.color_from_logo,
                        aria_label: "Background from logo",
                        on_checked_change: {
                            let id = color_from_logo_id.clone();
                            let logo = logo_value.clone();
                            let team_id = sample_id.clone();
                            move |checked| {
                                let rules = ColorRules {
                                    color_from_logo: checked,
                                    text_color_from_logo,
                                    computed_contrast,
                                };
                                store.update_team(&id, move |team| team.color_from_logo = checked);
                                if checked {
                                    if let Some(asset_id) = logo.clone() {
                                        let store = store;
                                        let team_id = team_id.clone();
                                        spawn(async move {
                                            sample_logo_colors(store, team_id, asset_id, rules).await;
                                        });
                                    } else {
                                        store.notify("Choose a logo first.");
                                    }
                                }
                            }
                        },
                    }
                }
                ColorField {
                    label: if team.text_color_from_logo {
                        "Text (from logo)".to_owned()
                    } else if team.computed_contrast {
                        "Text (computed)".to_owned()
                    } else {
                        "Text".to_owned()
                    },
                    value: team.text_color.clone(),
                    disabled: team.text_color_from_logo || team.computed_contrast,
                    on_change: {
                        let id = text_color_id.clone();
                        move |value| store.update_team(&id, move |team| team.text_color = value)
                    },
                }
                ToggleRow {
                    title: "Text from logo".to_owned(),
                    hint: Some("Fills Text with the recommended logo colour.".to_owned()),
                    Switch {
                        checked: team.text_color_from_logo,
                        aria_label: "Text from logo",
                        on_checked_change: {
                            let id = text_from_logo_id.clone();
                            let text_id = text_color_id.clone();
                            let logo = logo_value.clone();
                            let team_id = sample_id.clone();
                            let background = background_value.clone();
                            move |checked| {
                                let rules = ColorRules {
                                    color_from_logo,
                                    text_color_from_logo: checked,
                                    computed_contrast,
                                };
                                store.update_team(&id, move |team| team.text_color_from_logo = checked);
                                if checked {
                                    if let Some(asset_id) = logo.clone() {
                                        let store = store;
                                        let team_id = team_id.clone();
                                        spawn(async move {
                                            sample_logo_colors(store, team_id, asset_id, rules).await;
                                        });
                                    } else {
                                        store.notify("Choose a logo first.");
                                    }
                                } else if rules.computed_contrast
                                    && let Some(text) = contrast_text_color(&background)
                                {
                                    store.update_team(&text_id, move |team| team.text_color = text);
                                }
                            }
                        },
                    }
                }
                ToggleRow {
                    title: "Computed contrast".to_owned(),
                    hint: Some("Applies to Text only and can run with automatic logo colours.".to_owned()),
                    Switch {
                        checked: team.computed_contrast,
                        aria_label: "Computed contrast",
                        on_checked_change: {
                            let id = computed_contrast_id.clone();
                            let text_id = text_color_id.clone();
                            let background = background_value.clone();
                            let text = text_value.clone();
                            move |checked| {
                                let rules = ColorRules {
                                    color_from_logo,
                                    text_color_from_logo,
                                    computed_contrast: checked,
                                };
                                store.update_team(&id, move |team| team.computed_contrast = checked);
                                let text = if rules.text_color_from_logo {
                                    harmonize_text_color(&text, &background, checked)
                                } else if checked {
                                    contrast_text_color(&background)
                                } else {
                                    None
                                };
                                if let Some(text) = text {
                                    store.update_team(&text_id, move |team| team.text_color = text);
                                }
                            }
                        },
                    }
                }
                p { class: "fgc-note",
                    "Computed contrast changes Text only. Turn off an automatic option to edit that colour by hand."
                }
                ColorField {
                    label: "Logo background".to_owned(),
                    value: team.aux_color.clone(),
                    on_change: move |value| store.update_team(&aux_color_id, move |team| team.aux_color = value),
                }
                ToggleRow {
                    title: "Separate logo background".to_owned(),
                    hint: Some(
                        if team.logo_background {
                            "The logo sits on its own colour."
                        } else {
                            "The logo uses the normal background colour."
                        }
                        .to_owned(),
                    ),
                    Switch {
                        checked: team.logo_background,
                        aria_label: "Separate logo background",
                        on_checked_change: move |checked| {
                            store.update_team(&logo_background_id, move |team| team.logo_background = checked);
                        },
                    }
                }
            }
            CollapsiblePanel { title: "Preview".to_owned(),
                TeamPreview { team: team.clone(), logo }
            }
        }
    }
}

/// A native colour input with its current hex value.
#[component]
fn ColorField(
    label: String,
    value: String,
    on_change: EventHandler<String>,
    #[props(default)] disabled: bool,
) -> Element {
    rsx! {
        Field { label: label.clone(),
            div { class: "fgc-color-field",
                input {
                    r#type: "color",
                    value: "{value}",
                    aria_label: "{label}",
                    disabled: disabled,
                    oninput: move |event: FormEvent| on_change.call(event.value()),
                }
                code { class: "fgc-color-field__value", "{value}" }
            }
        }
    }
}

/// A team badge with the logo when one is selected.
#[component]
fn TeamBadge(team: Team, logo: Option<CoreAsset>, #[props(default)] class: String) -> Element {
    let badge_class = if class.is_empty() {
        "fgc-team-badge".to_owned()
    } else {
        format!("fgc-team-badge {class}")
    };
    let background = if team.logo_background {
        team.aux_color.clone()
    } else {
        team.color.clone()
    };

    rsx! {
        div { class: "{badge_class}", style: "background: {background};",
            if let Some(asset) = logo {
                AssetThumb {
                    key: "{asset.id}",
                    asset,
                    class: "fgc-team-badge__image".to_owned(),
                }
            } else {
                span { style: "color: {team.text_color};", "{team.tag}" }
            }
        }
    }
}

/// A preview of the team as it appears on the overlays.
#[component]
fn TeamPreview(team: Team, logo: Option<CoreAsset>) -> Element {
    rsx! {
        div {
            class: "fgc-team-preview",
            style: "background: {team.color}; color: {team.text_color};",
            TeamBadge {
                team: team.clone(),
                logo,
                class: "fgc-team-badge--preview".to_owned(),
            }
            div { class: "fgc-team-preview__text",
                span { class: "fgc-team-preview__tag", "[{team.tag}]" }
                span { class: "fgc-team-preview__name", "{team.name}" }
            }
        }
    }
}

/// One row in the team sidebar.
#[component]
fn TeamListItem(
    team: Team,
    logo: Option<CoreAsset>,
    selected: bool,
    on_select: EventHandler<TeamId>,
) -> Element {
    let team_id = team.id.clone();
    let item_class = if selected {
        "fgc-team-list__item fgc-team-list__item--active"
    } else {
        "fgc-team-list__item"
    };

    rsx! {
        button {
            r#type: "button",
            class: "{item_class}",
            onclick: move |_| on_select.call(team_id.clone()),
            div { class: "fgc-team-list__swatch", style: "background: {team.color};" }
            TeamBadge {
                team: team.clone(),
                logo,
                class: "fgc-team-list__badge".to_owned(),
            }
            div { class: "fgc-team-list__info",
                span { class: "fgc-team-list__tag", "{team.tag}" }
                span { class: "fgc-team-list__name", "{team.name}" }
            }
            span { class: "fgc-team-list__count", "{team.players.len()}" }
        }
    }
}

/// Ordered roster list or the editor for the selected player.
#[component]
fn RosterPanel(
    team: Team,
    assets: Vec<CoreAsset>,
    editing_player_id: Option<PlayerId>,
    on_add_player: EventHandler<()>,
    on_edit_player: EventHandler<PlayerId>,
    on_done_player: EventHandler<()>,
    on_delete_player: EventHandler<PlayerId>,
) -> Element {
    let editing_player = editing_player_id
        .as_ref()
        .and_then(|id| team.players.iter().find(|player| &player.id == id).cloned());

    rsx! {
        if let Some(player) = editing_player {
            PlayerEditor {
                key: "{player.id}",
                team_id: team.id.clone(),
                player,
                assets: assets.clone(),
                on_done: move |_| on_done_player.call(()),
                on_request_delete: move |id| on_delete_player.call(id),
            }
        } else {
            Panel {
                title: format!("Players ({})", team.players.len()),
                description: "Roster order is preserved in the saved project and overlays.".to_owned(),
                div { class: "fgc-roster-header",
                    p { class: "fgc-note",
                        "Core players, substitutes, managers and coaches share one ordered roster."
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Sm,
                        id: "fgc-add-player",
                        onclick: move |_| on_add_player.call(()),
                        "+ Add player"
                    }
                }
                if team.players.is_empty() {
                    div { class: "fgc-asset-empty",
                        p { class: "fgc-asset-empty__title", "No players yet." }
                        p { class: "fgc-note", "Add a player to build the roster." }
                    }
                } else {
                    div { class: "fgc-player-list",
                        for player in team.players {
                            {
                                let portrait = find_asset(&assets, player.portrait.as_ref()).cloned();
                                let hero = find_asset(&assets, player.featured_hero.as_ref()).cloned();
                                rsx! {
                                    PlayerCard {
                                        key: "{player.id}",
                                        player: player.clone(),
                                        portrait,
                                        hero,
                                        on_edit: move |id: PlayerId| on_edit_player.call(id),
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

/// Player fields with asset pickers for portrait and featured hero.
#[component]
fn PlayerEditor(
    team_id: TeamId,
    player: Player,
    assets: Vec<CoreAsset>,
    on_done: EventHandler<()>,
    on_request_delete: EventHandler<PlayerId>,
) -> Element {
    let store = use_context::<AppStore>();
    let player_id = player.id.clone();

    let name_team = team_id.clone();
    let name_player = player_id.clone();
    let role_team = team_id.clone();
    let role_player = player_id.clone();
    let position_team = team_id.clone();
    let position_player = player_id.clone();
    let portrait_team = team_id.clone();
    let portrait_player = player_id.clone();
    let hero_team = team_id.clone();
    let hero_player = player_id.clone();
    let pronouns_team = team_id.clone();
    let pronouns_player = player_id.clone();

    rsx! {
        Panel {
            title: format!("Edit {}", player.name),
            description: "Changes are saved automatically.".to_owned(),
            Field { label: "Player name".to_owned(),
                Input {
                    value: "{player.name}",
                    aria_label: "Player name",
                    id: "fgc-player-name",
                    maxlength: MAX_PLAYER_NAME_CHARS,
                    oninput: move |event: FormEvent| {
                        let value = event.value();
                        store.update_player(&name_team, &name_player, move |player| player.name = value);
                    },
                }
            }
            div { class: "fgc-form-row",
                Field { label: "Role".to_owned(),
                    select {
                        class: "fgc-native-select",
                        value: "{player.role.slug()}",
                        aria_label: "Player role",
                        onchange: move |event: FormEvent| {
                            if let Some(role) = Role::from_slug(&event.value()) {
                                store.update_player(&role_team, &role_player, move |player| player.role = role);
                            }
                        },
                        for role in Role::ALL {
                            option {
                                value: "{role.slug()}",
                                selected: role == player.role,
                                "{role.label()}"
                            }
                        }
                    }
                }
                Field { label: "Position".to_owned(),
                    select {
                        class: "fgc-native-select",
                        value: "{player.position.slug()}",
                        aria_label: "Player position",
                        onchange: move |event: FormEvent| {
                            if let Some(position) = Position::from_slug(&event.value()) {
                                store.update_player(&position_team, &position_player, move |player| player.position = position);
                            }
                        },
                        for position in Position::ALL {
                            option {
                                value: "{position.slug()}",
                                selected: position == player.position,
                                "{position.label()}"
                            }
                        }
                    }
                }
            }
            div { class: "fgc-grid fgc-grid--two",
                AssetPicker {
                    label: "Portrait".to_owned(),
                    category: AssetCategory::Portrait,
                    value: player.portrait.clone(),
                    on_change: move |id| store.update_player(&portrait_team, &portrait_player, move |player| player.portrait = id),
                }
                AssetPicker {
                    label: "Featured hero".to_owned(),
                    category: AssetCategory::Hero,
                    value: player.featured_hero.clone(),
                    on_change: move |id| store.update_player(&hero_team, &hero_player, move |player| player.featured_hero = id),
                }
            }
            Field {
                label: "Pronouns".to_owned(),
                hint: Some("Optional, for example he/him".to_owned()),
                Input {
                    value: player.pronouns.clone().unwrap_or_default(),
                    aria_label: "Player pronouns",
                    oninput: move |event: FormEvent| {
                        let value = event.value();
                        let pronouns = (!value.trim().is_empty()).then_some(value);
                        store.update_player(&pronouns_team, &pronouns_player, move |player| player.pronouns = pronouns);
                    },
                }
            }
            CollapsiblePanel {
                title: "Roster card preview".to_owned(),
                description: "Exactly how this player appears in the roster list.".to_owned(),
                div { class: "fgc-player-preview",
                    PlayerCardBody {
                        player: player.clone(),
                        portrait: find_asset(&assets, player.portrait.as_ref()).cloned(),
                        hero: find_asset(&assets, player.featured_hero.as_ref()).cloned(),
                    }
                }
            }
            div { class: "fgc-actions",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| on_done.call(()),
                    "Done"
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: move |_| on_request_delete.call(player_id.clone()),
                    "Delete player"
                }
            }
        }
    }
}

/// One roster card with portrait, role, position and featured hero.
#[component]
fn PlayerCard(
    player: Player,
    portrait: Option<CoreAsset>,
    hero: Option<CoreAsset>,
    on_edit: EventHandler<PlayerId>,
) -> Element {
    let player_id = player.id.clone();

    rsx! {
        button {
            r#type: "button",
            class: "fgc-player-card",
            id: "fgc-player-card-{player_id}",
            onclick: move |_| on_edit.call(player_id.clone()),
            PlayerCardBody { player, portrait, hero }
        }
    }
}

/// The shared visual body of a roster card, also used by the editor preview.
#[component]
fn PlayerCardBody(player: Player, portrait: Option<CoreAsset>, hero: Option<CoreAsset>) -> Element {
    let initial = player
        .name
        .chars()
        .next()
        .map(String::from)
        .unwrap_or_default();

    rsx! {
        div { class: "fgc-player-card__avatar",
            if let Some(asset) = portrait {
                AssetThumb {
                    key: "{asset.id}",
                    asset,
                    class: "fgc-player-card__portrait".to_owned(),
                }
            } else {
                span { class: "fgc-player-card__initial", "{initial}" }
            }
        }
        div { class: "fgc-player-card__info",
            span { class: "fgc-player-card__name", "{player.name}" }
            span { class: "fgc-player-card__meta",
                "{player.role.label()} · {player.position.label()}"
            }
            if let Some(pronouns) = player.pronouns.clone() {
                span { class: "fgc-player-card__pronouns", "{pronouns}" }
            }
        }
        if let Some(asset) = hero {
            div { class: "fgc-player-card__hero",
                AssetThumb {
                    key: "{asset.id}",
                    asset,
                    class: "fgc-player-card__hero-image".to_owned(),
                }
            }
        }
    }
}
