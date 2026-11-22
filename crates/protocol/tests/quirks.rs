//! Quirk tests for behaviours the overlays depend on.
//!
//! Each test names the original behaviour it locks in. Any change here is a
//! wire-breaking change and must be documented in `docs/protocol.md`.

mod support;

use fgc_core::{
    Asset, AssetCategory, AssetId, AssetOrigin, Match, Scoring, Side, TeamId, TeamSide,
};
use fgc_protocol::{
    AssetCatalog, AssetUrlResolver, DataUrlResolver, Projector, SCOREBOARD_TYPES, scores,
};
use serde_json::Value;

fn full_parts() -> (
    fgc_core::AppState,
    AssetCatalog,
    fgc_protocol::ServedAssetResolver,
) {
    (
        support::fixture_state(),
        support::fixture_catalog(),
        support::resolver(),
    )
}

fn scoreboard_value(projector: &Projector<'_>, match_state: &Match) -> Value {
    serde_json::to_value(projector.scoreboard(match_state)).expect("scoreboard JSON")
}

#[test]
fn five_identical_scoreboard_payloads() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let messages = projector.scoreboard_messages(match_state);
    assert_eq!(messages.len(), SCOREBOARD_TYPES.len());
    assert_eq!(
        messages
            .iter()
            .map(|message| message.kind.as_str())
            .collect::<Vec<_>>(),
        SCOREBOARD_TYPES
    );
    for message in &messages[1..] {
        assert_eq!(message.data, messages[0].data);
    }
}

#[test]
fn casters_data_wrapper_and_order() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = serde_json::to_value(projector.casters_message(match_state)).expect("casters");
    let casters = value["data"]["castersData"]
        .as_array()
        .expect("castersData array");

    assert_eq!(casters.len(), 2);
    assert_eq!(casters[0]["displayName"], "Caster One");
    assert_eq!(casters[1]["displayName"], "Caster Two");
    assert!(value["data"].get("teamA").is_none());
}

#[test]
fn casters_skip_slots_with_no_matching_talent() {
    let (mut state, catalog, resolver) = full_parts();
    state
        .current_match
        .as_mut()
        .expect("fixture match")
        .talent
        .caster1 = Some(fgc_core::TalentRef {
        talent_id: fgc_core::TalentId::from_raw("missing-talent"),
        role: fgc_core::TalentType::Caster,
    });
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let casters = projector.casters(match_state);
    assert_eq!(casters.casters_data.len(), 1);
    assert_eq!(casters.casters_data[0].display_name, "Caster Two");
}

#[test]
fn role_icons_have_lowercase_and_original_case_keys() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["roleIcons"].as_object().expect("roleIcons object");

    for role in ["Tank", "Damage", "Support"] {
        let original = icons.get(role).expect("original-case key");
        let lower = icons.get(&role.to_lowercase()).expect("lower-case key");
        assert_eq!(original, lower);
    }
}

#[test]
fn game_mode_icons_are_lowercase() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["gameModeIcons"].as_object().expect("gameModeIcons");

    assert!(icons.contains_key("control"));
    assert!(!icons.contains_key("Control"));
}

#[test]
fn hero_icon_and_portrait_keys_are_case_sensitive() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["heroIcons"].as_object().expect("heroIcons");
    let portraits = value["heroPortraits"].as_object().expect("heroPortraits");

    assert!(icons.contains_key("Ana"));
    assert!(!icons.contains_key("ana"));
    assert!(portraits.contains_key("Ana"));
    assert!(!portraits.contains_key("ana"));
}

#[test]
fn disabling_hero_bans_removes_icons_and_bans_from_every_scoreboard() {
    let (mut state, catalog, resolver) = full_parts();
    state.settings.display.show_hero_bans = false;
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    for message in projector.scoreboard_messages(match_state) {
        let value = serde_json::to_value(&message.data).expect("scoreboard JSON");
        assert_eq!(
            value["heroIcons"]
                .as_object()
                .map_or(0, |icons| icons.len()),
            0
        );
        assert_eq!(value["heroBans"].as_array().map_or(0, Vec::len), 0);
        assert!(
            !value["heroPortraits"]
                .as_object()
                .expect("heroPortraits")
                .is_empty()
        );
    }
}

#[test]
fn linked_icons_are_not_listed_as_heroes() {
    let (mut state, catalog, resolver) = full_parts();
    // Ban the linked icon as well, so the portrait path is exercised too.
    state
        .current_match
        .as_mut()
        .expect("fixture match")
        .hero_bans[0]
        .team_b
        .push("Ana Icon".to_owned());
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["heroIcons"].as_object().expect("heroIcons");
    let portraits = value["heroPortraits"].as_object().expect("heroPortraits");

    // The fixture links "Ana Icon" as Ana's icon, so it must not appear as a
    // hero in its own right.
    assert!(icons.contains_key("Ana"));
    assert!(!icons.contains_key("Ana Icon"));
    assert!(portraits.contains_key("Ana"));
    assert!(!portraits.contains_key("Ana Icon"));
}

