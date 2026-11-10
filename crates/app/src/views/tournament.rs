//! Tournament destination placeholder (milestone M8).

use dioxus::prelude::*;

use crate::theme::Destination;
use crate::views::placeholder::PlaceholderView;

#[component]
pub fn TournamentView() -> Element {
    rsx! { PlaceholderView { destination: Destination::Tournament } }
}
