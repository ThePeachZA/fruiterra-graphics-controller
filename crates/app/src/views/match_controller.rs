//! Match Controller destination placeholder (milestone M7).

use dioxus::prelude::*;

use crate::theme::Destination;
use crate::views::placeholder::PlaceholderView;

#[component]
pub fn MatchControllerView() -> Element {
    rsx! { PlaceholderView { destination: Destination::MatchController } }
}
