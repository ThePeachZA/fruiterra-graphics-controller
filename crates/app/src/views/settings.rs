//! Settings destination: overlay network, startup and display defaults.

use dioxus::prelude::*;
use fgc_server::AssetMode;

use crate::assets_io::format_bytes;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::components::{Field, KeyValue, KeyValues, Panel, ToggleRow};
use crate::server::DEFAULT_PORT;
use crate::state::AppStore;

/// Editable settings that are live in milestone M4 and M5.
#[component]
pub fn SettingsView() -> Element {
    let store = use_context::<AppStore>();
    let state = store.state();
    let server = store.server();
    let settings = state.read().settings.clone();
    let asset_mode = server.asset_mode();
    let unused = store.unused();
    let busy = store.asset_busy();
    let overlays_path = fgc_core::overlays_override_dir().map_or_else(
        |_| "Unavailable".to_owned(),
        |path| path.display().to_string(),
    );

    let port_text = use_signal(|| settings.overlay.port.to_string());
    let port_error = use_signal(|| None::<String>);

    use_effect(move || store.refresh_unused());

    rsx! {
        div { class: "fgc-view",
            Panel {
                title: "Overlay network".to_owned(),
                description: "Where the overlay server listens and how assets are addressed.".to_owned(),
                Field {
                    label: "Port".to_owned(),
                    hint: Some(format!("Default {DEFAULT_PORT}")),
                    error: port_error(),
                    div { class: "fgc-inline",
                        Input {
                            r#type: "number",
                            min: "1",
                            max: "65535",
                            value: "{port_text}",
                            aria_label: "Overlay server port",
                            oninput: move |event: FormEvent| {
                                set_port_text(port_text, event.value());
                            },
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter {
                                    apply_port(store, port_text, port_error);
                                }
                            },
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| apply_port(store, port_text, port_error),
                            "Apply"
                        }
                    }
                }
                ToggleRow {
                    title: "Start server on launch".to_owned(),
                    hint: Some(
                        "Start the overlay server as soon as the application opens."
                            .to_owned(),
                    ),
                    Switch {
                        checked: settings.overlay.auto_start,
                        aria_label: "Start server on launch",
                        on_checked_change: move |value| store.set_auto_start(value),
                    }
                }
                ToggleRow {
                    title: "Allow LAN connections".to_owned(),
                    hint: Some(
                        "Bind on all interfaces so another machine on the network can render overlays. \
                         Loopback-only is safer."
                            .to_owned(),
                    ),
                    Switch {
                        checked: settings.overlay.allow_lan,
                        aria_label: "Allow LAN connections",
                        on_checked_change: move |value| store.set_allow_lan(value),
                    }
                }
                Field {
                    label: "Asset URLs".to_owned(),
                    hint: Some(
                        "Served URLs keep payloads small. Inline data URLs embed every image in each message."
                            .to_owned(),
                    ),
                    select {
                        class: "fgc-native-select",
                        value: if asset_mode == AssetMode::Served { "served" } else { "data-url" },
                        aria_label: "Asset URL mode",
                        onchange: move |event| {
                            let mode = if event.value() == "data-url" {
                                AssetMode::DataUrl
                            } else {
                                AssetMode::Served
                            };
                            store.set_asset_mode(mode);
                        },
                        option { value: "served", "Served URLs (/assets/…)" }
                        option { value: "data-url", "Inline data URLs" }
                    }
                }
            }
            Panel {
                title: "Display".to_owned(),
                description: "Team colouring and the logos embedded in scoreboard payloads.".to_owned(),
                ToggleRow {
                    title: "Use team colours".to_owned(),
                    hint: Some("Tint scoreboard elements with each team's primary colour.".to_owned()),
                    Switch {
                        checked: settings.display.team_colors_enabled,
                        aria_label: "Use team colours",
                        on_checked_change: move |value| {
                            store.mutate(move |state| {
                                state.settings.display.team_colors_enabled = value;
                            });
                        },
                    }
                }
                KeyValues {
                    KeyValue {
                        label: "Watermark logo".to_owned(),
                        value: settings
                            .display
                            .watermark_logo
                            .as_ref()
                            .map_or_else(|| "Bundled default (Peach)".to_owned(), ToString::to_string),
                    }
                    KeyValue {
                        label: "Tournament logo".to_owned(),
                        value: settings
                            .display
                            .tournament_logo
                            .as_ref()
                            .map_or_else(|| "None".to_owned(), ToString::to_string),
                    }
                }
                p { class: "fgc-note", "Asset pickers arrive with the Teams, Talent and Tournament views." }
            }
            Panel {
                title: "Asset storage".to_owned(),
                description: "Content-addressed images used by the overlays and the asset library.".to_owned(),
                KeyValues {
                    KeyValue {
                        label: "Unused asset data".to_owned(),
                        value: format!("{} file(s), {}", unused.files, format_bytes(unused.bytes)),
                    }
                }
                div { class: "fgc-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: busy || unused.files == 0,
                        onclick: move |_| store.clean_up_unused(),
                        if busy { "Working…" } else { "Clean up now" }
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            match crate::assets_io::open_overlays_folder() {
                                Ok(directory) => store.notify(format!(
                                    "Opened {}.",
                                    directory.display()
                                )),
                                Err(error) => store.report_error(error),
                            }
                        },
                        "Open overlays folder"
                    }
                }
                p { class: "fgc-note",
                    "Unreferenced image data is deleted automatically after a successful save and on startup. Clean up now removes anything the saved and current projects no longer use."
                }
            }
            Panel {
                title: "Overlay overrides".to_owned(),
                description: "Tweak one overlay without rebuilding the application.".to_owned(),
                KeyValues {
                    KeyValue { label: "Overlays folder".to_owned(), value: overlays_path }
                }
                p { class: "fgc-note",
                    "Place a copy of any of the seven overlay HTML files in this folder to override it. A missing or broken override falls back to the embedded file, and the other six overlays are unaffected."
                }
            }
        }
    }
}

fn set_port_text(mut port_text: Signal<String>, value: String) {
    port_text.set(value);
}

fn apply_port(store: AppStore, port_text: Signal<String>, mut port_error: Signal<Option<String>>) {
    let raw = port_text.read().trim().to_owned();
    match raw.parse::<u16>() {
        Ok(port) if port != 0 => {
            port_error.set(None);
            store.set_port(port);
        }
        _ => port_error.set(Some("Enter a port between 1 and 65535.".to_owned())),
    }
}
