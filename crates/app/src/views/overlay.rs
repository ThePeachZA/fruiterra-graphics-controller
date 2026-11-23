//! Overlay destination: server controls and OBS Browser Source URLs.

use std::time::{Duration, Instant};

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use fgc_server::ServerStatus;

use crate::assets_io::open_browser;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::{CollapsiblePanel, Disclosure, KeyValue, KeyValues, Panel};
use crate::remote_access::{self, OverlayUrlGroup};
use crate::server::{
    OVERLAY_FILES, SCENE_OVERLAY_FILES, overlay_source_files, overlay_url_with_background,
};
use crate::state::AppStore;
use crate::status::{broadcast_age_text, client_count_text, status_text, status_tone};

/// Server lifecycle controls and copyable Browser Source URLs.
#[component]
pub fn OverlayView() -> Element {
    let store = use_context::<AppStore>();
    let server = store.server();
    let status = server.status();
    let busy = server.busy();
    let base_url = server.base_url();

    let running = matches!(&status, ServerStatus::Running { .. });
    let tone = status_tone(&status);
    let label = status_text(&status);
    let urls_visible = store.urls_visible();
    let overlays_visible = store.overlays_visible();
    let background_opaque = store.background_opaque();
    let mut clock = use_signal(Instant::now);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            clock.set(Instant::now());
        }
    });

    let clients = match &status {
        ServerStatus::Running { clients, .. } => *clients,
        _ => 0,
    };
    let last_broadcast_age = server
        .last_broadcast()
        .map(|last| clock.read().saturating_duration_since(last));
    let needs_reconnect = running && clients == 0;

    rsx! {
        div { class: "fgc-view",
            div { class: "fgc-grid fgc-grid--two",
                Panel {
                    title: "Overlay server".to_owned(),
                    description: "Serve the OBS-compatible overlays on the configured port.".to_owned(),
                    div { class: "fgc-status-row",
                        span { class: "fgc-status fgc-status--{tone}",
                            span { class: "fgc-dot fgc-dot--{tone}" }
                            "{label}"
                        }
                    }
                    KeyValues {
                        KeyValue {
                            label: "Base URL".to_owned(),
                            value: if urls_visible {
                                base_url.clone()
                            } else {
                                "Hidden".to_owned()
                            },
                        }
                        KeyValue {
                            label: "Overlay files".to_owned(),
                            value: OVERLAY_FILES.len().to_string(),
                        }
                        KeyValue {
                            label: "Live overlays".to_owned(),
                            value: if overlays_visible { "Visible".to_owned() } else { "Hidden".to_owned() },
                        }
                        KeyValue {
                            label: "Background".to_owned(),
                            value: if background_opaque { "Opaque".to_owned() } else { "Transparent".to_owned() },
                        }
                        KeyValue {
                            label: "Connected clients".to_owned(),
                            value: client_count_text(clients),
                        }
                        KeyValue {
                            label: "Last broadcast".to_owned(),
                            value: broadcast_age_text(last_broadcast_age),
                        }
                        if let ServerStatus::Running { port, .. } = &status {
                            KeyValue {
                                label: "Listening on".to_owned(),
                                value: format!("port {port}"),
                            }
                        }
                    }
                    div { class: "fgc-actions",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: !running || busy,
                            aria_pressed: overlays_visible,
                            onclick: move |_| store.set_overlays_visible(!overlays_visible),
                            if overlays_visible { "Hide overlays" } else { "Show overlays" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: !running || busy,
                            aria_pressed: background_opaque,
                            onclick: move |_| store.set_background_opaque(!background_opaque),
                            if background_opaque { "Use transparent background" } else { "Use opaque background" }
                        }
                        if running {
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: busy,
                                onclick: move |_| store.stop_server(),
                                "Stop server"
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                disabled: busy,
                                onclick: move |_| store.restart_server(),
                                "Restart"
                            }
                        } else {
                            Button {
                                variant: ButtonVariant::Primary,
                                disabled: busy,
                                onclick: move |_| store.start_server(),
                                "Start server"
                            }
                        }
                    }
                    p { class: "fgc-note",
                        "The base URL and Browser Source URLs stay hidden until you reveal them below. Port, remote access, asset URLs and auto-start are configured in Settings."
                    }
                    p { class: "fgc-note",
                        "The background toggle changes the generated Browser Source URLs. Refresh each OBS source after changing it; connected sources keep their current mode until then."
                    }
                    if needs_reconnect {
                        p { class: "fgc-note", role: "status",
                            "No Browser Sources are connected. Open or refresh an OBS Browser Source to reconnect."
                        }
                    }
                }
                CollapsiblePanel {
                    title: "OBS setup".to_owned(),
                    description: "Every overlay is a plain HTML Browser Source.".to_owned(),
                    ol { class: "fgc-steps",
                        li { "Add a Browser Source in OBS." }
                        li { "Set the width to 1920 and the height to 1080." }
                        li { "Paste one of the URLs below into the URL field." }
                        li {
                            "Turn off \"Shutdown source when not visible\" so the overlay keeps its state."
                        }
                    }
                    p { class: "fgc-note",
                        "Overlays connect over WebSocket and repaint whenever the control UI changes."
                    }
                }
            }
            AutoOverlayController {}
            OverlayUrls {}
        }
    }
}

