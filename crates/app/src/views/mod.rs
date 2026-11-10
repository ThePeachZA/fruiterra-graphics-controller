//! Destination views rendered inside the application shell.
//!
//! Milestone M4 implements the Overlay and Settings destinations, M5 adds
//! Assets, and M6 adds Talent and Teams. The remaining two render
//! [`placeholder::PlaceholderView`] until their milestones land.

pub mod assets;
pub mod match_controller;
pub mod overlay;
pub mod placeholder;
pub mod settings;
pub mod talent;
pub mod teams;
pub mod tournament;

use dioxus::prelude::*;

use crate::theme::Destination;

/// Renders the view for `destination`.
#[component]
pub fn DestinationView(destination: Destination) -> Element {
    match destination {
        Destination::Overlay => rsx! { overlay::OverlayView {} },
        Destination::Settings => rsx! { settings::SettingsView {} },
        Destination::Talent => rsx! { talent::TalentView {} },
        Destination::Teams => rsx! { teams::TeamsView {} },
        Destination::MatchController => rsx! { match_controller::MatchControllerView {} },
        Destination::Tournament => rsx! { tournament::TournamentView {} },
        Destination::Assets => rsx! { assets::AssetsView {} },
    }
}
