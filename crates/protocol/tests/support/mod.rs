#![allow(dead_code)]

use fgc_core::{
    AppState, Asset, AssetCategory, AssetId, AssetOrigin, DisplaySettings, MapHeroBans, MapSlot,
    Match, MatchId, MatchStatus, MatchTalent, OverlaySettings, Player, PlayerId, Position, Role,
    Scoring, Settings, Side, SocialLink, SocialPlatform, Talent, TalentId, TalentRef, TalentType,
    Team, TeamId, TeamSide, ThemePreset, TimerAlignment, TimerSettings,
};
use fgc_protocol::{
    AssetCatalog, Projector, SCOREBOARD_TYPES, SectionCache, ServedAssetResolver, TimerPayload,
    TimerSettingsDto, TimerTickDto,
};

/// Deterministic 64-character lowercase hash.
pub fn hash(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn asset(
    id: &str,
    category: AssetCategory,
    name: &str,
    byte: u8,
    ext: &str,
    origin: AssetOrigin,
) -> Asset {
    Asset {
        id: AssetId::from_raw(id),
        name: name.to_owned(),
        category,
        hash: hash(byte),
        ext: ext.to_owned(),
        origin,
        original_filename: None,
        added_at: None,
        icon: None,
        portrait: None,
        transparent: None,
        role: None,
        game_mode: None,
        attachment: None,
    }
}

fn user_asset(id: &str, category: AssetCategory, name: &str, byte: u8, ext: &str) -> Asset {
    asset(id, category, name, byte, ext, AssetOrigin::User)
}

fn bundled_asset(id: &str, category: AssetCategory, name: &str, byte: u8, ext: &str) -> Asset {
    asset(id, category, name, byte, ext, AssetOrigin::Bundled)
}

/// User assets referenced by teams, players and talents.
pub fn user_assets() -> Vec<Asset> {
    vec![
        user_asset("asset-logo-a", AssetCategory::Logo, "Alpha", 0x11, "png"),
        user_asset(
            "asset-logo-t",
            AssetCategory::Logo,
            "Tournament",
            0x12,
            "png",
        ),
        user_asset(
            "asset-portrait-a",
            AssetCategory::Portrait,
            "Ace Portrait",
            0x14,
            "png",
        ),
        user_asset(
            "asset-portrait-t1",
            AssetCategory::Portrait,
            "Caster One Portrait",
            0x15,
            "png",
        ),
    ]
}

/// Bundled assets, including maps, modes, roles, sides, heroes and the Peach watermark.
pub fn bundled_assets() -> Vec<Asset> {
    let mut assets = vec![
        bundled_asset("asset-peach", AssetCategory::Logo, "Peach", 0x13, "png"),
        bundled_asset("asset-ana", AssetCategory::Hero, "Ana", 0x18, "png"),
        bundled_asset(
            "asset-ana-icon",
            AssetCategory::Hero,
            "Ana Icon",
            0x17,
            "png",
        ),
    ];

    for (id, name, byte) in [
        ("asset-map-ilios", "Ilios", 0x21),
        ("asset-map-nepal", "Nepal", 0x22),
        ("asset-map-dorado", "Dorado", 0x23),
        ("asset-map-kings-row", "King's Row", 0x24),
        ("asset-map-suravasa", "Suravasa", 0x25),
        ("asset-map-colosseo", "Colosseo", 0x26),
    ] {
        assets.push(bundled_asset(id, AssetCategory::Map, name, byte, "png"));
    }

    for (id, name, byte) in [
        ("asset-mode-control", "Control", 0x31),
        ("asset-mode-escort", "Escort", 0x32),
        ("asset-mode-hybrid", "Hybrid", 0x33),
        ("asset-mode-flashpoint", "Flashpoint", 0x34),
        ("asset-mode-push", "Push", 0x35),
    ] {
        assets.push(bundled_asset(
            id,
            AssetCategory::GameMode,
            name,
            byte,
            "png",
        ));
    }

    for (id, name, byte) in [
        ("asset-role-tank", "Tank", 0x41),
        ("asset-role-damage", "Damage", 0x42),
        ("asset-role-support", "Support", 0x43),
    ] {
        assets.push(bundled_asset(id, AssetCategory::Role, name, byte, "png"));
    }

    for (id, name, byte) in [
        ("asset-side-attack", "Attack", 0x51),
        ("asset-side-defend", "Defend", 0x52),
    ] {
        assets.push(bundled_asset(id, AssetCategory::Side, name, byte, "png"));
    }

    assets
}

/// Adds the hero-art links and map game-mode links.
pub fn linked_bundled_assets() -> Vec<Asset> {
    let mut assets = bundled_assets();
    for asset in &mut assets {
        match asset.id.as_str() {
            "asset-ana" => {
                asset.icon = Some(AssetId::from_raw("asset-ana-icon"));
            }
            "asset-map-ilios" | "asset-map-nepal" => {
                asset.game_mode = Some(AssetId::from_raw("asset-mode-control"));
            }
            "asset-map-dorado" => {
                asset.game_mode = Some(AssetId::from_raw("asset-mode-escort"));
            }
            "asset-map-kings-row" => {
                asset.game_mode = Some(AssetId::from_raw("asset-mode-hybrid"));
            }
            "asset-map-suravasa" => {
                asset.game_mode = Some(AssetId::from_raw("asset-mode-flashpoint"));
            }
            "asset-map-colosseo" => {
                asset.game_mode = Some(AssetId::from_raw("asset-mode-push"));
            }
            _ => {}
        }
    }
    assets
}

/// Catalogue containing user assets followed by linked bundled assets.
pub fn fixture_catalog() -> AssetCatalog {
    let mut assets = user_assets();
    assets.extend(linked_bundled_assets());
    AssetCatalog::new(assets)
}

/// Served-URL resolver at the default loopback address.
pub fn resolver() -> ServedAssetResolver {
    ServedAssetResolver::new("http://127.0.0.1:3001")
}

/// Full fixture state used by the golden tests.
pub fn fixture_state() -> AppState {
    let team_a = Team {
        id: TeamId::from_raw("team-a"),
        name: "Alpha".to_owned(),
        tag: "ALP".to_owned(),
        record: Some("12-3".to_owned()),
        color: "#ff0000".to_owned(),
        text_color: "#ffffff".to_owned(),
        aux_color: "#111111".to_owned(),
        logo: Some(AssetId::from_raw("asset-logo-a")),
        logo_background: true,
        players: vec![
            Player {
                id: PlayerId::from_raw("player-ace"),
                name: "Ace".to_owned(),
                role: Role::Tank,
                position: Position::Core,
                portrait: Some(AssetId::from_raw("asset-portrait-a")),
                featured_hero: Some(AssetId::from_raw("asset-ana")),
                pronouns: Some("they/them".to_owned()),
            },
            Player {
                id: PlayerId::from_raw("player-bench"),
                name: "Bench".to_owned(),
                role: Role::Damage,
                position: Position::Sub,
                portrait: None,
                featured_hero: None,
                pronouns: None,
            },
        ],
    };

    let team_b = Team {
        id: TeamId::from_raw("team-b"),
        name: "Bravo".to_owned(),
        tag: "BRV".to_owned(),
        record: None,
        color: "#00ff00".to_owned(),
        text_color: "#000000".to_owned(),
        aux_color: "#090909".to_owned(),
        logo: None,
        logo_background: true,
        players: vec![Player {
            id: PlayerId::from_raw("player-blaze"),
            name: "Blaze".to_owned(),
            role: Role::Support,
            position: Position::Core,
            portrait: None,
            featured_hero: None,
            pronouns: Some("she/her".to_owned()),
        }],
    };

    let talents = vec![
        Talent {
            id: TalentId::from_raw("talent-one"),
            display_name: "Caster One".to_owned(),
            real_name: Some("Real One".to_owned()),
            pronouns: Some("he/him".to_owned()),
            social: Some(SocialLink {
                platform: SocialPlatform::Twitter,
                handle: "@caster1".to_owned(),
            }),
            portrait: Some(AssetId::from_raw("asset-portrait-t1")),
        },
        Talent {
            id: TalentId::from_raw("talent-two"),
            display_name: "Caster Two".to_owned(),
            real_name: None,
            pronouns: None,
            social: None,
            portrait: None,
        },
    ];

    let match_state = Match {
        id: MatchId::from_raw("match-grand-final"),
        title: "Grand Final".to_owned(),
        subtitle: "Upper Bracket".to_owned(),
        scheduled_at: None,
        team_a: Some(TeamId::from_raw("team-a")),
        team_b: Some(TeamId::from_raw("team-b")),
        maps: vec![
            MapSlot {
                name: "Ilios".to_owned(),
                game_mode_name: "Control".to_owned(),
                game_mode_manual: false,
                user_added: false,
                completed: true,
                score_a: 2,
                score_b: 1,
                chosen_by: Some(TeamSide::A),
            },
            MapSlot {
                name: "Nepal".to_owned(),
                game_mode_name: "Control".to_owned(),
                game_mode_manual: false,
                user_added: false,
                completed: false,
                score_a: 0,
                score_b: 0,
                chosen_by: None,
            },
            MapSlot {
                name: "Dorado".to_owned(),
                game_mode_name: "Escort".to_owned(),
                game_mode_manual: false,
                user_added: false,
                completed: false,
                score_a: 0,
                score_b: 0,
                chosen_by: Some(TeamSide::B),
            },
        ],
        hero_bans: vec![
            MapHeroBans {
                first_ban_team: Some(TeamSide::A),
                team_a: vec!["Ana".to_owned()],
                team_b: vec!["Kiriko".to_owned()],
            },
            MapHeroBans {
                first_ban_team: Some(TeamSide::B),
                team_a: vec!["Tracer".to_owned()],
                team_b: Vec::new(),
            },
        ],
        map_pool: vec![
            "Nepal".to_owned(),
            "Ilios".to_owned(),
            "Dorado".to_owned(),
            "King's Row".to_owned(),
            "Suravasa".to_owned(),
            "Colosseo".to_owned(),
            "Unknown Map".to_owned(),
        ],
        first_to: 3,
        status: MatchStatus::Live,
        current_map: Some(0),
        swap_sides: true,
        scoring: Scoring::Derived,
        side: Side::HomeAttacks,
        talent: MatchTalent {
            caster1: Some(TalentRef {
                talent_id: TalentId::from_raw("talent-one"),
                role: TalentType::Caster,
            }),
            caster2: Some(TalentRef {
                talent_id: TalentId::from_raw("talent-two"),
                role: TalentType::Caster,
            }),
            ..MatchTalent::default()
        },
        loser_picks: true,
        show_player_portraits: true,
    };

    AppState {
        teams: vec![team_a, team_b],
        talents,
        current_match: Some(match_state),
        tournaments: Vec::new(),
        assets: user_assets(),
        settings: Settings {
            overlay: OverlaySettings::default(),
            display: DisplaySettings {
                team_colors_enabled: true,
                show_hero_bans: true,
                team_abbreviations: false,
                theme: ThemePreset::NightBroadcast,
                theme_tokens: None,
                tournament_logo: Some(AssetId::from_raw("asset-logo-t")),
                watermark_logo: None,
            },
            timer: TimerSettings {
                mode: fgc_core::TimerMode::CountUp,
                alignment: TimerAlignment::Center,
                leading_zeroes: true,
                show_hours: true,
                standalone: false,
            },
        },
        ..AppState::default()
    }
}

/// A match with no teams, maps, bans or pool.
pub fn minimal_match() -> Match {
    Match {
        id: MatchId::from_raw("match-minimal"),
        title: String::new(),
        subtitle: String::new(),
        scheduled_at: None,
        team_a: None,
        team_b: None,
        maps: Vec::new(),
        hero_bans: Vec::new(),
        map_pool: Vec::new(),
        first_to: 3,
        status: MatchStatus::Pre,
        current_map: None,
        swap_sides: false,
        scoring: Scoring::Derived,
        side: Side::Neutral,
        talent: MatchTalent::default(),
        loser_picks: false,
        show_player_portraits: true,
    }
}

/// Cache populated the way the server does after a full broadcast.
pub fn fixture_cache(state: &AppState, projector: &Projector<'_>) -> SectionCache {
    let match_state = state.current_match.as_ref().expect("fixture match");
    let scoreboard = projector.scoreboard(match_state);
    let mut cache = SectionCache::new();
    for kind in SCOREBOARD_TYPES {
        cache.set(kind, &scoreboard).expect("cache scoreboard");
    }
    cache
        .set("casters", &projector.casters(match_state))
        .expect("cache casters");
    cache
        .set(
            "matchTimer",
            &TimerPayload::Tick(TimerTickDto {
                hours: 0,
                minutes: 12,
                seconds: 34,
                settings: TimerSettingsDto::from(&state.settings.timer),
            }),
        )
        .expect("cache timer");
    cache
}
