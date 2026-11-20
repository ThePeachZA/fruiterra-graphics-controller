#![allow(dead_code)]

use std::collections::BTreeMap;

use fgc_core::{
    AppState, Asset, AssetCategory, AssetId, AssetOrigin, BracketFormat, BracketMatch,
    BracketMatchStatus, Entrant, Group, GroupId, GroupMatch, MapHeroBans, MapSlot, Match, MatchId,
    MatchStatus, MatchTalent, Player, PlayerId, Position, Role, Round, Scoring, Side, SocialLink,
    SocialPlatform, Standing, Talent, TalentId, Team, TeamId, TeamSide, Tiebreaker, Tournament,
    TournamentId, TournamentStatus,
};

pub const HASH_A: &str = "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa99887766554433221100";
pub const HASH_B: &str = "9b77d0c1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d";

pub fn team(id: &str, name: &str) -> Team {
    Team {
        id: TeamId::from_raw(id),
        name: name.to_owned(),
        tag: "TAG".to_owned(),
        record: Some("0-0".to_owned()),
        color: "#ff0000".to_owned(),
        text_color: "#ffffff".to_owned(),
        aux_color: "#090909".to_owned(),
        logo: None,
        logo_background: true,
        players: Vec::new(),
    }
}

pub fn player(id: &str, name: &str) -> Player {
    Player {
        id: PlayerId::from_raw(id),
        name: name.to_owned(),
        role: Role::Tank,
        position: Position::Core,
        portrait: None,
        featured_hero: None,
        pronouns: None,
    }
}

pub fn talent(id: &str, name: &str) -> Talent {
    Talent {
        id: TalentId::from_raw(id),
        display_name: name.to_owned(),
        real_name: Some("Real Name".to_owned()),
        pronouns: Some("they/them".to_owned()),
        social: Some(SocialLink {
            platform: SocialPlatform::Twitter,
            handle: "@caster".to_owned(),
        }),
        portrait: None,
    }
}

pub fn asset(id: &str, hash: &str, ext: &str) -> Asset {
    Asset {
        id: AssetId::from_raw(id),
        name: "Asset".to_owned(),
        category: AssetCategory::Logo,
        hash: hash.to_owned(),
        ext: ext.to_owned(),
        origin: AssetOrigin::User,
        original_filename: None,
        added_at: None,
        icon: None,
        portrait: None,
        game_mode: None,
        attachment: None,
    }
}

pub fn match_state(id: &str) -> Match {
    Match {
        id: MatchId::from_raw(id),
        title: "Grand Final".to_owned(),
        subtitle: "Season 1".to_owned(),
        scheduled_at: None,
        team_a: Some(TeamId::from_raw("team-a")),
        team_b: Some(TeamId::from_raw("team-b")),
        maps: vec![MapSlot {
            name: "Ilios".to_owned(),
            game_mode_name: "Control".to_owned(),
            game_mode_manual: false,
            user_added: false,
            completed: false,
            score_a: 0,
            score_b: 0,
            chosen_by: Some(TeamSide::A),
        }],
        hero_bans: vec![MapHeroBans {
            first_ban_team: Some(TeamSide::A),
            team_a: vec!["Ana".to_owned()],
            team_b: Vec::new(),
        }],
        map_pool: vec!["Ilios".to_owned()],
        first_to: 3,
        status: MatchStatus::Live,
        current_map: Some(0),
        swap_sides: false,
        scoring: Scoring::Derived,
        side: Side::HomeAttacks,
        talent: MatchTalent::default(),
        loser_picks: false,
        show_player_portraits: true,
    }
}

pub fn bracket_match(id: &str) -> BracketMatch {
    BracketMatch {
        id: MatchId::from_raw(id),
        round: 1,
        number: 1,
        slot1: Some(TeamId::from_raw("team-a")),
        slot2: Some(TeamId::from_raw("team-b")),
        slot1_manual: false,
        slot2_manual: false,
        score1: 0,
        score2: 0,
        winner: None,
        winner_manual: false,
        status: BracketMatchStatus::Pending,
        next: None,
        loser_next: None,
        first_to: 3,
    }
}

pub fn group(id: &str) -> Group {
    Group {
        id: GroupId::from_raw(id),
        name: "Group A".to_owned(),
        team_ids: vec![TeamId::from_raw("team-a"), TeamId::from_raw("team-b")],
    }
}

pub fn group_match(id: &str) -> GroupMatch {
    GroupMatch {
        id: MatchId::from_raw(id),
        group: GroupId::from_raw("group-a"),
        team1: Some(TeamId::from_raw("team-a")),
        team2: Some(TeamId::from_raw("team-b")),
        score1: 0,
        score2: 0,
        status: BracketMatchStatus::Pending,
        first_to: 3,
        winner: None,
        winner_manual: false,
    }
}

fn standing(team: &str) -> Standing {
    Standing {
        team_id: TeamId::from_raw(team),
        played: 0,
        wins: 0,
        losses: 0,
        draws: 0,
        points_for: 0,
        points_against: 0,
        map_differential: 0,
        points: 0,
    }
}

pub fn standings() -> BTreeMap<GroupId, Vec<Standing>> {
    let mut standings = BTreeMap::new();
    standings.insert(
        GroupId::from_raw("group-a"),
        vec![standing("team-a"), standing("team-b")],
    );
    standings
}

pub fn tournament(id: &str) -> Tournament {
    Tournament {
        id: TournamentId::from_raw(id),
        name: "Cup".to_owned(),
        format: BracketFormat::SingleElim,
        entrants: vec![Entrant {
            team_id: TeamId::from_raw("team-a"),
            seed: 1,
        }],
        winners: vec![Round {
            name: "Final".to_owned(),
            matches: vec![bracket_match("bracket-match-a")],
        }],
        losers: Vec::new(),
        grand_final: Vec::new(),
        groups: vec![group("group-a")],
        group_matches: vec![group_match("group-match-a")],
        standings: standings(),
        tiebreakers: vec![
            Tiebreaker::Wins,
            Tiebreaker::HeadToHead,
            Tiebreaker::MapDifferential,
            Tiebreaker::PointsFor,
            Tiebreaker::Seed,
        ],
        playoff_qualifiers: 2,
        playoff_format: BracketFormat::SingleElim,
        champion: None,
        champion_manual: false,
        status: TournamentStatus::Setup,
        default_first_to: 3,
        grand_finals_reset: true,
    }
}

pub fn sample_state() -> AppState {
    let mut state = AppState::default();

    let mut team_a = team("team-a", "Alpha");
    team_a.players.push(player("player-a", "Ace"));
    state.teams.push(team_a);
    state.teams.push(team("team-b", "Bravo"));
    state.talents.push(talent("talent-a", "Caster"));
    state.assets.push(asset("asset-a", HASH_A, "png"));
    state.current_match = Some(match_state("match-a"));
    state.tournaments.push(tournament("tournament-a"));

    state
}
