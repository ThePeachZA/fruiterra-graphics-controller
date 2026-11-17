//! Tournament generation, propagation, standings and board editing.
//!
//! Generation produces a valid starting board, but the board is the source of
//! truth afterwards: every slot, match, round and link can be edited. The
//! helpers in this module are pure so the same rules run in the UI, in tests
//! and during load-time repair.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    BracketFormat, BracketMatch, BracketMatchStatus, Entrant, Group, GroupId, GroupMatch, MatchId,
    MatchLink, Round, Standing, TeamId, Tiebreaker, Tournament, TournamentId, TournamentStatus,
};

/// Tiebreakers used by a new tournament.
pub const DEFAULT_TIEBREAKERS: [Tiebreaker; 5] = [
    Tiebreaker::Wins,
    Tiebreaker::HeadToHead,
    Tiebreaker::MapDifferential,
    Tiebreaker::PointsFor,
    Tiebreaker::Seed,
];

/// Teams advancing from each group by default.
pub const DEFAULT_PLAYOFF_QUALIFIERS: u8 = 2;

/// Smallest bracket this module generates.
pub const MIN_ENTRANTS: usize = 2;

/// Largest entrant count covered by the milestone's acceptance tests.
pub const MAX_GENERATED_ENTRANTS: usize = 32;

/// Default first-to target for new matches.
const DEFAULT_FIRST_TO: u8 = 3;

/// Creates an empty tournament with the documented defaults.
#[must_use]
pub fn new_tournament(name: impl Into<String>, format: BracketFormat) -> Tournament {
    Tournament {
        id: TournamentId::new(),
        name: name.into(),
        format,
        entrants: Vec::new(),
        winners: Vec::new(),
        losers: Vec::new(),
        grand_final: Vec::new(),
        groups: Vec::new(),
        group_matches: Vec::new(),
        standings: BTreeMap::new(),
        tiebreakers: DEFAULT_TIEBREAKERS.to_vec(),
        playoff_qualifiers: DEFAULT_PLAYOFF_QUALIFIERS,
        playoff_format: BracketFormat::SingleElim,
        champion: None,
        champion_manual: false,
        status: TournamentStatus::Setup,
        default_first_to: DEFAULT_FIRST_TO,
        grand_finals_reset: true,
    }
}

// ---------------------------------------------------------------------------
// Entrants and seeds
// ---------------------------------------------------------------------------

/// Replaces the entrant list, assigning seeds in iteration order.
pub fn set_entrants(tournament: &mut Tournament, teams: impl IntoIterator<Item = TeamId>) {
    tournament.entrants = teams
        .into_iter()
        .enumerate()
        .map(|(index, team_id)| Entrant {
            team_id,
            seed: u32::try_from(index + 1).unwrap_or(u32::MAX),
        })
        .collect();
    normalize_seeds(tournament);
}

/// Adds an entrant and returns its seed, or the existing seed when present.
pub fn add_entrant(tournament: &mut Tournament, team_id: TeamId) -> u32 {
    if let Some(existing) = tournament
        .entrants
        .iter()
        .find(|entrant| entrant.team_id == team_id)
    {
        return existing.seed;
    }

    let seed = tournament
        .entrants
        .iter()
        .map(|entrant| entrant.seed)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    tournament.entrants.push(Entrant { team_id, seed });
    seed
}

/// Removes an entrant and normalises the remaining seeds.
pub fn remove_entrant(tournament: &mut Tournament, team_id: &TeamId) -> bool {
    let before = tournament.entrants.len();
    tournament
        .entrants
        .retain(|entrant| &entrant.team_id != team_id);
    normalize_seeds(tournament);
    tournament.entrants.len() != before
}

/// Returns entrants sorted by seed, then identifier for deterministic ties.
#[must_use]
pub fn entrants_in_seed_order(tournament: &Tournament) -> Vec<Entrant> {
    let mut entrants = tournament.entrants.clone();
    entrants.sort_by(|left, right| {
        left.seed
            .cmp(&right.seed)
            .then_with(|| left.team_id.as_str().cmp(right.team_id.as_str()))
    });
    entrants
}

/// Returns entrant team identifiers in seed order.
#[must_use]
pub fn entrant_teams(tournament: &Tournament) -> Vec<TeamId> {
    entrants_in_seed_order(tournament)
        .into_iter()
        .map(|entrant| entrant.team_id)
        .collect()
}

/// Reassigns missing and duplicate seeds so the list is contiguous from 1.
///
/// Relative seed order is preserved: an entrant with the smaller seed keeps
/// the smaller seed, and zero/missing seeds are placed after known seeds in
/// their original list order. Returns the number of seeds that changed.
pub fn normalize_seeds(tournament: &mut Tournament) -> usize {
    let mut order: Vec<usize> = (0..tournament.entrants.len()).collect();
    order.sort_by_key(|&index| {
        let seed = tournament
            .entrants
            .get(index)
            .map_or(0, |entrant| entrant.seed);
        (seed == 0, seed, index)
    });

    let mut changed = 0;
    for (rank, index) in order.into_iter().enumerate() {
        let next = u32::try_from(rank + 1).unwrap_or(u32::MAX);
        if let Some(entrant) = tournament.entrants.get_mut(index)
            && entrant.seed != next
        {
            entrant.seed = next;
            changed += 1;
        }
    }
    changed
}

/// Returns the seed for `team`, or `u32::MAX` when it is not an entrant.
#[must_use]
pub fn seed_of(tournament: &Tournament, team: &TeamId) -> u32 {
    tournament
        .entrants
        .iter()
        .find(|entrant| &entrant.team_id == team)
        .map_or(u32::MAX, |entrant| entrant.seed)
}

// ---------------------------------------------------------------------------
// Seeding and round names
// ---------------------------------------------------------------------------

/// Returns the standard seeding slot order for a power-of-two bracket.
///
/// For eight teams this is `1, 8, 4, 5, 3, 6, 2, 7`, i.e. the documented
/// `1v8, 4v5, 3v6, 2v7` first round.
#[must_use]
pub fn seeding_order(size: usize) -> Vec<u32> {
    let size = size.max(2).next_power_of_two();
    let mut order = Vec::with_capacity(size);
    for (top, bottom) in seeding_pairs(size as u32) {
        order.push(top);
        order.push(bottom);
    }
    order
}

fn seeding_pairs(size: u32) -> Vec<(u32, u32)> {
    if size <= 2 {
        return vec![(1, 2)];
    }

    let previous = seeding_pairs(size / 2);
    let top_half = previous.len().div_ceil(2);
    let mut pairs = Vec::with_capacity(usize::try_from(size / 2).unwrap_or(0));

    for (index, (top, bottom)) in previous.into_iter().enumerate() {
        let expanded = [(top, size + 1 - top), (bottom, size + 1 - bottom)];
        if index < top_half {
            pairs.extend(expanded);
        } else {
            pairs.extend(expanded.into_iter().rev());
        }
    }

    pairs
}

/// Names a winners round based on how many rounds remain.
#[must_use]
pub fn round_name(index: usize, total: usize) -> String {
    match total.saturating_sub(index) {
        0 | 1 => "Final".to_owned(),
        2 => "Semifinal".to_owned(),
        3 => "Quarterfinal".to_owned(),
        _ => format!("Round {}", index + 1),
    }
}

fn bracket_size(entrant_count: usize) -> usize {
    entrant_count.max(2).next_power_of_two()
}

fn log2_size(size: usize) -> usize {
    size.trailing_zeros() as usize
}

