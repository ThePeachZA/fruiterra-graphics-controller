//! Integration coverage for the tournament board.
//!
//! Each test maps to one of the acceptance criteria in `docs/tournament.md`
//! §8 (T1–T14). Generation is exercised for every supported entrant count so
//! the recursive seeding and drop mapping are covered, not just the tidy
//! power-of-two cases.

use std::collections::HashSet;

use fgc_core::{
    AppState, BracketFormat, BracketMatch, BracketMatchStatus, BracketSection, BracketSlot,
    Entrant, LinkError, LinkKind, MatchId, MatchLink, SlotError, TeamId, Tiebreaker, Tournament,
    TournamentId, TournamentRepairKind, TournamentStatus, add_group_fixture, add_match, add_round,
    generate_double_elimination, generate_playoff, generate_round_robin,
    generate_single_elimination, match_address, match_mut, migrate, move_match, new_tournament,
    refresh, remove_group, remove_group_fixture, remove_match, remove_round, rename_group,
    rename_round, repair_tournament, reset_bracket_match, rounds, seeding_order,
    set_bracket_result, set_bracket_winner, set_entrants, set_group_result, set_group_teams,
    set_link, set_slot, swap_slots,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn match_ids(tournament: &Tournament) -> HashSet<MatchId> {
    all_matches(tournament)
        .into_iter()
        .map(|game| game.id.clone())
        .collect()
}

fn slot(section: BracketSection, round: usize, match_index: usize, slot: u8) -> BracketSlot {
    BracketSlot {
        section,
        round,
        match_index,
        slot,
    }
}

fn complete(tournament: &mut Tournament, id: &MatchId, winner: &TeamId) {
    let game = match_mut(tournament, id).expect("match exists");
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

fn reset_match(tournament: &mut Tournament, id: &MatchId) {
    let game = match_mut(tournament, id).expect("match exists");
    game.status = BracketMatchStatus::Pending;
    game.winner = None;
    game.winner_manual = false;
    game.score1 = 0;
    game.score2 = 0;
}

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
                || (fixture.team1.as_ref() == Some(right) && fixture.team2.as_ref() == Some(left))
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

fn row_order(tournament: &Tournament, group_index: usize) -> Vec<TeamId> {
    let group = &tournament.groups[group_index];
    tournament.standings[&group.id]
        .iter()
        .map(|row| row.team_id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// T1: single elimination
// ---------------------------------------------------------------------------

#[test]
fn t1_single_elimination_is_valid_for_every_size() {
    for count in 2..=32 {
        let mut tournament = tournament_with_teams(count, BracketFormat::SingleElim);
        generate_single_elimination(&mut tournament);

        let size = count.next_power_of_two();
        assert_eq!(
            tournament.format,
            BracketFormat::SingleElim,
            "{count} entrants"
        );
        assert_eq!(
            tournament.winners.len(),
            size.trailing_zeros() as usize,
            "{count} entrants"
        );
        assert!(tournament.losers.is_empty(), "{count} entrants");
        assert!(tournament.grand_final.is_empty(), "{count} entrants");

        let first_round = &tournament.winners[0].matches;
        assert_eq!(first_round.len(), size / 2, "{count} entrants");
        let filled: usize = first_round
            .iter()
            .map(|game| usize::from(game.slot1.is_some()) + usize::from(game.slot2.is_some()))
            .sum();
        assert_eq!(filled, count, "{count} entrants");

        // Byes go to the highest seeds and complete immediately.
        for seed in 1..=(size - count) {
            let id = team(&format!("team-{seed}"));
            let game = first_round
                .iter()
                .find(|game| game.slot1.as_ref() == Some(&id) || game.slot2.as_ref() == Some(&id))
                .unwrap_or_else(|| panic!("seed {seed} must appear in round one"));
            assert_eq!(game.status, BracketMatchStatus::Completed);
            assert_eq!(game.winner.as_ref(), Some(&id));
        }

        // Every match is uniquely identified and addressable.
        let matches = all_matches(&tournament);
        assert_eq!(match_ids(&tournament).len(), matches.len());
        for game in matches {
            assert!(match_address(&tournament, &game.id).is_some());
        }

        // Rounds are numbered and named top to bottom.
        for (round_index, round) in tournament.winners.iter().enumerate() {
            for (match_index, game) in round.matches.iter().enumerate() {
                assert_eq!(game.round, u32::try_from(round_index + 1).unwrap());
                assert_eq!(game.number, u32::try_from(match_index + 1).unwrap());
            }
        }
    }
}

#[test]
fn t1_seeding_uses_the_documented_order() {
    assert_eq!(seeding_order(2), vec![1, 2]);
    assert_eq!(seeding_order(4), vec![1, 4, 2, 3]);
    assert_eq!(seeding_order(8), vec![1, 8, 4, 5, 3, 6, 2, 7]);

    for size in [2, 4, 8, 16, 32] {
        let order = seeding_order(size);
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            (1..=u32::try_from(size).unwrap()).collect::<Vec<_>>()
        );
        for pair in order.chunks(2) {
            assert_eq!(pair[0] + pair[1], u32::try_from(size).unwrap() + 1);
        }
    }
}

// ---------------------------------------------------------------------------
// T2: double elimination
// ---------------------------------------------------------------------------

#[test]
fn t2_double_elimination_is_valid_for_every_size() {
    for count in 2..=32 {
        let mut tournament = tournament_with_teams(count, BracketFormat::DoubleElim);
        generate_double_elimination(&mut tournament);

        let size = count.next_power_of_two();
        let winners_rounds = size.trailing_zeros() as usize;
        assert_eq!(tournament.format, BracketFormat::DoubleElim);
        assert_eq!(tournament.winners.len(), winners_rounds, "{count} entrants");
        assert_eq!(
            tournament.losers.len(),
            winners_rounds.saturating_sub(1) * 2,
            "{count} entrants"
        );
        assert_eq!(tournament.grand_final.len(), 1, "{count} entrants");
        assert!(
            tournament
                .winners
                .iter()
                .chain(tournament.losers.iter())
                .all(|round| !round.matches.is_empty()),
            "{count} entrants"
        );

        let matches = all_matches(&tournament);
        assert_eq!(match_ids(&tournament).len(), matches.len());
        for game in matches {
            assert!(match_address(&tournament, &game.id).is_some());
        }

        // The winners final feeds slot 1 of the grand final and the losers
        // final feeds slot 2.
        let winners_final = tournament
            .winners
            .last()
            .and_then(|round| round.matches.first())
            .expect("winners final");
        assert_eq!(winners_final.next.as_ref().map(|link| link.slot), Some(1));
        let losers_final = tournament
            .losers
            .last()
            .and_then(|round| round.matches.first());
        if let Some(losers_final) = losers_final {
            assert_eq!(losers_final.next.as_ref().map(|link| link.slot), Some(2));
        }
    }
}

#[test]
fn t2_double_elimination_match_counts_and_drop_mapping() {
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
        assert_eq!(losers_counts, expected_losers, "{count} entrants");
        assert_eq!(all_matches(&tournament).len(), count * 2 - 2);
    }

    // 8-team drop mapping: first-round losers pair off, later winners-bracket
    // losers drop into the second slot of the matching lower round.
    let mut tournament = tournament_with_teams(8, BracketFormat::DoubleElim);
    generate_double_elimination(&mut tournament);

    let lower_first = tournament.losers[0].matches[0].id.clone();
    assert_eq!(
        tournament.winners[0].matches[0]
            .loser_next
            .as_ref()
            .map(|link| (link.match_id.clone(), link.slot)),
        Some((lower_first.clone(), 1))
    );
    assert_eq!(
        tournament.winners[0].matches[1]
            .loser_next
            .as_ref()
            .map(|link| (link.match_id.clone(), link.slot)),
        Some((lower_first, 2))
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
}

// ---------------------------------------------------------------------------
// T3: round robin
// ---------------------------------------------------------------------------

#[test]
fn t3_round_robin_fixtures_and_standings_are_complete() {
    for count in 2..=12 {
        for group_count in 1..=3 {
            let mut tournament = tournament_with_teams(count, BracketFormat::RoundRobin);
            generate_round_robin(&mut tournament, group_count);

            assert_eq!(tournament.format, BracketFormat::RoundRobin);
            assert!(tournament.winners.is_empty());
            assert!(tournament.losers.is_empty());
            assert!(tournament.grand_final.is_empty());
            assert_eq!(tournament.groups.len(), group_count.min(count));
            assert_eq!(tournament.standings.len(), tournament.groups.len());

            let mut expected_fixtures = 0;
            for group in &tournament.groups {
                let size = group.team_ids.len();
                expected_fixtures += size * (size - 1) / 2;
                let rows = &tournament.standings[&group.id];
                assert_eq!(rows.len(), size);
                assert_eq!(
                    rows.iter()
                        .map(|row| row.team_id.clone())
                        .collect::<HashSet<_>>(),
                    group.team_ids.iter().cloned().collect::<HashSet<_>>()
                );
            }
            assert_eq!(tournament.group_matches.len(), expected_fixtures);

            for fixture in &tournament.group_matches {
                let group = tournament
                    .groups
                    .iter()
                    .find(|group| group.id == fixture.group)
                    .expect("fixture group exists");
                for fixture_team in [&fixture.team1, &fixture.team2].into_iter().flatten() {
                    assert!(group.team_ids.contains(fixture_team));
                }
            }

            // Every pair of teams in a group meets exactly once.
            let mut pairs = HashSet::new();
            for fixture in &tournament.group_matches {
                let (Some(left), Some(right)) = (&fixture.team1, &fixture.team2) else {
                    continue;
                };
                let pair = if left.as_str() < right.as_str() {
                    (left.clone(), right.clone())
                } else {
                    (right.clone(), left.clone())
                };
                assert!(pairs.insert(pair), "duplicate fixture");
            }
        }
    }
}

#[test]
fn t3_standings_rank_by_wins_and_differential() {
    let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
    generate_round_robin(&mut tournament, 1);

    complete_fixture(&mut tournament, &team("team-1"), &team("team-2"), 2, 0);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 1);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-4"), 0, 2);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-3"), 2, 1);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-4"), 1, 2);
    complete_fixture(&mut tournament, &team("team-3"), &team("team-4"), 0, 2);
    refresh(&mut tournament);

    let rows = &tournament.standings[&tournament.groups[0].id];
    assert_eq!(rows.iter().map(|row| row.played).sum::<u32>(), 12);
    assert_eq!(rows.iter().map(|row| row.wins).sum::<u32>(), 6);
    assert_eq!(rows[0].team_id, team("team-4"));
    assert_eq!(rows[0].wins, 3);
    assert_eq!(rows[0].points, 3);
    assert_eq!(rows[3].team_id, team("team-3"));
    assert_eq!(rows[3].losses, 3);
}

