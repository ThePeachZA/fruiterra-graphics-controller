//! Application shell: providers, navigation, page chrome and status bar.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};

use crate::components::toast::ToastProvider;
use crate::state::AppStore;
use crate::status::{status_text, status_tone};
use crate::theme::Destination;
use crate::views::DestinationView;

const DX_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");
const THEME_CSS: Asset = asset!("/assets/theme.css");
const APP_CSS: Asset = asset!("/assets/app.css");
const COMPONENTS_CSS: Asset = asset!("/assets/components.css");
const OVERLAY_CSS: Asset = asset!("/assets/overlay.css");
const SETTINGS_CSS: Asset = asset!("/assets/settings.css");
const ASSETS_CSS: Asset = asset!("/assets/assets.css");
const TEAMS_CSS: Asset = asset!("/assets/teams.css");
const MATCH_CONTROLLER_CSS: Asset = asset!("/assets/match-controller.css");
const PLACEHOLDER_CSS: Asset = asset!("/assets/placeholder.css");

const INTER_LATIN: Asset = asset!("/assets/fonts/Inter-latin.woff2");
const INTER_LATIN_EXT: Asset = asset!("/assets/fonts/Inter-latin-ext.woff2");

/// `@font-face` rules for the locally bundled Inter variable font.
///
/// Manganis does not rewrite `url()` references inside stylesheets, so the
/// rules are generated here from [`Asset`] constants and injected as a
/// `<style>` element.
const FONT_FACES: &str = r#"
@font-face {
    font-family: "Inter";
    font-style: normal;
    font-weight: 100 900;
    font-display: swap;
    src: url("__LATIN__") format("woff2");
    unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA,
        U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193,
        U+2212, U+2215, U+FEFF, U+FFFD;
}
@font-face {
    font-family: "Inter";
    font-style: normal;
    font-weight: 100 900;
    font-display: swap;
    src: url("__LATIN_EXT__") format("woff2");
    unicode-range: U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF,
        U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020,
        U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF;
}
"#;

/// Root component.
#[component]
pub fn App() -> Element {
    let store = AppStore::use_store();
    use_context_provider(|| store);
    let mut destination = use_signal(|| Destination::Overlay);

    let font_faces = FONT_FACES
        .replace("__LATIN__", &INTER_LATIN.to_string())
        .replace("__LATIN_EXT__", &INTER_LATIN_EXT.to_string());

    rsx! {
        document::Link { rel: "stylesheet", href: DX_THEME_CSS }
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Link { rel: "stylesheet", href: APP_CSS }
        document::Link { rel: "stylesheet", href: COMPONENTS_CSS }
        document::Link { rel: "stylesheet", href: OVERLAY_CSS }
        document::Link { rel: "stylesheet", href: SETTINGS_CSS }
        document::Link { rel: "stylesheet", href: ASSETS_CSS }
        document::Link { rel: "stylesheet", href: TEAMS_CSS }
        document::Link { rel: "stylesheet", href: MATCH_CONTROLLER_CSS }
        document::Link { rel: "stylesheet", href: PLACEHOLDER_CSS }
        style { dangerous_inner_html: font_faces }
        ToastProvider {
            ErrorToasts {}
            NoticeToasts {}
            div { class: "fgc-app",
                Sidebar {
                    destination: destination(),
                    on_navigate: move |next| destination.set(next),
                }
                main { class: "fgc-main",
                    PageHeader { destination: destination() }
                    ErrorBanner {}
                    div { class: "fgc-content",
                        DestinationView { destination: destination() }
                    }
                    StatusBar {}
                }
            }
        }
    }
}