fn seeded_slots(teams: &[TeamId], size: usize) -> Vec<Option<TeamId>> {
    seeding_order(size)
        .into_iter()
        .map(|seed| {
            let index = usize::try_from(seed.saturating_sub(1)).ok()?;
            teams.get(index).cloned()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generates a single-elimination board from the tournament's entrants.
pub fn generate_single_elimination(tournament: &mut Tournament) {
    tournament.format = BracketFormat::SingleElim;
    let teams = entrant_teams(tournament);
    generate_elimination_from_teams(tournament, BracketFormat::SingleElim, &teams);
}

/// Generates a double-elimination board from the tournament's entrants.
pub fn generate_double_elimination(tournament: &mut Tournament) {
    tournament.format = BracketFormat::DoubleElim;
    let teams = entrant_teams(tournament);
    generate_elimination_from_teams(tournament, BracketFormat::DoubleElim, &teams);
}

/// Generates an elimination board from an explicit seed-ordered team list.
///
/// The tournament's entrant list is left untouched, which lets the round-robin
/// playoff generator reuse the same bracket code with group qualifiers.
pub fn generate_elimination_from_teams(
    tournament: &mut Tournament,
    format: BracketFormat,
    teams: &[TeamId],
) {
    tournament.winners.clear();
    tournament.losers.clear();
    tournament.grand_final.clear();
    tournament.champion = None;
    tournament.champion_manual = false;
    tournament.status = TournamentStatus::Setup;

    if teams.len() < MIN_ENTRANTS {
        refresh(tournament);
        return;
    }

    let size = bracket_size(teams.len());
    let (rounds, ids) = create_winners_rounds(tournament, teams, size);
    tournament.winners = rounds;

    if format == BracketFormat::DoubleElim {
        create_double_elimination(tournament, size, &ids);
    }

    refresh(tournament);
}

fn create_winners_rounds(
    tournament: &Tournament,
    teams: &[TeamId],
    size: usize,
) -> (Vec<Round>, Vec<Vec<MatchId>>) {
    let slots = seeded_slots(teams, size);
    let round_count = log2_size(size);
    let mut rounds = Vec::with_capacity(round_count);
    let mut ids = Vec::with_capacity(round_count);

    for round_index in 0..round_count {
        let match_count = size >> (round_index + 1);
        let mut round = Round {
            name: round_name(round_index, round_count),
            matches: Vec::with_capacity(match_count),
        };
        let mut round_ids = Vec::with_capacity(match_count);

        for match_index in 0..match_count {
            let mut game = empty_match(
                u32::try_from(round_index + 1).unwrap_or(u32::MAX),
                u32::try_from(match_index + 1).unwrap_or(u32::MAX),
                tournament.default_first_to,
            );

            if round_index == 0 {
                game.slot1 = slots.get(match_index * 2).cloned().flatten();
                game.slot2 = slots.get(match_index * 2 + 1).cloned().flatten();
                apply_initial_bye(&mut game);
            }

            round_ids.push(game.id.clone());
            round.matches.push(game);
        }

        ids.push(round_ids);
        rounds.push(round);
    }

    for round_index in 0..rounds.len().saturating_sub(1) {
        for (match_index, game) in rounds[round_index].matches.iter_mut().enumerate() {
            game.next = Some(MatchLink {
                match_id: ids[round_index + 1][match_index / 2].clone(),
                slot: u8::try_from(match_index % 2 + 1).unwrap_or(1),
            });
        }
    }

    (rounds, ids)
}

fn create_double_elimination(tournament: &mut Tournament, size: usize, winners: &[Vec<MatchId>]) {
    let winners_rounds = log2_size(size);
    let losers_rounds = winners_rounds.saturating_sub(1) * 2;
    let mut losers = Vec::with_capacity(losers_rounds);
    let mut losers_ids = Vec::with_capacity(losers_rounds);

    for round_index in 0..losers_rounds {
        let match_count = losers_match_count(size, round_index);
        let mut round = Round {
            name: format!("Lower Round {}", round_index + 1),
            matches: Vec::with_capacity(match_count),
        };
        let mut ids = Vec::with_capacity(match_count);

        for match_index in 0..match_count {
            let game = empty_match(
                u32::try_from(round_index + 1).unwrap_or(u32::MAX),
                u32::try_from(match_index + 1).unwrap_or(u32::MAX),
                tournament.default_first_to,
            );
            ids.push(game.id.clone());
            round.matches.push(game);
        }

        losers_ids.push(ids);
        losers.push(round);
    }

    let grand_final_match = empty_match(1, 1, tournament.default_first_to);
    let grand_final_id = grand_final_match.id.clone();
    let grand_final_round = Round {
        name: "Grand final".to_owned(),
        matches: vec![grand_final_match],
    };

    // Install the rounds before wiring so `match_mut` can find every target.
    tournament.losers = losers;
    tournament.grand_final = vec![grand_final_round];

    wire_double_elimination(tournament, size, winners, &losers_ids, &grand_final_id);
}

fn wire_double_elimination(
    tournament: &mut Tournament,
    size: usize,
    winners: &[Vec<MatchId>],
    losers: &[Vec<MatchId>],
    grand_final: &MatchId,
) {
    let winners_rounds = log2_size(size);
    let last_winners = winners_rounds.saturating_sub(1);

    for (round_index, round_ids) in winners.iter().enumerate() {
        for (match_index, match_id) in round_ids.iter().enumerate() {
            let Some(game) = match_mut(tournament, match_id) else {
                continue;
            };

            if round_index == last_winners {
                game.next = Some(MatchLink {
                    match_id: grand_final.clone(),
                    slot: 1,
                });
            }

            let loser_target = if winners_rounds == 1 {
                Some((grand_final.clone(), 2))
            } else if round_index == last_winners {
                losers
                    .last()
                    .and_then(|round| round.first())
                    .map(|id| (id.clone(), 2))
            } else if round_index == 0 {
                losers.first().and_then(|round| {
                    round
                        .get(match_index / 2)
                        .map(|id| (id.clone(), u8::try_from(match_index % 2 + 1).unwrap_or(1)))
                })
            } else {
                losers
                    .get(round_index * 2 - 1)
                    .and_then(|round| round.get(match_index))
                    .map(|id| (id.clone(), 2))
            };

            game.loser_next = loser_target.map(|(match_id, slot)| MatchLink { match_id, slot });
        }
    }

    for (round_index, round_ids) in losers.iter().enumerate() {
        for (match_index, match_id) in round_ids.iter().enumerate() {
            let Some(game) = match_mut(tournament, match_id) else {
                continue;
            };

            let next = if round_index + 1 == losers.len() {
                Some((grand_final.clone(), 2))
            } else if (round_index + 1) % 2 == 1 {
                losers
                    .get(round_index + 1)
                    .and_then(|round| round.get(match_index))
                    .map(|id| (id.clone(), 1))
            } else {
                losers.get(round_index + 1).and_then(|round| {
                    round
                        .get(match_index / 2)
                        .map(|id| (id.clone(), u8::try_from(match_index % 2 + 1).unwrap_or(1)))
                })
            };

            game.next = next.map(|(match_id, slot)| MatchLink { match_id, slot });
        }
    }
}

fn losers_match_count(size: usize, index: usize) -> usize {
    if index == 0 {
        size / 4
    } else if index % 2 == 1 {
        size >> ((index + 1).div_ceil(2) + 1)
    } else {
        size >> ((index / 2) + 2)
    }
}

fn empty_match(round: u32, number: u32, first_to: u8) -> BracketMatch {
    BracketMatch {
        id: MatchId::new(),
        round,
        number,
        slot1: None,
        slot2: None,
        slot1_manual: false,
        slot2_manual: false,
        score1: 0,
        score2: 0,
        winner: None,
        winner_manual: false,
        status: BracketMatchStatus::Pending,
        next: None,
        loser_next: None,
        first_to,
    }
}

fn apply_initial_bye(game: &mut BracketMatch) {
    match (&game.slot1, &game.slot2) {
        (Some(team), None) | (None, Some(team)) => {
            game.status = BracketMatchStatus::Completed;
            game.winner = Some(team.clone());
            game.score1 = 0;
            game.score2 = 0;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Round robin
// ---------------------------------------------------------------------------

/// Creates a group, generates its fixtures and returns the group identifier.
pub fn create_group(
    tournament: &mut Tournament,
    name: impl Into<String>,
    team_ids: Vec<TeamId>,
) -> GroupId {
    let id = GroupId::new();
    tournament.groups.push(Group {
        id: id.clone(),
        name: name.into(),
        team_ids,
    });
    sync_group_fixtures(tournament, &id);
    refresh(tournament);
    id
}

/// Splits the entrants into evenly sized groups and generates round-robin
/// fixtures for each group.
pub fn generate_round_robin(tournament: &mut Tournament, group_count: usize) {
    tournament.format = BracketFormat::RoundRobin;
    let teams = entrant_teams(tournament);

    tournament.winners.clear();
    tournament.losers.clear();
    tournament.grand_final.clear();
    tournament.groups.clear();
    tournament.group_matches.clear();
    tournament.standings.clear();
    tournament.champion = None;
    tournament.champion_manual = false;
    tournament.status = TournamentStatus::Setup;

    if teams.is_empty() {
        refresh(tournament);
        return;
    }

    let count = group_count.clamp(1, teams.len());
    let mut groups: Vec<Group> = (0..count)
        .map(|index| Group {
            id: GroupId::new(),
            name: group_name(index),
            team_ids: Vec::new(),
        })
        .collect();

    for (index, team) in teams.into_iter().enumerate() {
        groups[index % count].team_ids.push(team);
    }

    tournament.groups = groups;
    sync_all_group_fixtures(tournament);
    refresh(tournament);
}

fn group_name(index: usize) -> String {
    let letter = u8::try_from(index)
        .ok()
        .and_then(|value| (value < 26).then(|| char::from(b'A'.saturating_add(value))));
    letter.map_or_else(
        || format!("Group {}", index + 1),
        |letter| format!("Group {letter}"),
    )
}

/// Regenerates missing fixtures for every group.
pub fn sync_all_group_fixtures(tournament: &mut Tournament) {
    let ids: Vec<GroupId> = tournament
        .groups
        .iter()
        .map(|group| group.id.clone())
        .collect();
    for id in ids {
        sync_group_fixtures(tournament, &id);
    }
}

/// Adds missing fixtures and removes unplayed fixtures for a group.
pub fn sync_group_fixtures(tournament: &mut Tournament, group_id: &GroupId) {
    let Some(group) = tournament
        .groups
        .iter()
        .find(|group| &group.id == group_id)
        .cloned()
    else {
        return;
    };

    let team_set: HashSet<TeamId> = group.team_ids.iter().cloned().collect();
    let mut kept = Vec::new();
    let mut existing_pairs = HashSet::new();

    for fixture in tournament
        .group_matches
        .iter()
        .filter(|fixture| &fixture.group == group_id)
    {
        let mut fixture = fixture.clone();
        if fixture
            .team1
            .as_ref()
            .is_some_and(|team| !team_set.contains(team))
        {
            if fixture.status != BracketMatchStatus::Completed {
                continue;
            }
            fixture.team1 = None;
        }
        if fixture
            .team2
            .as_ref()
            .is_some_and(|team| !team_set.contains(team))
        {
            if fixture.status != BracketMatchStatus::Completed {
                continue;
            }
            fixture.team2 = None;
        }
        if fixture.team1.is_none()
            && fixture.team2.is_none()
            && fixture.status != BracketMatchStatus::Completed
        {
            continue;
        }

        if let (Some(team1), Some(team2)) = (&fixture.team1, &fixture.team2) {
            existing_pairs.insert(pair_key(team1, team2));
        }
        kept.push(fixture);
    }

    for (team1, team2) in round_robin_pairs(&group.team_ids) {
        if existing_pairs.insert(pair_key(&team1, &team2)) {
            kept.push(GroupMatch {
                id: MatchId::new(),
                group: group.id.clone(),
                team1: Some(team1),
                team2: Some(team2),
                score1: 0,
                score2: 0,
                status: BracketMatchStatus::Pending,
                first_to: tournament.default_first_to,
            });
        }
    }

    tournament
        .group_matches
        .retain(|fixture| &fixture.group != group_id);
    tournament.group_matches.extend(kept);
}

/// Renames a group.
pub fn rename_group(
    tournament: &mut Tournament,
    group_id: &GroupId,
    name: impl Into<String>,
) -> bool {
    let Some(group) = tournament
        .groups
        .iter_mut()
        .find(|group| &group.id == group_id)
    else {
        return false;
    };
    group.name = name.into();
    true
}

/// Replaces a group's team list, preserving order and dropping duplicates.
///
/// Missing fixtures are generated and unplayed fixtures for removed teams are
/// dropped; played fixtures stay as records with an empty slot.
pub fn set_group_teams(
    tournament: &mut Tournament,
    group_id: &GroupId,
    team_ids: Vec<TeamId>,
) -> bool {
    let Some(group) = tournament
        .groups
        .iter_mut()
        .find(|group| &group.id == group_id)
    else {
        return false;
    };

    let mut seen = HashSet::new();
    group.team_ids = team_ids
        .into_iter()
        .filter(|team| seen.insert(team.clone()))
        .collect();
    sync_group_fixtures(tournament, group_id);
    refresh(tournament);
    true
}

/// Removes a group and all of its fixtures.
pub fn remove_group(tournament: &mut Tournament, group_id: &GroupId) -> bool {
    let before = tournament.groups.len();
    tournament.groups.retain(|group| &group.id != group_id);
    tournament
        .group_matches
        .retain(|fixture| &fixture.group != group_id);
    refresh(tournament);
    tournament.groups.len() != before
}

/// Adds a group fixture by hand.
///
/// Returns `None` when the group is missing, both sides are the same team, or
/// a team is not a member of the group.
pub fn add_group_fixture(
    tournament: &mut Tournament,
    group_id: &GroupId,
    team1: TeamId,
    team2: TeamId,
) -> Option<MatchId> {
    let group = tournament
        .groups
        .iter()
        .find(|group| &group.id == group_id)?;
    if team1 == team2 || !group.team_ids.contains(&team1) || !group.team_ids.contains(&team2) {
        return None;
    }

    let fixture = GroupMatch {
        id: MatchId::new(),
        group: group_id.clone(),
        team1: Some(team1),
        team2: Some(team2),
        score1: 0,
        score2: 0,
        status: BracketMatchStatus::Pending,
        first_to: tournament.default_first_to,
    };
    let id = fixture.id.clone();
    tournament.group_matches.push(fixture);
    refresh(tournament);
    Some(id)
}

/// Removes a group fixture by hand.
pub fn remove_group_fixture(tournament: &mut Tournament, fixture_id: &MatchId) -> bool {
    let before = tournament.group_matches.len();
    tournament
        .group_matches
        .retain(|fixture| &fixture.id != fixture_id);
    let removed = tournament.group_matches.len() != before;
    if removed {
        refresh(tournament);
    }
    removed
}

/// Sets a group fixture's scores and status.
pub fn set_group_result(
    tournament: &mut Tournament,
    fixture_id: &MatchId,
    score1: u16,
    score2: u16,
    status: BracketMatchStatus,
) -> bool {
    let Some(fixture) = tournament
        .group_matches
        .iter_mut()
        .find(|fixture| &fixture.id == fixture_id)
    else {
        return false;
    };
    fixture.score1 = score1;
    fixture.score2 = score2;
    fixture.status = status;
    refresh(tournament);
    true
}

fn pair_key(team1: &TeamId, team2: &TeamId) -> (String, String) {
    let first = team1.as_str();
    let second = team2.as_str();
    if first <= second {
        (first.to_owned(), second.to_owned())
    } else {
        (second.to_owned(), first.to_owned())
    }
}

/// Returns every unordered pairing using the circle method.
#[must_use]
pub fn round_robin_pairs(team_ids: &[TeamId]) -> Vec<(TeamId, TeamId)> {
    if team_ids.len() < 2 {
        return Vec::new();
    }

    let mut teams: Vec<Option<TeamId>> = team_ids.iter().cloned().map(Some).collect();
    if teams.len() % 2 == 1 {
        teams.push(None);
    }

    let count = teams.len();
    let mut pairs = Vec::new();

    for round in 0..count.saturating_sub(1) {
        for index in 0..count / 2 {
            let left = teams.get(index).cloned().flatten();
            let right = teams.get(count - 1 - index).cloned().flatten();
            if let (Some(left), Some(right)) = (left, right) {
                if round % 2 == 0 {
                    pairs.push((left, right));
                } else {
                    pairs.push((right, left));
                }
            }
        }

        if let Some(last) = teams.pop() {
            teams.insert(1, last);
        }
    }

    pairs
}

/// Generates a playoff bracket from the group standings.
///
/// # Errors
///
/// Returns [`PlayoffError::NoGroups`] when there is no group stage, and
/// [`PlayoffError::NotEnoughQualifiers`] when fewer than two teams would
/// advance.
pub fn generate_playoff(tournament: &mut Tournament) -> Result<(), PlayoffError> {
    if tournament.groups.is_empty() {
        return Err(PlayoffError::NoGroups);
    }

    recompute_standings(tournament);
    let qualifiers_per_group = usize::from(tournament.playoff_qualifiers.max(1));
    let mut qualifiers = Vec::new();

    for rank in 0..qualifiers_per_group {
        for group in &tournament.groups {
            let Some(row) = tournament
                .standings
                .get(&group.id)
                .and_then(|rows| rows.get(rank))
            else {
                continue;
            };
            if !qualifiers.contains(&row.team_id) {
                qualifiers.push(row.team_id.clone());
            }
        }
    }

    if qualifiers.len() < MIN_ENTRANTS {
        return Err(PlayoffError::NotEnoughQualifiers);
    }

    let format = if tournament.playoff_format == BracketFormat::DoubleElim {
        BracketFormat::DoubleElim
    } else {
        BracketFormat::SingleElim
    };
    generate_elimination_from_teams(tournament, format, &qualifiers);
    Ok(())
}

// ---------------------------------------------------------------------------
// Standings and tiebreakers
// ---------------------------------------------------------------------------

/// Recomputes and caches standings for every group.
pub fn recompute_standings(tournament: &mut Tournament) {
    let standings = tournament
        .groups
        .iter()
        .map(|group| (group.id.clone(), compute_standings(tournament, group)))
        .collect();
    tournament.standings = standings;
}

/// Computes sorted standings for one group from completed fixtures.
#[must_use]
pub fn compute_standings(tournament: &Tournament, group: &Group) -> Vec<Standing> {
    let mut rows: BTreeMap<TeamId, Standing> = group
        .team_ids
        .iter()
        .map(|team_id| (team_id.clone(), empty_standing(team_id.clone())))
        .collect();

    for fixture in &tournament.group_matches {
        if fixture.group != group.id || fixture.status != BracketMatchStatus::Completed {
            continue;
        }
        let (Some(team1), Some(team2)) = (&fixture.team1, &fixture.team2) else {
            continue;
        };
        if team1 == team2 || !rows.contains_key(team1) || !rows.contains_key(team2) {
            continue;
        }

        if let Some(row) = rows.get_mut(team1) {
            row.played = row.played.saturating_add(1);
            row.points_for = row.points_for.saturating_add(u32::from(fixture.score1));
            row.points_against = row.points_against.saturating_add(u32::from(fixture.score2));
            match fixture.score1.cmp(&fixture.score2) {
                Ordering::Greater => row.wins = row.wins.saturating_add(1),
                Ordering::Less => row.losses = row.losses.saturating_add(1),
                Ordering::Equal => row.draws = row.draws.saturating_add(1),
            }
        }

        if let Some(row) = rows.get_mut(team2) {
            row.played = row.played.saturating_add(1);
            row.points_for = row.points_for.saturating_add(u32::from(fixture.score2));
            row.points_against = row.points_against.saturating_add(u32::from(fixture.score1));
            match fixture.score2.cmp(&fixture.score1) {
                Ordering::Greater => row.wins = row.wins.saturating_add(1),
                Ordering::Less => row.losses = row.losses.saturating_add(1),
                Ordering::Equal => row.draws = row.draws.saturating_add(1),
            }
        }
    }

    let mut rows: Vec<Standing> = rows.into_values().collect();
    for row in &mut rows {
        row.map_differential = i32::try_from(row.points_for).unwrap_or(i32::MAX)
            - i32::try_from(row.points_against).unwrap_or(i32::MAX);
        row.points = row.wins;
    }

    sort_standings_rows(tournament, group, &mut rows);
    rows
}

fn empty_standing(team_id: TeamId) -> Standing {
    Standing {
        team_id,
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

fn sort_standings_rows(tournament: &Tournament, group: &Group, rows: &mut [Standing]) {
    let tiebreakers = tournament.tiebreakers.clone();
    sort_rows_recursive(tournament, group, rows, &tiebreakers);
}

fn sort_rows_recursive(
    tournament: &Tournament,
    group: &Group,
    rows: &mut [Standing],
    criteria: &[Tiebreaker],
) {
    if rows.len() <= 1 {
        return;
    }

    for (index, criterion) in criteria.iter().enumerate() {
        let keys: Vec<i64> = rows
            .iter()
            .map(|row| criterion_key(tournament, group, rows, row, *criterion))
            .collect();
        let first = keys.first().copied().unwrap_or_default();
        if keys.iter().all(|key| *key == first) {
            continue;
        }

        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|left, right| compare_key(*criterion, keys[*left], keys[*right]));

        let mut groups: Vec<Vec<Standing>> = Vec::new();
        let mut current_key: Option<i64> = None;
        for row_index in order {
            let key = keys.get(row_index).copied().unwrap_or_default();
            if current_key != Some(key) {
                groups.push(Vec::new());
                current_key = Some(key);
            }
            if let Some(group_rows) = groups.last_mut()
                && let Some(row) = rows.get(row_index)
            {
                group_rows.push(row.clone());
            }
        }

        let remaining = criteria.get(index + 1..).unwrap_or_default();
        let mut sorted = Vec::with_capacity(rows.len());
        for mut group_rows in groups {
            sort_rows_recursive(tournament, group, &mut group_rows, remaining);
            sorted.extend(group_rows);
        }
        rows.clone_from_slice(&sorted);
        return;
    }

    rows.sort_by(|left, right| {
        seed_of(tournament, &left.team_id)
            .cmp(&seed_of(tournament, &right.team_id))
            .then_with(|| left.team_id.as_str().cmp(right.team_id.as_str()))
    });
}

fn compare_key(criterion: Tiebreaker, left: i64, right: i64) -> Ordering {
    match criterion {
        Tiebreaker::PointsAgainst | Tiebreaker::Seed => left.cmp(&right),
        _ => right.cmp(&left),
    }
}

fn criterion_key(
    tournament: &Tournament,
    group: &Group,
    tied: &[Standing],
    row: &Standing,
    criterion: Tiebreaker,
) -> i64 {
    match criterion {
        Tiebreaker::Wins => i64::from(row.wins),
        Tiebreaker::HeadToHead => {
            i64::from(head_to_head_wins(tournament, group, tied, &row.team_id))
        }
        Tiebreaker::MapDifferential => i64::from(row.map_differential),
        Tiebreaker::MapsWon | Tiebreaker::PointsFor => i64::from(row.points_for),
        Tiebreaker::PointsAgainst => i64::from(row.points_against),
        Tiebreaker::Seed => i64::from(seed_of(tournament, &row.team_id)),
    }
}

fn head_to_head_wins(
    tournament: &Tournament,
    group: &Group,
    tied: &[Standing],
    team: &TeamId,
) -> u32 {
    let tied_teams: HashSet<&TeamId> = tied.iter().map(|row| &row.team_id).collect();
    let mut wins = 0;

    for fixture in &tournament.group_matches {
        if fixture.group != group.id || fixture.status != BracketMatchStatus::Completed {
            continue;
        }
        let (Some(team1), Some(team2)) = (&fixture.team1, &fixture.team2) else {
            continue;
        };
        if team1 == team2 || !tied_teams.contains(team1) || !tied_teams.contains(team2) {
            continue;
        }
        if (team1 == team && fixture.score1 > fixture.score2)
            || (team2 == team && fixture.score2 > fixture.score1)
        {
            wins += 1;
        }
    }

    wins
}

// ---------------------------------------------------------------------------
// Propagation, champion and reset
// ---------------------------------------------------------------------------

/// Recomputes every derived tournament value.
///
/// This is intentionally idempotent: board edits only change the raw fields,
/// then call this helper to advance winners, repair links, refresh standings
/// and update the champion.
pub fn refresh(tournament: &mut Tournament) {
    normalize_seeds(tournament);
    repair_links(tournament);
    repair_group_fixtures(tournament);
    reset_invalid_results(tournament);
    fill_structural_byes(tournament);

    let iterations = match_count(tournament).saturating_add(2);
    for _ in 0..iterations {
        let propagated = propagate_once(tournament);
        let reset = reset_invalid_results(tournament);
        let byes = fill_structural_byes(tournament);
        if !propagated && !reset && !byes {
            break;
        }
    }

    sync_grand_final_reset(tournament);
    recompute_standings(tournament);
    sync_champion(tournament);
}

fn match_count(tournament: &Tournament) -> usize {
    tournament
        .winners
        .iter()
        .chain(tournament.losers.iter())
        .chain(tournament.grand_final.iter())
        .map(|round| round.matches.len())
        .sum()
}

fn effective_winner(game: &BracketMatch) -> Option<&TeamId> {
    if game.status != BracketMatchStatus::Completed {
        return None;
    }
    let winner = game.winner.as_ref()?;
    if game.slot1.as_ref() == Some(winner) || game.slot2.as_ref() == Some(winner) {
        Some(winner)
    } else {
        None
    }
}

fn effective_loser(game: &BracketMatch) -> Option<&TeamId> {
    let winner = effective_winner(game)?;
    if game.slot1.as_ref() == Some(winner) {
        game.slot2.as_ref()
    } else if game.slot2.as_ref() == Some(winner) {
        game.slot1.as_ref()
    } else {
        None
    }
}

fn propagate_once(tournament: &mut Tournament) -> bool {
    let ids = all_match_ids(tournament);
    let mut changed = false;

    for id in ids {
        let (next, loser_next, winner, loser) = {
            let Some(game) = find_match(tournament, &id) else {
                continue;
            };
            (
                game.next.clone(),
                game.loser_next.clone(),
                effective_winner(game).cloned(),
                effective_loser(game).cloned(),
            )
        };

        if let Some(link) = next
            && let Some(target) = match_mut(tournament, &link.match_id)
            && !slot_is_manual(target, link.slot)
            && set_slot_value(target, link.slot, winner)
        {
            changed = true;
        }

        if let Some(link) = loser_next
            && let Some(target) = match_mut(tournament, &link.match_id)
            && !slot_is_manual(target, link.slot)
            && set_slot_value(target, link.slot, loser)
        {
            changed = true;
        }
    }

    changed
}

fn reset_invalid_results(tournament: &mut Tournament) -> bool {
    let mut changed = false;
    for_each_match_mut(tournament, |game| {
        if game.status == BracketMatchStatus::Completed && effective_winner(game).is_none() {
            game.status = BracketMatchStatus::Pending;
            game.winner = None;
            game.winner_manual = false;
            changed = true;
        }
    });
    changed
}

fn fill_structural_byes(tournament: &mut Tournament) -> bool {
    let mut incoming: HashMap<(MatchId, u8), (MatchId, LinkKind)> = HashMap::new();
    for_each_match(tournament, |game| {
        if let Some(link) = &game.next {
            incoming.insert(
                (link.match_id.clone(), link.slot),
                (game.id.clone(), LinkKind::Winner),
            );
        }
        if let Some(link) = &game.loser_next {
            incoming.insert(
                (link.match_id.clone(), link.slot),
                (game.id.clone(), LinkKind::Loser),
            );
        }
    });

    let mut byes = Vec::new();
    for_each_match(tournament, |game| {
        if game.status != BracketMatchStatus::Pending {
            return;
        }
        let filled = usize::from(game.slot1.is_some()) + usize::from(game.slot2.is_some());
        if filled != 1 {
            return;
        }

        let empty_slot = if game.slot1.is_none() { 1 } else { 2 };
        let manual = if empty_slot == 1 {
            game.slot1_manual
        } else {
            game.slot2_manual
        };
        if manual {
            return;
        }

        let Some((source_id, kind)) = incoming.get(&(game.id.clone(), empty_slot)) else {
            return;
        };
        let Some(source) = find_match(tournament, source_id) else {
            return;
        };
        if source.status != BracketMatchStatus::Completed {
            return;
        }
        let source_team = match kind {
            LinkKind::Winner => effective_winner(source),
            LinkKind::Loser => effective_loser(source),
        };
        if source_team.is_none() {
            byes.push(game.id.clone());
        }
    });

    let mut changed = false;
    for id in byes {
        if let Some(game) = match_mut(tournament, &id)
            && game.status == BracketMatchStatus::Pending
        {
            game.status = BracketMatchStatus::Completed;
            game.winner = game.slot1.clone().or_else(|| game.slot2.clone());
            game.score1 = 0;
            game.score2 = 0;
            game.winner_manual = false;
            changed = true;
        }
    }

    changed
}

fn sync_grand_final_reset(tournament: &mut Tournament) {
    if tournament.grand_final.is_empty() {
        return;
    }

    let (slot1, slot2, winner, completed) = {
        let Some(game) = tournament
            .grand_final
            .first()
            .and_then(|round| round.matches.first())
        else {
            return;
        };
        (
            game.slot1.clone(),
            game.slot2.clone(),
            effective_winner(game).cloned(),
            game.status == BracketMatchStatus::Completed,
        )
    };

    let losers_team_won = completed && winner.is_some() && winner == slot2 && slot2.is_some();
    let should_have_reset = tournament.grand_finals_reset && losers_team_won;

    if should_have_reset {
        if tournament.grand_final.len() < 2 {
            let reset = empty_match(1, 1, tournament.default_first_to);
            tournament.grand_final.push(Round {
                name: "Reset".to_owned(),
                matches: vec![reset],
            });
        }

        if let Some(reset) = tournament
            .grand_final
            .get_mut(1)
            .and_then(|round| round.matches.first_mut())
        {
            if !reset.slot1_manual {
                reset.slot1 = slot1;
            }
            if !reset.slot2_manual {
                reset.slot2 = slot2;
            }
            if reset.status != BracketMatchStatus::Completed
                && !reset.slot1_manual
                && !reset.slot2_manual
            {
                reset.status = BracketMatchStatus::Pending;
                reset.winner = None;
                reset.score1 = 0;
                reset.score2 = 0;
            }
        }
        return;
    }

    if tournament.grand_final.len() <= 1 {
        return;
    }

    let keep_reset = tournament
        .grand_final
        .get(1)
        .and_then(|round| round.matches.first())
        .is_some_and(|reset| reset.slot1_manual || reset.slot2_manual);
    if !keep_reset {
        tournament.grand_final.truncate(1);
    }
}

fn compute_champion(tournament: &Tournament) -> Option<TeamId> {
    if let Some(reset) = tournament
        .grand_final
        .get(1)
        .and_then(|round| round.matches.first())
        && let Some(winner) = effective_winner(reset)
    {
        return Some(winner.clone());
    }

    if let Some(game) = tournament
        .grand_final
        .first()
        .and_then(|round| round.matches.first())
    {
        let winner = effective_winner(game)?;
        if game.slot1.as_ref() == Some(winner) {
            return Some(winner.clone());
        }
        if game.slot2.as_ref() == Some(winner) {
            if tournament.grand_finals_reset {
                return None;
            }
            return Some(winner.clone());
        }
        return None;
    }

    tournament
        .winners
        .last()
        .and_then(|round| round.matches.last())
        .and_then(effective_winner)
        .cloned()
}

fn sync_champion(tournament: &mut Tournament) {
    let previous = tournament.champion.clone();
    let candidate = if tournament.champion_manual {
        tournament.champion.clone()
    } else {
        compute_champion(tournament)
    };
    let next = candidate.filter(|team| {
        tournament
            .entrants
            .iter()
            .any(|entrant| &entrant.team_id == team)
    });

    tournament.champion = next.clone();
    if tournament.champion_manual && tournament.champion.is_none() {
        tournament.champion_manual = false;
    }
    if previous.is_none() && next.is_some() && tournament.status != TournamentStatus::Completed {
        tournament.status = TournamentStatus::Completed;
    }
}

// ---------------------------------------------------------------------------
// Repair
// ---------------------------------------------------------------------------

/// A repair applied to a tournament while loading or editing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TournamentRepairKind {
    /// A missing or duplicate seed was reassigned.
    DuplicateSeed,
    /// A completed match had no valid winner and was reset.
    CompletedWithoutWinner,
    /// A link pointed at a missing match, itself or an invalid slot.
    DanglingLink,
    /// A group fixture referenced a team outside its group.
    GroupTeamOutsideGroup,
    /// The champion was not an entrant.
    ChampionNotEntrant,
    /// Links formed a cycle and were cleared.
    LinkCycle,
}

/// One repair applied to a tournament.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TournamentRepair {
    /// Repair category.
    pub kind: TournamentRepairKind,
    /// Human-readable detail.
    pub message: String,
}

/// Repairs malformed tournament data and refreshes derived values.
pub fn repair_tournament(tournament: &mut Tournament) -> Vec<TournamentRepair> {
    let mut repairs = Vec::new();

    let duplicate_seeds = duplicate_seed_count(tournament);
    if duplicate_seeds > 0 {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::DuplicateSeed,
            message: format!("{duplicate_seeds} duplicate or missing seed(s) were reassigned"),
        });
    }

    let invalid_results = invalid_result_count(tournament);
    if invalid_results > 0 {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::CompletedWithoutWinner,
            message: format!(
                "{invalid_results} completed match(es) without a valid winner were reset"
            ),
        });
    }

    let dangling = dangling_link_count(tournament);
    if dangling > 0 {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::DanglingLink,
            message: format!("{dangling} dangling or self-referential link(s) were cleared"),
        });
    }

    let cycles = cycle_count(tournament);
    if cycles > 0 {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::LinkCycle,
            message: format!("{cycles} link cycle(s) were broken"),
        });
    }

    let outside = group_fixture_outside_count(tournament);
    if outside > 0 {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::GroupTeamOutsideGroup,
            message: format!("{outside} group fixture slot(s) referenced a team outside the group"),
        });
    }

    if let Some(champion) = &tournament.champion
        && !tournament
            .entrants
            .iter()
            .any(|entrant| &entrant.team_id == champion)
    {
        repairs.push(TournamentRepair {
            kind: TournamentRepairKind::ChampionNotEntrant,
            message: "the champion was not an entrant and was cleared".to_owned(),
        });
    }

    refresh(tournament);
    repairs
}

