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

impl Role {
    /// Every role in display order.
    pub const ALL: [Self; 3] = [Self::Tank, Self::Damage, Self::Support];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tank => "Tank",
            Self::Damage => "Damage",
            Self::Support => "Support",
        }
    }

    /// Stable lowercase slug matching the serde representation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Tank => "tank",
            Self::Damage => "damage",
            Self::Support => "support",
        }
    }

    /// Parses a slug produced by [`Self::slug`].
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.slug() == slug)
    }
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

impl Position {
    /// Every position in display order.
    pub const ALL: [Self; 4] = [Self::Core, Self::Sub, Self::Manager, Self::Coach];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Sub => "Sub",
            Self::Manager => "Manager",
            Self::Coach => "Coach",
        }
    }

    /// Stable lowercase slug matching the serde representation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Sub => "sub",
            Self::Manager => "manager",
            Self::Coach => "coach",
        }
    }

    /// Parses a slug produced by [`Self::slug`].
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|position| position.slug() == slug)
    }
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

impl AssetCategory {
    /// Every category in display order.
    pub const ALL: [Self; 7] = [
        Self::Logo,
        Self::Portrait,
        Self::Map,
        Self::Hero,
        Self::GameMode,
        Self::Role,
        Self::Side,
    ];

    /// Human-readable label used by filters and badges.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Logo => "Logo",
            Self::Portrait => "Portrait",
            Self::Map => "Map",
            Self::Hero => "Hero",
            Self::GameMode => "Game mode",
            Self::Role => "Role",
            Self::Side => "Side",
        }
    }

    /// Stable kebab-case slug matching the serde representation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Logo => "logo",
            Self::Portrait => "portrait",
            Self::Map => "map",
            Self::Hero => "hero",
            Self::GameMode => "game-mode",
            Self::Role => "role",
            Self::Side => "side",
        }
    }

    /// Parses a slug produced by [`Self::slug`].
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.slug() == slug)
    }
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
    /// Group stage followed by an elimination bracket.
    MultiStage,
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
    /// Count towards a manually entered target time.
    Target,
    /// Count down to the current match's scheduled start.
    Scheduled,
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

/// On-air role selected for a talent assignment in Match Controller.
///
/// Roles are per assignment, not per person: the same talent can be a host in
/// one match and an analyst in the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TalentType {
    /// Play-by-play or colour commentary.
    #[default]
    Caster,
    /// Show host.
    Host,
    /// Desk or analyst panel member.
    #[serde(alias = "panelist")]
    Analyst,
    /// Interviewer.
    Interviewer,
    /// Anything else, including legacy observers and producers.
    #[serde(alias = "observer")]
    Other,
}

impl TalentType {
    /// Every role in display order.
    pub const ALL: [Self; 5] = [
        Self::Caster,
        Self::Host,
        Self::Analyst,
        Self::Interviewer,
        Self::Other,
    ];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Caster => "Caster",
            Self::Host => "Host",
            Self::Analyst => "Analyst",
            Self::Interviewer => "Interviewer",
            Self::Other => "Other",
        }
    }

    /// Stable kebab-case slug matching the serde representation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Caster => "caster",
            Self::Host => "host",
            Self::Analyst => "analyst",
            Self::Interviewer => "interviewer",
            Self::Other => "other",
        }
    }

    /// Parses a slug produced by [`Self::slug`], including legacy aliases.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "panelist" => Some(Self::Analyst),
            "observer" => Some(Self::Other),
            _ => Self::ALL.into_iter().find(|kind| kind.slug() == slug),
        }
    }
}

/// Social platform shown by the casters overlay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SocialPlatform {
    /// Twitter / X.
    #[default]
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

