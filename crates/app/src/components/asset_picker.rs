//! Asset picker backed by the merged bundled and user image library.

use dioxus::prelude::*;
use fgc_core::{Asset as CoreAsset, AssetCategory, AssetId};

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use crate::components::input::Input;
use crate::state::AppStore;

/// Returns the bundled catalogue followed by the user's imported assets.
#[must_use]
pub fn library_assets(user_assets: &[CoreAsset]) -> Vec<CoreAsset> {
    let mut assets = fgc_server::bundled_catalog();
    assets.extend(
        user_assets
            .iter()
            .filter(|asset| !asset.is_attachment())
            .cloned(),
    );
    assets
}

/// Finds one asset by identifier.
#[must_use]
pub fn find_asset<'a>(assets: &'a [CoreAsset], id: Option<&AssetId>) -> Option<&'a CoreAsset> {
    id.and_then(|id| assets.iter().find(|asset| &asset.id == id))
}

/// A labelled picker for one category of the asset library.
#[component]
pub fn AssetPicker(
    label: String,
    category: AssetCategory,
    value: Option<AssetId>,
    on_change: EventHandler<Option<AssetId>>,
) -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let catalog = store.catalog();
    let mut open = use_signal(|| false);
    let mut search = use_signal(String::new);

    let assets = library_assets(&state.read().assets);
    let selected = value
        .as_ref()
        .and_then(|id| assets.iter().find(|asset| &asset.id == id))
        .cloned();
    let search_text = search.read().trim().to_lowercase();
    let filtered: Vec<(CoreAsset, bool)> = assets
        .iter()
        .filter(|asset| asset.category == category)
        .filter(|asset| category != AssetCategory::Hero || catalog.is_selectable_hero(asset))
        .filter(|asset| search_text.is_empty() || asset.name.to_lowercase().contains(&search_text))
        .map(|asset| {
            let is_selected = selected
                .as_ref()
                .is_some_and(|selected| selected.id == asset.id);
            (asset.clone(), is_selected)
        })
        .collect();

    let category_label = category.label();
    let placeholder = format!("No {} selected", category_label.to_lowercase());
    let quick_upload = move || {
        store.import_assets_from_files_then(category, move |ids| {
            if let Some(id) = ids.first() {
                on_change.call(Some(id.clone()));
                open.set(false);
            }
        });
    };

    rsx! {
        div { class: "fgc-asset-picker",
            p { class: "fgc-field__label", span { "{label}" } }
            div { class: "fgc-asset-picker__preview",
                if let Some(asset) = selected.clone() {
                    AssetThumb {
                        key: "{asset.id}",
                        asset: asset.clone(),
                        class: "fgc-asset-picker__image".to_owned(),
                    }
                    span { class: "fgc-asset-picker__selected", "{asset.name}" }
                } else {
                    span { class: "fgc-asset-picker__placeholder", "{placeholder}" }
                }
            }
            div { class: "fgc-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| open.set(true),
                    "Choose {category_label.to_lowercase()}…"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    disabled: store.asset_busy(),
                    onclick: move |_| quick_upload(),
                    "Quick upload…"
                }
                if selected.is_some() {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |_| on_change.call(None),
                        "Clear"
                    }
                }
            }
        }

        Dialog {
            open: open(),
            on_open_change: move |is_open| open.set(is_open),
            DialogTitle { "Choose {label}" }
            DialogDescription {
                "Images from the {category_label} category. Quick upload adds a new image here without leaving this picker."
            }
            Input {
                r#type: "search",
                placeholder: "Search {category_label.to_lowercase()} assets",
                value: "{search}",
                aria_label: "Search assets",
                oninput: move |event: FormEvent| search.set(event.value()),
            }
            if filtered.is_empty() {
                div { class: "fgc-asset-picker__empty",
                    p { "No assets match." }
                    p { class: "fgc-note", "Quick upload adds one here, or import a batch in Assets." }
                }
            } else {
                div { class: "fgc-asset-picker__grid",
                    for (asset, is_selected) in filtered {
                        button {
                            key: "{asset.id}",
                            r#type: "button",
                            aria_label: "{asset.name}",
                            class: if is_selected {
                                "fgc-asset-picker__item fgc-asset-picker__item--selected"
                            } else {
                                "fgc-asset-picker__item"
                            },
                            onclick: {
                                let asset_id = asset.id.clone();
                                move |_| {
                                    on_change.call(Some(asset_id.clone()));
                                    open.set(false);
                                }
                            },
                            AssetThumb {
                                asset: asset.clone(),
                                class: "fgc-asset-picker__thumb".to_owned(),
                            }
                            span { class: "fgc-asset-picker__name", title: "{asset.name}", "{asset.name}" }
                        }
                    }
                }
            }
            div { class: "fgc-dialog-actions",
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: store.asset_busy(),
                    onclick: move |_| quick_upload(),
                    "Quick upload…"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| open.set(false),
                    "Close"
                }
            }
        }
    }
}

/// Where a thumbnail's bytes come from.
#[derive(Clone)]
enum ThumbSource {
    /// The overlay server is serving the bytes, so the WebView fetches them.
    Served(String),
    /// The bytes had to be inlined because the server is not running.
    Inline(String),
}

/// An image thumbnail resolved from the asset library.
#[component]
pub fn AssetThumb(
    asset: CoreAsset,
    #[props(default)] class: String,
    #[props(default)] placeholder_class: String,
) -> Element {
    let store = use_context::<AppStore>();
    let mut current = use_signal(|| asset.clone());
    let changed = {
        let current = current.peek();
        current.clone() != asset
    };
    if changed {
        current.set(asset.clone());
    }
    let url = use_resource(move || {
        let asset = current.read().clone();
        // The overlay server already serves the bytes and the WebView caches
        // them by content hash, so a thumbnail costs one request instead of a
        // multi-megabyte base64 string kept in the app process for ever.
        let served = store.asset_served_url(&asset);
        async move {
            match served {
                Some(url) => Some(ThumbSource::Served(url)),
                None => store.asset_data_url(asset).await.map(ThumbSource::Inline),
            }
        }
    });
    let label = asset.category.label();
    let placeholder = if placeholder_class.is_empty() {
        format!("fgc-thumb-placeholder {}", class).trim().to_owned()
    } else {
        placeholder_class
    };
    let resolved = url();
    let src = match &resolved {
        Some(Some(ThumbSource::Served(src))) => Some(src.clone()),
        Some(Some(ThumbSource::Inline(src))) => Some(src.clone()),
        _ => None,
    };
    let loading = resolved.is_none();

    rsx! {
        if let Some(src) = src {
            img {
                class: "{class}",
                src: "{src}",
                alt: "{asset.name}",
                loading: "lazy",
                decoding: "async",
            }
        } else if loading {
            div { class: "{placeholder} fgc-skeleton", "{label}" }
        } else {
            div { class: "{placeholder}", "{label}" }
        }
    }
}