fn duplicate_seed_count(tournament: &Tournament) -> usize {
    let mut used = HashSet::new();
    tournament
        .entrants
        .iter()
        .filter(|entrant| entrant.seed == 0 || !used.insert(entrant.seed))
        .count()
}

fn invalid_result_count(tournament: &Tournament) -> usize {
    let mut count = 0;
    for_each_match(tournament, |game| {
        if game.status == BracketMatchStatus::Completed && effective_winner(game).is_none() {
            count += 1;
        }
    });
    count
}

fn dangling_link_count(tournament: &Tournament) -> usize {
    let mut count = 0;
    for_each_match(tournament, |game| {
        for link in [&game.next, &game.loser_next].into_iter().flatten() {
            let valid = matches!(link.slot, 1 | 2)
                && link.match_id != game.id
                && find_match(tournament, &link.match_id).is_some();
            if !valid {
                count += 1;
            }
        }
    });
    count
}

fn group_fixture_outside_count(tournament: &Tournament) -> usize {
    let mut count = 0;
    for fixture in &tournament.group_matches {
        let Some(group) = tournament
            .groups
            .iter()
            .find(|group| group.id == fixture.group)
        else {
            count += 1;
            continue;
        };
        if fixture
            .team1
            .as_ref()
            .is_some_and(|team| !group.team_ids.contains(team))
        {
            count += 1;
        }
        if fixture
            .team2
            .as_ref()
            .is_some_and(|team| !group.team_ids.contains(team))
        {
            count += 1;
        }
    }
    count
}

