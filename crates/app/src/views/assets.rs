//! Assets destination: the content-addressed image library.
//!
//! Bundled assets are shown read-only because they live inside the binary.
//! User assets can be imported from a file or URL, previewed, renamed and
//! deleted. Deleting an asset clears every reference to it and lets garbage
//! collection reclaim the bytes after the next save.

use dioxus::prelude::*;
use fgc_core::{Asset as CoreAsset, AssetCategory, AssetId, AssetOrigin};

use crate::components::asset_picker::AssetThumb;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::input::Input;
use crate::components::{Field, Panel};
use crate::state::AppStore;

/// The asset library with filtering, import, preview and bundle controls.
#[component]
pub fn AssetsView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let busy = store.asset_busy();

    let mut category = use_signal(|| None::<AssetCategory>);
    let mut search = use_signal(String::new);
    let mut preview = use_signal(|| None::<CoreAsset>);
    let mut rename_target = use_signal(|| None::<CoreAsset>);
    let mut rename_text = use_signal(String::new);
    let mut delete_target = use_signal(|| None::<CoreAsset>);
    let mut property_target = use_signal(|| None::<AssetId>);
    let mut import_category = use_signal(|| AssetCategory::Logo);
    let mut show_url_dialog = use_signal(|| false);
    let mut url_text = use_signal(String::new);
    let mut url_error = use_signal(|| None::<String>);
    let mut confirm_import = use_signal(|| false);

    let user_assets = state
        .read()
        .assets
        .iter()
        .filter(|asset| !asset.is_attachment())
        .cloned()
        .collect::<Vec<_>>();
    let bundled = fgc_server::bundled_catalog();
    let mut all_assets: Vec<CoreAsset> = bundled;
    all_assets.extend(user_assets);

    let counts: Vec<(AssetCategory, usize)> = AssetCategory::ALL
        .into_iter()
        .map(|category| {
            let count = all_assets
                .iter()
                .filter(|asset| asset.category == category)
                .count();
            (category, count)
        })
        .collect();

    let search_text = search.read().trim().to_lowercase();
    let selected = category();
    let filtered: Vec<CoreAsset> = all_assets
        .iter()
        .filter(|asset| {
            selected.is_none_or(|category| asset.category == category)
                && (search_text.is_empty() || asset.name.to_lowercase().contains(&search_text))
        })
        .cloned()
        .collect();

    let mode_options: Vec<CoreAsset> = all_assets
        .iter()
        .filter(|asset| asset.category == AssetCategory::GameMode)
        .cloned()
        .collect();

    let (property_asset, property_icon) = {
        let assets = &state.read().assets;
        let asset = property_target()
            .and_then(|id| assets.iter().find(|candidate| candidate.id == id).cloned());
        let icon = asset
            .as_ref()
            .and_then(|asset| asset.icon.as_ref())
            .and_then(|id| assets.iter().find(|candidate| &candidate.id == id).cloned());
        (asset, icon)
    };

    let preview_url = use_resource(move || {
        let asset = preview();
        async move {
            match asset {
                Some(asset) => store.asset_data_url(asset).await,
                None => None,
            }
        }
    });

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-grid fgc-grid--two",
                Panel {
                    title: "Import".to_owned(),
                    description: "Add an image to the library. Identical bytes are stored once.".to_owned(),
                    Field { label: "Category".to_owned(),
                        select {
                            class: "fgc-native-select",
                            value: "{import_category().slug()}",
                            aria_label: "Import category",
                            onchange: move |event| {
                                if let Some(category) = AssetCategory::from_slug(&event.value()) {
                                    import_category.set(category);
                                }
                            },
                            for category in AssetCategory::ALL {
                                option {
                                    value: "{category.slug()}",
                                    selected: category == import_category(),
                                    "{category.label()}"
                                }
                            }
                        }
                    }
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Primary,
                            disabled: busy,
                            onclick: move |_| store.import_assets_from_files(import_category()),
                            if busy { "Working…" } else { "Import files…" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| {
                                url_error.set(None);
                                show_url_dialog.set(true);
                            },
                            "Import from URL…"
                        }
                    }
                    p { class: "fgc-note",
                        "PNG, JPEG, WebP, GIF and SVG up to 20 MB. SVG files are checked for script and external references before they are stored."
                    }
                }
                Panel {
                    title: "Project bundles".to_owned(),
                    description: "Move a project and its referenced images to another machine.".to_owned(),
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| store.export_bundle(),
                            "Export project…"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| confirm_import.set(true),
                            "Import project…"
                        }
                    }
                    p { class: "fgc-note",
                        "An export is a folder containing app.json, export.json and the referenced user images. Bundled art comes from the application and is never copied."
                    }
                }
            }
            Panel {
                title: format!("Library ({})", filtered.len()),
                description: "Bundled assets ship with the application; imported assets are content-addressed and read-only once stored.".to_owned(),
                div { class: "fgc-asset-toolbar",
                    Input {
                        r#type: "search",
                        placeholder: "Search assets by name",
                        value: "{search}",
                        aria_label: "Search assets",
                        oninput: move |event: FormEvent| search.set(event.value()),
                    }
                    div { class: "fgc-asset-filters",
                        button {
                            r#type: "button",
                            class: chip_class(selected.is_none()),
                            onclick: move |_| category.set(None),
                            "All ({all_assets.len()})"
                        }
                        for (option, count) in counts {
                            button {
                                key: "{option.slug()}",
                                r#type: "button",
                                class: chip_class(selected == Some(option)),
                                onclick: move |_| category.set(Some(option)),
                                "{option.label()} ({count})"
                            }
                        }
                    }
                }
                if filtered.is_empty() {
                    div { class: "fgc-asset-empty",
                        p { class: "fgc-asset-empty__title", "No assets match this filter." }
                        p { class: "fgc-note", "Try another category, clear the search, or import an image above." }
                    }
                } else {
                    div { class: "fgc-asset-grid",
                        for asset in filtered {
                            {
                                rsx! {
                                    AssetCard {
                                        key: "{asset.id}",
                                        asset: asset.clone(),
                                        mode_options: mode_options.clone(),
                                        on_preview: move |asset: CoreAsset| preview.set(Some(asset)),
                                        on_rename: move |asset: CoreAsset| {
                                            rename_text.set(asset.name.clone());
                                            rename_target.set(Some(asset));
                                        },
                                        on_delete: move |asset: CoreAsset| delete_target.set(Some(asset)),
                                        on_game_mode: move |(asset, value): (CoreAsset, Option<AssetId>)| {
                                            store.set_asset_game_mode(asset.id, value);
                                        },
                                        on_properties: move |asset: CoreAsset| {
                                            property_target.set(Some(asset.id));
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Dialog {
            open: preview().is_some(),
            on_open_change: move |open: bool| if !open { preview.set(None) },
            if let Some(asset) = preview() {
                DialogTitle { "{asset.name}" }
                DialogDescription {
                    "{asset.category.label()} · {asset.ext.to_uppercase()} · {short_hash(&asset.hash)}"
                }
                div { class: "fgc-asset-preview",
                    if let Some(url) = preview_url().flatten() {
                        img { src: "{url}", alt: "{asset.name}" }
                    } else {
                        p { class: "fgc-muted", "Preview unavailable." }
                    }
                }
                div { class: "fgc-dialog-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| preview.set(None),
                        "Close"
                    }
                }
            }
        }

        Dialog {
            open: rename_target().is_some(),
            on_open_change: move |open: bool| if !open { rename_target.set(None) },
            DialogTitle { "Rename asset" }
            DialogDescription {
                "Only the display name changes. The image data and its content-addressed URL stay the same."
            }
            Field { label: "Name".to_owned(),
                Input {
                    value: "{rename_text}",
                    aria_label: "Asset name",
                    oninput: move |event: FormEvent| rename_text.set(event.value()),
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| rename_target.set(None),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        let Some(asset) = rename_target() else { return; };
                        let name = rename_text.read().trim().to_owned();
                        if name.is_empty() {
                            store.report_error("Asset names cannot be empty.");
                            return;
                        }
                        store.rename_asset(asset.id, &name);
                        rename_target.set(None);
                    },
                    "Save"
                }
            }
        }

        Dialog {
            open: delete_target().is_some(),
            on_open_change: move |open: bool| if !open { delete_target.set(None) },
            DialogTitle { "Delete asset?" }
            DialogDescription {
                "The image is removed from the library and any team, player, talent or display setting that used it is cleared. This cannot be undone."
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
                        if let Some(asset) = delete_target() {
                            store.delete_asset(asset.id);
                            delete_target.set(None);
                        }
                    },
                    "Delete"
                }
            }
        }

        Dialog {
            open: property_asset.is_some(),
            on_open_change: move |open: bool| if !open { property_target.set(None) },
            if let Some(asset) = property_asset.clone() {
                DialogTitle { "{asset.name} properties" }
                DialogDescription {
                    "Attached images belong to this hero. They are not added to the library and cannot be chosen for another hero."
                }
                div { class: "fgc-asset-properties",
                    div { class: "fgc-asset-properties__hero",
                        AssetThumb {
                            asset: asset.clone(),
                            class: "fgc-asset-properties__image".to_owned(),
                            placeholder_class: "fgc-asset-properties__placeholder".to_owned(),
                        }
                        p { class: "fgc-note", "{asset.category.label()} · {asset.ext.to_uppercase()} · {short_hash(&asset.hash)}" }
                    }
                    div { class: "fgc-asset-properties__section",
                        h3 { "Ban icon" }
                        if let Some(icon) = property_icon.clone() {
                            div { class: "fgc-asset-properties__icon",
                                AssetThumb {
                                    asset: icon,
                                    class: "fgc-asset-properties__icon-image".to_owned(),
                                }
                                p { class: "fgc-note", "Uploaded for this hero. It is hidden from hero pickers and ban lists." }
                            }
                        } else {
                            p { class: "fgc-note", "No ban icon yet. The hero art is used in the ban strip." }
                        }
                        div { class: "fgc-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: busy,
                                onclick: {
                                    let hero_id = asset.id.clone();
                                    move |_| store.upload_hero_ban_icon(hero_id.clone())
                                },
                                if property_icon.is_some() { "Replace ban icon…" } else { "Upload ban icon…" }
                            }
                            if property_icon.is_some() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    onclick: {
                                        let hero_id = asset.id.clone();
                                        move |_| store.clear_hero_ban_icon(hero_id.clone())
                                    },
                                    "Remove ban icon"
                                }
                            }
                        }
                    }
                }
                div { class: "fgc-dialog-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| property_target.set(None),
                        "Close"
                    }
                }
            }
        }

        Dialog {
            open: show_url_dialog(),
            on_open_change: move |open: bool| if !open { show_url_dialog.set(false) },
            DialogTitle { "Import from URL" }
            DialogDescription {
                "The image is downloaded once, validated and stored locally. Only http and https URLs are accepted."
            }
            Field { label: "Image URL".to_owned(), error: url_error(),
                Input {
                    r#type: "url",
                    placeholder: "https://example.com/logo.png",
                    value: "{url_text}",
                    aria_label: "Image URL",
                    oninput: move |event: FormEvent| {
                        url_text.set(event.value());
                        url_error.set(None);
                    },
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| show_url_dialog.set(false),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    disabled: busy,
                    onclick: move |_| {
                        let raw = url_text.read().clone();
                        match crate::assets_io::validate_import_url(&raw) {
                            Ok(_) => {
                                store.import_asset_from_url(import_category(), &raw);
                                url_error.set(None);
                                show_url_dialog.set(false);
                            }
                            Err(error) => url_error.set(Some(error)),
                        }
                    },
                    "Import"
                }
            }
        }

        Dialog {
            open: confirm_import(),
            on_open_change: move |open: bool| if !open { confirm_import.set(false) },
            DialogTitle { "Replace the current project?" }
            DialogDescription {
                "Importing a bundle replaces teams, talent, tournaments, settings and the asset library. The current project is not kept."
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| confirm_import.set(false),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    disabled: busy,
                    onclick: move |_| {
                        store.import_bundle();
                        confirm_import.set(false);
                    },
                    "Import project"
                }
            }
        }
    }
}