#[test]
fn duplicate_icon_files_do_not_become_hero_portraits() {
    let (state, catalog, resolver) = full_parts();
    let mut assets = catalog.assets().to_vec();
    let icon = catalog
        .get(&AssetId::from_raw("asset-ana-icon"))
        .expect("linked icon");
    let mut duplicate = icon.clone();
    duplicate.id = AssetId::from_raw("asset-ana-icon-duplicate");
    duplicate.name = "Ana".to_owned();
    assets.push(duplicate);

    let catalog = AssetCatalog::new(assets);
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["heroIcons"].as_object().expect("heroIcons");
    let portraits = value["heroPortraits"].as_object().expect("heroPortraits");

    let icon_url = format!("http://127.0.0.1:3001/assets/{}.png", support::hash(0x17));
    let art_url = format!("http://127.0.0.1:3001/assets/{}.png", support::hash(0x18));
    assert_eq!(icons["Ana"], icon_url);
    assert_eq!(portraits["Ana"], art_url);
}

#[test]
fn hero_icons_fall_back_to_art_without_an_icon_link() {
    let (state, mut catalog, resolver) = full_parts();
    let mut assets = catalog.assets().to_vec();
    assets.push(Asset {
        id: AssetId::from_raw("asset-hero-imported"),
        name: "Imported".to_owned(),
        category: AssetCategory::Hero,
        hash: support::hash(0x1a),
        ext: "png".to_owned(),
        origin: AssetOrigin::User,
        original_filename: None,
        added_at: None,
        icon: None,
        portrait: None,
        transparent: None,
        role: None,
        game_mode: None,
        attachment: None,
    });
    catalog = AssetCatalog::new(assets);

    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let icons = value["heroIcons"].as_object().expect("heroIcons");

    let expected = format!("http://127.0.0.1:3001/assets/{}.png", support::hash(0x1a));
    assert_eq!(icons["Imported"], expected);
}

#[test]
fn map_pool_data_groups_and_orders_modes() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let groups = value["mapPoolData"].as_array().expect("mapPoolData");

    let modes: Vec<&str> = groups
        .iter()
        .map(|group| group["modeName"].as_str().expect("mode name"))
        .collect();
    assert_eq!(modes, ["Control", "Escort", "Hybrid", "Flashpoint", "Push"]);

    let control = &groups[0]["maps"].as_array().expect("control maps");
    assert_eq!(control.len(), 2);
    assert_eq!(control[0]["name"], "Nepal");
    assert_eq!(control[1]["name"], "Ilios");

    for group in groups {
        assert!(
            group["modeIcons"].is_array(),
            "modeIcons must always be an array"
        );
    }
}

#[test]
fn map_pool_data_skips_unknown_maps() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let serialised = value["mapPoolData"].to_string();

    assert!(!serialised.contains("Unknown Map"));
}

#[test]
fn map_pool_data_carries_chosen_by_and_logo() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let control = &value["mapPoolData"][0]["maps"];

    assert_eq!(control[0]["name"], "Nepal");
    assert_eq!(control[0]["chosenBy"], Value::Null);
    assert_eq!(control[0]["chosenByLogo"], Value::Null);

    assert_eq!(control[1]["name"], "Ilios");
    assert_eq!(control[1]["chosenBy"], "A");
    assert_eq!(
        control[1]["chosenByLogo"],
        value["teamA"]["logoUrl"].clone()
    );
}

#[test]
fn current_map_index_is_raw_zero_based() {
    let (mut state, catalog, resolver) = full_parts();
    state
        .current_match
        .as_mut()
        .expect("fixture match")
        .current_map = Some(2);
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["currentMapIndex"], 2);
    assert_eq!(value["currentMap"], "Dorado");
    assert_eq!(value["mapType"], "Escort");
}

#[test]
fn no_current_map_yields_empty_strings_and_null_index() {
    let (mut state, catalog, resolver) = full_parts();
    state
        .current_match
        .as_mut()
        .expect("fixture match")
        .current_map = None;
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["currentMapIndex"], Value::Null);
    assert_eq!(value["currentMap"], "");
    assert_eq!(value["mapType"], "");
}

#[test]
fn out_of_range_current_map_is_empty_but_index_is_preserved() {
    let (mut state, catalog, resolver) = full_parts();
    state
        .current_match
        .as_mut()
        .expect("fixture match")
        .current_map = Some(99);
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["currentMapIndex"], 99);
    assert_eq!(value["currentMap"], "");
    assert_eq!(value["mapType"], "");
}

