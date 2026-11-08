use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Generates a new random identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4().to_string())
            }

            /// Wraps an existing identifier, such as a bundled asset key.
            #[must_use]
            pub fn from_raw(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the identifier as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

define_id! {
    /// Identifies a team.
    TeamId
}
define_id! {
    /// Identifies a player.
    PlayerId
}
define_id! {
    /// Identifies a match or bracket fixture.
    MatchId
}
define_id! {
    /// Identifies a talent member.
    TalentId
}
define_id! {
    /// Identifies a tournament.
    TournamentId
}
define_id! {
    /// Identifies an asset.
    AssetId
}
define_id! {
    /// Identifies a round-robin group.
    GroupId
}

/// Player role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Tank role.
    Tank,
    /// Damage role.
    #[default]
    Damage,
    /// Support role.
    Support,
}

/// Roster position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    /// Starting roster.
    #[default]
    Core,
    /// Substitute.
    Sub,
    /// Manager.
    Manager,
    /// Coach.
    Coach,
}

/// Match lifecycle state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    /// Match has not started.
    #[default]
    Pre,
    /// Match is live.
    Live,
    /// Match is complete.
    Post,
}

/// Which team picked or banned a map or hero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TeamSide {
    /// Team A.
    A,
    /// Team B.
    B,
}

/// Side assignment shown on the game overlay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// No side indicator.
    #[default]
    Neutral,
    /// Home team attacks.
    HomeAttacks,
    /// Home team defends.
    HomeDefends,
}

impl Side {
    /// Numeric representation used by the persistence schema.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Neutral => 0,
            Self::HomeAttacks => 1,
            Self::HomeDefends => 2,
        }
    }

    /// Builds a side from its persisted numeric representation.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Neutral),
            1 => Some(Self::HomeAttacks),
            2 => Some(Self::HomeDefends),
            _ => None,
        }
    }
}

impl Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::from_u8(value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid side value {value}; expected 0 (neutral), 1 (home attacks) or 2 (home defends)"
            ))
        })
    }
}

/// Whether match scores are derived from maps or entered by hand.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Scoring {
    /// Scores are derived from completed maps.
    #[default]
    Derived,
    /// Scores are entered directly.
    Manual {
        /// Team A score.
        #[serde(deserialize_with = "clamped_u16")]
        a: u16,
        /// Team B score.
        #[serde(deserialize_with = "clamped_u16")]
        b: u16,
    },
}

/// Asset category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetCategory {
    /// Team logo.
    Logo,
    /// Player or talent portrait.
    Portrait,
    /// Map image.
    Map,
    /// Hero art.
    Hero,
    /// Game-mode icon.
    GameMode,
    /// Role icon.
    Role,
    /// Side icon.
    Side,
}

/// Where an asset's bytes come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetOrigin {
    /// Embedded in the binary.
    Bundled,
    /// Stored in the user's blob directory.
    User,
}

/// Tournament bracket format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BracketFormat {
    /// Single elimination.
    #[default]
    SingleElim,
    /// Double elimination.
    DoubleElim,
    /// Round robin.
    RoundRobin,
}

/// Tournament lifecycle state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TournamentStatus {
    /// Tournament is being set up.
    #[default]
    Setup,
    /// Tournament is in progress.
    InProgress,
    /// Tournament is complete.
    Completed,
}

/// Bracket fixture lifecycle state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BracketMatchStatus {
    /// Fixture has not started.
    #[default]
    Pending,
    /// Fixture is live.
    Live,
    /// Fixture is complete.
    Completed,
}

/// Ordered round-robin tiebreaker criteria.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tiebreaker {
    /// Most wins.
    Wins,
    /// Results between the tied teams.
    HeadToHead,
    /// Map-score differential.
    MapDifferential,
    /// Maps won.
    MapsWon,
    /// Total map points scored.
    PointsFor,
    /// Total map points conceded.
    PointsAgainst,
    /// Entrant seed.
    Seed,
}

