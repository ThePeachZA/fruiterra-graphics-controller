mod support;

use fgc_core::{AppState, CoreError, SCHEMA_VERSION, TalentType, migrate};
use serde_json::json;

#[test]
fn empty_document_migrates_to_default_state() {
    let state = migrate(json!({})).expect("empty document should load");
    assert_eq!(state.schema_version, SCHEMA_VERSION);
    assert_eq!(state, AppState::default());
}

#[test]
fn schema_v0_is_upgraded() {
    let state = migrate(json!({"schemaVersion": 0, "teams": []})).expect("v0 should migrate");
    assert_eq!(state.schema_version, SCHEMA_VERSION);
    assert!(state.teams.is_empty());
}

#[test]
fn newer_schema_is_refused() {
    let error = migrate(json!({"schemaVersion": SCHEMA_VERSION + 1})).unwrap_err();
    assert!(matches!(
        error,
        CoreError::UnsupportedSchema {
            found,
            supported
        } if found == SCHEMA_VERSION + 1 && supported == SCHEMA_VERSION
    ));
}

#[test]
fn invalid_schema_version_is_refused() {
    for value in [json!({"schemaVersion": -1}), json!({"schemaVersion": "1"})] {
        let error = migrate(value).unwrap_err();
        assert!(matches!(error, CoreError::InvalidSchemaVersion));
    }
}

#[test]
fn unknown_enum_reports_field_path() {
    let error = migrate(json!({
        "schemaVersion": 2,
        "teams": [{
            "id": "team",
            "name": "Team",
            "players": [{
                "id": "player",
                "name": "Player",
                "role": "wizard"
            }]
        }]
    }))
    .unwrap_err();

    let message = error.to_string();
    assert!(
        message.contains("teams[0].players[0].role"),
        "expected a field path, got: {message}"
    );
    assert!(
        message.contains("unknown variant"),
        "expected an unknown variant error, got: {message}"
    );
}

#[test]
fn top_level_unknown_fields_are_preserved() {
    let state = migrate(json!({
        "schemaVersion": 2,
        "futureField": {"enabled": true}
    }))
    .expect("unknown top-level fields should not fail");

    assert_eq!(
        state.extra.get("futureField"),
        Some(&json!({"enabled": true}))
    );

    let value = serde_json::to_value(&state).expect("state should serialise");
    assert_eq!(
        value.get("futureField"),
        Some(&json!({"enabled": true})),
        "unknown top-level fields should survive a round-trip"
    );
}

#[test]
fn nested_unknown_fields_are_ignored() {
    let state = migrate(json!({
        "schemaVersion": 2,
        "teams": [{"id": "team", "name": "Team", "futureField": 1}]
    }))
    .expect("unknown nested fields should not fail");

    assert_eq!(state.teams.len(), 1);
    assert_eq!(state.teams[0].name, "Team");
}

#[test]
fn sample_state_round_trips() {
    let state = support::sample_state();
    let value = serde_json::to_value(&state).expect("state should serialise");
    let loaded = migrate(value).expect("serialised state should load");

    assert_eq!(loaded, state);
}

#[test]
fn schema_v1_moves_talent_types_into_assignment_roles() {
    let state = support::sample_state();
    let mut raw = serde_json::to_value(&state).expect("state should serialise");
    raw["schemaVersion"] = json!(1);
    raw["talents"][0]["type"] = json!("panelist");

    let talent = raw["currentMatch"]["talent"]
        .as_object_mut()
        .expect("talent map");
    talent.insert("caster1".to_owned(), json!({"talentId": "talent-a"}));
    talent.insert("caster2".to_owned(), json!({"talentId": "unknown"}));
    talent.insert("panel1".to_owned(), json!({"talentId": "talent-a"}));

    let loaded = migrate(raw).expect("v1 state should migrate");
    let assignments = &loaded.current_match.expect("fixture match").talent;

    assert_eq!(
        assignments.caster1.as_ref().expect("caster one").role,
        TalentType::Analyst,
        "legacy per-person type should win over the slot default"
    );
    assert_eq!(
        assignments.caster2.as_ref().expect("caster two").role,
        TalentType::Caster,
        "unclassified talents should inherit the slot default"
    );
    assert_eq!(
        assignments.panel1.as_ref().expect("panel one").role,
        TalentType::Analyst,
        "legacy panelist should normalise to analyst"
    );
}

#[test]
fn tournament_round_trips_without_standings() {
    let mut state = support::sample_state();
    let tournament = &mut state.tournaments[0];
    tournament.standings.insert(
        fgc_core::GroupId::from_raw("group-a"),
        vec![fgc_core::Standing {
            team_id: fgc_core::TeamId::from_raw("team-a"),
            played: 1,
            wins: 1,
            losses: 0,
            draws: 0,
            points_for: 3,
            points_against: 1,
            map_differential: 2,
            points: 1,
        }],
    );

    let value = serde_json::to_value(&state).expect("state should serialise");
    assert!(
        value["tournaments"][0].get("standings").is_none(),
        "standings must not be persisted"
    );

    let loaded = migrate(value).expect("serialised state should load");
    let standings = &loaded.tournaments[0].standings[&fgc_core::GroupId::from_raw("group-a")];
    assert_eq!(
        standings.len(),
        2,
        "standings must be recomputed from the group"
    );
    assert!(
        standings
            .iter()
            .all(|row| row.played == 0 && row.points_for == 0),
        "standings must be recomputed, not restored"
    );
}