/// Session-only scene switcher for the single auto Browser Source.
#[component]
fn AutoOverlayController() -> Element {
    let store = use_context::<AppStore>();
    let included = store.auto_overlay_included();
    let active = store.auto_overlay_active();
    let hotkey_included = included.clone();
    let active_text = active.clone().unwrap_or_else(|| "None".to_owned());
    let status_text = if active.is_some() {
        format!("Active scene: {active_text}")
    } else {
        "Active scene: none — include at least one overlay".to_owned()
    };

    rsx! {
        Panel {
            title: "Auto-overlay scenes".to_owned(),
            description: "Use one Browser Source and switch between a chosen subset of overlays.".to_owned(),
            div {
                class: "fgc-auto-scenes",
                tabindex: 0,
                aria_label: "Auto-overlay scene controller",
                onkeydown: move |event: KeyboardEvent| {
                    match event.key() {
                        Key::Character(key) if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() => {
                            let Some(number) = key.parse::<u8>().ok() else {
                                return;
                            };
                            let Some(file) = SCENE_OVERLAY_FILES.get(usize::from(number.saturating_sub(1))) else {
                                return;
                            };
                            if number == 0 || !hotkey_included.iter().any(|item| item == file) {
                                return;
                            }
                            event.prevent_default();
                            event.stop_propagation();
                            store.activate_auto_overlay_number(number);
                        }
                        Key::ArrowLeft | Key::ArrowRight if !hotkey_included.is_empty() => {
                            event.prevent_default();
                            event.stop_propagation();
                            let direction = if event.key() == Key::ArrowLeft { -1 } else { 1 };
                            store.cycle_auto_overlay(direction);
                        }
                        _ => {}
                    }
                },
                div { class: "fgc-auto-scenes__list", role: "group", aria_label: "Overlay scenes",
                    for (index, file) in SCENE_OVERLAY_FILES.iter().enumerate() {
                        {
                            let file = (*file).to_owned();
                            let is_included = included.contains(&file);
                            let is_active = active.as_deref() == Some(file.as_str());
                            let include_label = format!("Include {file} in the auto scene switcher");
                            let active_label = format!("Make {file} the active scene");
                            let active_file = file.clone();
                            rsx! {
                                div {
                                    key: "{file}",
                                    class: if is_included { "fgc-auto-scene" } else { "fgc-auto-scene fgc-auto-scene--excluded" },
                                    div { class: "fgc-auto-scene__head",
                                        label { class: "fgc-auto-scene__include",
                                            input {
                                                r#type: "checkbox",
                                                checked: is_included,
                                                aria_label: include_label,
                                                onkeydown: |event: KeyboardEvent| event.stop_propagation(),
                                                onchange: {
                                                    let file = file.clone();
                                                    move |event: FormEvent| {
                                                        store.set_auto_overlay_included(&file, event.checked());
                                                    }
                                                },
                                            }
                                            span { "Include" }
                                        }
                                        span { class: "fgc-auto-scene__key", aria_hidden: "true", "{index + 1}" }
                                    }
                                    p { class: "fgc-auto-scene__file", "{file}" }
                                    label { class: "fgc-auto-scene__active",
                                        input {
                                            r#type: "radio",
                                            name: "auto-overlay-active",
                                            checked: is_active,
                                            disabled: !is_included,
                                            aria_label: active_label,
                                            onkeydown: |event: KeyboardEvent| event.stop_propagation(),
                                            onchange: move |_| store.set_auto_overlay_active(&active_file),
                                        }
                                        span { "Active" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "fgc-auto-scenes__side",
                    p { class: "fgc-auto-scenes__status", role: "status", "{status_text}" }
                    Disclosure {
                        title: "Keyboard shortcuts".to_owned(),
                        hint: Some("while this controller has focus".to_owned()),
                        p { class: "fgc-note", "Press 1–9 to activate that numbered overlay when it is included." }
                        p { class: "fgc-note", "Use Left and Right arrow keys to cycle through the included overlays." }
                    }
                    p { class: "fgc-note",
                        "Selection and active scene last for this app session only. The auto Browser Source is listed under Browser sources below."
                    }
                }
            }
        }
    }
}

/// Every Browser Source URL set currently available, hidden until revealed.
///
/// The same component is used on Overlay, in Settings and in Match Controller →
/// Config so the visibility choice stays consistent and no URL leaks anywhere.
#[component]
pub(crate) fn OverlayUrls() -> Element {
    let store = use_context::<AppStore>();
    let visible = store.urls_visible();
    let dns_name = store.tailscale_dns_name();
    let groups = {
        let state = store.state();
        let snapshot = state.read();
        remote_access::browser_source_groups(&snapshot, dns_name.as_deref())
    };
    let actions = rsx! {
        Button {
            variant: ButtonVariant::Outline,
            size: ButtonSize::Sm,
            aria_pressed: visible,
            onclick: move |_| store.set_urls_visible(!visible),
            if visible { "Hide URLs" } else { "Show URLs" }
        }
    };

    rsx! {
        Panel {
            title: "Browser sources".to_owned(),
            description: "One set for each enabled access path. Paste a URL into OBS at 1920×1080.".to_owned(),
            actions: Some(actions),
            div { class: "fgc-url-groups",
                for group in groups {
                    UrlGroupCard {
                        key: "{group.key}",
                        group,
                        visible,
                    }
                }
            }
            p { class: "fgc-note",
                "Each set contains the auto scene switcher plus all ten overlays. Use the URLs for the network your OBS computer can reach."
            }
        }
    }
}

/// One access path's ten Browser Source URLs.
#[component]
fn UrlGroupCard(group: OverlayUrlGroup, visible: bool) -> Element {
    let store = use_context::<AppStore>();
    let background_opaque = store.background_opaque();
    let key = group.key;
    let base_url = group.base_url.clone();
    let count = format!("{} sources", overlay_source_files().count());

    rsx! {
        section { class: "fgc-url-group fgc-url-group--{key}",
            header { class: "fgc-url-group__head",
                div { class: "fgc-url-group__identity",
                    div { class: "fgc-url-group__title-row",
                        h3 { class: "fgc-url-group__title", "{group.label}" }
                        span { class: "fgc-url-group__count", "{count}" }
                    }
                    p { class: "fgc-url-group__description", "{group.description}" }
                }
                if visible {
                    code { class: "fgc-code fgc-url-group__base", "{base_url}" }
                } else {
                    code { class: "fgc-code fgc-code--hidden fgc-url-group__base", "Base URL hidden" }
                }
            }
            div { class: "fgc-source-list",
                for file in overlay_source_files() {
                    SourceRow {
                        key: "{file}",
                        file: file.to_owned(),
                        url: overlay_url_with_background(&group.base_url, file, background_opaque),
                        visible,
                    }
                }
            }
        }
    }
}

/// One overlay URL with open and copy buttons.
#[component]
fn SourceRow(file: String, url: String, visible: bool) -> Element {
    let store = use_context::<AppStore>();
    let toast = use_toast();
    let url_for_code = url.clone();
    let url_for_click = url.clone();
    let url_for_open = url.clone();
    let file_for_toast = file.clone();
    let file_for_open = file.clone();
    let open_label = format!("Open {file} in browser");
    let copy_label = format!("Copy {file} URL");

    rsx! {
        div { class: "fgc-source",
            div { class: "fgc-source__meta",
                p { class: "fgc-source__file", "{file}" }
                if visible {
                    code { class: "fgc-code", "{url_for_code}" }
                } else {
                    code { class: "fgc-code fgc-code--hidden", "URL hidden" }
                }
            }
            div { class: "fgc-actions",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    disabled: !visible,
                    aria_label: open_label,
                    onclick: move |_| {
                        if let Err(error) = open_browser(&url_for_open) {
                            store.report_error(error);
                            return;
                        }
                        toast.success(
                            "Overlay opened".to_owned(),
                            ToastOptions::new().description(file_for_open.clone()),
                        );
                    },
                    "Open"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    disabled: !visible,
                    aria_label: copy_label,
                    onclick: move |_| {
                        copy_to_clipboard(&url_for_click);
                        toast.success(
                            "Browser source copied".to_owned(),
                            ToastOptions::new().description(file_for_toast.clone()),
                        );
                    },
                    "Copy"
                }
            }
        }
    }
}

/// Copies `text` to the clipboard, falling back to `execCommand` when the
/// async clipboard API is unavailable in the WebView.
pub(crate) fn copy_to_clipboard(text: &str) {
    let script = format!(
        "(() => {{ \
            const text = {text:?}; \
            const fallback = (value) => {{ \
                const area = document.createElement('textarea'); \
                area.value = value; \
                area.style.position = 'fixed'; \
                area.style.opacity = '0'; \
                document.body.appendChild(area); \
                area.select(); \
                try {{ document.execCommand('copy'); }} finally {{ area.remove(); }} \
            }}; \
            if (navigator.clipboard && navigator.clipboard.writeText) {{ \
                navigator.clipboard.writeText(text).catch(() => fallback(text)); \
            }} else {{ \
                fallback(text); \
            }} \
        }})();"
    );
    let _ = document::eval(&script);
}
