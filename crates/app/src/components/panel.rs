//! Hand-rolled layout panel used by every view.
//!
//! The vendored Dioxus Components cover form controls, but the original
//! application's card/section layout is simpler to express directly than to
//! bend a generic card into shape.

use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// A titled section with a body.
#[component]
pub fn Panel(
    /// Optional heading rendered in the panel header.
    #[props(default)]
    title: Option<String>,
    /// Optional supporting copy rendered under the heading.
    #[props(default)]
    description: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(section { class: "fgc-panel" });
    let merged = merge_attributes(vec![base, attributes]);
    let has_header = title.is_some() || description.is_some();

    rsx! {
        section { ..merged,
            if has_header {
                header { class: "fgc-panel__header",
                    if let Some(title) = title {
                        h2 { class: "fgc-panel__title", "{title}" }
                    }
                    if let Some(description) = description {
                        p { class: "fgc-panel__description", "{description}" }
                    }
                }
            }
            div { class: "fgc-panel__body", {children} }
        }
    }
}

/// A labelled form control with optional hint text.
#[component]
pub fn Field(
    /// Visible label.
    label: String,
    /// Optional hint shown beside the label.
    #[props(default)]
    hint: Option<String>,
    /// Optional validation message shown below the control.
    #[props(default)]
    error: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "fgc-field",
            div { class: "fgc-field__label",
                span { "{label}" }
                if let Some(hint) = hint {
                    span { class: "fgc-field__hint", "{hint}" }
                }
            }
            {children}
            if let Some(error) = error {
                p { class: "fgc-field__error", role: "alert", "{error}" }
            }
        }
    }
}

/// A title, description and toggle laid out on one row.
#[component]
pub fn ToggleRow(
    /// Row title.
    title: String,
    /// Explanation of what the toggle does.
    #[props(default)]
    hint: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "fgc-toggle-row",
            div { class: "fgc-toggle-row__text",
                p { class: "fgc-toggle-row__title", "{title}" }
                if let Some(hint) = hint {
                    p { class: "fgc-toggle-row__hint", "{hint}" }
                }
            }
            {children}
        }
    }
}

/// A read-only key/value list.
#[component]
pub fn KeyValues(children: Element) -> Element {
    rsx! {
        dl { class: "fgc-kv", {children} }
    }
}

/// One key/value pair inside [`KeyValues`].
#[component]
pub fn KeyValue(
    /// Left-hand label.
    label: String,
    /// Right-hand value.
    value: String,
) -> Element {
    rsx! {
        dt { class: "fgc-kv__key", "{label}" }
        dd { class: "fgc-kv__value", "{value}" }
    }
}