/// Timer counting mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerMode {
    /// Count upwards.
    #[default]
    CountUp,
    /// Count downwards.
    CountDown,
    /// Count towards a target time.
    Target,
}

/// Timer text alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerAlignment {
    /// Left aligned.
    Left,
    /// Centre aligned.
    #[default]
    Center,
    /// Right aligned.
    Right,
}

/// Social platform shown by the casters overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SocialPlatform {
    /// Twitter / X.
    Twitter,
    /// Twitch.
    Twitch,
    /// YouTube.
    YouTube,
    /// Instagram.
    Instagram,
    /// TikTok.
    TikTok,
}

/// A team and its roster.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Stable team identifier.
    pub id: TeamId,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Short tag.
    #[serde(default)]
    pub tag: String,
    /// Win-loss record, if entered.
    #[serde(default)]
    pub record: Option<String>,
    /// Primary colour.
    #[serde(default = "default_team_color")]
    pub color: String,
    /// Text colour.
    #[serde(default = "default_text_color")]
    pub text_color: String,
    /// Auxiliary colour.
    #[serde(default = "default_aux_color")]
    pub aux_color: String,
    /// Logo asset.
    #[serde(default)]
    pub logo: Option<AssetId>,
    /// Roster.
    #[serde(default)]
    pub players: Vec<Player>,
}

/// A player on a team roster.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    /// Stable player identifier.
    pub id: PlayerId,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Role.
    #[serde(default)]
    pub role: Role,
    /// Roster position.
    #[serde(default)]
    pub position: Position,
    /// Portrait asset.
    #[serde(default)]
    pub portrait: Option<AssetId>,
    /// Featured hero asset.
    #[serde(default)]
    pub featured_hero: Option<AssetId>,
    /// Pronouns.
    #[serde(default)]
    pub pronouns: Option<String>,
}

/// The current match and everything the overlays show.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    /// Stable match identifier.
    pub id: MatchId,
    /// Match title.
    #[serde(default)]
    pub title: String,
    /// Match subtitle.
    #[serde(default)]
    pub subtitle: String,
    /// Team A.
    #[serde(default)]
    pub team_a: Option<TeamId>,
    /// Team B.
    #[serde(default)]
    pub team_b: Option<TeamId>,
    /// Selected maps.
    #[serde(default)]
    pub maps: Vec<MapSlot>,
    /// Hero bans per map.
    #[serde(default)]
    pub hero_bans: Vec<MapHeroBans>,
    /// Map pool names.
    #[serde(default)]
    pub map_pool: Vec<String>,
    /// First to this many map wins.
    #[serde(default = "default_first_to")]
    pub first_to: u8,
    /// Match status.
    #[serde(default)]
    pub status: MatchStatus,
    /// Zero-based current map index.
    #[serde(default)]
    pub current_map: Option<usize>,
    /// Whether the teams' sides are swapped.
    #[serde(default)]
    pub swap_sides: bool,
    /// Derived or manual score.
    #[serde(default)]
    pub scoring: Scoring,
    /// Side indicator.
    #[serde(default)]
    pub side: Side,
    /// Talent assignments.
    #[serde(default)]
    pub talent: MatchTalent,
    /// Whether the loser of a map picks the next one.
    #[serde(default)]
    pub loser_picks: bool,
    /// Whether player portraits are shown.
    #[serde(default = "default_true")]
    pub show_player_portraits: bool,
}

/// One map in the match.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSlot {
    /// Map name.
    #[serde(default)]
    pub name: String,
    /// Free-text game mode, sent as the wire `type` field.
    #[serde(default)]
    pub game_mode_name: String,
    /// Whether the map is complete.
    #[serde(default)]
    pub completed: bool,
    /// Team A score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score_a: u16,
    /// Team B score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score_b: u16,
    /// Team that chose the map.
    #[serde(default)]
    pub chosen_by: Option<TeamSide>,
}