#[test]
fn maps_type_is_the_game_mode() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let maps = value["maps"].as_array().expect("maps");

    assert_eq!(maps[0]["name"], "Ilios");
    assert_eq!(maps[0]["type"], "Control");
    assert_eq!(maps[2]["name"], "Dorado");
    assert_eq!(maps[2]["type"], "Escort");
}

#[test]
fn record_defaults_to_zero_zero() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["teamA"]["record"], "12-3");
    assert_eq!(value["teamB"]["record"], "0-0");
}

#[test]
fn empty_record_defaults_to_zero_zero() {
    let (mut state, catalog, resolver) = full_parts();
    state.teams[1].record = Some(String::new());
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["teamB"]["record"], "0-0");
}

#[test]
fn empty_aux_color_uses_the_original_default() {
    let (mut state, catalog, resolver) = full_parts();
    state.teams[0].aux_color = String::new();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["teamA"]["logoBgColor"], "#090909");
}

#[test]
fn logo_background_off_uses_the_normal_team_colour() {
    let (mut state, catalog, resolver) = full_parts();
    state.teams[0].logo_background = false;
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["teamA"]["logoBgColor"], "#ff0000");
}

#[test]
fn pronouns_are_omitted_when_unset() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let players = value["teamAPlayers"].as_array().expect("players");

    assert_eq!(players[0]["pronouns"], "they/them");
    assert!(players[1].get("pronouns").is_none());
}

#[test]
fn caster_pronouns_and_social_are_omitted_when_unset() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = serde_json::to_value(projector.casters_message(match_state)).expect("casters");
    let casters = value["data"]["castersData"].as_array().expect("casters");

    assert_eq!(casters[0]["pronouns"], "he/him");
    assert!(casters[1].get("pronouns").is_none());
    assert!(casters[1].get("socialMedia").is_none());
}

#[test]
fn side_icons_always_have_both_keys() {
    let state = fgc_core::AppState::default();
    let catalog = AssetCatalog::default();
    let resolver = support::resolver();
    let projector = Projector::new(&state, &catalog, &resolver);

    let value =
        serde_json::to_value(projector.scoreboard(&support::minimal_match())).expect("scoreboard");
    let icons = value["sideIcons"].as_object().expect("sideIcons");

    assert!(icons.contains_key("attack"));
    assert!(icons.contains_key("defend"));
    assert_eq!(icons["attack"], Value::Null);
    assert_eq!(icons["defend"], Value::Null);
}

#[test]
fn watermark_falls_back_to_bundled_peach() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let peach = catalog
        .find_exact(AssetCategory::Logo, "Peach")
        .expect("Peach asset");
    let expected = resolver.url(peach).expect("Peach URL");

    assert_eq!(value["watermarkLogoUrl"], expected);
}

#[test]
fn watermark_falls_back_when_the_configured_asset_is_missing() {
    let (mut state, catalog, resolver) = full_parts();
    state.settings.display.watermark_logo = Some(AssetId::from_raw("missing-logo"));
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let peach = catalog
        .find_exact(AssetCategory::Logo, "Peach")
        .expect("Peach asset");
    let expected = resolver.url(peach).expect("Peach URL");

    assert_eq!(value["watermarkLogoUrl"], expected);
}

#[test]
fn configured_watermark_that_cannot_be_read_is_null() {
    let (mut state, catalog, _) = full_parts();
    state.settings.display.watermark_logo = Some(AssetId::from_raw("asset-logo-t"));
    let resolver = DataUrlResolver::new(|_: &Asset| None);
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["watermarkLogoUrl"], Value::Null);
}

#[test]
fn hero_art_prefers_a_portrait_link() {
    let (mut state, mut catalog, resolver) = full_parts();
    let mut assets = catalog.assets().to_vec();
    assets.push(Asset {
        id: AssetId::from_raw("asset-ana-alt"),
        name: "Ana Alt".to_owned(),
        category: AssetCategory::Hero,
        hash: support::hash(0x19),
        ext: "png".to_owned(),
        origin: AssetOrigin::Bundled,
        original_filename: None,
        added_at: None,
        icon: None,
        portrait: Some(AssetId::from_raw("asset-portrait-a")),
        transparent: None,
        role: None,
        game_mode: None,
        attachment: None,
    });
    catalog = AssetCatalog::new(assets);

    let match_state = state.current_match.as_mut().expect("fixture match");
    match_state.hero_bans[0].team_a.push("Ana Alt".to_owned());

    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let portraits = value["heroPortraits"].as_object().expect("heroPortraits");

    // The override link wins over the hero asset's own file.
    let expected = format!("http://127.0.0.1:3001/assets/{}.png", support::hash(0x14));
    assert_eq!(portraits["Ana Alt"], expected);
}

