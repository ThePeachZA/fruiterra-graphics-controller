// Vendored from Dioxus Components (https://github.com/DioxusLabs/components)
// at revision 9a758255ea26e2b20c8cecf4c1c946feb9e71da7.
// Copyright (c) Dioxus Labs. Licensed under MIT OR Apache-2.0.

use dioxus::prelude::*;
use dioxus_primitives::switch::{self, SwitchProps};

#[css_module("/src/components/switch/style.css")]
struct Styles;

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    rsx! {
        switch::Switch {
            class: Styles::dx_switch,
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: props.attributes,
            switch::SwitchThumb { class: Styles::dx_switch_thumb }
        }
    }
}