impl SocialPlatform {
    /// Every platform in display order.
    pub const ALL: [Self; 5] = [
        Self::Twitter,
        Self::Twitch,
        Self::YouTube,
        Self::Instagram,
        Self::TikTok,
    ];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Twitter => "Twitter / X",
            Self::Twitch => "Twitch",
            Self::YouTube => "YouTube",
            Self::Instagram => "Instagram",
            Self::TikTok => "TikTok",
        }
    }

    /// Stable lowercase slug matching the serde representation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Twitter => "twitter",
            Self::Twitch => "twitch",
            Self::YouTube => "youtube",
            Self::Instagram => "instagram",
            Self::TikTok => "tiktok",
        }
    }

    /// Parses a slug produced by [`Self::slug`].
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|platform| platform.slug() == slug)
    }
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
    /// Primary colour.
    #[serde(default = "default_team_color")]
    pub color: String,
    /// Text colour.
    #[serde(default = "default_text_color")]
    pub text_color: String,
    /// Whether the background colour is recommended from the selected logo.
    #[serde(default)]
    pub color_from_logo: bool,
    /// Whether the text colour is recommended from the selected logo.
    #[serde(default)]
    pub text_color_from_logo: bool,
    /// Whether a high-contrast colour is computed for the text only.
    #[serde(default)]
    pub computed_contrast: bool,
    /// Auxiliary colour.
    #[serde(default = "default_aux_color")]
    pub aux_color: String,
    /// Logo asset.
    #[serde(default)]
    pub logo: Option<AssetId>,
    /// Whether the logo sits on its own background colour.
    #[serde(default)]
    pub logo_background: bool,
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
    /// Scheduled start, as entered in a `datetime-local` field.
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// Team A.
    #[serde(default)]
    pub team_a: Option<TeamId>,
    /// Team B.
    #[serde(default)]
    pub team_b: Option<TeamId>,
    /// Tournament whose seed or standings supply the default team records.
    #[serde(default)]
    pub tournament_id: Option<TournamentId>,
    /// Manual win-loss or seed override for team A.
    #[serde(default)]
    pub team_a_record: Option<String>,
    /// Manual win-loss or seed override for team B.
    #[serde(default)]
    pub team_b_record: Option<String>,
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
    /// Game mode sent as the wire `type` field.
    ///
    /// Derived from the map's game-mode tag until the owner types a value.
    #[serde(default)]
    pub game_mode_name: String,
    /// True when `game_mode_name` was typed and must survive tag changes.
    #[serde(default)]
    pub game_mode_manual: bool,
    /// True when the slot was added explicitly with **+ Add map**.
    ///
    /// An empty user-added slot is still user data, so load-time normalisation
    /// must not trim it even though it has no name or mode yet.
    #[serde(default)]
    pub user_added: bool,
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
    /// Role chosen for this match assignment.
    #[serde(default)]
    pub role: TalentType,
}

/// An interview subject, which may be a talent member, a rostered player, or
/// a custom name.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervieweeRef {
    /// Talent identifier, when the interviewee is in the talent library.
    #[serde(default)]
    pub talent_id: Option<TalentId>,
    /// Player identifier, when the interviewee is on one of the match teams.
    #[serde(default)]
    pub player_id: Option<PlayerId>,
    /// Custom display name.
    #[serde(default)]
    pub custom_name: Option<String>,
    /// Custom subtext.
    #[serde(default)]
    pub custom_subtext: Option<String>,
    /// Custom portrait asset.
    #[serde(default)]
    pub custom_portrait: Option<AssetId>,
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

/// Role an attached image plays for its owner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetAttachmentKind {
    /// Small ban-strip image owned by a hero.
    #[default]
    HeroBanIcon,
    /// Transparent cut-out art owned by a hero.
    HeroTransparent,
    /// Hexagonal badge art owned by a hero.
    HeroBadge,
}

/// Marks an asset as owned by another asset instead of listed in the library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetAttachment {
    /// Asset that owns this file.
    pub owner: AssetId,
    /// Role this file plays for the owner.
    pub kind: AssetAttachmentKind,
}

impl AssetAttachment {
    /// Creates a hero ban-icon attachment.
    #[must_use]
    pub const fn hero_ban_icon(owner: AssetId) -> Self {
        Self {
            owner,
            kind: AssetAttachmentKind::HeroBanIcon,
        }
    }

    /// Creates a hero transparent-art attachment.
    #[must_use]
    pub const fn hero_transparent(owner: AssetId) -> Self {
        Self {
            owner,
            kind: AssetAttachmentKind::HeroTransparent,
        }
    }