fn cycle_count(tournament: &Tournament) -> usize {
    let edges = link_edges(tournament);
    let mut colors: HashMap<MatchId, u8> = HashMap::new();
    let mut cycles = 0;
    for id in edges.keys() {
        if colors.get(id).copied().unwrap_or(0) == 0 {
            count_cycles_from(id, &edges, &mut colors, &mut cycles);
        }
    }
    cycles
}

fn count_cycles_from(
    node: &MatchId,
    edges: &HashMap<MatchId, Vec<(MatchId, LinkKind)>>,
    colors: &mut HashMap<MatchId, u8>,
    cycles: &mut usize,
) {
    colors.insert(node.clone(), 1);
    if let Some(nexts) = edges.get(node) {
        for (next, _) in nexts {
            match colors.get(next).copied().unwrap_or(0) {
                1 => *cycles += 1,
                0 => count_cycles_from(next, edges, colors, cycles),
                _ => {}
            }
        }
    }
    colors.insert(node.clone(), 2);
}

fn repair_links(tournament: &mut Tournament) {
    let ids: HashSet<MatchId> = all_match_ids(tournament).into_iter().collect();
    let mut clear: Vec<(MatchId, LinkKind)> = Vec::new();

    for_each_match(tournament, |game| {
        for (kind, link) in [
            (LinkKind::Winner, &game.next),
            (LinkKind::Loser, &game.loser_next),
        ] {
            if let Some(link) = link
                && (!ids.contains(&link.match_id)
                    || link.match_id == game.id
                    || !matches!(link.slot, 1 | 2))
            {
                clear.push((game.id.clone(), kind));
            }
        }
    });

    for (id, kind) in clear {
        clear_link(tournament, &id, kind);
    }

    // A slot can only hold one team, so keep the first link (in board order)
    // that feeds each slot and drop any later duplicate.
    let mut fed_slots: HashSet<(MatchId, u8)> = HashSet::new();
    let mut duplicates: Vec<(MatchId, LinkKind)> = Vec::new();
    for_each_match(tournament, |game| {
        for (kind, link) in [
            (LinkKind::Winner, &game.next),
            (LinkKind::Loser, &game.loser_next),
        ] {
            if let Some(link) = link
                && !fed_slots.insert((link.match_id.clone(), link.slot))
            {
                duplicates.push((game.id.clone(), kind));
            }
        }
    });
    for (id, kind) in duplicates {
        clear_link(tournament, &id, kind);
    }

    loop {
        let edges = link_edges(tournament);
        let mut colors: HashMap<MatchId, u8> = HashMap::new();
        let mut to_clear: HashSet<(MatchId, LinkKind)> = HashSet::new();
        for id in edges.keys() {
            if colors.get(id).copied().unwrap_or(0) == 0 {
                collect_back_edges(id, &edges, &mut colors, &mut to_clear);
            }
        }
        if to_clear.is_empty() {
            break;
        }
        for (id, kind) in to_clear {
            clear_link(tournament, &id, kind);
        }
    }
}

fn link_edges(tournament: &Tournament) -> HashMap<MatchId, Vec<(MatchId, LinkKind)>> {
    let mut edges: HashMap<MatchId, Vec<(MatchId, LinkKind)>> = HashMap::new();
    for_each_match(tournament, |game| {
        if let Some(link) = &game.next {
            edges
                .entry(game.id.clone())
                .or_default()
                .push((link.match_id.clone(), LinkKind::Winner));
        }
        if let Some(link) = &game.loser_next {
            edges
                .entry(game.id.clone())
                .or_default()
                .push((link.match_id.clone(), LinkKind::Loser));
        }
    });
    edges
}

fn collect_back_edges(
    node: &MatchId,
    edges: &HashMap<MatchId, Vec<(MatchId, LinkKind)>>,
    colors: &mut HashMap<MatchId, u8>,
    to_clear: &mut HashSet<(MatchId, LinkKind)>,
) {
    colors.insert(node.clone(), 1);
    if let Some(nexts) = edges.get(node) {
        for (next, kind) in nexts {
            match colors.get(next).copied().unwrap_or(0) {
                1 => {
                    to_clear.insert((node.clone(), *kind));
                }
                0 => collect_back_edges(next, edges, colors, to_clear),
                _ => {}
            }
        }
    }
    colors.insert(node.clone(), 2);
}

