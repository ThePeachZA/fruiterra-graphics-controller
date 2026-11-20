use fgc_core::{
    MatchStatus, Position, Role, Side, SocialLink, TalentType, TeamSide, TimerAlignment,
    TimerSettings,
};
use serde::{Deserialize, Serialize};

/// A `{ "type": ..., "data": ... }` envelope.
///
/// `show` is the one exception to the envelope shape; see [`ShowMessage`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message<T> {
    /// Message type read by the overlay subscriber.
    #[serde(rename = "type")]
    pub kind: String,
    /// Section payload.
    pub data: T,
}

impl<T> Message<T> {
    /// Wraps `data` in an envelope with `kind`.
    #[must_use]
    pub fn new(kind: impl Into<String>, data: T) -> Self {
        Self {
            kind: kind.into(),
            data,
        }
    }
}

/// The `show` message.
///
/// The original overlays read `data.visible` from the **top level** of the
/// frame (`frame.visible`), not from a `data` object. The app never sends this
/// message in v1, but it must remain wire-compatible for third-party overlays
/// and future UI toggles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowMessage {
    /// Message type; always `show`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Whether the overlay is visible.
    pub visible: bool,
}

impl ShowMessage {
    /// Creates a `show` frame.
    #[must_use]
    pub fn new(visible: bool) -> Self {
        Self {
            kind: "show".to_owned(),
            visible,
        }
    }
}

/// The scoreboard payload shared by five message types.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreboardDto {
    /// Team A summary, or `null`.
    pub team_a: Option<TeamDto>,
    /// Team B summary, or `null`.
    pub team_b: Option<TeamDto>,
    /// Team A score.
    pub score_a: u16,
    /// Team B score.
    pub score_b: u16,
    /// Match title.
    pub title: String,
    /// Match subtitle.
    pub subtitle: String,
    /// Current map name, or `""`.
    pub current_map: String,
    /// Raw zero-based current map index, or `null`.
    pub current_map_index: Option<usize>,
    /// First to this many map wins.
    pub first_to: u8,
    /// Current map game-mode name, or `""`.
    pub map_type: String,
    /// Match status.
    pub status: MatchStatus,
    /// Numeric side indicator.
    pub side: Side,
    /// Whether the teams' sides are swapped.
    pub swap_sides: bool,
    /// Whether the loser of a map picks the next one.
    pub loser_picks: bool,
    /// Game-mode icons keyed by lower-case mode name.
    pub game_mode_icons: std::collections::BTreeMap<String, String>,
    /// Hero icons keyed by case-sensitive hero name.
    pub hero_icons: std::collections::BTreeMap<String, String>,
    /// Hero portraits keyed by case-sensitive hero name.
    pub hero_portraits: std::collections::BTreeMap<String, String>,
    /// Attack and defend icons; values may be `null`.
    pub side_icons: SideIconsDto,
    /// Role icons keyed by both lower-case and original-case role name.
    pub role_icons: std::collections::BTreeMap<String, String>,
    /// Team A roster in display order.
    pub team_a_players: Vec<PlayerDto>,
    /// Team B roster in display order.
    pub team_b_players: Vec<PlayerDto>,
    /// Every map slot, including empty ones.
    pub maps: Vec<MapDto>,
    /// Hero bans per map.
    pub hero_bans: Vec<HeroBansDto>,
    /// Map pool names.
    pub map_pool: Vec<String>,
    /// Map pool grouped by game mode.
    pub map_pool_data: Vec<MapPoolGroupDto>,
    /// Casters and panel members.
    pub casters_data: Vec<CasterDto>,
    /// Tournament logo URL, or `null`.
    pub tournament_logo_url: Option<String>,
    /// Watermark logo URL, or `null`.
    pub watermark_logo_url: Option<String>,
    /// Whether team colours are used.
    pub team_colors_enabled: bool,
    /// Whether player portraits are shown.
    pub show_player_portraits: bool,
}

/// A team summary inside the scoreboard payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDto {
    /// Display name.
    pub name: String,
    /// Short tag.
    pub tag: String,
    /// Win-loss record; defaults to `"0-0"`.
    pub record: String,
    /// Primary colour.
    pub color: String,
    /// Logo background colour.
    pub logo_bg_color: String,
    /// Logo URL, or `null`.
    pub logo_url: Option<String>,
}

/// A player inside the scoreboard payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDto {
    /// Display name.
    pub name: String,
    /// Role.
    pub role: Role,
    /// Roster position.
    pub position: Position,
    /// Pronouns, omitted when unset (matching the original's `undefined`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    /// Portrait URL, or `null`.
    pub portrait_url: Option<String>,
    /// Featured hero image URL, or `null`.
    pub hero_image_url: Option<String>,
}