#[test]
fn derived_scores_count_wins_and_ignore_draws() {
    let mut match_state = support::minimal_match();
    match_state.scoring = Scoring::Derived;
    match_state.maps = vec![
        fgc_core::MapSlot {
            name: "One".to_owned(),
            game_mode_name: "Control".to_owned(),
            game_mode_manual: false,
            user_added: false,
            completed: true,
            score_a: 2,
            score_b: 1,
            chosen_by: None,
        },
        fgc_core::MapSlot {
            name: "Two".to_owned(),
            game_mode_name: "Control".to_owned(),
            game_mode_manual: false,
            user_added: false,
            completed: true,
            score_a: 1,
            score_b: 1,
            chosen_by: None,
        },
        fgc_core::MapSlot {
            name: "Three".to_owned(),
            game_mode_name: "Control".to_owned(),
            game_mode_manual: false,
            user_added: false,
            completed: false,
            score_a: 3,
            score_b: 0,
            chosen_by: None,
        },
        fgc_core::MapSlot {
            name: "Four".to_owned(),
            game_mode_name: "Control".to_owned(),
            game_mode_manual: false,
            user_added: false,
            completed: true,
            score_a: 0,
            score_b: 2,
            chosen_by: None,
        },
    ];

    assert_eq!(scores(&match_state), (1, 1));
}

#[test]
fn manual_scores_override_derived_scores() {
    let mut match_state = support::minimal_match();
    match_state.scoring = Scoring::Manual { a: 3, b: 2 };
    match_state.maps = vec![fgc_core::MapSlot {
        name: "One".to_owned(),
        game_mode_name: "Control".to_owned(),
        game_mode_manual: false,
        user_added: false,
        completed: true,
        score_a: 2,
        score_b: 1,
        chosen_by: None,
    }];

    assert_eq!(scores(&match_state), (3, 2));
}

#[test]
fn team_side_and_swap_flags_are_passed_through() {
    let (mut state, catalog, resolver) = full_parts();
    {
        let match_state = state.current_match.as_mut().expect("fixture match");
        match_state.side = Side::HomeDefends;
        match_state.swap_sides = false;
        match_state.loser_picks = false;
    }
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(
        value["side"].as_u64(),
        Some(u64::from(Side::HomeDefends.as_u8()))
    );
    assert_eq!(value["swapSides"], Value::Bool(false));
    assert_eq!(value["loserPicks"], Value::Bool(false));
}

#[test]
fn chosen_by_name_uses_the_team_tag() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let maps = value["maps"].as_array().expect("maps");

    assert_eq!(maps[0]["chosenBy"], "A");
    assert_eq!(maps[0]["chosenByName"], "ALP");
    assert_eq!(maps[2]["chosenBy"], "B");
    assert_eq!(maps[2]["chosenByName"], "BRV");
    assert_eq!(maps[1]["chosenBy"], Value::Null);
    assert_eq!(maps[1]["chosenByName"], Value::Null);
}

#[test]
fn missing_teams_produce_null_summaries_and_empty_rosters() {
    let (mut state, catalog, resolver) = full_parts();
    {
        let match_state = state.current_match.as_mut().expect("fixture match");
        match_state.team_a = None;
        match_state.team_b = Some(TeamId::from_raw("missing-team"));
    }
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(value["teamA"], Value::Null);
    assert_eq!(value["teamB"], Value::Null);
    assert_eq!(value["teamAPlayers"].as_array().expect("players").len(), 0);
    assert_eq!(value["teamBPlayers"].as_array().expect("players").len(), 0);
}

#[test]
fn hero_bans_are_passed_through() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);
    let bans = value["heroBans"].as_array().expect("heroBans");

    assert_eq!(bans.len(), 2);
    assert_eq!(bans[0]["firstBanTeam"], "A");
    assert_eq!(bans[0]["teamA"][0], "Ana");
    assert_eq!(bans[1]["firstBanTeam"], "B");
}

#[test]
fn map_pool_is_copied_in_order() {
    let (state, catalog, resolver) = full_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let value = scoreboard_value(&projector, match_state);

    assert_eq!(
        value["mapPool"],
        serde_json::json!([
            "Nepal",
            "Ilios",
            "Dorado",
            "King's Row",
            "Suravasa",
            "Colosseo",
            "Unknown Map"
        ])
    );
}

#[test]
fn team_side_serialises_uppercase() {
    let bans = fgc_core::MapHeroBans {
        first_ban_team: Some(TeamSide::B),
        team_a: Vec::new(),
        team_b: Vec::new(),
    };
    let value = serde_json::to_value(bans).expect("bans");
    assert_eq!(value["firstBanTeam"], "B");
}
