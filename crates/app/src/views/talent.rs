//! Talent destination: casters, panel members and interviewers.
//!
//! A talent member is a display identity with optional real name, pronouns,
//! one social link and a portrait. Deleting a member clears any current-match
//! assignment that used them.

use dioxus::prelude::*;
use fgc_core::{
    Asset as CoreAsset, AssetCategory, MAX_TALENT_NAME_CHARS, SocialLink, SocialPlatform, Talent,
    TalentId,
};

use crate::components::asset_picker::{AssetPicker, AssetThumb, find_asset, library_assets};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::input::Input;
use crate::components::{Field, Panel};
use crate::state::AppStore;

/// Talent list and the editor for the selected member.
#[component]
pub fn TalentView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let mut selected_id = use_signal(|| None::<TalentId>);
    let mut delete_target = use_signal(|| None::<Talent>);

    let talents = state.read().talents.clone();
    let assets = library_assets(&state.read().assets);
    let selected =
        selected_id().and_then(|id| talents.iter().find(|talent| talent.id == id).cloned());

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-talent",
                Panel {
                    title: format!("Talent ({})", talents.len()),
                    description: "Casters, panel members and interviewers available to the match controller.".to_owned(),
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                let id = store.create_talent();
                                selected_id.set(Some(id));
                            },
                            "+ Add talent"
                        }
                    }
                    if talents.is_empty() {
                        div { class: "fgc-asset-empty",
                            p { class: "fgc-asset-empty__title", "No talent members yet." }
                            p { class: "fgc-note", "Add a caster, panel member or interviewer." }
                        }
                    } else {
                        div { class: "fgc-talent-list",
                            for talent in talents {
                                {
                                    let portrait = find_asset(&assets, talent.portrait.as_ref()).cloned();
                                    let is_selected = selected_id().as_ref() == Some(&talent.id);
                                    rsx! {
                                        TalentListItem {
                                            key: "{talent.id}",
                                            talent: talent.clone(),
                                            portrait,
                                            selected: is_selected,
                                            on_select: move |id: TalentId| selected_id.set(Some(id)),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(talent) = selected.clone() {
                    TalentEditor {
                        key: "{talent.id}",
                        talent: talent.clone(),
                        on_request_delete: move |talent: Talent| delete_target.set(Some(talent)),
                    }
                } else {
                    Panel {
                        title: "No talent selected".to_owned(),
                        description: "Choose a member from the list, or add a new one.".to_owned(),
                        div { class: "fgc-empty", "Select a talent member to edit their details." }
                    }
                }
            }
        }

        Dialog {
            open: delete_target().is_some(),
            on_open_change: move |is_open: bool| if !is_open { delete_target.set(None) },
            if let Some(talent) = delete_target() {
                {
                    let name = display_name(&talent);
                    rsx! {
                        DialogTitle { "Delete {name}?" }
                        DialogDescription {
                            "This removes the member from the talent library and clears any \
                             current-match assignment that used them. This cannot be undone."
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
                                    let Some(talent) = delete_target() else {
                                        return;
                                    };
                                    store.delete_talent(&talent.id);
                                    if selected_id().as_ref() == Some(&talent.id) {
                                        selected_id.set(None);
                                    }
                                    delete_target.set(None);
                                },
                                "Delete talent"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The display name, or a neutral fallback for an unnamed draft.
#[must_use]
fn display_name(talent: &Talent) -> String {
    let name = talent.display_name.trim();
    if name.is_empty() {
        "Unnamed talent".to_owned()
    } else {
        name.to_owned()
    }
}

/// One row in the talent sidebar.
#[component]
fn TalentListItem(
    talent: Talent,
    portrait: Option<CoreAsset>,
    selected: bool,
    on_select: EventHandler<TalentId>,
) -> Element {
    let talent_id = talent.id.clone();
    let item_class = if selected {
        "fgc-talent-list__item fgc-talent-list__item--active"
    } else {
        "fgc-talent-list__item"
    };
    let initial = talent
        .display_name
        .chars()
        .next()
        .map(String::from)
        .unwrap_or_default();
    let name = display_name(&talent);

    rsx! {
        button {
            r#type: "button",
            class: "{item_class}",
            onclick: move |_| on_select.call(talent_id.clone()),
            div { class: "fgc-talent-list__avatar",
                if let Some(asset) = portrait {
                    AssetThumb {
                        key: "{asset.id}",
                        asset,
                        class: "fgc-talent-list__portrait".to_owned(),
                    }
                } else if initial.is_empty() {
                    span { class: "fgc-talent-list__initial", "?" }
                } else {
                    span { class: "fgc-talent-list__initial", "{initial}" }
                }
            }
            div { class: "fgc-talent-list__info",
                span { class: "fgc-talent-list__name", "{name}" }
                if let Some(pronouns) = talent.pronouns.clone() {
                    span { class: "fgc-talent-list__pronouns", "{pronouns}" }
                }
            }
            if let Some(social) = talent.social.as_ref() {
                span { class: "fgc-talent-list__platform", "{social.platform.label()}" }
            }
        }
    }
}

/// Editable talent fields with portrait and social link.
#[component]
fn TalentEditor(talent: Talent, on_request_delete: EventHandler<Talent>) -> Element {
    let store = use_context::<AppStore>();
    let id = talent.id.clone();

    let display_id = id.clone();
    let real_name_id = id.clone();
    let pronouns_id = id.clone();
    let portrait_id = id.clone();
    let platform_id = id.clone();
    let handle_id = id.clone();
    let delete_talent = talent.clone();

    let social = talent.social.clone();
    let platform = social
        .as_ref()
        .map_or(SocialPlatform::Twitter, |social| social.platform);
    let handle = social.map_or_else(String::new, |social| social.handle);

    rsx! {
        div { class: "fgc-talent-detail",
            Panel {
                title: format!("Edit: {}", display_name(&talent)),
                description: "Changes are saved automatically.".to_owned(),
                Field { label: "Display name".to_owned(),
                    Input {
                        value: "{talent.display_name}",
                        aria_label: "Talent display name",
                        placeholder: "Enter display name",
                        maxlength: MAX_TALENT_NAME_CHARS,
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            store.update_talent(&display_id, move |talent| talent.display_name = value);
                        },
                    }
                }
                Field {
                    label: "Real name".to_owned(),
                    hint: Some("Optional".to_owned()),
                    Input {
                        value: talent.real_name.clone().unwrap_or_default(),
                        aria_label: "Talent real name",
                        placeholder: "Enter real name (optional)",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            let real_name = (!value.trim().is_empty()).then_some(value);
                            store.update_talent(&real_name_id, move |talent| talent.real_name = real_name);
                        },
                    }
                }
                Field {
                    label: "Pronouns".to_owned(),
                    hint: Some("Optional, for example they/them".to_owned()),
                    Input {
                        value: talent.pronouns.clone().unwrap_or_default(),
                        aria_label: "Talent pronouns",
                        placeholder: "e.g. he/him, she/her, they/them",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            let pronouns = (!value.trim().is_empty()).then_some(value);
                            store.update_talent(&pronouns_id, move |talent| talent.pronouns = pronouns);
                        },
                    }
                }
                div { class: "fgc-form-row",
                    Field { label: "Social platform".to_owned(),
                        select {
                            class: "fgc-native-select",
                            value: "{platform.slug()}",
                            aria_label: "Social platform",
                            onchange: move |event: FormEvent| {
                                let Some(next) = SocialPlatform::from_slug(&event.value()) else {
                                    return;
                                };
                                store.update_talent(&platform_id, move |talent| {
                                    let handle = talent
                                        .social
                                        .as_ref()
                                        .map_or_else(String::new, |social| social.handle.clone());
                                    talent.social = Some(SocialLink {
                                        platform: next,
                                        handle,
                                    });
                                });
                            },
                            for option in SocialPlatform::ALL {
                                option {
                                    value: "{option.slug()}",
                                    selected: option == platform,
                                    "{option.label()}"
                                }
                            }
                        }
                    }
                    Field { label: "Handle".to_owned(),
                        Input {
                            value: "{handle}",
                            aria_label: "Social handle",
                            placeholder: "@handle",
                            oninput: move |event: FormEvent| {
                                let value = event.value();
                                store.update_talent(&handle_id, move |talent| {
                                    let platform = talent
                                        .social
                                        .as_ref()
                                        .map_or(SocialPlatform::Twitter, |social| social.platform);
                                    talent.social = if value.trim().is_empty() {
                                        None
                                    } else {
                                        Some(SocialLink {
                                            platform,
                                            handle: value,
                                        })
                                    };
                                });
                            },
                        }
                    }
                }
                AssetPicker {
                    label: "Portrait".to_owned(),
                    category: AssetCategory::Portrait,
                    value: talent.portrait.clone(),
                    on_change: move |asset| {
                        store.update_talent(&portrait_id, move |talent| talent.portrait = asset);
                    },
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| on_request_delete.call(delete_talent.clone()),
                        "Delete talent"
                    }
                }
            }
        }
    }
}