// ---------------------------------------------------------------------------
// T4: board editing
// ---------------------------------------------------------------------------

#[test]
fn t4_slots_swaps_matches_rounds_and_links_are_editable() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);

    // Swap two teams between slots.
    let left = slot(BracketSection::Winners, 0, 0, 1);
    let right = slot(BracketSection::Winners, 0, 1, 1);
    let left_team = fgc_core::find_slot_match(&tournament, left)
        .expect("left")
        .slot1
        .clone();
    let right_team = fgc_core::find_slot_match(&tournament, right)
        .expect("right")
        .slot1
        .clone();
    swap_slots(&mut tournament, left, right).expect("swap");
    assert_eq!(
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1,
        right_team
    );
    assert_eq!(
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1,
        left_team
    );
    assert!(
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1_manual
    );
    assert!(
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1_manual
    );

    // Adding and removing a match keeps numbering contiguous.
    let added = add_match(&mut tournament, BracketSection::Winners, 0).expect("added match");
    assert_eq!(tournament.winners[0].matches.len(), 3);
    assert!(remove_match(&mut tournament, &added));
    assert_eq!(tournament.winners[0].matches.len(), 2);

    // Adding, renaming, reordering and removing a round.
    let extra_round = add_round(&mut tournament, BracketSection::Winners);
    assert_eq!(extra_round, 2);
    assert!(rename_round(
        &mut tournament,
        BracketSection::Winners,
        extra_round,
        "Consolation"
    ));
    let first_id = tournament.winners[0].matches[0].id.clone();
    let second_id = tournament.winners[0].matches[1].id.clone();
    assert!(move_match(
        &mut tournament,
        BracketSection::Winners,
        0,
        0,
        1
    ));
    assert_eq!(tournament.winners[0].matches[0].id, second_id);
    assert_eq!(tournament.winners[0].matches[1].id, first_id);
    assert!(remove_round(
        &mut tournament,
        BracketSection::Winners,
        extra_round
    ));
    assert_eq!(tournament.winners.len(), 2);

    // Link editing rejects cycles, self-links and bad targets.
    let source = tournament.winners[0].matches[0].id.clone();
    let target = tournament.winners[1].matches[0].id.clone();
    assert_eq!(
        set_link(
            &mut tournament,
            &source,
            LinkKind::Winner,
            Some(MatchLink {
                match_id: target.clone(),
                slot: 1,
            }),
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
            }),
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
            }),
        ),
        Err(LinkError::SelfLink)
    );
    assert_eq!(
        set_link(
            &mut tournament,
            &source,
            LinkKind::Winner,
            Some(MatchLink {
                match_id: MatchId::from_raw("missing"),
                slot: 1,
            }),
        ),
        Err(LinkError::TargetNotFound)
    );
    assert_eq!(
        set_link(
            &mut tournament,
            &source,
            LinkKind::Winner,
            Some(MatchLink {
                match_id: target.clone(),
                slot: 3,
            }),
        ),
        Err(LinkError::InvalidSlot)
    );

    // Removing a match clears links that pointed at it.
    assert!(remove_match(&mut tournament, &target));
    assert!(
        fgc_core::find_match(&tournament, &source)
            .expect("source")
            .next
            .is_none()
    );
}

