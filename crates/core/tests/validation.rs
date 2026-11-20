mod support;

use std::collections::HashSet;

use fgc_core::{AppState, CoreError, RepairKind, Scoring, TeamId, migrate, validate_and_repair};
use serde_json::json;

#[test]
fn duplicate_ids_are_regenerated() {
    let mut state = AppState {
        teams: vec![support::team("dup", "Alpha"), support::team("dup", "Bravo")],
        talents: vec![
            support::talent("talent-dup", "One"),
            support::talent("talent-dup", "Two"),
        ],
        assets: vec![
            support::asset("asset-dup", support::HASH_A, "png"),
            support::asset("asset-dup", support::HASH_B, "png"),
        ],
        tournaments: vec![
            support::tournament("tour-dup"),
            support::tournament("tour-dup"),
        ],
        ..AppState::default()
    };
    state.teams[0]
        .players
        .push(support::player("player-dup", "Ace"));
    state.teams[1]
        .players
        .push(support::player("player-dup", "Blaze"));

    let first = &mut state.tournaments[0];
    first.groups = vec![support::group("group-dup"), support::group("group-dup")];
    first.winners[0].matches = vec![
        support::bracket_match("bracket-dup"),
        support::bracket_match("bracket-dup"),
    ];
    first.group_matches = vec![
        support::group_match("group-match-dup"),
        support::group_match("group-match-dup"),
    ];
    let group_id = first.groups[0].id.clone();
    for fixture in &mut first.group_matches {
        fixture.group = group_id.clone();
    }

    let repairs = validate_and_repair(&mut state);
    assert_eq!(
        repairs
            .iter()
            .filter(|repair| repair.kind == RepairKind::DuplicateId)
            .count(),
        8
    );

    assert_eq!(state.teams[0].id.as_str(), "dup");
    assert_ne!(state.teams[1].id.as_str(), "dup");

    let player_ids: HashSet<_> = state
        .teams
        .iter()
        .flat_map(|team| team.players.iter())
        .map(|player| player.id.as_str())
        .collect();
    assert_eq!(player_ids.len(), 2);

    let talent_ids: HashSet<_> = state
        .talents
        .iter()
        .map(|talent| talent.id.as_str())
        .collect();
    assert_eq!(talent_ids.len(), 2);

    let asset_ids: HashSet<_> = state.assets.iter().map(|asset| asset.id.as_str()).collect();
    assert_eq!(asset_ids.len(), 2);

    let tournament_ids: HashSet<_> = state
        .tournaments
        .iter()
        .map(|tournament| tournament.id.as_str())
        .collect();
    assert_eq!(tournament_ids.len(), 2);

    let group_ids: HashSet<_> = state.tournaments[0]
        .groups
        .iter()
        .map(|group| group.id.as_str())
        .collect();
    assert_eq!(group_ids.len(), 2);

    let bracket_ids: HashSet<_> = state.tournaments[0].winners[0]
        .matches
        .iter()
        .map(|game| game.id.as_str())
        .collect();
    assert_eq!(bracket_ids.len(), 2);

    let group_match_ids: HashSet<_> = state.tournaments[0]
        .group_matches
        .iter()
        .map(|game| game.id.as_str())
        .collect();
    assert_eq!(group_match_ids.len(), 2);
}

#[test]
fn same_team_on_both_sides_is_cleared() {
    let mut state = AppState::default();
    let mut current = support::match_state("match");
    current.team_a = Some(TeamId::from_raw("same-team"));
    current.team_b = Some(TeamId::from_raw("same-team"));
    state.current_match = Some(current);

    let repairs = validate_and_repair(&mut state);

    assert_eq!(
        state
            .current_match
            .as_ref()
            .expect("current match should remain")
            .team_b,
        None
    );
    assert!(
        repairs
            .iter()
            .any(|repair| repair.kind == RepairKind::TeamOnBothSides)
    );
}

#[test]
fn current_map_out_of_range_is_cleared() {
    let mut state = AppState::default();
    let mut current = support::match_state("match");
    current.current_map = Some(1);
    state.current_match = Some(current);

    let repairs = validate_and_repair(&mut state);
    assert_eq!(
        state
            .current_match
            .as_ref()
            .expect("current match should remain")
            .current_map,
        None
    );
    assert!(
        repairs
            .iter()
            .any(|repair| repair.kind == RepairKind::CurrentMapOutOfRange)
    );

    let mut state = AppState::default();
    let mut current = support::match_state("match");
    current.current_map = Some(0);
    state.current_match = Some(current);
    let repairs = validate_and_repair(&mut state);
    assert_eq!(
        state
            .current_match
            .as_ref()
            .expect("current match should remain")
            .current_map,
        Some(0)
    );
    assert!(repairs.is_empty());
}

