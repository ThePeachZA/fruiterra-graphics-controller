//! Builds the manual-QA fixture around assets that are already imported.
//!
//! Run this with the desktop app closed:
//!
//! ```powershell
//! cargo run -p fgc-core --example qa_fixture
//! ```
//!
//! The builder keeps the imported asset library, replaces the teams, talent,
//! current match and tournaments, and writes `app.json` through [`JsonStore`].
//! It expects the asset names listed in `docs/manual-qa-runbook.md` §3.4:
//! two logos, five portraits, five maps and six heroes.
//!
//! The fixture is deterministic apart from generated identifiers. Re-running it
//! rebuilds the same project from the same asset library.

use std::collections::BTreeMap;
use std::error::Error;
use std::io;
use std::path::Path;

use fgc_core::{
    AppState, AssetCategory, AssetId, BracketFormat, IntervieweeRef, JsonStore, MapHeroBans,
    MapSlot, Match, MatchStatus, MatchTalent, Player, PlayerId, Position, Role, Scoring, Side,
    SocialLink, SocialPlatform, Store, Talent, TalentId, TalentRef, TalentType, Team, TeamId,
    TeamSide, TimerMode, empty_hero_bans, generate_double_elimination, generate_round_robin,
    generate_single_elimination, new_match, new_tournament, set_entrants, sha256_hex,
    validate_and_repair,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let path = std::env::args().nth(1).map_or_else(
        || fgc_core::state_file().map_err(|error| -> Box<dyn Error> { error.into() }),
        |value| Ok(value.into()),
    )?;

    let store = JsonStore::new(&path);
    let mut state = store.load()?;

    rename_map(&mut state, "Kings Row", "King's Row");
    link_map_modes(&mut state)?;

    state.teams = build_teams(&state)?;
    state.talents = build_talent(&state)?;
    state.current_match = Some(build_match(&state)?);
    state.tournaments = build_tournaments(&state)?;

    state.settings.overlay.port = 3001;
    state.settings.overlay.allow_lan = false;
    state.settings.display.team_colors_enabled = true;
    state.settings.timer.mode = TimerMode::CountDown;
    state.settings.timer.leading_zeroes = true;
    state.settings.timer.show_hours = true;
    state.settings.timer.standalone = false;

    let repairs = validate_and_repair(&mut state);
    if !repairs.is_empty() {
        for repair in &repairs {
            eprintln!("repair applied: {:?}: {}", repair.kind, repair.message);
        }
        return Err(io::Error::other("fixture was repaired; refusing to write it").into());
    }

    store.save(&state)?;
    println!("wrote {}", path.display());
    println!(
        "{} teams, {} talent, {} assets, {} tournaments",
        state.teams.len(),
        state.talents.len(),
        state.assets.len(),
        state.tournaments.len()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

fn asset(state: &AppState, category: AssetCategory, name: &str) -> Result<AssetId> {
    state
        .assets
        .iter()
        .find(|asset| asset.category == category && asset.name == name)
        .map(|asset| asset.id.clone())
        .ok_or_else(|| {
            io::Error::other(format!(
                "missing {category:?} asset named {name:?}; import it on the Assets tab first"
            ))
            .into()
        })
}

fn rename_map(state: &mut AppState, from: &str, to: &str) {
    let already_named = state
        .assets
        .iter()
        .any(|asset| asset.category == AssetCategory::Map && asset.name == to);
    if already_named {
        return;
    }
    for asset in &mut state.assets {
        if asset.category == AssetCategory::Map && asset.name == from {
            asset.name = to.to_owned();
        }
    }
}

/// Links each map to the bundled game-mode icon so the map pool groups by mode.
fn link_map_modes(state: &mut AppState) -> Result<()> {
    let control = bundled_id("game-modes/control.png")?;
    let hybrid = bundled_id("game-modes/hybrid.png")?;
    let escort = bundled_id("game-modes/escort.png")?;
    let modes: BTreeMap<&str, &AssetId> = [
        ("Ilios", &control),
        ("Nepal", &control),
        ("King's Row", &hybrid),
        ("Dorado", &escort),
        ("Antarctic Peninsula", &control),
    ]
    .into_iter()
    .collect();

    for asset in &mut state.assets {
        if asset.category != AssetCategory::Map {
            continue;
        }
        if let Some(mode) = modes.get(asset.name.as_str()) {
            asset.game_mode = Some((*mode).clone());
        }
    }
    Ok(())
}

fn bundled_id(relative: &str) -> Result<AssetId> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../server/assets/bundled")
        .join(relative);
    let bytes = std::fs::read(&path)?;
    Ok(AssetId::from_raw(format!("b:{}", sha256_hex(&bytes))))
}

// ---------------------------------------------------------------------------
// Teams and talent
// ---------------------------------------------------------------------------

fn build_teams(state: &AppState) -> Result<Vec<Team>> {
    let northwind_logo = asset(state, AssetCategory::Logo, "Pink Salamanders")?;
    let wxm_logo = asset(state, AssetCategory::Logo, "Rage Quit")?;
    let blasters_logo = asset(state, AssetCategory::Logo, "Inglorious Blasters")?;
    let curro_logo = asset(state, AssetCategory::Logo, "Curro")?;

    let babs = asset(state, AssetCategory::Portrait, "Babs")?;
    let blur = asset(state, AssetCategory::Portrait, "Blur")?;
    let ulturism = asset(state, AssetCategory::Portrait, "Ulturism")?;
    let sunpheonyx = asset(state, AssetCategory::Portrait, "SunPheonyx")?;
    let tocs = asset(state, AssetCategory::Portrait, "Tocs")?;

    let ana = asset(state, AssetCategory::Hero, "Ana")?;
    let kiriko = asset(state, AssetCategory::Hero, "Kiriko")?;
    let winston = asset(state, AssetCategory::Hero, "Winston")?;
    let tracer = asset(state, AssetCategory::Hero, "Tracer")?;
    let sojourn = asset(state, AssetCategory::Hero, "Sojourn")?;
    let lucio = asset(state, AssetCategory::Hero, "Lucio")?;

    let mut northwind = team(
        "Northwind Sentinels",
        "NWS",
        Some("12-3"),
        "#1f6feb",
        "#ffffff",
        Some(northwind_logo),
    );
    northwind.players = vec![
        player(
            "SØREN",
            Role::Tank,
            Position::Core,
            "he/him",
            Some(babs.clone()),
            Some(ana.clone()),
        ),
        player(
            "ДМИТРИЙ ВОЛКОВ",
            Role::Damage,
            Position::Core,
            "they/them",
            Some(blur.clone()),
            Some(kiriko.clone()),
        ),
        player(
            "王小明",
            Role::Support,
            Position::Core,
            "he/him",
            Some(ulturism.clone()),
            Some(winston),
        ),
        player(
            "Longplayer Name Overflow",
            Role::Damage,
            Position::Sub,
            "she/her",
            Some(sunpheonyx.clone()),
            Some(tracer),
        ),
    ];

    let mut wxm = team(
        "王小明战队",
        "WXM",
        Some("9-6"),
        "#d29922",
        "#111111",
        Some(wxm_logo),
    );
    wxm.players = vec![
        player(
            "Alex",
            Role::Tank,
            Position::Core,
            "he/him",
            Some(tocs.clone()),
            Some(sojourn),
        ),
        player(
            "Priya",
            Role::Damage,
            Position::Core,
            "she/her",
            Some(babs),
            Some(lucio),
        ),
        player(
            "Marco",
            Role::Support,
            Position::Core,
            "they/them",
            Some(blur),
            Some(ana),
        ),
        player(
            "Sam",
            Role::Support,
            Position::Sub,
            "they/them",
            Some(ulturism),
            Some(kiriko),
        ),
    ];

    let long = team(
        "Long Team Name That Overflows",
        "LONG",
        Some("0-0"),
        "#2ea043",
        "#ffffff",
        Some(blasters_logo),
    );
    let cyrillic = team(
        "ДМИТРИЙ ВОЛКОВ",
        "ДВ",
        Some("3-1"),
        "#8250df",
        "#ffffff",
        Some(curro_logo),
    );

    let mut teams = vec![northwind, wxm, long, cyrillic];
    for index in 1..=6 {
        teams.push(team(
            &format!("Team {index}"),
            &format!("T{index}"),
            None,
            "#30363d",
            "#ffffff",
            None,
        ));
    }
    for index in 1..=22 {
        teams.push(team(
            &format!("Stress {index}"),
            &format!("S{index}"),
            None,
            "#21262d",
            "#ffffff",
            None,
        ));
    }
    Ok(teams)
}

fn build_talent(state: &AppState) -> Result<Vec<Talent>> {
    let babs = asset(state, AssetCategory::Portrait, "Babs")?;
    let blur = asset(state, AssetCategory::Portrait, "Blur")?;
    let sunpheonyx = asset(state, AssetCategory::Portrait, "SunPheonyx")?;
    let tocs = asset(state, AssetCategory::Portrait, "Tocs")?;

    Ok(vec![
        talent("Arc", SocialPlatform::Twitch, "@arc", "he/him", tocs),
        talent(
            "Priya",
            SocialPlatform::Twitter,
            "@priya",
            "she/her",
            sunpheonyx,
        ),
        talent("Marco", SocialPlatform::YouTube, "@marco", "he/him", blur),
        talent("Sam", SocialPlatform::Instagram, "@sam", "they/them", babs),
    ])
}

fn team(
    name: &str,
    tag: &str,
    record: Option<&str>,
    color: &str,
    text_color: &str,
    logo: Option<AssetId>,
) -> Team {
    Team {
        id: TeamId::new(),
        name: name.to_owned(),
        tag: tag.to_owned(),
        record: record.map(ToOwned::to_owned),
        color: color.to_owned(),
        text_color: text_color.to_owned(),
        aux_color: "#090909".to_owned(),
        logo,
        logo_background: false,
        players: Vec::new(),
    }
}

fn player(
    name: &str,
    role: Role,
    position: Position,
    pronouns: &str,
    portrait: Option<AssetId>,
    featured_hero: Option<AssetId>,
) -> Player {
    Player {
        id: PlayerId::new(),
        name: name.to_owned(),
        role,
        position,
        portrait,
        featured_hero,
        pronouns: Some(pronouns.to_owned()),
    }
}

fn talent(
    display_name: &str,
    platform: SocialPlatform,
    handle: &str,
    pronouns: &str,
    portrait: AssetId,
) -> Talent {
    Talent {
        id: TalentId::new(),
        display_name: display_name.to_owned(),
        real_name: None,
        pronouns: Some(pronouns.to_owned()),
        social: Some(SocialLink {
            platform,
            handle: handle.to_owned(),
        }),
        portrait: Some(portrait),
    }
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

fn build_match(state: &AppState) -> Result<Match> {
    let northwind = find_team(state, "Northwind Sentinels")?;
    let wxm = find_team(state, "王小明战队")?;

    let arc = find_talent(state, "Arc")?;
    let priya = find_talent(state, "Priya")?;
    let marco = find_talent(state, "Marco")?;
    let sam = find_talent(state, "Sam")?;

    let mut current = new_match();
    current.title = "Grand Final".to_owned();
    current.subtitle = "Upper Bracket".to_owned();
    current.team_a = Some(northwind);
    current.team_b = Some(wxm);
    current.maps = vec![
        map_slot("Ilios", "Control", Some(TeamSide::A), true, 2, 1),
        map_slot("Nepal", "Control", Some(TeamSide::B), true, 1, 2),
        map_slot("King's Row", "Hybrid", Some(TeamSide::A), true, 3, 2),
        map_slot("Dorado", "Escort", Some(TeamSide::B), false, 0, 0),
        map_slot(
            "Antarctic Peninsula",
            "Control",
            Some(TeamSide::A),
            false,
            0,
            0,
        ),
    ];
    current.hero_bans = vec![
        hero_bans(Some(TeamSide::A), &["Ana"], &["Kiriko"]),
        hero_bans(Some(TeamSide::B), &["Winston"], &["Tracer"]),
        hero_bans(Some(TeamSide::A), &["Sojourn"], &["Lucio"]),
        empty_hero_bans(),
        empty_hero_bans(),
    ];
    current.map_pool = vec![
        "Ilios".to_owned(),
        "Nepal".to_owned(),
        "King's Row".to_owned(),
        "Dorado".to_owned(),
        "Antarctic Peninsula".to_owned(),
    ];
    current.first_to = 3;
    current.status = MatchStatus::Live;
    current.current_map = Some(0);
    current.swap_sides = false;
    current.scoring = Scoring::Derived;
    current.side = Side::HomeAttacks;
    current.loser_picks = true;
    current.show_player_portraits = true;
    current.talent = MatchTalent {
        caster1: Some(TalentRef {
            talent_id: arc,
            role: TalentType::Caster,
        }),
        caster2: Some(TalentRef {
            talent_id: priya,
            role: TalentType::Caster,
        }),
        panel1: Some(TalentRef {
            talent_id: marco,
            role: TalentType::Analyst,
        }),
        panel2: Some(TalentRef {
            talent_id: sam.clone(),
            role: TalentType::Analyst,
        }),
        panel3: None,
        interviewer: None,
        interviewee: Some(IntervieweeRef {
            talent_id: Some(sam),
            custom_name: None,
            custom_subtext: None,
        }),
    };

    Ok(current)
}

fn find_team(state: &AppState, name: &str) -> Result<TeamId> {
    state
        .teams
        .iter()
        .find(|team| team.name == name)
        .map(|team| team.id.clone())
        .ok_or_else(|| io::Error::other(format!("missing team {name:?}")).into())
}

fn find_talent(state: &AppState, name: &str) -> Result<TalentId> {
    state
        .talents
        .iter()
        .find(|talent| talent.display_name == name)
        .map(|talent| talent.id.clone())
        .ok_or_else(|| io::Error::other(format!("missing talent {name:?}")).into())
}

fn map_slot(
    name: &str,
    mode: &str,
    chosen_by: Option<TeamSide>,
    completed: bool,
    score_a: u16,
    score_b: u16,
) -> MapSlot {
    MapSlot {
        name: name.to_owned(),
        game_mode_name: mode.to_owned(),
        game_mode_manual: false,
        user_added: false,
        completed,
        score_a,
        score_b,
        chosen_by,
    }
}

fn hero_bans(first_ban: Option<TeamSide>, team_a: &[&str], team_b: &[&str]) -> MapHeroBans {
    MapHeroBans {
        first_ban_team: first_ban,
        team_a: team_a.iter().map(|hero| (*hero).to_owned()).collect(),
        team_b: team_b.iter().map(|hero| (*hero).to_owned()).collect(),
    }
}

// ---------------------------------------------------------------------------
// Tournaments
// ---------------------------------------------------------------------------

fn build_tournaments(state: &AppState) -> Result<Vec<fgc_core::Tournament>> {
    let northwind = find_team(state, "Northwind Sentinels")?;
    let wxm = find_team(state, "王小明战队")?;
    let eight: Vec<TeamId> = std::iter::once(northwind.clone())
        .chain(std::iter::once(wxm.clone()))
        .chain(
            (1..=6)
                .map(|index| find_team(state, &format!("Team {index}")))
                .collect::<Result<Vec<_>>>()?,
        )
        .collect();

    let mut autumn = new_tournament("Autumn Invitational", BracketFormat::DoubleElim);
    autumn.default_first_to = 3;
    autumn.grand_finals_reset = true;
    set_entrants(&mut autumn, eight.clone());
    generate_double_elimination(&mut autumn);

    let mut groups = new_tournament("Group Stage", BracketFormat::RoundRobin);
    groups.default_first_to = 3;
    set_entrants(&mut groups, eight);
    generate_round_robin(&mut groups, 2);

    let mut stress = new_tournament("Board Stress", BracketFormat::SingleElim);
    stress.default_first_to = 3;
    let all: Vec<TeamId> = state.teams.iter().map(|team| team.id.clone()).collect();
    set_entrants(&mut stress, all);
    generate_single_elimination(&mut stress);

    Ok(vec![autumn, groups, stress])
}
