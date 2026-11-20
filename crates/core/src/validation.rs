use std::collections::HashSet;

use crate::{
    AppState, AssetId, GroupId, MatchId, PlayerId, RemoteAccessMode, TalentId, TeamId,
    TournamentId, TournamentRepairKind, is_allowed_extension, is_valid_hash, repair_tournament,
    trim_trailing_empty_maps_for_first_to,
};

/// Longest team name the overlays can render without truncation.
pub const MAX_TEAM_NAME_CHARS: usize = 32;
/// Longest player name the overlays can render without truncation.
pub const MAX_PLAYER_NAME_CHARS: usize = 24;
/// Longest talent name the overlays can render without truncation.
pub const MAX_TALENT_NAME_CHARS: usize = 24;

/// The kind of repair applied while loading user data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairKind {
    /// A duplicate identifier was regenerated.
    DuplicateId,
    /// `teamA` and `teamB` referred to the same team.
    TeamOnBothSides,
    /// The current map index no longer exists.
    CurrentMapOutOfRange,
    /// A first-to value outside 1..=4 was reset.
    FirstToClamped,
    /// Trailing empty map slots beyond the first-to format were removed.
    MapCountNormalized,
    /// An asset with a malformed hash was removed from the library.
    AssetHashRejected,
    /// An asset with an unsupported extension was removed from the library.
    AssetExtensionRejected,
    /// A tournament entrant had a duplicate or missing seed.
    TournamentDuplicateSeed,
    /// A completed bracket match had no valid winner and was reset.
    BracketCompletedWithoutWinner,
    /// A bracket link pointed at a missing match, itself or an invalid slot.
    BracketLinkDangling,
    /// Bracket links formed a cycle and were broken.
    BracketLinkCycle,
    /// A group fixture referenced a team outside its group.
    GroupTeamOutsideGroup,
    /// The tournament champion was not an entrant.
    ChampionNotEntrant,
    /// A team, player or talent name exceeded the display limit.
    NameTooLong,
}

/// A single repair applied to loaded state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repair {
    /// Repair category.
    pub kind: RepairKind,
    /// Human-readable detail.
    pub message: String,
}

impl Repair {
    fn new(kind: RepairKind, message: String) -> Self {
        Self { kind, message }
    }
}