    /// Creates a hero badge-art attachment.
    #[must_use]
    pub const fn hero_badge(owner: AssetId) -> Self {
        Self {
            owner,
            kind: AssetAttachmentKind::HeroBadge,
        }
    }
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
    /// Hexagonal badge art for the hero bans overlay.
    #[serde(default)]
    pub badge: Option<AssetId>,
    /// Hero portrait asset.
    #[serde(default)]
    pub portrait: Option<AssetId>,
    /// Transparent cut-out art for the hero bans overlay.
    #[serde(default)]
    pub transparent: Option<AssetId>,
    /// Role of the hero, shown as a hexagon on the hero bans overlay.
    #[serde(default)]
    pub role: Option<Role>,
    /// Game-mode asset for a map.
    #[serde(default)]
    pub game_mode: Option<AssetId>,
    /// Attached media owned by another asset, hidden from the library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AssetAttachment>,
}

impl Asset {
    /// True when this asset is an attachment owned by another asset.
    #[must_use]
    pub const fn is_attachment(&self) -> bool {
        self.attachment.is_some()
    }
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
    /// Manually chosen winner, when the score-derived winner is overridden.
    #[serde(default)]
    pub winner: Option<TeamId>,
    /// Whether `winner` overrides the score-derived winner.
    #[serde(default)]
    pub winner_manual: bool,
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

/// Who can reach the overlay server.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RemoteAccessMode {
    /// Loopback only, the v1 default.
    #[default]
    Off,
    /// Any device on the local network.
    Lan,
    /// A device on the user's Tailscale tailnet.
    Tailscale,
    /// The public internet through Tailscale Funnel.
    Funnel,
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
    /// Whether the server starts with the application.
    #[serde(default)]
    pub auto_start: bool,
    /// Access scope shown in Settings.
    #[serde(default)]
    pub remote_access: RemoteAccessMode,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            port: default_port(),
            allow_lan: false,
            auto_start: false,
            remote_access: RemoteAccessMode::Off,
        }
    }
}

/// The built-in overlay themes.
///
/// The overlays inline the Night Broadcast tokens as their stylesheet
/// defaults. A different preset is delivered as a set of CSS custom property
/// overrides, so switching a theme never needs a rebuild.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreset {
    /// The dark Night Broadcast theme.
    #[default]
    NightBroadcast,
    /// The light Daylight theme.
    Light,
}

impl ThemePreset {
    /// The stable name used on the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NightBroadcast => "nightBroadcast",
            Self::Light => "light",
        }
    }

    /// The human-readable name shown in the app.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NightBroadcast => "Night Broadcast",
            Self::Light => "Light",
        }
    }

    /// Every preset, in the order the app lists them.
    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::NightBroadcast, Self::Light]
    }
}

