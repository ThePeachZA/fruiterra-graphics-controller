//! Authenticated local and remote control endpoint.
//!
//! The control surface is deliberately separate from the public overlay
//! routes. Loopback clients use a bearer token; remote clients sign the exact
//! request with HMAC-SHA256 so the token is never transmitted. Both modes share
//! one replay guard and forward validated actions to the application task.

use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::runtime::SharedState;

/// Path of the authenticated control endpoint.
pub const CONTROL_PATH: &str = "/control";

const DOMAIN_SEPARATOR: &str = "FGC-CONTROL-V1";
const HMAC_SCHEME: &str = "HMAC-SHA256";
const MAX_TIMESTAMP_SKEW_SECONDS: u64 = 60;
const NONCE_TTL: Duration = Duration::from_secs(300);
const RATE_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT: u32 = 30;
const NONCE_MIN_LEN: usize = 8;
const NONCE_MAX_LEN: usize = 128;
const UNDO_TTL: Duration = Duration::from_secs(300);
const UNDO_LIMIT: usize = 256;
const MAX_INDEX: usize = 64;
const MAX_SCORE: u16 = 999;
const MAX_TITLE_LEN: usize = 120;
const MAX_RECORD_LEN: usize = 24;
const MAX_HERO_LEN: usize = 80;
const MAX_SCHEDULED_LEN: usize = 32;
const MAX_ID_LEN: usize = 64;

type HmacSha256 = Hmac<Sha256>;

/// Team selector used by score and ban actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlTeamSide {
    /// Team A.
    A,
    /// Team B.
    B,
}

/// Team selector that can also clear a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlOptionalTeam {
    /// Team A.
    A,
    /// Team B.
    B,
    /// Clear the selection.
    None,
}

/// Match side indicator wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlMatchSide {
    /// No indicator.
    Neutral,
    /// Home attacks.
    HomeAttacks,
    /// Home defends.
    HomeDefends,
}

/// Match lifecycle wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMatchStatus {
    /// Upcoming.
    Pre,
    /// Live.
    Live,
    /// Complete.
    Post,
}

/// Timer run commands dispatched to the desktop timer controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlTimerCommand {
    /// Start or resume the clock.
    Start,
    /// Pause the clock.
    Pause,
    /// Stop and reset the clock.
    Stop,
}

/// Timer counting-mode wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlTimerMode {
    /// Count upwards.
    CountUp,
    /// Count downwards.
    CountDown,
    /// Count towards a target time.
    Target,
    /// Count towards the scheduled start.
    Scheduled,
}

/// Timer alignment wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlTimerAlignment {
    /// Left aligned.
    Left,
    /// Centre aligned.
    Center,
    /// Right aligned.
    Right,
}

/// Map result wire values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMapOutcome {
    /// Team A wins.
    A,
    /// Team B wins.
    B,
    /// Draw.
    Draw,
    /// Clear the result.
    Unplayed,
}

/// Show and Match Controller actions accepted by the control endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    /// Show all overlays.
    Show,
    /// Hide all overlays.
    Hide,
    /// Recalculate and resend current overlays.
    ForceUpdate,
    /// Select an auto-overlay scene by its configured label.
    SetScene(String),
    /// Advance to the next enabled auto-overlay scene.
    NextScene,
    /// Return to the previous enabled auto-overlay scene.
    PreviousScene,
    /// Set the current match title.
    SetMatchTitle { value: String },
    /// Set the current match subtitle.
    SetMatchSubtitle { value: String },
    /// Set or clear the scheduled start.
    SetMatchScheduledAt { value: String },
    /// Set the first-to target.
    SetMatchFirstTo { value: u8 },
    /// Set match status.
    SetMatchStatus { status: ControlMatchStatus },
    /// Set the side indicator.
    SetMatchSide { side: ControlMatchSide },
    /// Set or clear the tournament link.
    SetMatchTournament { value: Option<String> },
    /// Set or clear one team.
    SetMatchTeam {
        team: ControlTeamSide,
        value: Option<String>,
    },
    /// Set or clear a manual team record.
    SetMatchRecord {
        team: ControlTeamSide,
        value: Option<String>,
    },
    /// Toggle the side swap.
    SetSwapSides { enabled: bool },
    /// Toggle manual score override.
    SetManualScoring { enabled: bool },
    /// Set one manual score.
    SetManualScore { team: ControlTeamSide, score: u16 },
    /// Adjust one manual score by a single point.
    ScoreAdjust { team: ControlTeamSide, delta: i8 },
    /// Snapshot the current match into the library.
    SaveCurrentMatch,
    /// Load a saved match into the controller.
    LoadSavedMatch { id: String },
    /// Delete a saved match.
    DeleteSavedMatch { id: String },
    /// Replace the current match with a fresh one.
    CreateMatch,
    /// Run the desktop timer.
    Timer(ControlTimerCommand),
    /// Change timer counting mode.
    SetTimerMode { mode: ControlTimerMode },
    /// Change timer alignment.
    SetTimerAlignment { alignment: ControlTimerAlignment },
    /// Toggle leading zeroes.
    SetTimerLeadingZeroes { enabled: bool },
    /// Toggle hour digits.
    SetTimerShowHours { enabled: bool },
    /// Toggle the standalone timer layout.
    SetTimerStandalone { enabled: bool },
    /// Mark one map slot current.
    SetCurrentMap { index: usize },
    /// Clear the current map selection.
    ClearCurrentMap,
    /// Set one map's picker.
    SetMapPick {
        index: usize,
        team: ControlOptionalTeam,
    },
    /// Set one map's result.
    SetMapResult {
        index: usize,
        outcome: ControlMapOutcome,
    },
    /// Set one map's name.
    SetMapName { index: usize, value: String },
    /// Set one map's game mode.
    SetMapMode { index: usize, mode: String },
    /// Set one map's two scores.
    SetMapScores {
        index: usize,
        score_a: u16,
        score_b: u16,
    },
    /// Mark one map complete or incomplete.
    SetMapCompleted { index: usize, enabled: bool },
    /// Add a map slot at an index.
    InsertMap { index: usize },
    /// Remove a map slot.
    RemoveMap { index: usize },
    /// Move a map slot by one position.
    MoveMap { index: usize, direction: i8 },
    /// Reset all map slots.
    ResetMaps,
    /// Clear all hero bans.
    ResetHeroBans,
    /// Set one map's first-ban team.
    SetFirstBanTeam {
        index: usize,
        team: ControlOptionalTeam,
    },
    /// Set or clear one hero ban.
    SetHeroBan {
        index: usize,
        team: ControlTeamSide,
        hero: String,
    },
    /// Add or remove a map-pool entry.
    SetMapPoolIncluded { name: String, enabled: bool },
    /// Move a map-pool entry by one position.
    MoveMapPool { index: usize, direction: i8 },
    /// Restore the inverse captured for an undo token.
    Undo(String),
    /// Internal envelope carrying the single-use undo token for a write.
    #[doc(hidden)]
    WithUndo {
        /// The validated action to apply.
        action: Box<ControlAction>,
        /// Token derived from the request nonce.
        undo: String,
    },
}