#[test]
fn first_to_is_clamped() {
    for (input, expected) in [(0, 3), (5, 3), (255, 3), (1, 1), (4, 4)] {
        let mut state = AppState::default();
        let mut current = support::match_state("match");
        current.first_to = input;
        state.current_match = Some(current);

        let repairs = validate_and_repair(&mut state);
        assert_eq!(
            state
                .current_match
                .as_ref()
                .expect("current match should remain")
                .first_to,
            expected
        );
        if input == expected {
            assert!(repairs.is_empty());
        } else {
            assert!(
                repairs
                    .iter()
                    .any(|repair| repair.kind == RepairKind::FirstToClamped)
            );
        }
    }
}

#[test]
fn negative_scores_are_clamped_when_parsed() {
    let state = migrate(json!({
        "schemaVersion": 2,
        "currentMatch": {
            "id": "match",
            "maps": [{
                "name": "Ilios",
                "gameModeName": "Control",
                "scoreA": -5,
                "scoreB": -1
            }],
            "scoring": {"mode": "manual", "a": -2, "b": -3}
        }
    }))
    .expect("negative scores should be repaired while parsing");

    let current = state.current_match.expect("current match should exist");
    assert_eq!(current.maps[0].score_a, 0);
    assert_eq!(current.maps[0].score_b, 0);
    assert_eq!(current.scoring, Scoring::Manual { a: 0, b: 0 });
}

#[test]
fn unknown_map_pool_names_are_kept() {
    let mut state = AppState::default();
    let mut current = support::match_state("match");
    current.map_pool = vec!["Unknown Map".to_owned()];
    state.current_match = Some(current);

    let repairs = validate_and_repair(&mut state);

    assert_eq!(
        state
            .current_match
            .as_ref()
            .expect("current match should remain")
            .map_pool,
        vec!["Unknown Map".to_owned()]
    );
    assert!(repairs.is_empty());
}

#[test]
fn invalid_assets_are_dropped() {
    let mut state = AppState {
        assets: vec![
            support::asset("ok", support::HASH_A, "png"),
            support::asset("bad-hash", "not-a-hash", "png"),
            support::asset("uppercase-hash", &support::HASH_A.to_uppercase(), "png"),
            support::asset("bad-extension", support::HASH_B, "exe"),
            support::asset("uppercase-extension", support::HASH_B, "PNG"),
        ],
        ..AppState::default()
    };

    let repairs = validate_and_repair(&mut state);

    assert_eq!(state.assets.len(), 1);
    assert_eq!(state.assets[0].id.as_str(), "ok");
    assert!(
        repairs
            .iter()
            .any(|repair| repair.kind == RepairKind::AssetHashRejected)
    );
    assert!(
        repairs
            .iter()
            .any(|repair| repair.kind == RepairKind::AssetExtensionRejected)
    );
}

#[test]
fn assets_sharing_a_hash_are_allowed() {
    let mut state = AppState {
        assets: vec![
            support::asset("first", support::HASH_A, "png"),
            support::asset("second", support::HASH_A, "png"),
        ],
        ..AppState::default()
    };

    let repairs = validate_and_repair(&mut state);

    assert_eq!(state.assets.len(), 2);
    assert!(repairs.is_empty());
}

#[test]
fn tournament_first_to_is_clamped() {
    let mut state = AppState::default();
    let mut tournament = support::tournament("tournament");
    tournament.default_first_to = 0;
    tournament.winners[0].matches[0].first_to = 5;
    tournament.group_matches[0].first_to = 255;
    state.tournaments.push(tournament);

    let repairs = validate_and_repair(&mut state);

    assert_eq!(state.tournaments[0].default_first_to, 3);
    assert_eq!(state.tournaments[0].winners[0].matches[0].first_to, 3);
    assert_eq!(state.tournaments[0].group_matches[0].first_to, 3);
    assert_eq!(
        repairs
            .iter()
            .filter(|repair| repair.kind == RepairKind::FirstToClamped)
            .count(),
        3
    );
}

#[test]
fn valid_state_is_unchanged() {
    let mut state = support::sample_state();
    let expected = state.clone();

    let repairs = validate_and_repair(&mut state);

    assert!(repairs.is_empty());
    assert_eq!(state, expected);
}

#[test]
fn invalid_document_reports_core_error() {
    let error = migrate(json!({"schemaVersion": 2, "teams": "not a list"})).unwrap_err();
    assert!(matches!(error, CoreError::InvalidField { .. }));
}
