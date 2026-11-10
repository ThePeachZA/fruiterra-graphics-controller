//! Assets destination placeholder (milestone M5).

use dioxus::prelude::*;

use crate::theme::Destination;
use crate::views::placeholder::PlaceholderView;

#[component]
pub fn AssetsView() -> Element {
    rsx! { PlaceholderView { destination: Destination::Assets } }
}