/// Applies every rule in the data-model repair table.
///
/// Returns the repairs that were applied so the caller can surface them. Each
/// repair is also logged at `warn` level.
pub fn validate_and_repair(state: &mut AppState) -> Vec<Repair> {
    let mut repairs = Vec::new();

    // Older saves have no scope field. A LAN bind means the owner already chose
    // the LAN scope, so keep the new selector consistent without a repair toast.
    if state.settings.overlay.remote_access == RemoteAccessMode::Off
        && state.settings.overlay.allow_lan
    {
        state.settings.overlay.remote_access = RemoteAccessMode::Lan;
    }

    dedupe_state_ids(state, &mut repairs);
    clamp_names(state, &mut repairs);

    if let Some(current) = state.current_match.as_mut() {
        if current.team_a.is_some() && current.team_a == current.team_b {
            current.team_b = None;
            repairs.push(Repair::new(
                RepairKind::TeamOnBothSides,
                "team B was the same as team A and was cleared".to_owned(),
            ));
        }

        if let Some(index) = current.current_map
            && index >= current.maps.len()
        {
            current.current_map = None;
            repairs.push(Repair::new(
                RepairKind::CurrentMapOutOfRange,
                format!("currentMap {index} is outside the map list and was cleared"),
            ));
        }

        clamp_first_to(&mut current.first_to, "match.firstTo", &mut repairs);

        let before = current.maps.len();
        trim_trailing_empty_maps_for_first_to(current);
        if current.maps.len() != before {
            repairs.push(Repair::new(
                RepairKind::MapCountNormalized,
                format!(
                    "match map slots were resized from {before} to {} for first-to {}",
                    current.maps.len(),
                    current.first_to
                ),
            ));
        }
    }

    state.assets.retain(|asset| {
        if !is_valid_hash(&asset.hash) {
            repairs.push(Repair::new(
                RepairKind::AssetHashRejected,
                format!(
                    "asset {} was dropped because its hash is not 64 lowercase hexadecimal characters",
                    asset.id
                ),
            ));
            return false;
        }
        if !is_allowed_extension(&asset.ext) {
            repairs.push(Repair::new(
                RepairKind::AssetExtensionRejected,
                format!(
                    "asset {} was dropped because extension {:?} is not allowed",
                    asset.id, asset.ext
                ),
            ));
            return false;
        }
        true
    });

    for tournament in &mut state.tournaments {
        clamp_first_to(
            &mut tournament.default_first_to,
            "tournament.defaultFirstTo",
            &mut repairs,
        );

        for round in tournament
            .winners
            .iter_mut()
            .chain(tournament.losers.iter_mut())
            .chain(tournament.grand_final.iter_mut())
        {
            for game in &mut round.matches {
                clamp_first_to(&mut game.first_to, "bracketMatch.firstTo", &mut repairs);
            }
        }

        for game in &mut tournament.group_matches {
            clamp_first_to(&mut game.first_to, "groupMatch.firstTo", &mut repairs);
        }

        for repair in repair_tournament(tournament) {
            let kind = match repair.kind {
                TournamentRepairKind::DuplicateSeed => RepairKind::TournamentDuplicateSeed,
                TournamentRepairKind::CompletedWithoutWinner => {
                    RepairKind::BracketCompletedWithoutWinner
                }
                TournamentRepairKind::DanglingLink => RepairKind::BracketLinkDangling,
                TournamentRepairKind::LinkCycle => RepairKind::BracketLinkCycle,
                TournamentRepairKind::GroupTeamOutsideGroup => RepairKind::GroupTeamOutsideGroup,
                TournamentRepairKind::ChampionNotEntrant => RepairKind::ChampionNotEntrant,
            };
            repairs.push(Repair::new(kind, repair.message));
        }
    }

    for repair in &repairs {
        tracing::warn!(
            kind = ?repair.kind,
            message = %repair.message,
            "repaired state"
        );
    }

    repairs
}

fn clamp_names(state: &mut AppState, repairs: &mut Vec<Repair>) {
    for team in &mut state.teams {
        clamp_name(
            &mut team.name,
            MAX_TEAM_NAME_CHARS,
            "team",
            team.id.as_str(),
            repairs,
        );
        for player in &mut team.players {
            clamp_name(
                &mut player.name,
                MAX_PLAYER_NAME_CHARS,
                "player",
                player.id.as_str(),
                repairs,
            );
        }
    }

    for talent in &mut state.talents {
        clamp_name(
            &mut talent.display_name,
            MAX_TALENT_NAME_CHARS,
            "talent",
            talent.id.as_str(),
            repairs,
        );
    }
}

fn clamp_name(value: &mut String, max: usize, label: &str, id: &str, repairs: &mut Vec<Repair>) {
    if value.chars().count() <= max {
        return;
    }

    let truncated = value.chars().take(max).collect::<String>();
    repairs.push(Repair::new(
        RepairKind::NameTooLong,
        format!("{label} {id} name was longer than {max} characters and was truncated"),
    ));
    *value = truncated.trim_end().to_owned();
}

fn clamp_first_to(value: &mut u8, field: &str, repairs: &mut Vec<Repair>) {
    if !matches!(*value, 1..=4) {
        let previous = *value;
        *value = 3;
        repairs.push(Repair::new(
            RepairKind::FirstToClamped,
            format!("{field} {previous} was outside 1..=4 and was reset to 3"),
        ));
    }
}

