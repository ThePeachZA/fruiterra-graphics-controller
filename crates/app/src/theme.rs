//! Navigation metadata and the visual theme identifiers.
//!
//! The seven destinations mirror the original application. The order follows
//! real room workflow: run the match, check OBS, then maintain the people and
//! images that feed the overlays. Settings and tournament admin come last.
//! Keeping labels and ordering here means the shell cannot drift apart.

/// One top-level destination in the application shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    /// Overlay server controls and OBS Browser Source URLs.
    Overlay,
    /// Application and overlay settings.
    Settings,
    /// Talent library.
    Talent,
    /// Teams and rosters.
    Teams,
    /// Match preparation and live control.
    MatchController,
    /// Tournament brackets and standings.
    Tournament,
    /// Asset library.
    Assets,
}

impl Destination {
    /// Every destination in navigation order.
    pub const ALL: [Self; 7] = [
        Self::MatchController,
        Self::Overlay,
        Self::Teams,
        Self::Talent,
        Self::Assets,
        Self::Tournament,
        Self::Settings,
    ];

    /// Sidebar label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overlay => "Overlay",
            Self::Settings => "Settings",
            Self::Talent => "Talent",
            Self::Teams => "Teams",
            Self::MatchController => "Match Controller",
            Self::Tournament => "Tournament",
            Self::Assets => "Assets",
        }
    }

    /// One-sentence description shown on placeholder pages.
    #[must_use]
    pub const fn blurb(self) -> &'static str {
        match self {
            Self::Overlay => "Run the overlay server and copy the Browser Source URLs into OBS.",
            Self::Settings => "Overlay network settings and timer display defaults.",
            Self::Talent => "Manage casters, panel members and interviewers.",
            Self::Teams => "Manage teams, colours, logos and player rosters.",
            Self::MatchController => {
                "Prepare the current match, maps, hero bans, talent and timer."
            }
            Self::Tournament => "Build brackets, run groups and publish standings.",
            Self::Assets => "Import and organise the images used by every overlay.",
        }
    }

    /// Milestone that fills this destination in.
    #[must_use]
    pub const fn milestone(self) -> &'static str {
        match self {
            Self::Overlay | Self::Settings => "M4",
            Self::Assets => "M5",
            Self::Talent | Self::Teams => "M6",
            Self::MatchController => "M7",
            Self::Tournament => "M8",
        }
    }

    /// True when the destination has real functionality in this milestone.
    #[must_use]
    pub const fn is_implemented(self) -> bool {
        matches!(
            self,
            Self::Overlay
                | Self::Settings
                | Self::Talent
                | Self::Teams
                | Self::MatchController
                | Self::Tournament
                | Self::Assets
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_destination_has_a_unique_label() {
        let mut labels: Vec<&str> = Destination::ALL.iter().map(|item| item.label()).collect();
        assert_eq!(labels.len(), 7);
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), 7);
    }

    #[test]
    fn navigation_starts_with_the_match_and_ends_with_settings() {
        assert_eq!(Destination::ALL[0], Destination::MatchController);
        assert_eq!(Destination::ALL[6], Destination::Settings);
    }

    #[test]
    fn implemented_destinations_are_the_live_views() {
        let implemented: Vec<Destination> = Destination::ALL
            .iter()
            .copied()
            .filter(|item| item.is_implemented())
            .collect();
        assert_eq!(
            implemented,
            vec![
                Destination::MatchController,
                Destination::Overlay,
                Destination::Teams,
                Destination::Talent,
                Destination::Assets,
                Destination::Tournament,
                Destination::Settings
            ]
        );
    }

    #[test]
    fn placeholders_report_their_milestone() {
        assert_eq!(Destination::Assets.milestone(), "M5");
        assert_eq!(Destination::Talent.milestone(), "M6");
        assert_eq!(Destination::Teams.milestone(), "M6");
        assert_eq!(Destination::MatchController.milestone(), "M7");
        assert_eq!(Destination::Tournament.milestone(), "M8");
    }
}