#[test]
fn t4_slot_edits_reject_duplicates_and_missing_slots() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);

    let first = slot(BracketSection::Winners, 0, 0, 1);
    let other = slot(BracketSection::Winners, 0, 0, 2);
    let duplicate = fgc_core::find_slot_match(&tournament, other)
        .expect("match")
        .slot2
        .clone();
    assert!(duplicate.is_some());
    assert_eq!(
        set_slot(&mut tournament, first, duplicate),
        Err(SlotError::SameTeam)
    );
    assert_eq!(
        set_slot(
            &mut tournament,
            slot(BracketSection::Winners, 9, 0, 1),
            None
        ),
        Err(SlotError::NotFound)
    );
    assert_eq!(
        set_slot(
            &mut tournament,
            slot(BracketSection::Winners, 0, 0, 3),
            None
        ),
        Err(SlotError::InvalidSlot)
    );
}

// ---------------------------------------------------------------------------
// T5: auto-advance and manual override
// ---------------------------------------------------------------------------

#[test]
fn t5_auto_advance_fills_slots_and_respects_manual_placements() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);

    let first = tournament.winners[0].matches[0].id.clone();
    let second = tournament.winners[0].matches[1].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();

    complete(&mut tournament, &first, &team("team-1"));
    refresh(&mut tournament);
    assert_eq!(
        fgc_core::find_match(&tournament, &final_id)
            .expect("final")
            .slot1,
        Some(team("team-1"))
    );

    // Manually place a team in the final; completing the other semifinal must
    // not overwrite it.
    set_slot(
        &mut tournament,
        slot(BracketSection::Winners, 1, 0, 1),
        Some(team("team-3")),
    )
    .expect("manual slot");
    complete(&mut tournament, &second, &team("team-2"));
    refresh(&mut tournament);

    let final_game = fgc_core::find_match(&tournament, &final_id).expect("final");
    assert_eq!(final_game.slot1, Some(team("team-3")));
    assert_eq!(final_game.slot2, Some(team("team-2")));
    assert!(final_game.slot1_manual);
    assert!(!final_game.slot2_manual);

    complete(&mut tournament, &final_id, &team("team-2"));
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-2")));
}