/// One image card with an async thumbnail and editable properties.
#[component]
fn AssetCard(
    asset: CoreAsset,
    mode_options: Vec<CoreAsset>,
    on_preview: EventHandler<CoreAsset>,
    on_rename: EventHandler<CoreAsset>,
    on_delete: EventHandler<CoreAsset>,
    on_game_mode: EventHandler<(CoreAsset, Option<AssetId>)>,
    on_properties: EventHandler<CoreAsset>,
) -> Element {
    let preview_asset = asset.clone();
    let rename_asset = asset.clone();
    let delete_asset = asset.clone();
    let mode_asset = asset.clone();
    let properties_asset = asset.clone();
    let bundled = asset.origin == AssetOrigin::Bundled;
    let extension = asset.ext.to_uppercase();
    let is_map = asset.category == AssetCategory::Map;
    let is_hero = asset.category == AssetCategory::Hero;
    let mode_value = asset
        .game_mode
        .as_ref()
        .map_or_else(String::new, ToString::to_string);
    let current_mode = asset.game_mode.clone();

    rsx! {
        article { class: "fgc-asset-card",
            button {
                class: "fgc-asset-card__preview",
                r#type: "button",
                aria_label: "Preview {asset.name}",
                onclick: move |_| on_preview.call(preview_asset.clone()),
                AssetThumb {
                    asset: asset.clone(),
                    class: "fgc-asset-card__image".to_owned(),
                    placeholder_class: "fgc-asset-card__placeholder".to_owned(),
                }
            }
            div { class: "fgc-asset-card__body",
                p { class: "fgc-asset-card__name", title: "{asset.name}", "{asset.name}" }
                div { class: "fgc-asset-card__meta",
                    span { class: "fgc-asset-card__badge", "{asset.category.label()}" }
                    span { class: "fgc-asset-card__badge", "{extension}" }
                    if bundled {
                        span { class: "fgc-asset-card__badge fgc-asset-card__badge--bundled", "Bundled" }
                    }
                }
                if is_map && !bundled {
                    Field { label: "Game mode".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Game mode",
                            value: "{mode_value}",
                            onchange: move |event: FormEvent| {
                                let value = event.value();
                                let id = if value.is_empty() {
                                    None
                                } else {
                                    Some(AssetId::from_raw(value))
                                };
                                on_game_mode.call((mode_asset.clone(), id));
                            },
                            option { value: "", "No game mode" }
                            for candidate in mode_options.iter() {
                                option {
                                    key: "{candidate.id}",
                                    value: "{candidate.id}",
                                    selected: current_mode.as_ref() == Some(&candidate.id),
                                    "{candidate.name}"
                                }
                            }
                        }
                    }
                }
            }
            if !bundled {
                div { class: if is_hero { "fgc-asset-card__actions fgc-asset-card__actions--hero" } else { "fgc-asset-card__actions" },
                    if is_hero {
                        Button {
                            class: "fgc-asset-card__properties",
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| on_properties.call(properties_asset.clone()),
                            "Properties…"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_rename.call(rename_asset.clone()),
                        "Rename"
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_delete.call(delete_asset.clone()),
                        "Delete"
                    }
                }
            }
        }
    }
}

fn chip_class(active: bool) -> &'static str {
    if active {
        "fgc-asset-chip fgc-asset-chip--active"
    } else {
        "fgc-asset-chip"
    }
}

fn short_hash(hash: &str) -> String {
    format!("{}…", hash.get(..12).unwrap_or(hash))
}