fn clear_link(tournament: &mut Tournament, id: &MatchId, kind: LinkKind) {
    if let Some(game) = match_mut(tournament, id) {
        match kind {
            LinkKind::Winner => game.next = None,
            LinkKind::Loser => game.loser_next = None,
        }
    }
}

fn repair_group_fixtures(tournament: &mut Tournament) {
    let mut groups = tournament.groups.clone();
    for group in &mut groups {
        let mut seen = HashSet::new();
        group.team_ids.retain(|team| seen.insert(team.clone()));
    }
    tournament.groups = groups;

    let group_ids: HashSet<GroupId> = tournament
        .groups
        .iter()
        .map(|group| group.id.clone())
        .collect();
    tournament
        .group_matches
        .retain(|fixture| group_ids.contains(&fixture.group));

    for fixture in &mut tournament.group_matches {
        let Some(group) = tournament
            .groups
            .iter()
            .find(|group| group.id == fixture.group)
        else {
            continue;
        };
        if fixture
            .team1
            .as_ref()
            .is_some_and(|team| !group.team_ids.contains(team))
        {
            fixture.team1 = None;
        }
        if fixture
            .team2
            .as_ref()
            .is_some_and(|team| !group.team_ids.contains(team))
        {
            fixture.team2 = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Board addresses and operations
// ---------------------------------------------------------------------------

/// One bracket section on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BracketSection {
    /// Winners bracket.
    Winners,
    /// Losers bracket.
    Losers,
    /// Grand final and reset.
    GrandFinal,
}

impl BracketSection {
    /// Every section in board order.
    pub const ALL: [Self; 3] = [Self::Winners, Self::Losers, Self::GrandFinal];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Winners => "Winners",
            Self::Losers => "Losers",
            Self::GrandFinal => "Grand final",
        }
    }
}

/// One slot on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BracketSlot {
    /// Section containing the match.
    pub section: BracketSection,
    /// Round index within the section.
    pub round: usize,
    /// Match index within the round.
    pub match_index: usize,
    /// Slot 1 or 2.
    pub slot: u8,
}

/// Whether a link carries the winner or the loser.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkKind {
    /// Winner advances.
    Winner,
    /// Loser drops.
    Loser,
}

/// Errors raised when generating a playoff from group standings.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PlayoffError {
    /// No group stage exists, so there are no standings to qualify from.
    #[error("Create at least one group first.")]
    NoGroups,
    /// Fewer than two teams would advance from the groups.
    #[error("At least two teams must qualify for a playoff.")]
    NotEnoughQualifiers,
}

/// Errors raised by slot edits.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SlotError {
    /// The match or round no longer exists.
    #[error("match not found")]
    NotFound,
    /// The slot number was not 1 or 2.
    #[error("slot must be 1 or 2")]
    InvalidSlot,
    /// The same team would occupy both sides of the match.
    #[error("the same team cannot occupy both slots")]
    SameTeam,
}

/// Errors raised by link edits.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LinkError {
    /// The source match no longer exists.
    #[error("match not found")]
    NotFound,
    /// The target match no longer exists.
    #[error("target match not found")]
    TargetNotFound,
    /// The target slot was not 1 or 2.
    #[error("slot must be 1 or 2")]
    InvalidSlot,
    /// A match cannot link to itself.
    #[error("a match cannot link to itself")]
    SelfLink,
    /// The link would create a cycle.
    #[error("this link would create a cycle")]
    Cycle,
}

/// Returns the rounds in `section`.
#[must_use]
pub fn rounds(tournament: &Tournament, section: BracketSection) -> &[Round] {
    match section {
        BracketSection::Winners => &tournament.winners,
        BracketSection::Losers => &tournament.losers,
        BracketSection::GrandFinal => &tournament.grand_final,
    }
}

/// Returns the rounds in `section` mutably.
#[must_use]
pub fn rounds_mut(tournament: &mut Tournament, section: BracketSection) -> &mut Vec<Round> {
    match section {
        BracketSection::Winners => &mut tournament.winners,
        BracketSection::Losers => &mut tournament.losers,
        BracketSection::GrandFinal => &mut tournament.grand_final,
    }
}

/// Finds a match anywhere on the board.
#[must_use]
pub fn find_match<'a>(tournament: &'a Tournament, id: &MatchId) -> Option<&'a BracketMatch> {
    tournament
        .winners
        .iter()
        .chain(tournament.losers.iter())
        .chain(tournament.grand_final.iter())
        .flat_map(|round| round.matches.iter())
        .find(|game| &game.id == id)
}

/// Finds a match mutably anywhere on the board.
#[must_use]
pub fn match_mut<'a>(tournament: &'a mut Tournament, id: &MatchId) -> Option<&'a mut BracketMatch> {
    let (section, round, index) = match_address(tournament, id)?;
    rounds_mut(tournament, section)
        .get_mut(round)?
        .matches
        .get_mut(index)
}

/// Returns the section, round and match index for `id`.
#[must_use]
pub fn match_address(
    tournament: &Tournament,
    id: &MatchId,
) -> Option<(BracketSection, usize, usize)> {
    for (round_index, round) in tournament.winners.iter().enumerate() {
        if let Some(index) = round.matches.iter().position(|game| &game.id == id) {
            return Some((BracketSection::Winners, round_index, index));
        }
    }
    for (round_index, round) in tournament.losers.iter().enumerate() {
        if let Some(index) = round.matches.iter().position(|game| &game.id == id) {
            return Some((BracketSection::Losers, round_index, index));
        }
    }
    for (round_index, round) in tournament.grand_final.iter().enumerate() {
        if let Some(index) = round.matches.iter().position(|game| &game.id == id) {
            return Some((BracketSection::GrandFinal, round_index, index));
        }
    }
    None
}

/// Returns the match at `address`.
#[must_use]
pub fn find_slot_match(tournament: &Tournament, address: BracketSlot) -> Option<&BracketMatch> {
    rounds(tournament, address.section)
        .get(address.round)?
        .matches
        .get(address.match_index)
}

fn match_mut_at(tournament: &mut Tournament, address: BracketSlot) -> Option<&mut BracketMatch> {
    rounds_mut(tournament, address.section)
        .get_mut(address.round)?
        .matches
        .get_mut(address.match_index)
}

/// Assigns a team to a slot by hand.
pub fn set_slot(
    tournament: &mut Tournament,
    address: BracketSlot,
    team: Option<TeamId>,
) -> Result<(), SlotError> {
    if !matches!(address.slot, 1 | 2) {
        return Err(SlotError::InvalidSlot);
    }

    let other = {
        let game = find_slot_match(tournament, address).ok_or(SlotError::NotFound)?;
        if address.slot == 1 {
            game.slot2.clone()
        } else {
            game.slot1.clone()
        }
    };
    if team.is_some() && team == other {
        return Err(SlotError::SameTeam);
    }

    let game = match_mut_at(tournament, address).ok_or(SlotError::NotFound)?;
    match address.slot {
        1 => {
            game.slot1 = team;
            game.slot1_manual = true;
        }
        2 => {
            game.slot2 = team;
            game.slot2_manual = true;
        }
        _ => return Err(SlotError::InvalidSlot),
    }
    refresh(tournament);
    Ok(())
}

/// Swaps the teams in two slots and marks both as manually placed.
pub fn swap_slots(
    tournament: &mut Tournament,
    left: BracketSlot,
    right: BracketSlot,
) -> Result<(), SlotError> {
    if left == right {
        return Ok(());
    }
    if !matches!(left.slot, 1 | 2) || !matches!(right.slot, 1 | 2) {
        return Err(SlotError::InvalidSlot);
    }

    let (left_team, left_other) = slot_values(tournament, left)?;
    let (right_team, right_other) = slot_values(tournament, right)?;

    let same_match = left.section == right.section
        && left.round == right.round
        && left.match_index == right.match_index;

    if same_match {
        if left_team.is_some() && left_team == right_team {
            return Err(SlotError::SameTeam);
        }
    } else {
        if right_team.is_some() && right_team == left_other {
            return Err(SlotError::SameTeam);
        }
        if left_team.is_some() && left_team == right_other {
            return Err(SlotError::SameTeam);
        }
    }

    let left_game = match_mut_at(tournament, left).ok_or(SlotError::NotFound)?;
    set_slot_raw(left_game, left.slot, right_team);
    let right_game = match_mut_at(tournament, right).ok_or(SlotError::NotFound)?;
    set_slot_raw(right_game, right.slot, left_team);

    if let Some(game) = match_mut_at(tournament, left) {
        mark_manual(game, left.slot);
    }
    if let Some(game) = match_mut_at(tournament, right) {
        mark_manual(game, right.slot);
    }

    refresh(tournament);
    Ok(())
}

fn slot_values(
    tournament: &Tournament,
    address: BracketSlot,
) -> Result<(Option<TeamId>, Option<TeamId>), SlotError> {
    let game = find_slot_match(tournament, address).ok_or(SlotError::NotFound)?;
    match address.slot {
        1 => Ok((game.slot1.clone(), game.slot2.clone())),
        2 => Ok((game.slot2.clone(), game.slot1.clone())),
        _ => Err(SlotError::InvalidSlot),
    }
}

fn set_slot_raw(game: &mut BracketMatch, slot: u8, team: Option<TeamId>) {
    match slot {
        1 => game.slot1 = team,
        2 => game.slot2 = team,
        _ => {}
    }
}

fn mark_manual(game: &mut BracketMatch, slot: u8) {
    match slot {
        1 => game.slot1_manual = true,
        2 => game.slot2_manual = true,
        _ => {}
    }
}

/// Adds an empty match to the end of a round.
pub fn add_match(
    tournament: &mut Tournament,
    section: BracketSection,
    round_index: usize,
) -> Option<MatchId> {
    let first_to = tournament.default_first_to;
    let round = rounds_mut(tournament, section).get_mut(round_index)?;
    let game = empty_match(
        u32::try_from(round_index + 1).unwrap_or(u32::MAX),
        u32::try_from(round.matches.len() + 1).unwrap_or(u32::MAX),
        first_to,
    );
    let id = game.id.clone();
    round.matches.push(game);
    renumber_round(round, round_index);
    refresh(tournament);
    Some(id)
}

/// Removes a match and clears every link that pointed at it.
pub fn remove_match(tournament: &mut Tournament, id: &MatchId) -> bool {
    let Some((section, round_index, match_index)) = match_address(tournament, id) else {
        return false;
    };

    {
        let rounds = rounds_mut(tournament, section);
        let Some(round) = rounds.get_mut(round_index) else {
            return false;
        };
        if match_index >= round.matches.len() {
            return false;
        }
        round.matches.remove(match_index);
        renumber_round(round, round_index);
    }

    clear_links_to(tournament, id);
    refresh(tournament);
    true
}

/// Adds an empty round to a section.
pub fn add_round(tournament: &mut Tournament, section: BracketSection) -> usize {
    let index = rounds(tournament, section).len();
    let name = match section {
        BracketSection::Winners => format!("Round {}", index + 1),
        BracketSection::Losers => format!("Lower Round {}", index + 1),
        BracketSection::GrandFinal => {
            if index == 0 {
                "Grand final".to_owned()
            } else {
                format!("Grand final {}", index + 1)
            }
        }
    };
    rounds_mut(tournament, section).push(Round {
        name,
        matches: Vec::new(),
    });
    refresh(tournament);
    index
}

/// Removes a round, its matches and every link that pointed at them.
pub fn remove_round(
    tournament: &mut Tournament,
    section: BracketSection,
    round_index: usize,
) -> bool {
    let removed = {
        let rounds = rounds_mut(tournament, section);
        if round_index >= rounds.len() {
            return false;
        }
        rounds.remove(round_index)
    };

    for game in &removed.matches {
        clear_links_to(tournament, &game.id);
    }
    renumber_section(tournament, section);
    refresh(tournament);
    true
}