/// Hero bans for one map.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapHeroBans {
    /// Team that placed the first ban.
    #[serde(default)]
    pub first_ban_team: Option<TeamSide>,
    /// Heroes banned by team A.
    #[serde(default)]
    pub team_a: Vec<String>,
    /// Heroes banned by team B.
    #[serde(default)]
    pub team_b: Vec<String>,
}

/// A talent member.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Talent {
    /// Stable talent identifier.
    pub id: TalentId,
    /// On-screen name.
    #[serde(default)]
    pub display_name: String,
    /// Real name.
    #[serde(default)]
    pub real_name: Option<String>,
    /// Pronouns.
    #[serde(default)]
    pub pronouns: Option<String>,
    /// Social link.
    #[serde(default)]
    pub social: Option<SocialLink>,
    /// Portrait asset.
    #[serde(default)]
    pub portrait: Option<AssetId>,
}

/// A social platform and handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialLink {
    /// Platform.
    pub platform: SocialPlatform,
    /// Handle, including the leading `@` where the platform uses one.
    #[serde(default)]
    pub handle: String,
}

/// A reference to a talent member from a match.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentRef {
    /// Talent identifier.
    pub talent_id: TalentId,
}

/// An interview subject, which may be a talent member or a custom name.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervieweeRef {
    /// Talent identifier, when the interviewee is in the talent library.
    #[serde(default)]
    pub talent_id: Option<TalentId>,
    /// Custom display name.
    #[serde(default)]
    pub custom_name: Option<String>,
    /// Custom subtext.
    #[serde(default)]
    pub custom_subtext: Option<String>,
}

/// Talent assignments for the current match.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchTalent {
    /// First caster.
    #[serde(default)]
    pub caster1: Option<TalentRef>,
    /// Second caster.
    #[serde(default)]
    pub caster2: Option<TalentRef>,
    /// First panel member.
    #[serde(default)]
    pub panel1: Option<TalentRef>,
    /// Second panel member.
    #[serde(default)]
    pub panel2: Option<TalentRef>,
    /// Third panel member.
    #[serde(default)]
    pub panel3: Option<TalentRef>,
    /// Interviewer.
    #[serde(default)]
    pub interviewer: Option<TalentRef>,
    /// Interviewee.
    #[serde(default)]
    pub interviewee: Option<IntervieweeRef>,
}

/// Metadata for a content-addressed asset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Logical identity referenced by teams, players and settings.
    pub id: AssetId,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Category.
    pub category: AssetCategory,
    /// SHA-256 of the file bytes, 64 lowercase hexadecimal characters.
    pub hash: String,
    /// File extension from the allow-list.
    pub ext: String,
    /// Bundled or user-supplied.
    pub origin: AssetOrigin,
    /// Original filename, kept for diagnostics.
    #[serde(default)]
    pub original_filename: Option<String>,
    /// Import timestamp.
    #[serde(default)]
    pub added_at: Option<String>,
    /// Hero icon asset.
    #[serde(default)]
    pub icon: Option<AssetId>,
    /// Hero portrait asset.
    #[serde(default)]
    pub portrait: Option<AssetId>,
    /// Game-mode asset for a map.
    #[serde(default)]
    pub game_mode: Option<AssetId>,
}