// ---------------------------------------------------------------------------
// T6: recursive clearing
// ---------------------------------------------------------------------------

#[test]
fn t6_clearing_a_result_recursively_clears_downstream_slots() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);

    let first = tournament.winners[0].matches[0].id.clone();
    let second = tournament.winners[0].matches[1].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();

    complete(&mut tournament, &first, &team("team-1"));
    complete(&mut tournament, &second, &team("team-2"));
    refresh(&mut tournament);
    complete(&mut tournament, &final_id, &team("team-1"));
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-1")));

    // Clearing a semifinal clears the final's auto-filled slot, the final
    // result and the champion.
    reset_match(&mut tournament, &first);
    refresh(&mut tournament);
    let final_game = fgc_core::find_match(&tournament, &final_id).expect("final");
    assert_eq!(final_game.slot1, None);
    assert_eq!(final_game.status, BracketMatchStatus::Pending);
    assert_eq!(final_game.winner, None);
    assert_eq!(tournament.champion, None);

    // Re-completing it restores propagation.
    complete(&mut tournament, &first, &team("team-4"));
    refresh(&mut tournament);
    assert_eq!(
        fgc_core::find_match(&tournament, &final_id)
            .expect("final")
            .slot1,
        Some(team("team-4"))
    );
    complete(&mut tournament, &final_id, &team("team-4"));
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-4")));
}

// ---------------------------------------------------------------------------
// T7: grand-final reset and manual overrides
// ---------------------------------------------------------------------------

#[test]
fn t7_grand_final_reset_creates_a_decider_and_crowns_the_winner() {
    let mut tournament = tournament_with_teams(4, BracketFormat::DoubleElim);
    generate_double_elimination(&mut tournament);
    let grand_final_id = tournament.grand_final[0].matches[0].id.clone();

    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 1),
        Some(team("team-1")),
    )
    .expect("slot 1");
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 2),
        Some(team("team-2")),
    )
    .expect("slot 2");

    // The losers-bracket team wins the first grand final.
    complete(&mut tournament, &grand_final_id, &team("team-2"));
    refresh(&mut tournament);
    assert_eq!(tournament.grand_final.len(), 2);
    let reset_id = tournament.grand_final[1].matches[0].id.clone();
    let reset = fgc_core::find_match(&tournament, &reset_id).expect("reset");
    assert_eq!(reset.slot1, Some(team("team-1")));
    assert_eq!(reset.slot2, Some(team("team-2")));
    assert_eq!(tournament.champion, None);

    complete(&mut tournament, &reset_id, &team("team-1"));
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-1")));
}

#[test]
fn t7_reset_can_be_disabled_and_manual_resets_are_kept() {
    let mut tournament = tournament_with_teams(4, BracketFormat::DoubleElim);
    tournament.grand_finals_reset = false;
    generate_double_elimination(&mut tournament);
    let grand_final_id = tournament.grand_final[0].matches[0].id.clone();
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 1),
        Some(team("team-1")),
    )
    .expect("slot 1");
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 2),
        Some(team("team-2")),
    )
    .expect("slot 2");
    complete(&mut tournament, &grand_final_id, &team("team-2"));
    refresh(&mut tournament);
    assert_eq!(tournament.grand_final.len(), 1);
    assert_eq!(tournament.champion, Some(team("team-2")));

    // A manually edited reset survives when the winners-bracket team wins.
    let mut tournament = tournament_with_teams(4, BracketFormat::DoubleElim);
    generate_double_elimination(&mut tournament);
    let grand_final_id = tournament.grand_final[0].matches[0].id.clone();
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 1),
        Some(team("team-1")),
    )
    .expect("slot 1");
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 0, 0, 2),
        Some(team("team-2")),
    )
    .expect("slot 2");
    complete(&mut tournament, &grand_final_id, &team("team-2"));
    refresh(&mut tournament);
    assert_eq!(tournament.grand_final.len(), 2);
    set_slot(
        &mut tournament,
        slot(BracketSection::GrandFinal, 1, 0, 1),
        Some(team("team-1")),
    )
    .expect("manual reset slot");
    complete(&mut tournament, &grand_final_id, &team("team-1"));
    refresh(&mut tournament);
    assert_eq!(tournament.grand_final.len(), 2);
    assert_eq!(tournament.champion, Some(team("team-1")));
}