/// Sidebar with the seven original destinations.
#[component]
fn Sidebar(destination: Destination, on_navigate: EventHandler<Destination>) -> Element {
    rsx! {
        aside { class: "fgc-sidebar",
            div { class: "fgc-brand",
                div { class: "fgc-brand__mark", "FG" }
                div {
                    p { class: "fgc-brand__name", "Fruiterra" }
                    p { class: "fgc-brand__sub", "Graphics Controller" }
                }
            }
            nav { class: "fgc-nav", aria_label: "Destinations",
                for item in Destination::ALL {
                    button {
                        key: "{item.label()}",
                        class: if item == destination {
                            "fgc-nav__item fgc-nav__item--active"
                        } else {
                            "fgc-nav__item"
                        },
                        r#type: "button",
                        aria_current: if item == destination { "page" } else { "false" },
                        onclick: move |_| on_navigate.call(item),
                        span { class: "fgc-nav__monogram", "{item.monogram()}" }
                        span { "{item.label()}" }
                        if !item.is_implemented() {
                            span { class: "fgc-nav__soon", "{item.milestone()}" }
                        }
                    }
                }
            }
            div { class: "fgc-sidebar__footer",
                span { "Version {fgc_core::VERSION}" }
                span { "Overlays · 1920×1080" }
            }
        }
    }
}

/// Page title, blurb and milestone pill for the current destination.
#[component]
fn PageHeader(destination: Destination) -> Element {
    let pill = if destination.is_implemented() {
        "Live"
    } else {
        destination.milestone()
    };

    rsx! {
        header { class: "fgc-page-header",
            div {
                p { class: "fgc-eyebrow", "Fruiterra Graphics Controller" }
                h1 { class: "fgc-page-header__title", "{destination.label()}" }
                p { class: "fgc-page-header__blurb", "{destination.blurb()}" }
            }
            div { class: "fgc-page-header__actions",
                span { class: "fgc-pill", "{pill}" }
            }
        }
    }
}

/// Dismissible banner for the most recent user-facing error.
#[component]
fn ErrorBanner() -> Element {
    let store = use_context::<AppStore>();
    let Some(message) = store.last_error() else {
        return rsx! {};
    };

    rsx! {
        div { class: "fgc-banner", role: "alert",
            div { class: "fgc-banner__body",
                p { class: "fgc-banner__title", "Something needs attention" }
                p { class: "fgc-banner__message", "{message}" }
            }
            button {
                class: "fgc-banner__close",
                r#type: "button",
                aria_label: "Dismiss error",
                onclick: move |_| store.dismiss_error(),
                "×"
            }
        }
    }
}

/// Surfaces new user-facing errors as toasts.
#[component]
fn ErrorToasts() -> Element {
    let store = use_context::<AppStore>();
    let toast = use_toast();
    let mut seen = use_signal(|| None::<String>);

    use_effect(move || {
        let current = store.last_error();
        if current != *seen.peek() {
            if let Some(message) = &current {
                toast.error(
                    "Something went wrong".to_owned(),
                    ToastOptions::new().description(message.clone()),
                );
            }
            seen.set(current);
        }
    });

    rsx! {}
}

/// Surfaces new success messages as toasts.
#[component]
fn NoticeToasts() -> Element {
    let store = use_context::<AppStore>();
    let toast = use_toast();
    let mut seen = use_signal(|| None::<String>);

    use_effect(move || {
        let current = store.notice();
        if current != *seen.peek() {
            if let Some(message) = &current {
                toast.success(message.clone(), ToastOptions::new());
                store.dismiss_notice();
            }
            seen.set(current);
        }
    });

    rsx! {}
}

/// Bottom status bar with server, asset and save state.
#[component]
fn StatusBar() -> Element {
    let store = use_context::<AppStore>();
    let server = store.server();
    let status = server.status();
    let tone = status_tone(&status);
    let label = status_text(&status);
    let base_url = server.base_url();
    let assets = store.asset_count();
    let dirty = store.is_dirty();
    let saving = store.is_saving();

    rsx! {
        footer { class: "fgc-statusbar",
            span { class: "fgc-statusbar__group",
                span { class: "fgc-dot fgc-dot--{tone}" }
                span { "{label}" }
            }
            span { class: "fgc-statusbar__group fgc-mono", "{base_url}" }
            span { class: "fgc-statusbar__spacer" }
            span { class: "fgc-statusbar__group", "{assets} assets" }
            span { class: "fgc-statusbar__group",
                if saving {
                    "Saving…"
                } else if dirty {
                    "Unsaved changes"
                } else {
                    "Saved"
                }
            }
        }
    }
}