/// Display settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySettings {
    /// Whether team colours are used.
    #[serde(default)]
    pub team_colors_enabled: bool,
    /// Whether the Hero Bans section is shown in Map Selection.
    #[serde(default = "default_true")]
    pub show_hero_bans: bool,
    /// Whether the overlays show the short team tag instead of the full name.
    #[serde(default)]
    pub team_abbreviations: bool,
    /// The built-in overlay theme.
    #[serde(default)]
    pub theme: ThemePreset,
    /// Token overrides loaded from an imported theme file.
    ///
    /// They are applied over the built-in preset, so a theme file may set only
    /// the tokens it cares about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme_tokens: Option<BTreeMap<String, String>>,
    /// Tournament logo asset.
    #[serde(default)]
    pub tournament_logo: Option<AssetId>,
    /// Watermark logo asset.
    #[serde(default)]
    pub watermark_logo: Option<AssetId>,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            team_colors_enabled: false,
            show_hero_bans: true,
            team_abbreviations: false,
            theme: ThemePreset::default(),
            theme_tokens: None,
            tournament_logo: None,
            watermark_logo: None,
        }
    }
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
    /// Whether hour digits are shown on the overlay.
    #[serde(default = "default_true")]
    pub show_hours: bool,
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
            show_hours: true,
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
    /// Saved matches, including scheduled matches for a later overlay.
    #[serde(default)]
    pub matches: Vec<Match>,
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
            matches: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_match;

    #[test]
    fn role_slugs_round_trip() {
        for role in Role::ALL {
            assert_eq!(Role::from_slug(role.slug()), Some(role));
            assert!(!role.label().is_empty());
        }
        assert_eq!(Role::from_slug("unknown"), None);
    }

    #[test]
    fn position_slugs_round_trip() {
        for position in Position::ALL {
            assert_eq!(Position::from_slug(position.slug()), Some(position));
            assert!(!position.label().is_empty());
        }
        assert_eq!(Position::from_slug("unknown"), None);
    }

    #[test]
    fn social_platform_slugs_round_trip() {
        for platform in SocialPlatform::ALL {
            assert_eq!(SocialPlatform::from_slug(platform.slug()), Some(platform));
            assert!(!platform.label().is_empty());
        }
        assert_eq!(SocialPlatform::from_slug("unknown"), None);
    }

    #[test]
    fn talent_type_slugs_round_trip() {
        for kind in TalentType::ALL {
            assert_eq!(TalentType::from_slug(kind.slug()), Some(kind));
            assert!(!kind.label().is_empty());
            let json = serde_json::to_string(&kind).expect("serialise talent type");
            assert_eq!(json, format!("\"{}\"", kind.slug()));
        }
        assert_eq!(TalentType::from_slug("unknown"), None);
    }

    #[test]
    fn assignment_defaults_to_caster_when_role_is_missing() {
        let assignment: TalentRef =
            serde_json::from_str(r#"{"talentId":"talent-one"}"#).expect("load legacy assignment");
        assert_eq!(assignment.role, TalentType::Caster);

        let value = serde_json::to_value(&assignment).expect("serialise assignment");
        assert_eq!(value["role"], "caster");
    }

    #[test]
    fn new_schema_fields_have_documented_defaults() {
        let overlay: OverlaySettings =
            serde_json::from_str(r#"{"port":3001,"allowLan":false}"#).expect("overlay");
        assert!(!overlay.auto_start);
        assert_eq!(overlay.remote_access, RemoteAccessMode::Off);

        let display: DisplaySettings = serde_json::from_str(r#"{}"#).expect("display");
        assert!(display.show_hero_bans);

        let team: Team = serde_json::from_str(r#"{"id":"team-one","name":"Team"}"#)
            .expect("team without logo background");
        assert!(!team.logo_background);

        let current: Match = serde_json::from_str(r#"{"id":"match-one"}"#).expect("match");
        assert_eq!(current.scheduled_at, None);

        let state: AppState =
            serde_json::from_value(serde_json::json!({"schemaVersion": 2})).expect("state");
        assert!(state.matches.is_empty());
    }

    #[test]
    fn scheduled_timer_mode_round_trips() {
        let json = serde_json::to_string(&TimerMode::Scheduled).expect("serialise timer mode");
        assert_eq!(json, "\"scheduled\"");
        let mode: TimerMode = serde_json::from_str("\"scheduled\"").expect("load timer mode");
        assert_eq!(mode, TimerMode::Scheduled);
    }

    #[test]
    fn timer_show_hours_defaults_on_and_round_trips() {
        let legacy: TimerSettings = serde_json::from_str("{}").expect("legacy timer settings");
        assert!(legacy.show_hours);

        let hidden: TimerSettings =
            serde_json::from_str(r#"{"showHours":false}"#).expect("hidden-hour timer settings");
        assert!(!hidden.show_hours);
        let json = serde_json::to_string(&hidden).expect("serialise timer settings");
        assert!(json.contains("\"showHours\":false"), "{json}");
    }

    #[test]
    fn team_abbreviations_default_off_and_round_trip() {
        assert!(!DisplaySettings::default().team_abbreviations);

        let legacy: DisplaySettings = serde_json::from_str("{}").expect("legacy display settings");
        assert!(!legacy.team_abbreviations);

        let abbreviated: DisplaySettings =
            serde_json::from_str(r#"{"teamAbbreviations":true}"#).expect("abbreviated settings");
        assert!(abbreviated.team_abbreviations);
        let json = serde_json::to_string(&abbreviated).expect("serialise display settings");
        assert!(json.contains("\"teamAbbreviations\":true"), "{json}");
    }

    #[test]
    fn theme_preset_defaults_to_night_broadcast_and_round_trips() {
        assert_eq!(
            DisplaySettings::default().theme,
            ThemePreset::NightBroadcast
        );

        let legacy: DisplaySettings = serde_json::from_str("{}").expect("legacy display settings");
        assert_eq!(legacy.theme, ThemePreset::NightBroadcast);
        assert!(legacy.theme_tokens.is_none());

        let light: DisplaySettings =
            serde_json::from_str(r#"{"theme":"light"}"#).expect("light display settings");
        assert_eq!(light.theme, ThemePreset::Light);
        let json = serde_json::to_string(&light).expect("serialise display settings");
        assert!(json.contains("\"theme\":\"light\""), "{json}");

        let imported: DisplaySettings =
            serde_json::from_str(r##"{"themeTokens":{"--fgc-surface":"#ffffff"}}"##)
                .expect("imported theme settings");
        let tokens = imported.theme_tokens.expect("imported tokens");
        assert_eq!(
            tokens.get("--fgc-surface").map(String::as_str),
            Some("#ffffff")
        );
    }

    #[test]
    fn theme_preset_names_are_stable() {
        assert_eq!(ThemePreset::NightBroadcast.as_str(), "nightBroadcast");
        assert_eq!(ThemePreset::Light.as_str(), "light");
        assert_eq!(ThemePreset::all().len(), 2);
    }

    #[test]
    fn hero_badge_round_trips() {
        let badge = AssetId::from_raw("asset-badge");
        let mut asset = Asset::new_user("Ana", AssetCategory::Hero, "a".repeat(64), "png", None)
            .expect("badge asset");
        asset.badge = Some(badge);

        let json = serde_json::to_string(&asset).expect("serialise asset");
        assert!(json.contains("\"badge\":\"asset-badge\""), "{json}");

        let legacy: Asset = serde_json::from_value(serde_json::json!({
            "id": "asset-legacy",
            "category": "hero",
            "hash": "a".repeat(64),
            "ext": "png",
            "origin": "user",
        }))
        .expect("legacy asset");
        assert_eq!(legacy.badge, None);
    }

    #[test]
    fn new_schema_fields_round_trip() {
        let overlay = OverlaySettings {
            auto_start: true,
            remote_access: RemoteAccessMode::Lan,
            ..OverlaySettings::default()
        };
        let json = serde_json::to_string(&overlay).expect("serialise overlay");
        assert!(json.contains("\"autoStart\":true"));
        assert!(json.contains("\"remoteAccess\":\"lan\""));

        let display = DisplaySettings {
            show_hero_bans: false,
            ..DisplaySettings::default()
        };
        let json = serde_json::to_string(&display).expect("serialise display");
        assert!(json.contains("\"showHeroBans\":false"));

        let mut team: Team =
            serde_json::from_str(r#"{"id":"team-one","name":"Team"}"#).expect("team");
        team.logo_background = true;
        let json = serde_json::to_string(&team).expect("serialise team");
        assert!(json.contains("\"logoBackground\":true"));

        let mut current = new_match();
        current.scheduled_at = Some("2026-11-21T18:00".to_owned());
        let json = serde_json::to_string(&current).expect("serialise match");
        assert!(json.contains("\"scheduledAt\":\"2026-11-21T18:00\""));

        let mut state = AppState::default();
        state.matches.push(current);
        let json = serde_json::to_string(&state).expect("serialise state");
        assert!(json.contains("\"matches\":["));
    }

    #[test]
    fn role_and_position_serialise_to_their_slugs() {
        for role in Role::ALL {
            let json = serde_json::to_string(&role).expect("serialise role");
            assert_eq!(json, format!("\"{}\"", role.slug()));
        }
        for position in Position::ALL {
            let json = serde_json::to_string(&position).expect("serialise position");
            assert_eq!(json, format!("\"{}\"", position.slug()));
        }
    }
}
