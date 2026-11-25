//! Assets destination: the content-addressed image library.
//!
//! Bundled assets are shown read-only because they live inside the binary.
//! User assets can be imported from a file or URL, previewed, replaced,
//! renamed and deleted. Replacing keeps the asset identity so every reference
//! follows the new image; deleting clears references and lets garbage
//! collection reclaim the bytes after the next save.

use std::cmp::Reverse;

use dioxus::prelude::*;
use fgc_core::{Asset as CoreAsset, AssetCategory, AssetId, AssetOrigin, Role};

use crate::components::asset_picker::AssetThumb;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::input::Input;
use crate::components::{CollapsiblePanel, Field, Panel};
use crate::state::AppStore;

/// How many asset cards one page holds.
///
/// A large library is slow to draw and slow to scroll, so the grid is cut
/// into pages the operator can step through.
const ASSETS_PER_PAGE: usize = 24;

/// The order the asset library is listed in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AssetSort {
    /// Alphabetical by name, A to Z.
    NameAsc,
    /// Alphabetical by name, Z to A.
    NameDesc,
    /// Grouped by category, then by name.
    Category,
    /// The most recently imported first.
    Newest,
    /// The oldest import first.
    Oldest,
}

impl AssetSort {
    /// The label shown in the sort dropdown.
    const fn label(self) -> &'static str {
        match self {
            Self::NameAsc => "Name (A-Z)",
            Self::NameDesc => "Name (Z-A)",
            Self::Category => "Category",
            Self::Newest => "Newest first",
            Self::Oldest => "Oldest first",
        }
    }

    /// The value the sort dropdown uses.
    const fn slug(self) -> &'static str {
        match self {
            Self::NameAsc => "name-asc",
            Self::NameDesc => "name-desc",
            Self::Category => "category",
            Self::Newest => "newest",
            Self::Oldest => "oldest",
        }
    }

    /// Every order, in the order the dropdown lists them.
    const ALL: [Self; 5] = [
        Self::NameAsc,
        Self::NameDesc,
        Self::Category,
        Self::Newest,
        Self::Oldest,
    ];

    /// The order a dropdown value names.
    fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|order| order.slug() == slug)
    }
}

