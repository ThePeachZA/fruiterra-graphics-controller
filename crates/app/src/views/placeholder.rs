//! Placeholder for destinations that arrive in a later milestone.

use dioxus::prelude::*;

use crate::theme::Destination;

/// A consistent "not yet implemented" page.
#[component]
pub fn PlaceholderView(destination: Destination) -> Element {
    rsx! {
        div { class: "fgc-placeholder",
            span { class: "fgc-placeholder__badge", "{destination.milestone()}" }
            h2 { class: "fgc-placeholder__title", "{destination.label()}" }
            p { class: "fgc-placeholder__blurb", "{destination.blurb()}" }
            p { class: "fgc-placeholder__note",
                "This destination arrives in milestone {destination.milestone()}. \
                 The application shell, persistence and overlay server are already live."
            }
        }
    }
}