// ---------------------------------------------------------------------------
// T8: champion derivation and manual override
// ---------------------------------------------------------------------------

#[test]
fn t8_champion_is_derived_unless_manually_overridden() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);
    let first = tournament.winners[0].matches[0].id.clone();
    let second = tournament.winners[0].matches[1].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();
    complete(&mut tournament, &first, &team("team-1"));
    complete(&mut tournament, &second, &team("team-2"));
    refresh(&mut tournament);
    complete(&mut tournament, &final_id, &team("team-1"));
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-1")));
    assert_eq!(tournament.status, TournamentStatus::Completed);

    // Manual override wins over the derived champion.
    tournament.champion = Some(team("team-2"));
    tournament.champion_manual = true;
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-2")));

    // An override outside the entrant list is rejected and the flag cleared.
    tournament.champion = Some(team("outsider"));
    tournament.champion_manual = true;
    refresh(&mut tournament);
    assert_eq!(tournament.champion, None);
    assert!(!tournament.champion_manual);

    // Releasing the override falls back to the derived champion.
    tournament.champion = Some(team("team-2"));
    tournament.champion_manual = false;
    refresh(&mut tournament);
    assert_eq!(tournament.champion, Some(team("team-1")));
}

// ---------------------------------------------------------------------------
// T9: ordered tiebreakers and head-to-head
// ---------------------------------------------------------------------------

#[test]
fn t9_tiebreakers_are_ordered_and_head_to_head_uses_a_mini_table() {
    let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
    generate_round_robin(&mut tournament, 1);

    // team-1 and team-2 both finish 2-1. team-2 won the head-to-head, while
    // team-1 has the better map differential.
    complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 0);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-4"), 2, 0);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-1"), 2, 1);
    complete_fixture(&mut tournament, &team("team-3"), &team("team-2"), 2, 0);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-4"), 2, 0);
    complete_fixture(&mut tournament, &team("team-4"), &team("team-3"), 2, 0);

    tournament.tiebreakers = vec![Tiebreaker::Wins, Tiebreaker::HeadToHead];
    refresh(&mut tournament);
    assert_eq!(
        row_order(&tournament, 0)
            .into_iter()
            .take(2)
            .collect::<Vec<_>>(),
        vec![team("team-2"), team("team-1")]
    );

    tournament.tiebreakers = vec![Tiebreaker::Wins, Tiebreaker::MapDifferential];
    refresh(&mut tournament);
    assert_eq!(
        row_order(&tournament, 0)
            .into_iter()
            .take(2)
            .collect::<Vec<_>>(),
        vec![team("team-1"), team("team-2")]
    );

    // The configured order survives persistence.
    tournament.tiebreakers = vec![
        Tiebreaker::MapDifferential,
        Tiebreaker::HeadToHead,
        Tiebreaker::Wins,
    ];
    let state = AppState {
        tournaments: vec![tournament.clone()],
        ..AppState::default()
    };
    let loaded =
        migrate(serde_json::to_value(&state).expect("serialise")).expect("migrate tournament");
    assert_eq!(loaded.tournaments[0].tiebreakers, tournament.tiebreakers);
}

// ---------------------------------------------------------------------------
// T10: playoff generation
// ---------------------------------------------------------------------------

#[test]
fn t10_playoff_cross_seeds_group_qualifiers_and_stays_editable() {
    let mut tournament = tournament_with_teams(8, BracketFormat::RoundRobin);
    generate_round_robin(&mut tournament, 2);

    // Seed order wins every fixture 2-0, making the standings deterministic.
    for group_index in 0..tournament.groups.len() {
        let teams = tournament.groups[group_index].team_ids.clone();
        for (left_index, left) in teams.iter().enumerate() {
            for right in &teams[left_index + 1..] {
                complete_fixture(&mut tournament, left, right, 2, 0);
            }
        }
    }
    refresh(&mut tournament);

    let group_a = tournament.groups[0].id.clone();
    let group_b = tournament.groups[1].id.clone();
    let a1 = tournament.standings[&group_a][0].team_id.clone();
    let a2 = tournament.standings[&group_a][1].team_id.clone();
    let b1 = tournament.standings[&group_b][0].team_id.clone();
    let b2 = tournament.standings[&group_b][1].team_id.clone();

    tournament.playoff_qualifiers = 2;
    generate_playoff(&mut tournament).expect("playoff");

    // Groups are preserved; the bracket uses cross-group seeding.
    assert_eq!(tournament.groups.len(), 2);
    assert_eq!(tournament.winners.len(), 2);
    let first_round = &tournament.winners[0].matches;
    assert_eq!(first_round[0].slot1, Some(a1));
    assert_eq!(first_round[0].slot2, Some(b2));
    assert_eq!(first_round[1].slot1, Some(b1));
    assert_eq!(first_round[1].slot2, Some(a2));

    // The generated playoff is still fully editable.
    let left = slot(BracketSection::Winners, 0, 0, 1);
    let right = slot(BracketSection::Winners, 0, 1, 1);
    let before = (
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1
            .clone(),
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1
            .clone(),
    );
    swap_slots(&mut tournament, left, right).expect("swap");
    assert_eq!(
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1,
        before.1
    );
    assert_eq!(
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1,
        before.0
    );

    // A double-elimination playoff builds the full board.
    tournament.playoff_format = BracketFormat::DoubleElim;
    generate_playoff(&mut tournament).expect("double-elim playoff");
    assert!(!tournament.losers.is_empty());
    assert_eq!(tournament.grand_final.len(), 1);
}

