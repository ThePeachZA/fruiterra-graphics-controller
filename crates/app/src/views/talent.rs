//! Talent destination placeholder (milestone M6).

use dioxus::prelude::*;

use crate::theme::Destination;
use crate::views::placeholder::PlaceholderView;

#[component]
pub fn TalentView() -> Element {
    rsx! { PlaceholderView { destination: Destination::Talent } }
}
