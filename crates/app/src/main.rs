#![cfg_attr(
    all(feature = "bundle", target_os = "windows"),
    windows_subsystem = "windows"
)]

//! Fruiterra Graphics Controller — desktop entry point.
//!
//! M0 is a hello-world shell. Navigation, state and server integration arrive
//! in milestone M4.

use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        main { class: "shell",
            header { class: "masthead",
                p { class: "eyebrow", "Fruiterra" }
                h1 { "{fgc_core::APP_NAME}" }
            }
            section { class: "card",
                p { class: "status", "M0 scaffold" }
                p { "Workspace, toolchain and bundle pipeline are wired." }
                p { class: "meta",
                    "Version {fgc_core::VERSION} · overlay port {fgc_server::default_port()}"
                }
            }
        }
    }
}