#[test]
fn t10_playoff_requires_groups_and_two_qualifiers() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    assert!(generate_playoff(&mut tournament).is_err());

    generate_round_robin(&mut tournament, 1);
    tournament.playoff_qualifiers = 1;
    // Only one team qualifies per group, so a single-group stage has one team.
    assert!(generate_playoff(&mut tournament).is_err());
}

// ---------------------------------------------------------------------------
// T11: match modal operations
// ---------------------------------------------------------------------------

#[test]
fn t11_match_scores_status_first_to_and_winner_override_are_editable() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);
    let first = tournament.winners[0].matches[0].id.clone();
    let second = tournament.winners[0].matches[1].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();

    {
        let game = match_mut(&mut tournament, &first).expect("first");
        game.first_to = 5;
        game.status = BracketMatchStatus::Live;
        game.score1 = 1;
        game.score2 = 0;
    }
    refresh(&mut tournament);
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.first_to, 5);
    assert_eq!(game.status, BracketMatchStatus::Live);
    assert_eq!((game.score1, game.score2), (1, 0));

    complete(&mut tournament, &first, &team("team-1"));
    complete(&mut tournament, &second, &team("team-2"));
    refresh(&mut tournament);

    // Explicit winner override: the scores favour slot 1 but slot 2 wins.
    {
        let game = match_mut(&mut tournament, &final_id).expect("final");
        game.status = BracketMatchStatus::Completed;
        game.score1 = 2;
        game.score2 = 0;
        game.winner = Some(team("team-2"));
        game.winner_manual = true;
    }
    refresh(&mut tournament);
    let final_game = fgc_core::find_match(&tournament, &final_id).expect("final");
    assert_eq!(final_game.winner, Some(team("team-2")));
    assert!(final_game.winner_manual);
    assert_eq!(tournament.champion, Some(team("team-2")));

    // Reset clears the result and the auto-filled downstream champion.
    reset_match(&mut tournament, &final_id);
    refresh(&mut tournament);
    assert_eq!(tournament.champion, None);
}

// ---------------------------------------------------------------------------
// T11: score-derived winners and group editing helpers
// ---------------------------------------------------------------------------

#[test]
fn t11_scores_derive_the_winner_and_manual_override_wins() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);
    let first = tournament.winners[0].matches[0].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();

    // Completing with scores derives the winner from the higher score.
    assert!(set_bracket_result(
        &mut tournament,
        &first,
        2,
        0,
        BracketMatchStatus::Completed
    ));
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.winner, Some(team("team-1")));
    assert!(!game.winner_manual);
    assert_eq!(
        fgc_core::find_match(&tournament, &final_id)
            .expect("final")
            .slot1,
        Some(team("team-1"))
    );

    // An explicit override beats the score-derived winner.
    let opponent = fgc_core::find_match(&tournament, &first)
        .expect("first")
        .slot2
        .clone()
        .expect("opponent");
    assert!(set_bracket_winner(
        &mut tournament,
        &first,
        Some(opponent.clone())
    ));
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.winner, Some(opponent));
    assert!(game.winner_manual);

    // A winner outside the two slots is rejected.
    assert!(!set_bracket_winner(
        &mut tournament,
        &first,
        Some(team("outsider"))
    ));

    // Clearing the override falls back to the scores.
    assert!(set_bracket_winner(&mut tournament, &first, None));
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.winner, Some(team("team-1")));
    assert!(!game.winner_manual);

    // A live score change clears the previous result.
    assert!(set_bracket_result(
        &mut tournament,
        &first,
        1,
        0,
        BracketMatchStatus::Live
    ));
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.status, BracketMatchStatus::Live);
    assert!(game.winner.is_none());

    // Resetting clears scores, winner and downstream auto-fills.
    assert!(reset_bracket_match(&mut tournament, &first));
    let game = fgc_core::find_match(&tournament, &first).expect("first");
    assert_eq!(game.status, BracketMatchStatus::Pending);
    assert_eq!((game.score1, game.score2), (0, 0));
    assert!(game.winner.is_none());
}

