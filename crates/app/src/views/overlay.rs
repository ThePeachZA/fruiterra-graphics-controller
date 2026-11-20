//! Overlay destination: server controls and OBS Browser Source URLs.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use fgc_server::ServerStatus;

use crate::assets_io::open_browser;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::{KeyValue, KeyValues, Panel};
use crate::server::{OVERLAY_FILES, overlay_url};
use crate::state::AppStore;
use crate::status::{status_text, status_tone};

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
                        if let ServerStatus::Running { port, .. } = &status {
                            KeyValue {
                                label: "Listening on".to_owned(),
                                value: format!("port {port}"),
                            }
                        }
                    }
                    div { class: "fgc-actions",
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
                        Button {
                            variant: ButtonVariant::Outline,
                            aria_pressed: urls_visible,
                            onclick: move |_| store.set_urls_visible(!urls_visible),
                            if urls_visible { "Hide URLs" } else { "Show URLs" }
                        }
                    }
                    p { class: "fgc-note",
                        "URLs are hidden until you reveal them. Port, LAN access, asset URLs and auto-start are configured in Settings."
                    }
                }
                Panel {
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
            OverlayUrls { base_url }
        }
    }
}

/// The seven Browser Source URLs, hidden until the operator reveals them.
///
/// The same component is used on Overlay and in Match Controller → Config so
/// the visibility choice stays consistent and no URL leaks into either view.
#[component]
pub(crate) fn OverlayUrls(base_url: String) -> Element {
    let store = use_context::<AppStore>();
    let visible = store.urls_visible();

    rsx! {
        Panel {
            title: "Browser sources".to_owned(),
            description: "Copy a URL into an OBS Browser Source at 1920×1080.".to_owned(),
            div { class: "fgc-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    aria_pressed: visible,
                    onclick: move |_| store.set_urls_visible(!visible),
                    if visible { "Hide URLs" } else { "Show URLs" }
                }
            }
            div { class: "fgc-source-list",
                for file in OVERLAY_FILES {
                    SourceRow {
                        key: "{file}",
                        file: file.to_owned(),
                        url: overlay_url(&base_url, file),
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
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
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
                    "Open in browser"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        copy_to_clipboard(&url_for_click);
                        toast.success(
                            "Browser source copied".to_owned(),
                            ToastOptions::new().description(file_for_toast.clone()),
                        );
                    },
                    "Copy URL"
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