/// A tournament, including the generated board and group stage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tournament {
    /// Stable tournament identifier.
    pub id: TournamentId,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Bracket format.
    #[serde(default)]
    pub format: BracketFormat,
    /// Entrants and seeds.
    #[serde(default)]
    pub entrants: Vec<Entrant>,
    /// Winners bracket.
    #[serde(default)]
    pub winners: Vec<Round>,
    /// Losers bracket.
    #[serde(default)]
    pub losers: Vec<Round>,
    /// Grand final, or grand final plus reset.
    #[serde(default)]
    pub grand_final: Vec<Round>,
    /// Round-robin groups.
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Round-robin fixtures.
    #[serde(default)]
    pub group_matches: Vec<GroupMatch>,
    /// Derived standings, never persisted.
    #[serde(skip)]
    pub standings: BTreeMap<GroupId, Vec<Standing>>,
    /// Ordered tiebreaker criteria.
    #[serde(default = "default_tiebreakers")]
    pub tiebreakers: Vec<Tiebreaker>,
    /// Number of teams advancing from each group.
    #[serde(default = "default_playoff_qualifiers")]
    pub playoff_qualifiers: u8,
    /// Playoff bracket format.
    #[serde(default)]
    pub playoff_format: BracketFormat,
    /// Champion, if known.
    #[serde(default)]
    pub champion: Option<TeamId>,
    /// Whether the champion was chosen by hand.
    #[serde(default)]
    pub champion_manual: bool,
    /// Tournament status.
    #[serde(default)]
    pub status: TournamentStatus,
    /// Default first-to for matches in this tournament.
    #[serde(default = "default_first_to")]
    pub default_first_to: u8,
    /// Whether a grand-final reset is enabled.
    #[serde(default = "default_true")]
    pub grand_finals_reset: bool,
}

/// A named bracket round.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    /// Round name.
    #[serde(default)]
    pub name: String,
    /// Matches in display order.
    #[serde(default)]
    pub matches: Vec<BracketMatch>,
}

/// A tournament entrant and seed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entrant {
    /// Team identifier.
    pub team_id: TeamId,
    /// Seed, starting at 1.
    #[serde(default)]
    pub seed: u32,
}

/// One match on the bracket board.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BracketMatch {
    /// Stable match identifier.
    pub id: MatchId,
    /// Round number.
    #[serde(default)]
    pub round: u32,
    /// Match number within the round.
    #[serde(default)]
    pub number: u32,
    /// First slot.
    #[serde(default)]
    pub slot1: Option<TeamId>,
    /// Second slot.
    #[serde(default)]
    pub slot2: Option<TeamId>,
    /// Whether slot 1 was placed by hand.
    #[serde(default)]
    pub slot1_manual: bool,
    /// Whether slot 2 was placed by hand.
    #[serde(default)]
    pub slot2_manual: bool,
    /// First score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score1: u16,
    /// Second score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score2: u16,
    /// Winner.
    #[serde(default)]
    pub winner: Option<TeamId>,
    /// Whether the winner was chosen by hand.
    #[serde(default)]
    pub winner_manual: bool,
    /// Fixture status.
    #[serde(default)]
    pub status: BracketMatchStatus,
    /// Where the winner advances.
    #[serde(default)]
    pub next: Option<MatchLink>,
    /// Where the loser drops.
    #[serde(default)]
    pub loser_next: Option<MatchLink>,
    /// First to this many map wins.
    #[serde(default = "default_first_to")]
    pub first_to: u8,
}

/// A link to a target match and slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchLink {
    /// Target match identifier.
    #[serde(rename = "match")]
    pub match_id: MatchId,
    /// Target slot, 1 or 2.
    #[serde(default)]
    pub slot: u8,
}

/// A round-robin group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    /// Stable group identifier.
    pub id: GroupId,
    /// Group name.
    #[serde(default)]
    pub name: String,
    /// Teams in the group.
    #[serde(default)]
    pub team_ids: Vec<TeamId>,
}

/// One round-robin fixture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMatch {
    /// Stable fixture identifier.
    pub id: MatchId,
    /// Group the fixture belongs to.
    pub group: GroupId,
    /// First team.
    #[serde(default)]
    pub team1: Option<TeamId>,
    /// Second team.
    #[serde(default)]
    pub team2: Option<TeamId>,
    /// First score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score1: u16,
    /// Second score.
    #[serde(default, deserialize_with = "clamped_u16")]
    pub score2: u16,
    /// Fixture status.
    #[serde(default)]
    pub status: BracketMatchStatus,
    /// First to this many map wins.
    #[serde(default = "default_first_to")]
    pub first_to: u8,
}