#[test]
fn t11_groups_can_be_renamed_resized_and_fixtures_edited() {
    let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
    generate_round_robin(&mut tournament, 1);
    let group_id = tournament.groups[0].id.clone();
    assert_eq!(tournament.group_matches.len(), 6);

    // Rename.
    assert!(rename_group(&mut tournament, &group_id, "Group Alpha"));
    assert_eq!(tournament.groups[0].name, "Group Alpha");

    // Removing a team drops its unplayed fixtures and keeps played records.
    let fixture = tournament.group_matches[0].id.clone();
    assert!(set_group_result(
        &mut tournament,
        &fixture,
        2,
        0,
        BracketMatchStatus::Completed
    ));
    let removed_team = tournament.groups[0].team_ids[3].clone();
    let remaining = tournament.groups[0].team_ids[..3].to_vec();
    assert!(set_group_teams(
        &mut tournament,
        &group_id,
        remaining.clone()
    ));
    assert_eq!(tournament.groups[0].team_ids, remaining);
    for fixture in &tournament.group_matches {
        assert_ne!(fixture.team1.as_ref(), Some(&removed_team));
        assert_ne!(fixture.team2.as_ref(), Some(&removed_team));
    }

    // Duplicate team entries are dropped.
    assert!(set_group_teams(
        &mut tournament,
        &group_id,
        vec![
            remaining[0].clone(),
            remaining[0].clone(),
            remaining[1].clone(),
        ]
    ));
    assert_eq!(tournament.groups[0].team_ids.len(), 2);

    // Manual fixtures can be added and removed.
    let manual = add_group_fixture(
        &mut tournament,
        &group_id,
        remaining[0].clone(),
        remaining[1].clone(),
    )
    .expect("manual fixture");
    assert!(
        tournament
            .group_matches
            .iter()
            .any(|fixture| fixture.id == manual)
    );
    assert!(remove_group_fixture(&mut tournament, &manual));
    assert!(
        !tournament
            .group_matches
            .iter()
            .any(|fixture| fixture.id == manual)
    );

    // Removing the group removes its fixtures and standings.
    assert!(remove_group(&mut tournament, &group_id));
    assert!(tournament.groups.is_empty());
    assert!(tournament.group_matches.is_empty());
    assert!(tournament.standings.is_empty());
}

// ---------------------------------------------------------------------------
// T12: persistence and repair
// ---------------------------------------------------------------------------

#[test]
fn t12_completed_round_robin_round_trips_and_recomputes_standings() {
    let mut tournament = tournament_with_teams(4, BracketFormat::RoundRobin);
    generate_round_robin(&mut tournament, 1);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-2"), 2, 0);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-3"), 2, 1);
    complete_fixture(&mut tournament, &team("team-1"), &team("team-4"), 1, 2);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-3"), 2, 0);
    complete_fixture(&mut tournament, &team("team-2"), &team("team-4"), 2, 1);
    complete_fixture(&mut tournament, &team("team-3"), &team("team-4"), 2, 0);
    refresh(&mut tournament);

    let expected = tournament.standings.clone();
    let state = AppState {
        tournaments: vec![tournament.clone()],
        ..AppState::default()
    };
    let value = serde_json::to_value(&state).expect("serialise");
    assert!(
        value["tournaments"][0].get("standings").is_none(),
        "standings must not be persisted"
    );

    let loaded = migrate(value).expect("migrate tournament");
    assert_eq!(loaded.tournaments[0].standings, expected);
    assert_eq!(loaded.tournaments[0].group_matches.len(), 6);
    assert_eq!(
        loaded.tournaments[0]
            .group_matches
            .iter()
            .filter(|fixture| fixture.status == BracketMatchStatus::Completed)
            .count(),
        6
    );
}

#[test]
fn t12_malformed_tournament_data_is_repaired() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);
    let state = AppState {
        tournaments: vec![tournament],
        ..AppState::default()
    };
    let mut value = serde_json::to_value(&state).expect("serialise");

    // Duplicate seed.
    value["tournaments"][0]["entrants"][1]["seed"] =
        value["tournaments"][0]["entrants"][0]["seed"].clone();
    // Completed match without a winner.
    value["tournaments"][0]["winners"][0]["matches"][0]["status"] = serde_json::json!("completed");
    value["tournaments"][0]["winners"][0]["matches"][0]["winner"] = serde_json::Value::Null;
    // Dangling link.
    value["tournaments"][0]["winners"][1]["matches"][0]["next"] =
        serde_json::json!({ "match": "missing", "slot": 1 });
    // Champion outside the entrant list.
    value["tournaments"][0]["champion"] = serde_json::json!("outsider");

    let loaded = migrate(value).expect("migrate malformed tournament");
    let tournament = &loaded.tournaments[0];

    let seeds: Vec<u32> = tournament
        .entrants
        .iter()
        .map(|entrant| entrant.seed)
        .collect();
    assert_eq!(seeds, vec![1, 2, 3, 4]);
    assert_eq!(
        tournament.winners[0].matches[0].status,
        BracketMatchStatus::Pending
    );
    assert!(tournament.winners[0].matches[0].winner.is_none());
    assert!(tournament.winners[1].matches[0].next.is_none());
    assert!(tournament.champion.is_none());
}