/// The asset library with filtering, sorting, paging, import and preview.
#[component]
pub fn AssetsView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let busy = store.asset_busy();

    let mut category = use_signal(|| None::<AssetCategory>);
    let mut search = use_signal(String::new);
    let mut sort = use_signal(|| AssetSort::NameAsc);
    let mut page = use_signal(|| 0usize);
    let mut preview = use_signal(|| None::<CoreAsset>);
    let mut rename_target = use_signal(|| None::<CoreAsset>);
    let mut rename_text = use_signal(String::new);
    let mut delete_target = use_signal(|| None::<CoreAsset>);
    let mut replace_target = use_signal(|| None::<CoreAsset>);
    let mut property_target = use_signal(|| None::<AssetId>);
    let mut import_category = use_signal(|| AssetCategory::Logo);
    let mut show_url_dialog = use_signal(|| false);
    let mut url_text = use_signal(String::new);
    let mut url_error = use_signal(|| None::<String>);
    let mut show_pack_export = use_signal(|| false);
    let mut show_pack_url = use_signal(|| false);
    let mut pack_name = use_signal(String::new);
    let mut pack_description = use_signal(String::new);
    let mut pack_author = use_signal(String::new);
    let mut pack_license = use_signal(String::new);
    let mut pack_homepage = use_signal(String::new);
    let mut pack_url_text = use_signal(String::new);
    let mut pack_url_error = use_signal(|| None::<String>);

    // A different filter, search or order starts the library at the first page.
    use_effect(move || {
        let _ = category();
        let _ = search();
        let _ = sort();
        page.set(0);
    });

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
    let mut filtered: Vec<CoreAsset> = all_assets
        .iter()
        .filter(|asset| {
            selected.is_none_or(|category| asset.category == category)
                && (search_text.is_empty() || asset.name.to_lowercase().contains(&search_text))
        })
        .cloned()
        .collect();
    match sort() {
        AssetSort::NameAsc => {
            filtered.sort_by_key(|asset| asset.name.to_lowercase());
        }
        AssetSort::NameDesc => {
            filtered.sort_by_key(|asset| Reverse(asset.name.to_lowercase()));
        }
        AssetSort::Category => {
            filtered.sort_by_key(|asset| (asset.category.label(), asset.name.to_lowercase()));
        }
        AssetSort::Newest => {
            filtered.sort_by_key(|asset| Reverse(asset.added_at.clone()));
        }
        AssetSort::Oldest => {
            filtered.sort_by_key(|asset| asset.added_at.clone());
        }
    }

    let page_count = filtered.len().div_ceil(ASSETS_PER_PAGE).max(1);
    let current_page = page().min(page_count - 1);
    let first = current_page * ASSETS_PER_PAGE;
    let last = (first + ASSETS_PER_PAGE).min(filtered.len());
    let visible: Vec<CoreAsset> = filtered[first..last].to_vec();

    let mode_options: Vec<CoreAsset> = all_assets
        .iter()
        .filter(|asset| asset.category == AssetCategory::GameMode)
        .cloned()
        .collect();

    let (property_asset, property_icon, property_transparent, property_badge) = {
        let assets = &state.read().assets;
        let asset = property_target()
            .and_then(|id| assets.iter().find(|candidate| candidate.id == id).cloned());
        let icon = asset
            .as_ref()
            .and_then(|asset| asset.icon.as_ref())
            .and_then(|id| assets.iter().find(|candidate| &candidate.id == id).cloned());
        let transparent = asset
            .as_ref()
            .and_then(|asset| asset.transparent.as_ref())
            .and_then(|id| assets.iter().find(|candidate| &candidate.id == id).cloned());
        let badge = asset
            .as_ref()
            .and_then(|asset| asset.badge.as_ref())
            .and_then(|id| assets.iter().find(|candidate| &candidate.id == id).cloned());
        (asset, icon, transparent, badge)
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
                    select {
                        class: "fgc-native-select",
                        value: "{sort().slug()}",
                        aria_label: "Sort assets",
                        onchange: move |event| {
                            if let Some(order) = AssetSort::from_slug(&event.value()) {
                                sort.set(order);
                            }
                        },
                        for order in AssetSort::ALL {
                            option {
                                value: "{order.slug()}",
                                selected: order == sort(),
                                "{order.label()}"
                            }
                        }
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
                div { class: "fgc-asset-import",
                    select {
                        class: "fgc-native-select",
                        value: "{import_category().slug()}",
                        aria_label: "Import category",
                        onchange: move |event| {
                            if let Some(category) = AssetCategory::from_slug(&event.value()) {
                                import_category.set(category);
                            }
                        },
                        for option in AssetCategory::ALL {
                            option {
                                value: "{option.slug()}",
                                selected: option == import_category(),
                                "{option.label()}"
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: busy,
                        onclick: move |_| store.import_assets_from_files(import_category()),
                        if busy { "Working…" } else { "Import files…" }
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: busy,
                        onclick: move |_| {
                            url_error.set(None);
                            show_url_dialog.set(true);
                        },
                        "Import from URL…"
                    }
                    span { class: "fgc-asset-import__note",
                        "PNG, JPEG, WebP, GIF and SVG up to 20 MB. Identical bytes are stored once."
                    }
                }
                if filtered.is_empty() {
                    div { class: "fgc-asset-empty",
                        p { class: "fgc-asset-empty__title", "No assets match this filter." }
                        p { class: "fgc-note", "Try another category, clear the search, or import an image above." }
                    }
                } else {
                    div { class: "fgc-asset-grid",
                        for asset in visible {
                            {
                                rsx! {
                                    AssetCard {
                                        key: "{asset.id}",
                                        asset: asset.clone(),
                                        mode_options: mode_options.clone(),
                                        busy,
                                        on_preview: move |asset: CoreAsset| preview.set(Some(asset)),
                                        on_replace: move |asset: CoreAsset| replace_target.set(Some(asset)),
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
                    div { class: "fgc-asset-pager",
                        span { class: "fgc-asset-pager__count",
                            "Showing {first + 1}-{last} of {filtered.len()}"
                        }
                        div { class: "fgc-asset-pager__controls",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: current_page == 0,
                                onclick: move |_| {
                                    page.set(current_page.saturating_sub(1));
                                },
                                "Previous"
                            }
                            span { class: "fgc-asset-pager__page",
                                "Page {current_page + 1} of {page_count}"
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: current_page + 1 >= page_count,
                                onclick: move |_| {
                                    page.set((current_page + 1).min(page_count - 1));
                                },
                                "Next"
                            }
                        }
                    }
                }
            }
            div { class: "fgc-grid",
                CollapsiblePanel {
                    title: "Content packs".to_owned(),
                    description: "Share artwork without touching anybody's teams or settings.".to_owned(),
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| {
                                pack_name.set(String::new());
                                pack_description.set(String::new());
                                pack_author.set(String::new());
                                pack_license.set(String::new());
                                pack_homepage.set(String::new());
                                show_pack_export.set(true);
                            },
                            "Export pack…"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| store.import_content_pack(),
                            "Import pack…"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy,
                            onclick: move |_| {
                                pack_url_error.set(None);
                                show_pack_url.set(true);
                            },
                            "Import pack from URL…"
                        }
                    }
                    p { class: "fgc-note",
                        "A pack holds images only. Installing one adds its artwork to the library and leaves teams, players, tournaments and the running match exactly as they are. To move a whole project - teams, matches, settings and images - use Settings -> Project backup instead."
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
                    if asset.origin != AssetOrigin::Bundled {
                        Button {
                            variant: ButtonVariant::Primary,
                            disabled: busy,
                            onclick: move |_| {
                                replace_target.set(Some(asset.clone()));
                                preview.set(None);
                            },
                            "Replace…"
                        }
                    }
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
            open: replace_target().is_some(),
            on_open_change: move |open: bool| if !open { replace_target.set(None) },
            if let Some(asset) = replace_target() {
                DialogTitle { "Replace {asset.name}?" }
                DialogDescription {
                    "Choose a new file for this asset. Its name and identity stay the same, so every team, player and overlay using it shows the new image. The old image is cleaned up after the next save."
                }
                div { class: "fgc-dialog-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| replace_target.set(None),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: busy,
                        onclick: move |_| {
                            store.replace_asset_image(asset.id.clone(), move |_| {
                                replace_target.set(None);
                            });
                        },
                        if busy { "Working…" } else { "Choose image…" }
                    }
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
                    div { class: "fgc-asset-properties__section",
                        h3 { "Transparent art" }
                        if let Some(transparent) = property_transparent.clone() {
                            div { class: "fgc-asset-properties__icon",
                                AssetThumb {
                                    asset: transparent,
                                    class: "fgc-asset-properties__icon-image".to_owned(),
                                }
                                p { class: "fgc-note", "Uploaded for this hero. The hero bans overlay uses it instead of the regular art." }
                            }
                        } else {
                            p { class: "fgc-note", "No transparent art yet. The regular hero art is used." }
                        }
                        div { class: "fgc-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: busy,
                                onclick: {
                                    let hero_id = asset.id.clone();
                                    move |_| store.upload_hero_transparent(hero_id.clone())
                                },
                                if property_transparent.is_some() { "Replace transparent art…" } else { "Upload transparent art…" }
                            }
                            if property_transparent.is_some() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    onclick: {
                                        let hero_id = asset.id.clone();
                                        move |_| store.clear_hero_transparent(hero_id.clone())
                                    },
                                    "Remove transparent art"
                                }
                            }
                        }
                    }
                    div { class: "fgc-asset-properties__section",
                        h3 { "Badge" }
                        if let Some(badge) = property_badge.clone() {
                            div { class: "fgc-asset-properties__icon",
                                AssetThumb {
                                    asset: badge,
                                    class: "fgc-asset-properties__icon-image".to_owned(),
                                }
                                p { class: "fgc-note", "The hexagonal badge for this hero. The hero bans overlay shows it beside the role hexagon." }
                            }
                        } else {
                            p { class: "fgc-note", "No badge yet. The hero bans overlay falls back to the regular art." }
                        }
                        div { class: "fgc-actions",
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: busy,
                                onclick: {
                                    let hero_id = asset.id.clone();
                                    move |_| store.upload_hero_badge(hero_id.clone())
                                },
                                if property_badge.is_some() { "Replace badge…" } else { "Upload badge…" }
                            }
                            if property_badge.is_some() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    onclick: {
                                        let hero_id = asset.id.clone();
                                        move |_| store.clear_hero_badge(hero_id.clone())
                                    },
                                    "Remove badge"
                                }
                            }
                        }
                    }
                    div { class: "fgc-asset-properties__section",
                        h3 { "Role" }
                        p { class: "fgc-note", "The hero bans overlay shows the role hexagon above the hero name." }
                        select {
                            class: "fgc-native-select",
                            aria_label: "Hero role",
                            value: asset.role.map(|role| role.slug().to_owned()).unwrap_or_default(),
                            onchange: move |event| {
                                let hero_id = asset.id.clone();
                                let slug = event.value();
                                let role = Role::ALL
                                    .into_iter()
                                    .find(|role| role.slug() == slug);
                                store.set_hero_role(hero_id, role);
                            },
                            option { value: "", selected: asset.role.is_none(), "No role" }
                            for role in Role::ALL {
                                option {
                                    value: role.slug(),
                                    selected: asset.role == Some(role),
                                    "{role.label()}"
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
            open: show_pack_export(),
            on_open_change: move |open: bool| if !open { show_pack_export.set(false) },
            DialogTitle { "Export a content pack" }
            DialogDescription {
                "The pack holds every image in the library. Teams, players, tournaments and settings are left out, so anybody can install it without changing their project."
            }
            Field { label: "Name".to_owned(),
                Input {
                    r#type: "text",
                    placeholder: "Fruiterra Overwatch",
                    value: "{pack_name}",
                    aria_label: "Pack name",
                    oninput: move |event: FormEvent| pack_name.set(event.value()),
                }
            }
            Field { label: "Description".to_owned(),
                Input {
                    r#type: "text",
                    placeholder: "Hero art, map art and game-mode icons",
                    value: "{pack_description}",
                    aria_label: "Pack description",
                    oninput: move |event: FormEvent| pack_description.set(event.value()),
                }
            }
            Field { label: "Author".to_owned(),
                Input {
                    r#type: "text",
                    placeholder: "Your name or studio",
                    value: "{pack_author}",
                    aria_label: "Pack author",
                    oninput: move |event: FormEvent| pack_author.set(event.value()),
                }
            }
            Field { label: "Licence".to_owned(),
                Input {
                    r#type: "text",
                    placeholder: "CC BY 4.0",
                    value: "{pack_license}",
                    aria_label: "Pack licence",
                    oninput: move |event: FormEvent| pack_license.set(event.value()),
                }
            }
            Field { label: "Homepage".to_owned(),
                Input {
                    r#type: "url",
                    placeholder: "https://github.com/your-name/pack",
                    value: "{pack_homepage}",
                    aria_label: "Pack homepage",
                    oninput: move |event: FormEvent| pack_homepage.set(event.value()),
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| show_pack_export.set(false),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    disabled: busy,
                    onclick: move |_| {
                        let info = fgc_core::PackInfo {
                            name: pack_name.read().clone(),
                            description: pack_description.read().clone(),
                            author: pack_author.read().clone(),
                            license: pack_license.read().clone(),
                            homepage: pack_homepage.read().clone(),
                        };
                        store.export_content_pack(info);
                        show_pack_export.set(false);
                    },
                    "Export"
                }
            }
        }

        Dialog {
            open: show_pack_url(),
            on_open_change: move |open: bool| if !open { show_pack_url.set(false) },
            DialogTitle { "Install a content pack from a URL" }
            DialogDescription {
                "The archive is downloaded once, unpacked and validated. Only its images are added; nothing in the project is replaced."
            }
            Field { label: "Pack URL".to_owned(), error: pack_url_error(),
                Input {
                    r#type: "url",
                    placeholder: "https://github.com/owner/repo/releases/download/v1/pack.zip",
                    value: "{pack_url_text}",
                    aria_label: "Pack URL",
                    oninput: move |event: FormEvent| {
                        pack_url_text.set(event.value());
                        pack_url_error.set(None);
                    },
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| show_pack_url.set(false),
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    disabled: busy,
                    onclick: move |_| {
                        let raw = pack_url_text.read().clone();
                        match crate::assets_io::validate_import_url(&raw) {
                            Ok(_) => {
                                store.import_content_pack_from_url(&raw);
                                pack_url_error.set(None);
                                show_pack_url.set(false);
                            }
                            Err(error) => pack_url_error.set(Some(error)),
                        }
                    },
                    "Install"
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
    busy: bool,
    on_preview: EventHandler<CoreAsset>,
    on_replace: EventHandler<CoreAsset>,
    on_rename: EventHandler<CoreAsset>,
    on_delete: EventHandler<CoreAsset>,
    on_game_mode: EventHandler<(CoreAsset, Option<AssetId>)>,
    on_properties: EventHandler<CoreAsset>,
) -> Element {
    let preview_asset = asset.clone();
    let replace_asset = asset.clone();
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
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: busy,
                        onclick: move |_| on_replace.call(replace_asset.clone()),
                        "Replace…"
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