/// A derived standings row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Standing {
    /// Team identifier.
    pub team_id: TeamId,
    /// Fixtures played.
    #[serde(default)]
    pub played: u32,
    /// Fixtures won.
    #[serde(default)]
    pub wins: u32,
    /// Fixtures lost.
    #[serde(default)]
    pub losses: u32,
    /// Fixtures drawn.
    #[serde(default)]
    pub draws: u32,
    /// Map points scored.
    #[serde(default)]
    pub points_for: u32,
    /// Map points conceded.
    #[serde(default)]
    pub points_against: u32,
    /// Map differential.
    #[serde(default)]
    pub map_differential: i32,
    /// League points (wins in v1).
    #[serde(default)]
    pub points: u32,
}

/// Overlay server settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlaySettings {
    /// HTTP/WebSocket port.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Whether to bind on the LAN interface.
    #[serde(default)]
    pub allow_lan: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            port: default_port(),
            allow_lan: false,
        }
    }
}

/// Display settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySettings {
    /// Whether team colours are used.
    #[serde(default)]
    pub team_colors_enabled: bool,
    /// Tournament logo asset.
    #[serde(default)]
    pub tournament_logo: Option<AssetId>,
    /// Watermark logo asset.
    #[serde(default)]
    pub watermark_logo: Option<AssetId>,
}

/// Timer settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSettings {
    /// Counting mode.
    #[serde(default)]
    pub mode: TimerMode,
    /// Text alignment.
    #[serde(default)]
    pub alignment: TimerAlignment,
    /// Whether single-digit values are padded.
    #[serde(default = "default_true")]
    pub leading_zeroes: bool,
    /// Whether the timer is shown standalone.
    #[serde(default)]
    pub standalone: bool,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            mode: TimerMode::default(),
            alignment: TimerAlignment::default(),
            leading_zeroes: true,
            standalone: false,
        }
    }
}

/// Persisted application settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Overlay server.
    #[serde(default)]
    pub overlay: OverlaySettings,
    /// Display options.
    #[serde(default)]
    pub display: DisplaySettings,
    /// Timer options.
    #[serde(default)]
    pub timer: TimerSettings,
}

/// Everything persisted in `app.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    /// Persistence schema version.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Teams and rosters.
    #[serde(default)]
    pub teams: Vec<Team>,
    /// Talent library.
    #[serde(default)]
    pub talents: Vec<Talent>,
    /// Current match, if one is being prepared.
    #[serde(default)]
    pub current_match: Option<Match>,
    /// Tournaments.
    #[serde(default)]
    pub tournaments: Vec<Tournament>,
    /// User asset metadata.
    #[serde(default)]
    pub assets: Vec<Asset>,
    /// Settings.
    #[serde(default)]
    pub settings: Settings,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: crate::SCHEMA_VERSION,
            teams: Vec::new(),
            talents: Vec::new(),
            current_match: None,
            tournaments: Vec::new(),
            assets: Vec::new(),
            settings: Settings::default(),
            extra: BTreeMap::new(),
        }
    }
}

fn clamped_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    Ok(value.clamp(0, i64::from(u16::MAX)) as u16)
}

const fn default_schema_version() -> u32 {
    crate::SCHEMA_VERSION
}

const fn default_first_to() -> u8 {
    3
}

const fn default_true() -> bool {
    true
}

const fn default_playoff_qualifiers() -> u8 {
    2
}

const fn default_port() -> u16 {
    3001
}

fn default_team_color() -> String {
    "#000000".to_owned()
}

fn default_text_color() -> String {
    "#ffffff".to_owned()
}

fn default_aux_color() -> String {
    "#090909".to_owned()
}

fn default_tiebreakers() -> Vec<Tiebreaker> {
    vec![
        Tiebreaker::Wins,
        Tiebreaker::HeadToHead,
        Tiebreaker::MapDifferential,
        Tiebreaker::PointsFor,
        Tiebreaker::Seed,
    ]
}