/// One map slot inside the scoreboard payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapDto {
    /// Map name.
    pub name: String,
    /// Game-mode display string (the wire `type` field).
    #[serde(rename = "type")]
    pub map_type: String,
    /// Whether the map is complete.
    pub completed: bool,
    /// Team A score.
    pub score_a: u16,
    /// Team B score.
    pub score_b: u16,
    /// Map image URL, or `null`.
    pub map_image_url: Option<String>,
    /// Team that chose the map, or `null`.
    pub chosen_by: Option<TeamSide>,
    /// Tag of the team that chose the map, or `null`.
    pub chosen_by_name: Option<String>,
}

/// Hero bans for one map.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroBansDto {
    /// Team that placed the first ban, or `null`.
    pub first_ban_team: Option<TeamSide>,
    /// Heroes banned by team A.
    pub team_a: Vec<String>,
    /// Heroes banned by team B.
    pub team_b: Vec<String>,
}

/// One game-mode group in the map pool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapPoolGroupDto {
    /// Game-mode name.
    pub mode_name: String,
    /// Zero or one icon URL; always an array.
    pub mode_icons: Vec<String>,
    /// Maps in pool order.
    pub maps: Vec<MapPoolMapDto>,
}

/// One map inside a map-pool group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapPoolMapDto {
    /// Map name.
    pub name: String,
    /// Map image URL, or `null`.
    pub image_url: Option<String>,
    /// Team that chose the map, or `null`.
    pub chosen_by: Option<TeamSide>,
    /// Logo URL of the team that chose the map, or `null`.
    pub chosen_by_logo: Option<String>,
}

/// Attack and defend icons.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideIconsDto {
    /// Attack icon URL, or `null`.
    pub attack: Option<String>,
    /// Defend icon URL, or `null`.
    pub defend: Option<String>,
}

/// The `casters` payload.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastersDto {
    /// Casters, panel members and interviewer in display order.
    pub casters_data: Vec<CasterDto>,
}

/// One on-air talent member.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CasterDto {
    /// Role selected for this match assignment.
    pub role: TalentType,
    /// On-screen name.
    pub display_name: String,
    /// Pronouns, omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    /// Social link, omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<SocialLink>,
    /// Portrait URL, or `null`.
    pub portrait_url: Option<String>,
}

/// Timer settings embedded in a tick payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSettingsDto {
    /// Text alignment.
    pub alignment: TimerAlignment,
    /// Whether single-digit values are padded.
    pub leading_zeroes: bool,
    /// Whether the timer is shown standalone.
    pub standalone: bool,
}

impl From<&TimerSettings> for TimerSettingsDto {
    fn from(settings: &TimerSettings) -> Self {
        Self {
            alignment: settings.alignment,
            leading_zeroes: settings.leading_zeroes,
            standalone: settings.standalone,
        }
    }
}

/// A timer tick payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerTickDto {
    /// Hours.
    pub hours: u32,
    /// Minutes.
    pub minutes: u32,
    /// Seconds.
    pub seconds: u32,
    /// Timer settings.
    pub settings: TimerSettingsDto,
}

/// Timer control action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerActionKind {
    /// Start counting.
    Start,
    /// Pause counting.
    Pause,
    /// Reset to zero.
    Reset,
}

/// A timer control payload.
///
/// `start` and `pause` omit the time fields entirely; `reset` includes them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerActionDto {
    /// Action to perform.
    pub action: TimerActionKind,
    /// Hours, present only for `reset`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<u32>,
    /// Minutes, present only for `reset`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    /// Seconds, present only for `reset`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,
}

impl TimerActionDto {
    /// A `start` action.
    #[must_use]
    pub fn start() -> Self {
        Self {
            action: TimerActionKind::Start,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    /// A `pause` action.
    #[must_use]
    pub fn pause() -> Self {
        Self {
            action: TimerActionKind::Pause,
            hours: None,
            minutes: None,
            seconds: None,
        }
    }

    /// A `reset` action carrying the reset time.
    #[must_use]
    pub fn reset(hours: u32, minutes: u32, seconds: u32) -> Self {
        Self {
            action: TimerActionKind::Reset,
            hours: Some(hours),
            minutes: Some(minutes),
            seconds: Some(seconds),
        }
    }
}

/// Either timer shape carried by `matchTimer`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimerPayload {
    /// A once-per-second tick.
    Tick(TimerTickDto),
    /// A control action.
    Action(TimerActionDto),
}