/// Renames a round.
pub fn rename_round(
    tournament: &mut Tournament,
    section: BracketSection,
    round_index: usize,
    name: impl Into<String>,
) -> bool {
    let Some(round) = rounds_mut(tournament, section).get_mut(round_index) else {
        return false;
    };
    round.name = name.into();
    true
}

/// Moves a match within its round.
pub fn move_match(
    tournament: &mut Tournament,
    section: BracketSection,
    round_index: usize,
    from: usize,
    to: usize,
) -> bool {
    let Some(round) = rounds_mut(tournament, section).get_mut(round_index) else {
        return false;
    };
    if from >= round.matches.len() || to >= round.matches.len() {
        return false;
    }
    if from == to {
        return true;
    }
    let game = round.matches.remove(from);
    round.matches.insert(to, game);
    renumber_round(round, round_index);
    refresh(tournament);
    true
}

/// Sets or clears a winner or loser link.
pub fn set_link(
    tournament: &mut Tournament,
    match_id: &MatchId,
    kind: LinkKind,
    link: Option<MatchLink>,
) -> Result<(), LinkError> {
    if find_match(tournament, match_id).is_none() {
        return Err(LinkError::NotFound);
    }

    if let Some(link) = &link {
        if find_match(tournament, &link.match_id).is_none() {
            return Err(LinkError::TargetNotFound);
        }
        if !matches!(link.slot, 1 | 2) {
            return Err(LinkError::InvalidSlot);
        }
        if &link.match_id == match_id {
            return Err(LinkError::SelfLink);
        }
        if link_would_cycle(tournament, match_id, &link.match_id) {
            return Err(LinkError::Cycle);
        }
    }

    if let Some(link) = &link {
        // A slot can only hold one team, so only one match may feed it. Taking
        // over a slot replaces whichever link previously pointed at it.
        let mut displaced = Vec::new();
        for_each_match(tournament, |game| {
            for (other_kind, other) in [
                (LinkKind::Winner, &game.next),
                (LinkKind::Loser, &game.loser_next),
            ] {
                if (&game.id != match_id || other_kind != kind)
                    && other.as_ref().is_some_and(|other| {
                        other.match_id == link.match_id && other.slot == link.slot
                    })
                {
                    displaced.push((game.id.clone(), other_kind));
                }
            }
        });
        for (id, other_kind) in displaced {
            clear_link(tournament, &id, other_kind);
        }
    }

    let game = match_mut(tournament, match_id).ok_or(LinkError::NotFound)?;
    match kind {
        LinkKind::Winner => game.next = link,
        LinkKind::Loser => game.loser_next = link,
    }
    refresh(tournament);
    Ok(())
}

/// Returns the winner implied by the scores, or `None` when the scores are
/// level.
#[must_use]
pub fn score_winner(game: &BracketMatch) -> Option<TeamId> {
    match game.score1.cmp(&game.score2) {
        Ordering::Greater => game.slot1.clone(),
        Ordering::Less => game.slot2.clone(),
        Ordering::Equal => None,
    }
}

fn winner_in_slots(game: &BracketMatch) -> bool {
    game.winner.as_ref().is_some_and(|winner| {
        game.slot1.as_ref() == Some(winner) || game.slot2.as_ref() == Some(winner)
    })
}

/// Sets a bracket match's scores and status, deriving the winner unless the
/// result was explicitly overridden by hand.
pub fn set_bracket_result(
    tournament: &mut Tournament,
    id: &MatchId,
    score1: u16,
    score2: u16,
    status: BracketMatchStatus,
) -> bool {
    let Some(game) = match_mut(tournament, id) else {
        return false;
    };
    game.score1 = score1;
    game.score2 = score2;
    game.status = status;

    if status == BracketMatchStatus::Completed {
        if !game.winner_manual || !winner_in_slots(game) {
            game.winner = score_winner(game);
            game.winner_manual = false;
        }
    } else {
        game.winner = None;
        game.winner_manual = false;
    }

    refresh(tournament);
    true
}

/// Sets or clears a manual winner override.
///
/// `Some(team)` must be one of the two slots and marks the match completed.
/// `None` removes the override and falls back to the score-derived winner.
pub fn set_bracket_winner(
    tournament: &mut Tournament,
    id: &MatchId,
    winner: Option<TeamId>,
) -> bool {
    let Some(game) = match_mut(tournament, id) else {
        return false;
    };

    match winner {
        Some(team) if game.slot1.as_ref() == Some(&team) || game.slot2.as_ref() == Some(&team) => {
            game.winner = Some(team);
            game.winner_manual = true;
            game.status = BracketMatchStatus::Completed;
        }
        Some(_) => return false,
        None => {
            game.winner_manual = false;
            if game.status == BracketMatchStatus::Completed {
                game.winner = score_winner(game);
            }
        }
    }

    refresh(tournament);
    true
}

/// Clears a bracket match's scores, winner and status.
///
/// The following [`refresh`] recursively clears any downstream slots that this
/// result had auto-filled.
pub fn reset_bracket_match(tournament: &mut Tournament, id: &MatchId) -> bool {
    let Some(game) = match_mut(tournament, id) else {
        return false;
    };
    game.score1 = 0;
    game.score2 = 0;
    game.winner = None;
    game.winner_manual = false;
    game.status = BracketMatchStatus::Pending;
    refresh(tournament);
    true
}

fn link_would_cycle(tournament: &Tournament, source: &MatchId, target: &MatchId) -> bool {
    let mut edges = link_edges(tournament);
    edges
        .entry(source.clone())
        .or_default()
        .push((target.clone(), LinkKind::Winner));

    let mut stack = vec![target.clone()];
    let mut seen = HashSet::new();
    while let Some(node) = stack.pop() {
        if &node == source {
            return true;
        }
        if !seen.insert(node.clone()) {
            continue;
        }
        if let Some(nexts) = edges.get(&node) {
            stack.extend(nexts.iter().map(|(next, _)| next.clone()));
        }
    }
    false
}

fn clear_links_to(tournament: &mut Tournament, id: &MatchId) {
    for_each_match_mut(tournament, |game| {
        if game.next.as_ref().is_some_and(|link| &link.match_id == id) {
            game.next = None;
        }
        if game
            .loser_next
            .as_ref()
            .is_some_and(|link| &link.match_id == id)
        {
            game.loser_next = None;
        }
    });
}

fn renumber_round(round: &mut Round, round_index: usize) {
    let round_number = u32::try_from(round_index + 1).unwrap_or(u32::MAX);
    for (index, game) in round.matches.iter_mut().enumerate() {
        game.round = round_number;
        game.number = u32::try_from(index + 1).unwrap_or(u32::MAX);
    }
}

fn renumber_section(tournament: &mut Tournament, section: BracketSection) {
    for (index, round) in rounds_mut(tournament, section).iter_mut().enumerate() {
        renumber_round(round, index);
    }
}

fn slot_is_manual(game: &BracketMatch, slot: u8) -> bool {
    match slot {
        1 => game.slot1_manual,
        2 => game.slot2_manual,
        _ => true,
    }
}

fn set_slot_value(game: &mut BracketMatch, slot: u8, team: Option<TeamId>) -> bool {
    let current = match slot {
        1 => &mut game.slot1,
        2 => &mut game.slot2,
        _ => return false,
    };
    if *current == team {
        return false;
    }
    *current = team;
    true
}

fn all_match_ids(tournament: &Tournament) -> Vec<MatchId> {
    tournament
        .winners
        .iter()
        .chain(tournament.losers.iter())
        .chain(tournament.grand_final.iter())
        .flat_map(|round| round.matches.iter().map(|game| game.id.clone()))
        .collect()
}

fn for_each_match(tournament: &Tournament, mut visit: impl FnMut(&BracketMatch)) {
    for round in tournament
        .winners
        .iter()
        .chain(tournament.losers.iter())
        .chain(tournament.grand_final.iter())
    {
        for game in &round.matches {
            visit(game);
        }
    }
}