/// Body accepted by `POST /control`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(try_from = "RawControlRequest")]
pub struct ControlRequest {
    /// Show action to apply.
    pub action: ControlAction,
    /// Unix timestamp in seconds; requests outside a 60-second window are
    /// rejected.
    pub timestamp: u64,
    /// Unique request identifier, retained for five minutes to stop replay.
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawControlRequest {
    action: String,
    timestamp: u64,
    nonce: String,
    #[serde(default)]
    scene: Option<String>,
    #[serde(default)]
    index: Option<usize>,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    hero: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    side: Option<String>,
    #[serde(default)]
    alignment: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    outcome: Option<String>,
    #[serde(default)]
    score: Option<u16>,
    #[serde(default)]
    score_a: Option<u16>,
    #[serde(default)]
    score_b: Option<u16>,
    #[serde(default)]
    delta: Option<i8>,
    #[serde(default)]
    direction: Option<i8>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    confirm: Option<bool>,
    #[serde(default)]
    undo: Option<String>,
}

fn required<T>(value: Option<T>) -> Result<T, ControlError> {
    value.ok_or(ControlError::InvalidRequest)
}

fn valid_index(value: Option<usize>) -> Result<usize, ControlError> {
    match value {
        Some(index) if index <= MAX_INDEX => Ok(index),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn valid_step(value: Option<i8>) -> Result<i8, ControlError> {
    match value {
        Some(-1 | 1) => Ok(value.expect("matched")),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn valid_score(value: Option<u16>) -> Result<u16, ControlError> {
    match value {
        Some(score) if score <= MAX_SCORE => Ok(score),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn valid_text(value: &str, max: usize, allow_empty: bool) -> bool {
    value.chars().count() <= max
        && !value.chars().any(char::is_control)
        && (allow_empty || !value.trim().is_empty())
}

fn text(raw: Option<String>, max: usize, allow_empty: bool) -> Result<String, ControlError> {
    match raw {
        Some(value) if valid_text(&value, max, allow_empty) => Ok(value),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn valid_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=MAX_ID_LEN).contains(&bytes.len())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn optional_id(raw: Option<String>) -> Result<Option<String>, ControlError> {
    match raw {
        None => Ok(None),
        Some(value) if value.is_empty() => Ok(None),
        Some(value) if valid_id(&value) => Ok(Some(value)),
        Some(_) => Err(ControlError::InvalidRequest),
    }
}

fn required_id(raw: Option<String>) -> Result<String, ControlError> {
    match raw {
        Some(value) if valid_id(&value) => Ok(value),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn valid_scheduled(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    let bytes = value.as_bytes();
    (bytes.len() == 16 || bytes.len() == 19)
        && [4, 7].iter().all(|index| bytes[*index] == b'-')
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && (bytes.len() == 16 || bytes[16] == b':')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit())
}

fn valid_undo_token(value: &str) -> bool {
    value
        .strip_prefix("u-")
        .is_some_and(|rest| rest.len() == 64 && rest.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn team_side(raw: Option<&str>) -> Result<ControlTeamSide, ControlError> {
    match raw {
        Some("a") => Ok(ControlTeamSide::A),
        Some("b") => Ok(ControlTeamSide::B),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn optional_team(raw: Option<&str>) -> Result<ControlOptionalTeam, ControlError> {
    match raw {
        Some("a") => Ok(ControlOptionalTeam::A),
        Some("b") => Ok(ControlOptionalTeam::B),
        Some("none") => Ok(ControlOptionalTeam::None),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn match_status(raw: Option<&str>) -> Result<ControlMatchStatus, ControlError> {
    match raw {
        Some("pre") => Ok(ControlMatchStatus::Pre),
        Some("live") => Ok(ControlMatchStatus::Live),
        Some("post") => Ok(ControlMatchStatus::Post),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn match_side(raw: Option<&str>) -> Result<ControlMatchSide, ControlError> {
    match raw {
        Some("neutral") => Ok(ControlMatchSide::Neutral),
        Some("home_attacks") => Ok(ControlMatchSide::HomeAttacks),
        Some("home_defends") => Ok(ControlMatchSide::HomeDefends),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn timer_command(raw: Option<&str>) -> Result<ControlTimerCommand, ControlError> {
    match raw {
        Some("start") => Ok(ControlTimerCommand::Start),
        Some("pause") => Ok(ControlTimerCommand::Pause),
        Some("stop") => Ok(ControlTimerCommand::Stop),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn timer_mode(raw: Option<&str>) -> Result<ControlTimerMode, ControlError> {
    match raw {
        Some("count_up") => Ok(ControlTimerMode::CountUp),
        Some("count_down") => Ok(ControlTimerMode::CountDown),
        Some("target") => Ok(ControlTimerMode::Target),
        Some("scheduled") => Ok(ControlTimerMode::Scheduled),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn timer_alignment(raw: Option<&str>) -> Result<ControlTimerAlignment, ControlError> {
    match raw {
        Some("left") => Ok(ControlTimerAlignment::Left),
        Some("center") => Ok(ControlTimerAlignment::Center),
        Some("right") => Ok(ControlTimerAlignment::Right),
        _ => Err(ControlError::InvalidRequest),
    }
}

fn map_outcome(raw: Option<&str>) -> Result<ControlMapOutcome, ControlError> {
    match raw {
        Some("a") => Ok(ControlMapOutcome::A),
        Some("b") => Ok(ControlMapOutcome::B),
        Some("draw") => Ok(ControlMapOutcome::Draw),
        Some("unplayed") => Ok(ControlMapOutcome::Unplayed),
        _ => Err(ControlError::InvalidRequest),
    }
}

impl TryFrom<RawControlRequest> for ControlRequest {
    type Error = ControlError;

    fn try_from(raw: RawControlRequest) -> Result<Self, Self::Error> {
        let scene = raw.scene.filter(|scene| {
            let bytes = scene.as_bytes();
            (1..=128).contains(&bytes.len())
                && bytes
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        });
        let index = || valid_index(raw.index);
        let step = |value: Option<i8>| valid_step(value);
        let flag = |value: Option<bool>| required(value);
        let team = || team_side(raw.team.as_deref());
        let action = match raw.action.as_str() {
            "show" => ControlAction::Show,
            "hide" => ControlAction::Hide,
            "force_update" => ControlAction::ForceUpdate,
            "set_scene" => ControlAction::SetScene(scene.ok_or(ControlError::InvalidRequest)?),
            "next_scene" => ControlAction::NextScene,
            "previous_scene" => ControlAction::PreviousScene,
            "set_match_title" => ControlAction::SetMatchTitle {
                value: text(raw.value, MAX_TITLE_LEN, true)?,
            },
            "set_match_subtitle" => ControlAction::SetMatchSubtitle {
                value: text(raw.value, MAX_TITLE_LEN, true)?,
            },
            "set_match_scheduled_at" => {
                let value = text(raw.value, MAX_SCHEDULED_LEN, true)?;
                if !valid_scheduled(&value) {
                    return Err(ControlError::InvalidRequest);
                }
                ControlAction::SetMatchScheduledAt { value }
            }
            "set_match_first_to" => {
                let value = required(raw.score)?;
                if !(1..=4).contains(&value) {
                    return Err(ControlError::InvalidRequest);
                }
                ControlAction::SetMatchFirstTo { value: value as u8 }
            }
            "set_match_status" => ControlAction::SetMatchStatus {
                status: match_status(raw.status.as_deref())?,
            },
            "set_match_side" => ControlAction::SetMatchSide {
                side: match_side(raw.side.as_deref())?,
            },
            "set_match_tournament" => ControlAction::SetMatchTournament {
                value: optional_id(raw.value)?,
            },
            "set_match_team" => ControlAction::SetMatchTeam {
                team: team()?,
                value: optional_id(raw.value)?,
            },
            "set_match_record" => ControlAction::SetMatchRecord {
                team: team()?,
                value: match raw.value {
                    None => None,
                    Some(value) if value.is_empty() => None,
                    Some(value) if valid_text(&value, MAX_RECORD_LEN, false) => Some(value),
                    Some(_) => return Err(ControlError::InvalidRequest),
                },
            },
            "set_swap_sides" => ControlAction::SetSwapSides {
                enabled: flag(raw.enabled)?,
            },
            "set_manual_scoring" => ControlAction::SetManualScoring {
                enabled: flag(raw.enabled)?,
            },
            "set_manual_score" => ControlAction::SetManualScore {
                team: team()?,
                score: valid_score(raw.score)?,
            },
            "score_adjust" => ControlAction::ScoreAdjust {
                team: team()?,
                delta: step(raw.delta)?,
            },
            "save_current_match" => ControlAction::SaveCurrentMatch,
            "load_saved_match" => ControlAction::LoadSavedMatch {
                id: required_id(raw.id)?,
            },
            "delete_saved_match" => ControlAction::DeleteSavedMatch {
                id: required_id(raw.id)?,
            },
            "create_match" => ControlAction::CreateMatch,
            "timer" => ControlAction::Timer(timer_command(raw.command.as_deref())?),
            "set_timer_mode" => ControlAction::SetTimerMode {
                mode: timer_mode(raw.mode.as_deref())?,
            },
            "set_timer_alignment" => ControlAction::SetTimerAlignment {
                alignment: timer_alignment(raw.alignment.as_deref())?,
            },
            "set_timer_leading_zeroes" => ControlAction::SetTimerLeadingZeroes {
                enabled: flag(raw.enabled)?,
            },
            "set_timer_show_hours" => ControlAction::SetTimerShowHours {
                enabled: flag(raw.enabled)?,
            },
            "set_timer_standalone" => ControlAction::SetTimerStandalone {
                enabled: flag(raw.enabled)?,
            },
            "set_current_map" => ControlAction::SetCurrentMap { index: index()? },
            "clear_current_map" => ControlAction::ClearCurrentMap,
            "set_map_pick" => ControlAction::SetMapPick {
                index: index()?,
                team: optional_team(raw.team.as_deref())?,
            },
            "set_map_result" => ControlAction::SetMapResult {
                index: index()?,
                outcome: map_outcome(raw.outcome.as_deref())?,
            },
            "set_map_name" => ControlAction::SetMapName {
                index: index()?,
                value: text(raw.value, MAX_TITLE_LEN, true)?,
            },
            "set_map_mode" => ControlAction::SetMapMode {
                index: index()?,
                mode: text(raw.mode, MAX_TITLE_LEN, true)?,
            },
            "set_map_scores" => ControlAction::SetMapScores {
                index: index()?,
                score_a: valid_score(raw.score_a)?,
                score_b: valid_score(raw.score_b)?,
            },
            "set_map_completed" => ControlAction::SetMapCompleted {
                index: index()?,
                enabled: flag(raw.enabled)?,
            },
            "insert_map" => ControlAction::InsertMap { index: index()? },
            "remove_map" => ControlAction::RemoveMap { index: index()? },
            "move_map" | "move_map_pool" => {
                let index = index()?;
                let direction = step(raw.direction)?;
                if raw.action == "move_map" {
                    ControlAction::MoveMap { index, direction }
                } else {
                    ControlAction::MoveMapPool { index, direction }
                }
            }
            "reset_maps" => ControlAction::ResetMaps,
            "reset_hero_bans" => ControlAction::ResetHeroBans,
            "set_first_ban_team" => ControlAction::SetFirstBanTeam {
                index: index()?,
                team: optional_team(raw.team.as_deref())?,
            },
            "set_hero_ban" => ControlAction::SetHeroBan {
                index: index()?,
                team: team()?,
                hero: text(raw.hero, MAX_HERO_LEN, true)?,
            },
            "set_map_pool_included" => ControlAction::SetMapPoolIncluded {
                name: text(raw.name, MAX_TITLE_LEN, false)?,
                enabled: flag(raw.enabled)?,
            },
            "undo" => ControlAction::Undo(match raw.undo {
                Some(token) if valid_undo_token(&token) => token,
                _ => return Err(ControlError::InvalidRequest),
            }),
            _ => return Err(ControlError::InvalidRequest),
        };
        if action.requires_confirmation() && raw.confirm != Some(true) {
            return Err(ControlError::ConfirmationRequired);
        }
        Ok(Self {
            action,
            timestamp: raw.timestamp,
            nonce: raw.nonce,
        })
    }
}

impl ControlAction {
    /// Stable wire name used by the control endpoint and its documentation.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Hide => "hide",
            Self::ForceUpdate => "force_update",
            Self::SetScene(_) => "set_scene",
            Self::NextScene => "next_scene",
            Self::PreviousScene => "previous_scene",
            Self::SetMatchTitle { .. } => "set_match_title",
            Self::SetMatchSubtitle { .. } => "set_match_subtitle",
            Self::SetMatchScheduledAt { .. } => "set_match_scheduled_at",
            Self::SetMatchFirstTo { .. } => "set_match_first_to",
            Self::SetMatchStatus { .. } => "set_match_status",
            Self::SetMatchSide { .. } => "set_match_side",
            Self::SetMatchTournament { .. } => "set_match_tournament",
            Self::SetMatchTeam { .. } => "set_match_team",
            Self::SetMatchRecord { .. } => "set_match_record",
            Self::SetSwapSides { .. } => "set_swap_sides",
            Self::SetManualScoring { .. } => "set_manual_scoring",
            Self::SetManualScore { .. } => "set_manual_score",
            Self::ScoreAdjust { .. } => "score_adjust",
            Self::SaveCurrentMatch => "save_current_match",
            Self::LoadSavedMatch { .. } => "load_saved_match",
            Self::DeleteSavedMatch { .. } => "delete_saved_match",
            Self::CreateMatch => "create_match",
            Self::Timer(_) => "timer",
            Self::SetTimerMode { .. } => "set_timer_mode",
            Self::SetTimerAlignment { .. } => "set_timer_alignment",
            Self::SetTimerLeadingZeroes { .. } => "set_timer_leading_zeroes",
            Self::SetTimerShowHours { .. } => "set_timer_show_hours",
            Self::SetTimerStandalone { .. } => "set_timer_standalone",
            Self::SetCurrentMap { .. } => "set_current_map",
            Self::ClearCurrentMap => "clear_current_map",
            Self::SetMapPick { .. } => "set_map_pick",
            Self::SetMapResult { .. } => "set_map_result",
            Self::SetMapName { .. } => "set_map_name",
            Self::SetMapMode { .. } => "set_map_mode",
            Self::SetMapScores { .. } => "set_map_scores",
            Self::SetMapCompleted { .. } => "set_map_completed",
            Self::InsertMap { .. } => "insert_map",
            Self::RemoveMap { .. } => "remove_map",
            Self::MoveMap { .. } => "move_map",
            Self::ResetMaps => "reset_maps",
            Self::ResetHeroBans => "reset_hero_bans",
            Self::SetFirstBanTeam { .. } => "set_first_ban_team",
            Self::SetHeroBan { .. } => "set_hero_ban",
            Self::SetMapPoolIncluded { .. } => "set_map_pool_included",
            Self::MoveMapPool { .. } => "move_map_pool",
            Self::Undo(_) => "undo",
            Self::WithUndo { action, .. } => action.name(),
        }
    }

    /// Whether this action needs an explicit `confirm: true` field.
    #[must_use]
    fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            Self::LoadSavedMatch { .. }
                | Self::DeleteSavedMatch { .. }
                | Self::CreateMatch
                | Self::SetMapPick { .. }
                | Self::SetMapResult { .. }
                | Self::SetMapName { .. }
                | Self::SetMapMode { .. }
                | Self::SetMapScores { .. }
                | Self::InsertMap { .. }
                | Self::RemoveMap { .. }
                | Self::MoveMap { .. }
                | Self::ResetMaps
                | Self::ResetHeroBans
                | Self::SetFirstBanTeam { .. }
                | Self::SetHeroBan { .. }
                | Self::SetMapPoolIncluded { .. }
                | Self::MoveMapPool { .. }
        )
    }

    /// Whether this action writes Match Controller state and needs an undo.
    #[must_use]
    fn is_write(&self) -> bool {
        !matches!(
            self,
            Self::Show
                | Self::Hide
                | Self::ForceUpdate
                | Self::SetScene(_)
                | Self::NextScene
                | Self::PreviousScene
                | Self::Undo(_)
                | Self::WithUndo { .. }
        )
    }

    /// Undo token attached by the control handler, if any.
    #[must_use]
    fn undo_token(&self) -> Option<&str> {
        match self {
            Self::WithUndo { undo, .. } => Some(undo),
            _ => None,
        }
    }
}

/// Generates a high-entropy bearer token for the app to persist beside
/// `app.json`.
#[must_use]
pub fn generate_control_token() -> String {
    Uuid::new_v4().to_string()
}

/// Raw token bytes kept in memory for HMAC verification. Its `Debug` output is
/// redacted so a log or diagnostic cannot leak the write credential.
#[derive(Clone)]
struct RemoteSecret(Arc<[u8]>);

impl fmt::Debug for RemoteSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RemoteSecret([REDACTED])")
    }
}

/// Validates the bearer token or HMAC signature and forwards accepted actions
/// to the UI task.
#[derive(Clone)]
pub struct ControlBridge {
    token_digest: [u8; 32],
    token: RemoteSecret,
    actions: mpsc::UnboundedSender<ControlAction>,
    remote_enabled: bool,
    /// Single-use undo tokens with their issue time.
    undo: Arc<Mutex<HashMap<String, Instant>>>,
}

impl ControlBridge {
    /// Creates a bridge for `token`. `remote_enabled` allows HMAC requests and
    /// remote peers; bearer requests remain restricted to loopback.
    #[must_use]
    pub fn new(
        token: &str,
        actions: mpsc::UnboundedSender<ControlAction>,
        remote_enabled: bool,
    ) -> Self {
        Self {
            token_digest: digest(token),
            token: RemoteSecret(Arc::from(token.as_bytes())),
            actions,
            remote_enabled,
            undo: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Whether remote access is currently enabled for this bridge.
    #[must_use]
    pub fn remote_enabled(&self) -> bool {
        self.remote_enabled
    }

    /// Returns whether `token` matches the bridge digest in constant time.
    #[must_use]
    pub fn accepts_token(&self, token: &str) -> bool {
        constant_time_eq(&self.token_digest, &digest(token))
    }

    fn authorises(
        &self,
        headers: &HeaderMap,
        peer: IpAddr,
        request: &ControlRequest,
        body: &[u8],
    ) -> Result<(), ControlError> {
        let Some(value) = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Err(ControlError::Unauthorised);
        };
        let mut parts = value.splitn(2, ' ');
        let scheme = parts.next().unwrap_or_default();
        let credential = parts
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or_default();

        if scheme.eq_ignore_ascii_case("Bearer") {
            if !peer.is_loopback() || !self.accepts_token(credential) {
                return Err(ControlError::Unauthorised);
            }
            return Ok(());
        }

        if scheme.eq_ignore_ascii_case(HMAC_SCHEME) {
            if !self.remote_enabled {
                return Err(ControlError::Unauthorised);
            }
            let provided = decode_hex_32(credential).ok_or(ControlError::Unauthorised)?;
            let mut mac = HmacSha256::new_from_slice(&self.token.0)
                .map_err(|_| ControlError::Unauthorised)?;
            mac.update(&canonical_message(request, body));
            let expected = mac.finalize().into_bytes();
            let mut expected_bytes = [0_u8; 32];
            expected_bytes.copy_from_slice(&expected);
            if !constant_time_eq(&provided, &expected_bytes) {
                return Err(ControlError::Unauthorised);
            }
            return Ok(());
        }

        Err(ControlError::Unauthorised)
    }

    fn forward(&self, action: ControlAction) -> Result<(), ControlError> {
        self.actions
            .send(action)
            .map_err(|_| ControlError::BridgeClosed)
    }

    /// Creates and remembers the single-use token for one accepted write.
    fn issue_undo(&self, nonce: &str, timestamp: u64) -> String {
        let material = format!("{DOMAIN_SEPARATOR}\n{nonce}\n{timestamp}");
        let token = format!("u-{}", hex_lower(Sha256::digest(material.as_bytes())));
        let mut undo = lock(&self.undo);
        let now = Instant::now();
        undo.retain(|_, issued| now.duration_since(*issued) < UNDO_TTL);
        let oldest = (undo.len() >= UNDO_LIMIT && !undo.contains_key(&token))
            .then(|| {
                undo.iter()
                    .min_by_key(|(_, issued)| **issued)
                    .map(|(token, _)| token.clone())
            })
            .flatten();
        if let Some(oldest) = oldest {
            undo.remove(&oldest);
        }
        undo.insert(token.clone(), now);
        token
    }

    /// Consumes a fresh undo token, or reports it as unavailable.
    fn validate_undo(&self, token: &str) -> Result<(), ControlError> {
        let mut undo = lock(&self.undo);
        let Some(issued) = undo.get(token).copied() else {
            return Err(ControlError::UndoUnavailable);
        };
        if issued.elapsed() >= UNDO_TTL {
            undo.remove(token);
            return Err(ControlError::UndoUnavailable);
        }
        undo.remove(token);
        Ok(())
    }
}

/// The exact signed message. The trailing newline is part of the contract.
fn canonical_message(request: &ControlRequest, body: &[u8]) -> Vec<u8> {
    format!(
        "{DOMAIN_SEPARATOR}\n{}\n{}\nPOST\n{CONTROL_PATH}\n{}\n",
        request.timestamp,
        request.nonce,
        hex_lower(Sha256::digest(body)),
    )
    .into_bytes()
}

/// Replay and abuse guards shared by all requests to the control endpoint.
#[derive(Default)]
pub(crate) struct ControlGuard {
    nonces: Mutex<HashMap<(IpAddr, String), Instant>>,
    rates: Mutex<HashMap<IpAddr, RateWindow>>,
}

#[derive(Clone, Copy)]
struct RateWindow {
    started: Instant,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlError {
    BridgeMissing,
    Unauthorised,
    NonLoopback,
    InvalidRequest,
    StaleTimestamp,
    InvalidNonce,
    ReplayedNonce,
    RateLimited,
    BridgeClosed,
    ConfirmationRequired,
    UndoUnavailable,
}

impl fmt::Display for ControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl ControlError {
    fn status(self) -> StatusCode {
        match self {
            Self::BridgeMissing | Self::BridgeClosed => StatusCode::SERVICE_UNAVAILABLE,
            Self::Unauthorised => StatusCode::UNAUTHORIZED,
            Self::NonLoopback => StatusCode::FORBIDDEN,
            Self::InvalidRequest | Self::ConfirmationRequired => StatusCode::UNPROCESSABLE_ENTITY,
            Self::StaleTimestamp | Self::InvalidNonce => StatusCode::BAD_REQUEST,
            Self::ReplayedNonce | Self::UndoUnavailable => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::BridgeMissing | Self::BridgeClosed => "control_unavailable",
            Self::Unauthorised => "unauthorised",
            Self::NonLoopback => "loopback_only",
            Self::InvalidRequest => "invalid_request",
            Self::StaleTimestamp => "stale_timestamp",
            Self::InvalidNonce => "invalid_nonce",
            Self::ReplayedNonce => "replayed_nonce",
            Self::RateLimited => "rate_limited",
            Self::ConfirmationRequired => "confirmation_required",
            Self::UndoUnavailable => "undo_unavailable",
        }
    }
}

impl IntoResponse for ControlError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.code() });
        (
            self.status(),
            [(header::CACHE_CONTROL, "no-store")],
            Json(body),
        )
            .into_response()
    }
}

impl ControlGuard {
    pub(crate) fn check_peer(
        &self,
        peer: IpAddr,
        remote_enabled: bool,
    ) -> Result<(), ControlError> {
        if peer.is_loopback() || remote_enabled {
            Ok(())
        } else {
            Err(ControlError::NonLoopback)
        }
    }

    pub(crate) fn rate_check(&self, peer: IpAddr) -> Result<(), ControlError> {
        let mut rates = lock(&self.rates);
        let now = Instant::now();
        rates.retain(|_, window| now.duration_since(window.started) < RATE_WINDOW);
        let window = rates.entry(peer).or_insert_with(|| RateWindow {
            started: now,
            count: 0,
        });
        if now.duration_since(window.started) >= RATE_WINDOW {
            *window = RateWindow {
                started: now,
                count: 0,
            };
        }
        window.count += 1;
        if window.count > RATE_LIMIT {
            Err(ControlError::RateLimited)
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_replay(
        &self,
        peer: IpAddr,
        timestamp: u64,
        nonce: &str,
    ) -> Result<(), ControlError> {
        if !(NONCE_MIN_LEN..=NONCE_MAX_LEN).contains(&nonce.len())
            || !nonce
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(ControlError::InvalidNonce);
        }
        let now = unix_seconds()?;
        if now.abs_diff(timestamp) > MAX_TIMESTAMP_SKEW_SECONDS {
            return Err(ControlError::StaleTimestamp);
        }

        let key = (peer, nonce.to_owned());
        let mut nonces = lock(&self.nonces);
        let seen_at = Instant::now();
        nonces.retain(|_, seen| seen_at.duration_since(*seen) < NONCE_TTL);
        if nonces.contains_key(&key) {
            return Err(ControlError::ReplayedNonce);
        }
        nonces.insert(key, seen_at);
        Ok(())
    }

    pub(crate) fn clear(&self) {
        lock(&self.nonces).clear();
        lock(&self.rates).clear();
    }
}

/// Handler for `POST /control`.
pub(crate) async fn control(
    State(shared): State<Arc<SharedState>>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let raw: RawControlRequest = match serde_json::from_slice(&body) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::debug!(%error, "malformed control request");
            return ControlError::InvalidRequest.into_response();
        }
    };
    let request = match ControlRequest::try_from(raw) {
        Ok(request) => request,
        Err(error) => return error.into_response(),
    };

    let result: Result<ControlAction, ControlError> = (|| {
        let bridge = shared.control_bridge().ok_or(ControlError::BridgeMissing)?;
        shared
            .control_guard()
            .check_peer(peer.ip(), bridge.remote_enabled())?;
        shared.control_guard().rate_check(peer.ip())?;
        bridge.authorises(&headers, peer.ip(), &request, &body)?;
        shared
            .control_guard()
            .check_replay(peer.ip(), request.timestamp, &request.nonce)?;
        let action = match request.action {
            ControlAction::Undo(token) => {
                bridge.validate_undo(&token)?;
                ControlAction::Undo(token)
            }
            action if action.is_write() => {
                let token = bridge.issue_undo(&request.nonce, request.timestamp);
                ControlAction::WithUndo {
                    action: Box::new(action),
                    undo: token,
                }
            }
            action => action,
        };
        bridge.forward(action.clone())?;
        Ok(action)
    })();

    match result {
        Ok(action) => {
            let mut body = serde_json::json!({
                "ok": true,
                "action": action.name(),
            });
            if let ControlAction::SetScene(scene) = &action {
                body["scene"] = serde_json::json!(scene);
            }
            if let Some(token) = action.undo_token() {
                body["undo"] = serde_json::json!(token);
            }
            (
                StatusCode::OK,
                [(header::CACHE_CONTROL, "no-store")],
                Json(body),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

fn unix_seconds() -> Result<u64, ControlError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| ControlError::StaleTimestamp)
}

fn digest(token: &str) -> [u8; 32] {
    let value = Sha256::digest(token.as_bytes());
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&value);
    bytes
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut difference = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        difference |= left_byte ^ right_byte;
    }
    difference == 0
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[usize::from(byte >> 4)] as char);
        output.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        let high = hex_nibble(bytes[index * 2])?;
        let low = hex_nibble(bytes[index * 2 + 1])?;
        *byte = (high << 4) | low;
    }
    Some(output)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer() -> IpAddr {
        "127.0.0.1".parse().expect("loopback")
    }

    fn signed_request(token: &str, body: &[u8], timestamp: u64, nonce: &str) -> String {
        let request = ControlRequest {
            action: ControlAction::Hide,
            timestamp,
            nonce: nonce.to_owned(),
        };
        let mut mac = HmacSha256::new_from_slice(token.as_bytes()).expect("key");
        mac.update(&canonical_message(&request, body));
        format!("{HMAC_SCHEME} {}", hex_lower(mac.finalize().into_bytes()))
    }

    #[test]
    fn generated_tokens_are_version_four_uuids() {
        let token = generate_control_token();
        assert_eq!(token.len(), 36);
        assert_eq!(token.as_bytes()[14], b'4');
        assert!(matches!(token.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
    }

    #[test]
    fn bridge_accepts_only_the_configured_token() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, false);
        assert!(bridge.accepts_token("secret"));
        assert!(!bridge.accepts_token("other"));
    }

    #[test]
    fn canonical_message_has_a_trailing_newline_and_body_hash() {
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = ControlRequest {
            action: ControlAction::Hide,
            timestamp: 100,
            nonce: "nonce-abcdef".to_owned(),
        };
        let message = String::from_utf8(canonical_message(&request, body)).expect("utf8");
        assert_eq!(
            message,
            format!(
                "{DOMAIN_SEPARATOR}\n100\nnonce-abcdef\nPOST\n{CONTROL_PATH}\n{}\n",
                hex_lower(Sha256::digest(body))
            )
        );
    }

    #[test]
    fn hmac_authorises_exact_body_when_remote_enabled() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            signed_request("secret", body, request.timestamp, &request.nonce)
                .parse()
                .expect("header"),
        );
        assert!(bridge.authorises(&headers, peer(), &request, body).is_ok());
    }

    #[test]
    fn hmac_rejects_tampered_body_and_wrong_key() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let signed = signed_request("secret", body, request.timestamp, &request.nonce);
        let tampered = br#"{"action":"show","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, signed.parse().expect("header"));
        assert_eq!(
            bridge.authorises(&headers, peer(), &request, tampered),
            Err(ControlError::Unauthorised)
        );

        let (tx, _rx) = mpsc::unbounded_channel();
        let other = ControlBridge::new("other", tx, true);
        assert_eq!(
            other.authorises(&headers, peer(), &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn hmac_is_refused_when_remote_is_disabled() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, false);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            signed_request("secret", body, request.timestamp, &request.nonce)
                .parse()
                .expect("header"),
        );
        assert_eq!(
            bridge.authorises(&headers, peer(), &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn bearer_is_rejected_from_a_remote_peer() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, true);
        let body = br#"{"action":"hide","timestamp":100,"nonce":"nonce-abcdef"}"#;
        let request = serde_json::from_slice::<ControlRequest>(body).expect("request");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer secret".parse().expect("header"),
        );
        let remote = "192.0.2.10".parse().expect("remote");
        assert_eq!(
            bridge.authorises(&headers, remote, &request, body),
            Err(ControlError::Unauthorised)
        );
    }

    #[test]
    fn guard_accepts_loopback_peers() {
        let guard = ControlGuard::default();
        assert!(guard.check_peer(peer(), false).is_ok());
    }

    #[test]
    fn guard_rejects_remote_peers_when_remote_is_disabled() {
        let guard = ControlGuard::default();
        assert_eq!(
            guard.check_peer("192.0.2.10".parse().expect("remote"), false),
            Err(ControlError::NonLoopback)
        );
        assert!(
            guard
                .check_peer("192.0.2.10".parse().expect("remote"), true)
                .is_ok()
        );
    }

    #[test]
    fn guard_rejects_stale_timestamps() {
        let guard = ControlGuard::default();
        let stale = unix_seconds().expect("clock").saturating_sub(120);
        assert_eq!(
            guard.check_replay(peer(), stale, "nonce-12345678"),
            Err(ControlError::StaleTimestamp)
        );
    }

    #[test]
    fn guard_rejects_malformed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert_eq!(
            guard.check_replay(peer(), now, "bad nonce"),
            Err(ControlError::InvalidNonce)
        );
    }

    #[test]
    fn guard_rejects_replayed_nonces() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        assert!(guard.check_replay(peer(), now, "nonce-abcdefgh").is_ok());
        assert_eq!(
            guard.check_replay(peer(), now, "nonce-abcdefgh"),
            Err(ControlError::ReplayedNonce)
        );
    }

    #[test]
    fn guard_scopes_nonces_to_each_peer() {
        let guard = ControlGuard::default();
        let now = unix_seconds().expect("clock");
        let other = "192.0.2.10".parse().expect("remote");
        assert!(guard.check_replay(peer(), now, "nonce-abcdefgh").is_ok());
        assert!(guard.check_replay(other, now, "nonce-abcdefgh").is_ok());
    }

    #[test]
    fn guard_limits_requests_per_peer() {
        let guard = ControlGuard::default();
        for _ in 0..RATE_LIMIT {
            assert!(guard.rate_check(peer()).is_ok());
        }
        assert_eq!(guard.rate_check(peer()), Err(ControlError::RateLimited));
    }

    fn parse(value: serde_json::Value) -> Result<ControlRequest, ControlError> {
        let raw: RawControlRequest =
            serde_json::from_value(value).map_err(|_| ControlError::InvalidRequest)?;
        ControlRequest::try_from(raw)
    }

    fn request(action: &str) -> serde_json::Value {
        serde_json::json!({
            "action": action,
            "timestamp": 100,
            "nonce": "nonce-abcdefgh",
        })
    }

    #[test]
    fn score_adjust_parses_a_one_point_delta() {
        let mut body = request("score_adjust");
        body["team"] = serde_json::json!("b");
        body["delta"] = serde_json::json!(-1);
        let request = parse(body).expect("score adjust");
        assert_eq!(
            request.action,
            ControlAction::ScoreAdjust {
                team: ControlTeamSide::B,
                delta: -1,
            }
        );
        assert!(request.action.is_write());
        assert!(!request.action.requires_confirmation());
    }

    #[test]
    fn destructive_edits_require_confirmation() {
        let mut body = request("set_map_pick");
        body["index"] = serde_json::json!(2);
        body["team"] = serde_json::json!("none");
        assert_eq!(parse(body.clone()), Err(ControlError::ConfirmationRequired));
        body["confirm"] = serde_json::json!(false);
        assert_eq!(parse(body.clone()), Err(ControlError::ConfirmationRequired));
        body["confirm"] = serde_json::json!(true);
        assert_eq!(
            parse(body).expect("confirmed pick").action,
            ControlAction::SetMapPick {
                index: 2,
                team: ControlOptionalTeam::None,
            }
        );
    }

    #[test]
    fn match_writes_reject_malformed_fields() {
        let mut bad_index = request("set_current_map");
        bad_index["index"] = serde_json::json!(MAX_INDEX + 1);
        assert_eq!(parse(bad_index), Err(ControlError::InvalidRequest));

        let mut bad_delta = request("score_adjust");
        bad_delta["team"] = serde_json::json!("a");
        bad_delta["delta"] = serde_json::json!(2);
        assert_eq!(parse(bad_delta), Err(ControlError::InvalidRequest));

        let mut bad_score = request("set_manual_score");
        bad_score["team"] = serde_json::json!("a");
        bad_score["score"] = serde_json::json!(MAX_SCORE + 1);
        assert_eq!(parse(bad_score), Err(ControlError::InvalidRequest));

        let mut bad_id = request("load_saved_match");
        bad_id["id"] = serde_json::json!("bad id!");
        bad_id["confirm"] = serde_json::json!(true);
        assert_eq!(parse(bad_id), Err(ControlError::InvalidRequest));

        let mut unknown = request("show");
        unknown["surprise"] = serde_json::json!(1);
        assert_eq!(parse(unknown), Err(ControlError::InvalidRequest));
    }

    #[test]
    fn undo_tokens_are_nonce_derived_and_single_use() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = ControlBridge::new("secret", tx, false);
        let token = bridge.issue_undo("nonce-abcdefgh", 100);
        assert!(valid_undo_token(&token));
        assert_eq!(token, bridge.issue_undo("nonce-abcdefgh", 100));
        assert_ne!(token, bridge.issue_undo("other-nonce-01", 100));
        assert!(bridge.validate_undo(&token).is_ok());
        assert_eq!(
            bridge.validate_undo(&token),
            Err(ControlError::UndoUnavailable)
        );
    }
}