#[test]
fn t12_repair_reports_every_documented_rule() {
    let mut tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    generate_single_elimination(&mut tournament);
    tournament.entrants[1].seed = tournament.entrants[0].seed;
    let first = tournament.winners[0].matches[0].id.clone();
    let final_id = tournament.winners[1].matches[0].id.clone();
    {
        let game = match_mut(&mut tournament, &first).expect("first");
        game.status = BracketMatchStatus::Completed;
        game.winner = None;
    }
    {
        let game = match_mut(&mut tournament, &final_id).expect("final");
        game.next = Some(MatchLink {
            match_id: MatchId::from_raw("missing"),
            slot: 1,
        });
    }
    tournament.champion = Some(team("outsider"));

    let repairs = repair_tournament(&mut tournament);
    let kinds: HashSet<TournamentRepairKind> = repairs.iter().map(|repair| repair.kind).collect();
    assert!(kinds.contains(&TournamentRepairKind::DuplicateSeed));
    assert!(kinds.contains(&TournamentRepairKind::CompletedWithoutWinner));
    assert!(kinds.contains(&TournamentRepairKind::DanglingLink));
    assert!(kinds.contains(&TournamentRepairKind::ChampionNotEntrant));
    assert_eq!(
        fgc_core::find_match(&tournament, &first)
            .expect("first")
            .status,
        BracketMatchStatus::Pending
    );
    assert!(tournament.champion.is_none());
}

// ---------------------------------------------------------------------------
// T14: a 32-team double-elimination board is complete and addressable
// ---------------------------------------------------------------------------

#[test]
fn t14_thirty_two_team_double_elimination_board_is_complete() {
    let mut tournament = tournament_with_teams(32, BracketFormat::DoubleElim);
    generate_double_elimination(&mut tournament);

    assert_eq!(all_matches(&tournament).len(), 62);
    assert_eq!(match_ids(&tournament).len(), 62);
    assert_eq!(tournament.winners.len(), 5);
    assert_eq!(tournament.losers.len(), 8);
    assert_eq!(tournament.grand_final.len(), 1);

    for (round_index, round) in rounds(&tournament, BracketSection::Winners)
        .iter()
        .enumerate()
    {
        for (match_index, game) in round.matches.iter().enumerate() {
            assert_eq!(
                match_address(&tournament, &game.id),
                Some((BracketSection::Winners, round_index, match_index))
            );
        }
    }

    // Keyboard slot movement is a UI binding for `swap_slots`; the core
    // operation must work on the largest supported board.
    let left = slot(BracketSection::Winners, 0, 0, 1);
    let right = slot(BracketSection::Winners, 0, 1, 1);
    let before = (
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1
            .clone(),
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1
            .clone(),
    );
    assert!(before.0.is_some() && before.1.is_some());
    swap_slots(&mut tournament, left, right).expect("swap");
    assert_eq!(
        fgc_core::find_slot_match(&tournament, left)
            .expect("left")
            .slot1,
        before.1
    );
    assert_eq!(
        fgc_core::find_slot_match(&tournament, right)
            .expect("right")
            .slot1,
        before.0
    );
}

// ---------------------------------------------------------------------------
// Entry helpers and normalization
// ---------------------------------------------------------------------------

#[test]
fn entrants_normalize_to_contiguous_seeds_without_reordering() {
    let mut tournament = new_tournament("Seeds", BracketFormat::SingleElim);
    tournament.entrants = vec![
        Entrant {
            team_id: team("team-a"),
            seed: 3,
        },
        Entrant {
            team_id: team("team-b"),
            seed: 0,
        },
        Entrant {
            team_id: team("team-c"),
            seed: 1,
        },
        Entrant {
            team_id: team("team-d"),
            seed: 3,
        },
    ];
    let changed = fgc_core::normalize_seeds(&mut tournament);
    assert!(changed > 0);
    // Seeds become contiguous without moving entrants around: the entrant
    // list order is untouched and relative seed order is preserved.
    let ids: Vec<&str> = tournament
        .entrants
        .iter()
        .map(|entrant| entrant.team_id.as_str())
        .collect();
    assert_eq!(ids, vec!["team-a", "team-b", "team-c", "team-d"]);
    let mut seeds: Vec<u32> = tournament
        .entrants
        .iter()
        .map(|entrant| entrant.seed)
        .collect();
    seeds.sort_unstable();
    assert_eq!(seeds, vec![1, 2, 3, 4]);
    assert_eq!(tournament.entrants[2].seed, 1);
    assert_eq!(tournament.entrants[0].seed, 2);
    assert_eq!(tournament.entrants[3].seed, 3);
    assert_eq!(tournament.entrants[1].seed, 4);
}

#[test]
fn tournament_id_round_trips_through_persistence() {
    let tournament = tournament_with_teams(4, BracketFormat::SingleElim);
    let id = TournamentId::from_raw("tournament-a");
    let mut tournament = tournament;
    tournament.id = id.clone();
    let state = AppState {
        tournaments: vec![tournament],
        ..AppState::default()
    };
    let loaded = migrate(serde_json::to_value(&state).expect("serialise")).expect("migrate");
    assert_eq!(loaded.tournaments[0].id, id);
}