fn for_each_match_mut(tournament: &mut Tournament, mut visit: impl FnMut(&mut BracketMatch)) {
    for round in tournament
        .winners
        .iter_mut()
        .chain(tournament.losers.iter_mut())
        .chain(tournament.grand_final.iter_mut())
    {
        for game in &mut round.matches {
            visit(game);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(id: &str) -> TeamId {
        TeamId::from_raw(id)
    }

    fn tournament_with_teams(count: usize, format: BracketFormat) -> Tournament {
        let mut tournament = new_tournament("Test Cup", format);
        set_entrants(
            &mut tournament,
            (1..=count).map(|index| team(&format!("team-{index}"))),
        );
        tournament
    }

    fn all_matches(tournament: &Tournament) -> Vec<&BracketMatch> {
        tournament
            .winners
            .iter()
            .chain(tournament.losers.iter())
            .chain(tournament.grand_final.iter())
            .flat_map(|round| round.matches.iter())
            .collect()
    }

    fn match_at(
        tournament: &Tournament,
        section: BracketSection,
        round: usize,
        index: usize,
    ) -> &BracketMatch {
        &rounds(tournament, section)[round].matches[index]
    }

    fn complete(game: &mut BracketMatch, winner: &TeamId) {
        game.status = BracketMatchStatus::Completed;
        game.winner = Some(winner.clone());
        if game.slot1.as_ref() == Some(winner) {
            game.score1 = 2;
            game.score2 = 0;
        } else {
            game.score1 = 0;
            game.score2 = 2;
        }
    }

    /// Completes the group fixture between `left` and `right` with explicit
    /// scores, regardless of which side the fixture stored each team on.
    fn complete_fixture(
        tournament: &mut Tournament,
        left: &TeamId,
        right: &TeamId,
        score_left: u16,
        score_right: u16,
    ) {
        let fixture = tournament
            .group_matches
            .iter_mut()
            .find(|fixture| {
                (fixture.team1.as_ref() == Some(left) && fixture.team2.as_ref() == Some(right))
                    || (fixture.team1.as_ref() == Some(right)
                        && fixture.team2.as_ref() == Some(left))
            })
            .expect("fixture exists");
        fixture.status = BracketMatchStatus::Completed;
        if fixture.team1.as_ref() == Some(left) {
            fixture.score1 = score_left;
            fixture.score2 = score_right;
        } else {
            fixture.score1 = score_right;
            fixture.score2 = score_left;
        }
    }

    #[test]
    fn standard_eight_team_seeding_is_documented_order() {
        assert_eq!(seeding_order(8), vec![1, 8, 4, 5, 3, 6, 2, 7]);
        assert_eq!(seeding_order(4), vec![1, 4, 2, 3]);
        assert_eq!(seeding_order(2), vec![1, 2]);
    }

    #[test]
    fn single_elimination_generates_every_team_count() {
        for count in 2..=MAX_GENERATED_ENTRANTS {
            let mut tournament = tournament_with_teams(count, BracketFormat::SingleElim);
            generate_single_elimination(&mut tournament);
            let size = count.next_power_of_two();
            assert_eq!(tournament.winners.len(), size.trailing_zeros() as usize);
            assert!(tournament.losers.is_empty());
            assert!(tournament.grand_final.is_empty());
            let first_round = &tournament.winners[0].matches;
            assert_eq!(first_round.len(), size / 2);
            let slot_count: usize = first_round
                .iter()
                .map(|game| usize::from(game.slot1.is_some()) + usize::from(game.slot2.is_some()))
                .sum();
            assert_eq!(slot_count, count);
            for (index, game) in first_round.iter().enumerate() {
                assert_eq!(game.round, 1);
                assert_eq!(game.number, index as u32 + 1);
            }
        }
    }

    #[test]
    fn single_elimination_byes_go_to_top_seeds() {
        let mut tournament = tournament_with_teams(6, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let first_round = &tournament.winners[0].matches;
        assert_eq!(first_round.len(), 4);
        assert_eq!(first_round[0].slot1, Some(team("team-1")));
        assert!(first_round[0].slot2.is_none());
        assert_eq!(first_round[0].status, BracketMatchStatus::Completed);
        assert_eq!(first_round[0].winner, Some(team("team-1")));
        assert_eq!(first_round[3].slot1, Some(team("team-2")));
        assert!(first_round[3].slot2.is_none());
        assert_eq!(first_round[3].winner, Some(team("team-2")));

        let semifinal = &tournament.winners[1].matches;
        assert_eq!(semifinal[0].slot1, Some(team("team-1")));
        assert_eq!(semifinal[1].slot2, Some(team("team-2")));
    }

    #[test]
    fn single_elimination_round_names_follow_bracket_size() {
        let mut tournament = tournament_with_teams(8, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let names: Vec<&str> = tournament
            .winners
            .iter()
            .map(|round| round.name.as_str())
            .collect();
        assert_eq!(names, vec!["Quarterfinal", "Semifinal", "Final"]);

        let mut tournament = tournament_with_teams(32, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let names: Vec<&str> = tournament
            .winners
            .iter()
            .map(|round| round.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["Round 1", "Round 2", "Quarterfinal", "Semifinal", "Final"]
        );
    }

    #[test]
    fn single_elimination_advances_and_clears_downstream() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let first = tournament.winners[0].matches[0].id.clone();
        let second = tournament.winners[0].matches[1].id.clone();
        let final_id = tournament.winners[1].matches[0].id.clone();

        complete(
            match_mut(&mut tournament, &first).expect("first match"),
            &team("team-1"),
        );
        refresh(&mut tournament);
        complete(
            match_mut(&mut tournament, &second).expect("second match"),
            &team("team-2"),
        );
        refresh(&mut tournament);

        let final_game = find_match(&tournament, &final_id).expect("final");
        assert_eq!(final_game.slot1, Some(team("team-1")));
        assert_eq!(final_game.slot2, Some(team("team-2")));

        complete(
            match_mut(&mut tournament, &final_id).expect("final match"),
            &team("team-1"),
        );
        refresh(&mut tournament);
        assert_eq!(tournament.champion, Some(team("team-1")));
        assert_eq!(tournament.status, TournamentStatus::Completed);

        let first_game = match_mut(&mut tournament, &first).expect("first match");
        first_game.status = BracketMatchStatus::Pending;
        first_game.winner = None;
        refresh(&mut tournament);
        let final_game = find_match(&tournament, &final_id).expect("final");
        assert_eq!(final_game.status, BracketMatchStatus::Pending);
        assert!(final_game.slot1.is_none());
        assert_eq!(tournament.champion, None);
    }

    #[test]
    fn linking_into_an_occupied_slot_takes_it_over_and_propagates() {
        let mut tournament = tournament_with_teams(32, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);

        let first = tournament.winners[0].matches[0].id.clone();
        let quarterfinal_two = tournament.winners[2].matches[1].id.clone();
        let semi = tournament.winners[3].matches[0].id.clone();
        let final_id = tournament.winners[4].matches[0].id.clone();

        let slot = BracketSlot {
            section: BracketSection::Winners,
            round: 0,
            match_index: 0,
            slot: 1,
        };
        set_slot(&mut tournament, slot, Some(team("team-1"))).expect("slot 1");
        set_slot(
            &mut tournament,
            BracketSlot { slot: 2, ..slot },
            Some(team("team-2")),
        )
        .expect("slot 2");
        assert!(set_bracket_result(
            &mut tournament,
            &first,
            2,
            1,
            BracketMatchStatus::Completed
        ));

        // Both target slots are already fed by the generated bracket, so the
        // new links must replace those sources rather than fight with them.
        assert_eq!(
            set_link(
                &mut tournament,
                &first,
                LinkKind::Winner,
                Some(MatchLink {
                    match_id: semi.clone(),
                    slot: 2,
                })
            ),
            Ok(())
        );
        assert_eq!(
            set_link(
                &mut tournament,
                &first,
                LinkKind::Loser,
                Some(MatchLink {
                    match_id: final_id.clone(),
                    slot: 1,
                })
            ),
            Ok(())
        );

        assert!(
            find_match(&tournament, &quarterfinal_two)
                .expect("quarterfinal two")
                .next
                .is_none()
        );
        assert!(find_match(&tournament, &semi).expect("semi").next.is_none());
        assert_eq!(
            find_match(&tournament, &semi).expect("semi").slot2,
            Some(team("team-1"))
        );
        assert_eq!(
            find_match(&tournament, &final_id).expect("final").slot1,
            Some(team("team-2"))
        );
    }

    #[test]
    fn refresh_keeps_one_link_per_target_slot() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let final_id = tournament.winners[1].matches[0].id.clone();

        // Corrupt the board by pointing both first-round matches at the same
        // slot, as an older save file could.
        tournament.winners[0].matches[0].next = Some(MatchLink {
            match_id: final_id.clone(),
            slot: 2,
        });

        refresh(&mut tournament);

        assert!(tournament.winners[0].matches[0].next.is_some());
        assert!(tournament.winners[0].matches[1].next.is_none());
    }

    #[test]
    fn manual_slots_are_never_overwritten() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let first = tournament.winners[0].matches[0].id.clone();
        let final_id = tournament.winners[1].matches[0].id.clone();

        complete(
            match_mut(&mut tournament, &first).expect("first match"),
            &team("team-1"),
        );
        refresh(&mut tournament);

        let address = BracketSlot {
            section: BracketSection::Winners,
            round: 1,
            match_index: 0,
            slot: 1,
        };
        set_slot(&mut tournament, address, Some(team("team-3"))).expect("manual slot");

        let first_game = match_mut(&mut tournament, &first).expect("first match");
        complete(first_game, &team("team-4"));
        refresh(&mut tournament);

        let final_game = find_match(&tournament, &final_id).expect("final");
        assert_eq!(final_game.slot1, Some(team("team-3")));
    }

    #[test]
    fn double_elimination_structure_and_drop_mapping() {
        let mut tournament = tournament_with_teams(8, BracketFormat::DoubleElim);
        generate_double_elimination(&mut tournament);

        assert_eq!(tournament.winners.len(), 3);
        assert_eq!(tournament.losers.len(), 4);
        assert_eq!(tournament.grand_final.len(), 1);
        let losers_counts: Vec<usize> = tournament
            .losers
            .iter()
            .map(|round| round.matches.len())
            .collect();
        assert_eq!(losers_counts, vec![2, 2, 1, 1]);

        let winners_first = &tournament.winners[0].matches;
        assert_eq!(
            winners_first[0].loser_next.as_ref().map(|link| link.slot),
            Some(1)
        );
        assert_eq!(
            winners_first[0]
                .loser_next
                .as_ref()
                .map(|link| link.match_id.clone()),
            Some(tournament.losers[0].matches[0].id.clone())
        );
        assert_eq!(
            winners_first[1].loser_next.as_ref().map(|link| link.slot),
            Some(2)
        );
        assert_eq!(
            winners_first[1]
                .loser_next
                .as_ref()
                .map(|link| link.match_id.clone()),
            Some(tournament.losers[0].matches[0].id.clone())
        );

        assert_eq!(
            tournament.winners[1].matches[0]
                .loser_next
                .as_ref()
                .map(|link| link.match_id.clone()),
            Some(tournament.losers[1].matches[0].id.clone())
        );
        assert_eq!(
            tournament.winners[2].matches[0]
                .loser_next
                .as_ref()
                .map(|link| link.match_id.clone()),
            Some(tournament.losers[3].matches[0].id.clone())
        );
        assert_eq!(
            tournament.winners[2].matches[0]
                .next
                .as_ref()
                .map(|link| link.match_id.clone()),
            Some(tournament.grand_final[0].matches[0].id.clone())
        );
        assert_eq!(
            tournament.losers[3].matches[0]
                .next
                .as_ref()
                .map(|link| link.slot),
            Some(2)
        );
    }

    #[test]
    fn double_elimination_has_the_documented_match_counts() {
        for (count, expected_losers) in [
            (4, vec![1, 1]),
            (8, vec![2, 2, 1, 1]),
            (16, vec![4, 4, 2, 2, 1, 1]),
            (32, vec![8, 8, 4, 4, 2, 2, 1, 1]),
        ] {
            let mut tournament = tournament_with_teams(count, BracketFormat::DoubleElim);
            generate_double_elimination(&mut tournament);
            let losers_counts: Vec<usize> = tournament
                .losers
                .iter()
                .map(|round| round.matches.len())
                .collect();
            assert_eq!(losers_counts, expected_losers);
            let total_losers: usize = losers_counts.iter().sum();
            assert_eq!(total_losers, count - 2);
        }
    }

    #[test]
    fn grand_final_reset_is_created_and_sets_the_champion() {
        let mut tournament = tournament_with_teams(4, BracketFormat::DoubleElim);
        generate_double_elimination(&mut tournament);
        let grand_final_id = tournament.grand_final[0].matches[0].id.clone();

        let slot = |slot: u8| BracketSlot {
            section: BracketSection::GrandFinal,
            round: 0,
            match_index: 0,
            slot,
        };
        set_slot(&mut tournament, slot(1), Some(team("team-1"))).expect("slot 1");
        set_slot(&mut tournament, slot(2), Some(team("team-2"))).expect("slot 2");

        let grand_final = match_mut(&mut tournament, &grand_final_id).expect("grand final");
        complete(grand_final, &team("team-2"));
        refresh(&mut tournament);

        assert_eq!(tournament.grand_final.len(), 2);
        let reset_id = tournament.grand_final[1].matches[0].id.clone();
        let reset = find_match(&tournament, &reset_id).expect("reset");
        assert_eq!(reset.slot1, Some(team("team-1")));
        assert_eq!(reset.slot2, Some(team("team-2")));
        assert_eq!(tournament.champion, None);

        complete(
            match_mut(&mut tournament, &reset_id).expect("reset"),
            &team("team-1"),
        );
        refresh(&mut tournament);
        assert_eq!(tournament.champion, Some(team("team-1")));

        let grand_final = match_mut(&mut tournament, &grand_final_id).expect("grand final");
        complete(grand_final, &team("team-1"));
        refresh(&mut tournament);
        assert_eq!(tournament.grand_final.len(), 1);
        assert_eq!(tournament.champion, Some(team("team-1")));
    }

    #[test]
    fn reset_can_be_disabled() {
        let mut tournament = tournament_with_teams(4, BracketFormat::DoubleElim);
        tournament.grand_finals_reset = false;
        generate_double_elimination(&mut tournament);
        let grand_final_id = tournament.grand_final[0].matches[0].id.clone();
        let slot = |slot: u8| BracketSlot {
            section: BracketSection::GrandFinal,
            round: 0,
            match_index: 0,
            slot,
        };
        set_slot(&mut tournament, slot(1), Some(team("team-1"))).expect("slot 1");
        set_slot(&mut tournament, slot(2), Some(team("team-2"))).expect("slot 2");
        let grand_final = match_mut(&mut tournament, &grand_final_id).expect("grand final");
        complete(grand_final, &team("team-2"));
        refresh(&mut tournament);
        assert_eq!(tournament.grand_final.len(), 1);
        assert_eq!(tournament.champion, Some(team("team-2")));
    }

    #[test]
    fn round_robin_fixtures_and_standings() {
        let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
        generate_round_robin(&mut tournament, 1);
        assert_eq!(tournament.groups.len(), 1);
        assert_eq!(tournament.group_matches.len(), 6);
        assert_eq!(tournament.standings[&tournament.groups[0].id].len(), 4);

        complete_fixture(&mut tournament, &team("team-1"), &team("team-2"), 2, 0);
        complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 0);
        complete_fixture(&mut tournament, &team("team-1"), &team("team-4"), 2, 0);
        complete_fixture(&mut tournament, &team("team-2"), &team("team-3"), 2, 0);
        complete_fixture(&mut tournament, &team("team-2"), &team("team-4"), 2, 0);
        complete_fixture(&mut tournament, &team("team-3"), &team("team-4"), 2, 0);
        refresh(&mut tournament);

        let standings = &tournament.standings[&tournament.groups[0].id];
        assert_eq!(standings.len(), 4);
        assert_eq!(standings.iter().map(|row| row.played).sum::<u32>(), 12);
        assert_eq!(standings[0].team_id, team("team-1"));
        assert_eq!(standings[0].wins, 3);
        assert_eq!(standings[0].points, 3);
        assert_eq!(standings[3].team_id, team("team-4"));
        assert_eq!(standings[3].losses, 3);
    }

    #[test]
    fn head_to_head_breaks_a_two_way_tie() {
        let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
        generate_round_robin(&mut tournament, 1);

        // Team 1 and team 2 both finish 2-1; team 2 won the head-to-head.
        complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 0);
        complete_fixture(&mut tournament, &team("team-1"), &team("team-4"), 2, 0);
        complete_fixture(&mut tournament, &team("team-2"), &team("team-1"), 2, 0);
        complete_fixture(&mut tournament, &team("team-3"), &team("team-2"), 2, 0);
        complete_fixture(&mut tournament, &team("team-2"), &team("team-4"), 2, 0);
        complete_fixture(&mut tournament, &team("team-4"), &team("team-3"), 2, 0);
        refresh(&mut tournament);

        let standings = &tournament.standings[&tournament.groups[0].id];
        assert_eq!(standings[0].team_id, team("team-2"));
        assert_eq!(standings[1].team_id, team("team-1"));
        assert_eq!(standings[0].wins, 2);
        assert_eq!(standings[1].wins, 2);
    }

    #[test]
    fn configured_tiebreaker_order_is_applied() {
        let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
        generate_round_robin(&mut tournament, 1);

        // Team 1 wins twice narrowly and loses once heavily; team 2 wins once
        // heavily and loses twice narrowly. Wins ranks team 1 higher, map
        // differential ranks team 2 higher.
        complete_fixture(&mut tournament, &team("team-1"), &team("team-2"), 2, 1);
        complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 1);
        complete_fixture(&mut tournament, &team("team-4"), &team("team-1"), 5, 0);
        complete_fixture(&mut tournament, &team("team-2"), &team("team-3"), 5, 0);
        complete_fixture(&mut tournament, &team("team-4"), &team("team-2"), 1, 0);
        complete_fixture(&mut tournament, &team("team-3"), &team("team-4"), 1, 0);

        tournament.tiebreakers = vec![Tiebreaker::Wins, Tiebreaker::MapDifferential];
        refresh(&mut tournament);
        let by_wins = &tournament.standings[&tournament.groups[0].id];
        assert_eq!(by_wins[0].team_id, team("team-4"));
        assert_eq!(by_wins[1].team_id, team("team-1"));
        assert_eq!(by_wins[2].team_id, team("team-2"));

        tournament.tiebreakers = vec![Tiebreaker::MapDifferential, Tiebreaker::Wins];
        refresh(&mut tournament);
        let by_differential = &tournament.standings[&tournament.groups[0].id];
        assert_eq!(by_differential[0].team_id, team("team-4"));
        assert_eq!(by_differential[1].team_id, team("team-2"));
        assert_eq!(by_differential[2].team_id, team("team-1"));
    }

    #[test]
    fn playoff_generation_cross_seeds_two_groups() {
        let mut tournament = tournament_with_teams(8, BracketFormat::RoundRobin);
        generate_round_robin(&mut tournament, 2);
        assert_eq!(tournament.groups.len(), 2);
        for group in &tournament.groups {
            let ids: Vec<MatchId> = tournament
                .group_matches
                .iter()
                .filter(|fixture| fixture.group == group.id)
                .map(|fixture| fixture.id.clone())
                .collect();
            for (index, id) in ids.iter().enumerate() {
                let fixture = tournament
                    .group_matches
                    .iter_mut()
                    .find(|fixture| &fixture.id == id)
                    .expect("fixture");
                fixture.status = BracketMatchStatus::Completed;
                if index % 2 == 0 {
                    fixture.score1 = 2;
                    fixture.score2 = 0;
                } else {
                    fixture.score1 = 0;
                    fixture.score2 = 2;
                }
            }
        }
        refresh(&mut tournament);
        generate_playoff(&mut tournament).expect("playoff");

        assert_eq!(tournament.winners.len(), 2);
        let first_round = &tournament.winners[0].matches;
        assert_eq!(first_round.len(), 2);
        let group_a = &tournament.groups[0];
        let group_b = &tournament.groups[1];
        let a1 = tournament.standings[&group_a.id][0].team_id.clone();
        let b1 = tournament.standings[&group_b.id][0].team_id.clone();
        let a2 = tournament.standings[&group_a.id][1].team_id.clone();
        let b2 = tournament.standings[&group_b.id][1].team_id.clone();

        assert_eq!(first_round[0].slot1, Some(a1));
        assert_eq!(first_round[0].slot2, Some(b2));
        assert_eq!(first_round[1].slot1, Some(b1));
        assert_eq!(first_round[1].slot2, Some(a2));
    }

    #[test]
    fn board_operations_edit_structure_and_links() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);

        let round_index = 0;
        let added =
            add_match(&mut tournament, BracketSection::Winners, round_index).expect("match");
        assert_eq!(tournament.winners[0].matches.len(), 3);
        assert!(remove_match(&mut tournament, &added));
        assert_eq!(tournament.winners[0].matches.len(), 2);

        let extra_round = add_round(&mut tournament, BracketSection::Winners);
        assert_eq!(extra_round, 2);
        assert!(rename_round(
            &mut tournament,
            BracketSection::Winners,
            extra_round,
            "Consolation"
        ));
        assert!(move_match(
            &mut tournament,
            BracketSection::Winners,
            round_index,
            0,
            1
        ));
        assert!(remove_round(
            &mut tournament,
            BracketSection::Winners,
            extra_round
        ));

        let source = tournament.winners[0].matches[0].id.clone();
        let target = tournament.winners[1].matches[0].id.clone();
        let link = MatchLink {
            match_id: target.clone(),
            slot: 1,
        };
        assert_eq!(
            set_link(
                &mut tournament,
                &source,
                LinkKind::Winner,
                Some(link.clone())
            ),
            Ok(())
        );
        assert_eq!(
            set_link(
                &mut tournament,
                &target,
                LinkKind::Winner,
                Some(MatchLink {
                    match_id: source.clone(),
                    slot: 1,
                })
            ),
            Err(LinkError::Cycle)
        );
        assert_eq!(
            set_link(
                &mut tournament,
                &source,
                LinkKind::Winner,
                Some(MatchLink {
                    match_id: source.clone(),
                    slot: 1,
                })
            ),
            Err(LinkError::SelfLink)
        );
    }

    #[test]
    fn swapping_slots_marks_both_manual_and_rejects_duplicates() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        let left = BracketSlot {
            section: BracketSection::Winners,
            round: 0,
            match_index: 0,
            slot: 1,
        };
        let right = BracketSlot {
            section: BracketSection::Winners,
            round: 1,
            match_index: 0,
            slot: 1,
        };
        let before = (
            find_slot_match(&tournament, left)
                .expect("left")
                .slot1
                .clone(),
            find_slot_match(&tournament, right)
                .expect("right")
                .slot1
                .clone(),
        );
        swap_slots(&mut tournament, left, right).expect("swap");
        let after = (
            find_slot_match(&tournament, left)
                .expect("left")
                .slot1
                .clone(),
            find_slot_match(&tournament, right)
                .expect("right")
                .slot1
                .clone(),
        );
        assert_eq!(before.0, after.1);
        assert_eq!(before.1, after.0);
        assert!(
            find_slot_match(&tournament, left)
                .expect("left")
                .slot1_manual
        );
        assert!(
            find_slot_match(&tournament, right)
                .expect("right")
                .slot1_manual
        );

        let duplicate_team = find_slot_match(
            &tournament,
            BracketSlot {
                section: BracketSection::Winners,
                round: 0,
                match_index: 0,
                slot: 2,
            },
        )
        .expect("left match")
        .slot2
        .clone();
        assert!(duplicate_team.is_some());
        assert_eq!(
            set_slot(&mut tournament, left, duplicate_team),
            Err(SlotError::SameTeam)
        );
    }

    #[test]
    fn repair_clears_invalid_results_links_and_champion() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        tournament.entrants[1].seed = 1;
        let first = tournament.winners[0].matches[0].id.clone();
        let final_id = tournament.winners[1].matches[0].id.clone();

        {
            let first_game = match_mut(&mut tournament, &first).expect("first");
            first_game.status = BracketMatchStatus::Completed;
            first_game.winner = None;
        }
        {
            let final_game = match_mut(&mut tournament, &final_id).expect("final");
            final_game.next = Some(MatchLink {
                match_id: MatchId::from_raw("missing"),
                slot: 1,
            });
        }
        tournament.champion = Some(team("outsider"));

        let repairs = repair_tournament(&mut tournament);
        let kinds: Vec<TournamentRepairKind> = repairs.iter().map(|repair| repair.kind).collect();
        assert!(kinds.contains(&TournamentRepairKind::DuplicateSeed));
        assert!(kinds.contains(&TournamentRepairKind::CompletedWithoutWinner));
        assert!(kinds.contains(&TournamentRepairKind::DanglingLink));
        assert!(kinds.contains(&TournamentRepairKind::ChampionNotEntrant));
        assert_eq!(
            find_match(&tournament, &first).expect("first").status,
            BracketMatchStatus::Pending
        );
        assert!(
            find_match(&tournament, &final_id)
                .expect("final")
                .next
                .is_none()
        );
        assert_eq!(tournament.champion, None);
    }

    #[test]
    fn repair_clears_group_fixtures_outside_the_group() {
        let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
        generate_round_robin(&mut tournament, 1);
        let group_id = tournament.groups[0].id.clone();
        let fixture = tournament
            .group_matches
            .iter_mut()
            .find(|fixture| fixture.group == group_id)
            .expect("fixture");
        fixture.team1 = Some(team("outsider"));
        let repairs = repair_tournament(&mut tournament);
        assert!(
            repairs
                .iter()
                .any(|repair| repair.kind == TournamentRepairKind::GroupTeamOutsideGroup)
        );
        assert!(
            tournament
                .group_matches
                .iter()
                .filter(|fixture| fixture.group == group_id)
                .all(|fixture| fixture.team1.as_ref() != Some(&team("outsider")))
        );
    }

    #[test]
    fn round_robin_pairs_cover_every_team_once_per_round() {
        let teams: Vec<TeamId> = (1..=5)
            .map(|index| team(&format!("team-{index}")))
            .collect();
        let pairs = round_robin_pairs(&teams);
        assert_eq!(pairs.len(), 10);
        let mut counts: HashMap<TeamId, usize> = HashMap::new();
        for (left, right) in &pairs {
            *counts.entry(left.clone()).or_default() += 1;
            *counts.entry(right.clone()).or_default() += 1;
        }
        assert!(counts.values().all(|count| *count == 4));
    }

    #[test]
    fn entrant_helpers_keep_seeds_contiguous() {
        let mut tournament = new_tournament("Seeds", BracketFormat::SingleElim);
        assert_eq!(add_entrant(&mut tournament, team("a")), 1);
        assert_eq!(add_entrant(&mut tournament, team("b")), 2);
        assert_eq!(add_entrant(&mut tournament, team("a")), 1);
        assert_eq!(tournament.entrants.len(), 2);
        assert!(remove_entrant(&mut tournament, &team("a")));
        assert_eq!(tournament.entrants[0].seed, 1);
    }

    #[test]
    fn new_tournament_uses_documented_defaults() {
        let tournament = new_tournament("Defaults", BracketFormat::SingleElim);
        assert_eq!(tournament.tiebreakers, DEFAULT_TIEBREAKERS.to_vec());
        assert_eq!(tournament.playoff_qualifiers, 2);
        assert_eq!(tournament.default_first_to, 3);
        assert!(tournament.grand_finals_reset);
        assert_eq!(tournament.status, TournamentStatus::Setup);
        assert!(tournament.standings.is_empty());
    }

    #[test]
    fn match_at_helpers_are_stable() {
        let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);
        assert!(all_matches(&tournament).len() >= 3);
        let game = match_at(&tournament, BracketSection::Winners, 0, 0);
        assert_eq!(game.round, 1);
    }
}