fn dedupe_state_ids(state: &mut AppState, repairs: &mut Vec<Repair>) {
    let mut seen = HashSet::new();
    dedupe_ids(
        &mut state.teams,
        &mut seen,
        |team| team.id.as_str().to_owned(),
        |team| team.id = TeamId::new(),
        "team",
        repairs,
    );

    let mut player_seen = HashSet::new();
    for team in &mut state.teams {
        dedupe_ids(
            &mut team.players,
            &mut player_seen,
            |player| player.id.as_str().to_owned(),
            |player| player.id = PlayerId::new(),
            "player",
            repairs,
        );
    }

    let mut seen = HashSet::new();
    dedupe_ids(
        &mut state.talents,
        &mut seen,
        |talent| talent.id.as_str().to_owned(),
        |talent| talent.id = TalentId::new(),
        "talent",
        repairs,
    );

    let mut seen = HashSet::new();
    dedupe_ids(
        &mut state.tournaments,
        &mut seen,
        |tournament| tournament.id.as_str().to_owned(),
        |tournament| tournament.id = TournamentId::new(),
        "tournament",
        repairs,
    );

    let mut seen = HashSet::new();
    dedupe_ids(
        &mut state.assets,
        &mut seen,
        |asset| asset.id.as_str().to_owned(),
        |asset| asset.id = AssetId::new(),
        "asset",
        repairs,
    );

    for tournament in &mut state.tournaments {
        let mut group_seen = HashSet::new();
        dedupe_ids(
            &mut tournament.groups,
            &mut group_seen,
            |group| group.id.as_str().to_owned(),
            |group| group.id = GroupId::new(),
            "group",
            repairs,
        );

        let mut match_seen = HashSet::new();
        for round in tournament
            .winners
            .iter_mut()
            .chain(tournament.losers.iter_mut())
            .chain(tournament.grand_final.iter_mut())
        {
            dedupe_ids(
                &mut round.matches,
                &mut match_seen,
                |game| game.id.as_str().to_owned(),
                |game| game.id = MatchId::new(),
                "bracket match",
                repairs,
            );
        }

        dedupe_ids(
            &mut tournament.group_matches,
            &mut match_seen,
            |game| game.id.as_str().to_owned(),
            |game| game.id = MatchId::new(),
            "group match",
            repairs,
        );
    }
}

