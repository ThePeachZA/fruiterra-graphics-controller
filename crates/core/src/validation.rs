use std::collections::HashSet;

use crate::{
    AppState, AssetId, GroupId, MatchId, PlayerId, TalentId, TeamId, TournamentId,
    is_allowed_extension, is_valid_hash,
};

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
    /// An asset with a malformed hash was removed from the library.
    AssetHashRejected,
    /// An asset with an unsupported extension was removed from the library.
    AssetExtensionRejected,
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

    dedupe_state_ids(state, &mut repairs);

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
    use serde_json::json;

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
