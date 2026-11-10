// Vendored from Dioxus Components (https://github.com/DioxusLabs/components)
// at revision 9a758255ea26e2b20c8cecf4c1c946feb9e71da7.
// Copyright (c) Dioxus Labs. Licensed under MIT OR Apache-2.0.

use dioxus::prelude::*;
use dioxus_primitives::label::{self, LabelProps};
#[css_module("/src/components/label/style.css")]
struct Styles;

#[component]
pub fn Label(props: LabelProps) -> Element {
    rsx! {
        label::Label {
            class: Styles::dx_label,
            html_for: props.html_for,
            attributes: props.attributes,
            {props.children}
        }
    }
}