fn dedupe_ids<T>(
    items: &mut [T],
    seen: &mut HashSet<String>,
    mut id_of: impl FnMut(&T) -> String,
    mut regenerate: impl FnMut(&mut T),
    label: &str,
    repairs: &mut Vec<Repair>,
) {
    for item in items {
        let original = id_of(item);
        if seen.insert(original.clone()) {
            continue;
        }

        loop {
            regenerate(item);
            if seen.insert(id_of(item)) {
                break;
            }
        }

        repairs.push(Repair::new(
            RepairKind::DuplicateId,
            format!("regenerated duplicate {label} id {original}"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{empty_hero_bans, empty_map_slot, insert_map, new_match};
    use serde_json::json;

    #[test]
    fn legacy_lan_bind_selects_the_lan_scope() {
        let mut state = AppState::default();
        state.settings.overlay.allow_lan = true;

        validate_and_repair(&mut state);

        assert_eq!(state.settings.overlay.remote_access, RemoteAccessMode::Lan);
    }

    #[test]
    fn trailing_empty_maps_are_normalized_to_first_to_on_load() {
        let mut current = new_match();
        current.first_to = 1;
        current.maps = (0..9).map(|_| empty_map_slot()).collect();
        current.maps[0].name = "Ilios".to_owned();
        current.maps[0].game_mode_name = "Control".to_owned();
        // A derived mode on an unnamed slot is not user data.
        current.maps[1].game_mode_name = "Escort".to_owned();
        current.hero_bans = (0..9).map(|_| empty_hero_bans()).collect();

        let mut state = AppState {
            current_match: Some(current),
            ..AppState::default()
        };
        let repairs = validate_and_repair(&mut state);
        let current = state.current_match.expect("current match");

        assert_eq!(current.maps.len(), 1);
        assert_eq!(current.hero_bans.len(), 1);
        assert_eq!(current.maps[0].name, "Ilios");
        assert!(
            repairs
                .iter()
                .any(|repair| repair.kind == RepairKind::MapCountNormalized)
        );
    }

    #[test]
    fn load_normalization_keeps_entered_tiebreakers_and_manual_modes() {
        let mut current = new_match();
        current.first_to = 1;
        current.maps = (0..9).map(|_| empty_map_slot()).collect();
        current.maps[0].name = "Ilios".to_owned();
        current.maps[1].name = "Tiebreaker".to_owned();
        current.maps[2].game_mode_name = "Push".to_owned();
        current.maps[2].game_mode_manual = true;
        current.hero_bans = (0..9).map(|_| empty_hero_bans()).collect();

        let mut state = AppState {
            current_match: Some(current),
            ..AppState::default()
        };
        validate_and_repair(&mut state);
        let current = state.current_match.expect("current match");

        assert_eq!(current.maps.len(), 3);
        assert_eq!(current.hero_bans.len(), 3);
        assert_eq!(current.maps[1].name, "Tiebreaker");
        assert!(current.maps[2].game_mode_manual);
    }

    #[test]
    fn user_added_empty_map_survives_validation() {
        let mut current = new_match();
        current.first_to = 1;
        current.maps.truncate(1);
        current.hero_bans.truncate(1);
        insert_map(&mut current, 1);
        assert!(current.maps[1].user_added);

        let mut state = AppState {
            current_match: Some(current),
            ..AppState::default()
        };
        let repairs = validate_and_repair(&mut state);
        let current = state.current_match.expect("current match");

        assert_eq!(current.maps.len(), 2);
        assert_eq!(current.hero_bans.len(), 2);
        assert!(current.maps[1].user_added);
        assert!(
            repairs
                .iter()
                .all(|repair| repair.kind != RepairKind::MapCountNormalized)
        );
    }

    #[test]
    fn overlong_names_are_truncated_to_the_display_limits() {
        let team_name = "T".repeat(MAX_TEAM_NAME_CHARS + 5);
        let player_name = "P".repeat(MAX_PLAYER_NAME_CHARS + 5);
        let talent_name = "C".repeat(MAX_TALENT_NAME_CHARS + 5);

        let mut state: AppState = serde_json::from_value(json!({
            "teams": [{
                "id": "team-1",
                "name": team_name,
                "players": [{ "id": "player-1", "name": player_name }]
            }],
            "talents": [{ "id": "talent-1", "displayName": talent_name }]
        }))
        .expect("valid fixture");

        let repairs = validate_and_repair(&mut state);

        assert_eq!(state.teams[0].name.chars().count(), MAX_TEAM_NAME_CHARS);
        assert_eq!(
            state.teams[0].players[0].name.chars().count(),
            MAX_PLAYER_NAME_CHARS
        );
        assert_eq!(
            state.talents[0].display_name.chars().count(),
            MAX_TALENT_NAME_CHARS
        );
        assert_eq!(
            repairs
                .iter()
                .filter(|repair| repair.kind == RepairKind::NameTooLong)
                .count(),
            3
        );
    }

    #[test]
    fn duplicate_player_and_talent_ids_are_regenerated() {
        let mut state: AppState = serde_json::from_value(json!({
            "teams": [{
                "id": "team-1",
                "players": [
                    { "id": "player-1", "name": "First" },
                    { "id": "player-1", "name": "Second" }
                ]
            }],
            "talents": [
                { "id": "talent-1", "displayName": "First" },
                { "id": "talent-1", "displayName": "Second" }
            ]
        }))
        .expect("valid fixture");

        let repairs = validate_and_repair(&mut state);

        assert_ne!(state.teams[0].players[0].id, state.teams[0].players[1].id);
        assert_ne!(state.talents[0].id, state.talents[1].id);
        assert_eq!(
            repairs
                .iter()
                .filter(|repair| repair.kind == RepairKind::DuplicateId)
                .count(),
            2
        );
    }
}
