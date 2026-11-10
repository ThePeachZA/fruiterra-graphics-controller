//! Cross-entity reference discovery for destructive edits.
//!
//! Deleting a team that a tournament or the current match still uses would
//! leave dangling identifiers. These helpers report every place a team is
//! referenced and remove those references when the user explicitly asks to
//! clean up and delete.

use crate::{AppState, Round, TalentId, TeamId, TournamentId};

/// One place a team is referenced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamReference {
    /// Tournament identifier, or `None` for the current match.
    pub tournament_id: Option<TournamentId>,
    /// Display name of the tournament, or "Current match".
    pub name: String,
    /// Distinct contexts within the tournament or match.
    pub contexts: Vec<&'static str>,
}

impl TeamReference {
    /// One-line description for the deletion dialog.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.contexts.is_empty() {
            self.name.clone()
        } else {
            format!("{} — {}", self.name, self.contexts.join(", "))
        }
    }
}

/// Returns every tournament and current-match reference to `team`.
#[must_use]
pub fn team_references(state: &AppState, team: &TeamId) -> Vec<TeamReference> {
    let mut references = Vec::new();

    for tournament in &state.tournaments {
        let mut contexts = Vec::new();
        if tournament
            .entrants
            .iter()
            .any(|entrant| &entrant.team_id == team)
        {
            contexts.push("entrants");
        }
        if rounds_reference(&tournament.winners, team) {
            contexts.push("winners bracket");
        }
        if rounds_reference(&tournament.losers, team) {
            contexts.push("losers bracket");
        }
        if rounds_reference(&tournament.grand_final, team) {
            contexts.push("grand final");
        }
        if tournament
            .groups
            .iter()
            .any(|group| group.team_ids.contains(team))
        {
            contexts.push("groups");
        }
        if tournament
            .group_matches
            .iter()
            .any(|game| game.team1.as_ref() == Some(team) || game.team2.as_ref() == Some(team))
        {
            contexts.push("group fixtures");
        }
        if tournament.champion.as_ref() == Some(team) {
            contexts.push("champion");
        }

        if !contexts.is_empty() {
            let name = if tournament.name.trim().is_empty() {
                "Untitled tournament".to_owned()
            } else {
                tournament.name.clone()
            };
            references.push(TeamReference {
                tournament_id: Some(tournament.id.clone()),
                name,
                contexts,
            });
        }
    }

    if let Some(current) = &state.current_match {
        let mut contexts = Vec::new();
        if current.team_a.as_ref() == Some(team) {
            contexts.push("team A");
        }
        if current.team_b.as_ref() == Some(team) {
            contexts.push("team B");
        }
        if !contexts.is_empty() {
            references.push(TeamReference {
                tournament_id: None,
                name: "Current match".to_owned(),
                contexts,
            });
        }
    }

    references
}

/// Removes every tournament and current-match reference to `team`.
///
/// This is the explicit "remove from tournaments and delete" cleanup path.
/// Scores and statuses are left untouched; the owning tournament view owns
/// bracket consistency once the slot or winner is cleared.
pub fn remove_team_references(state: &mut AppState, team: &TeamId) {
    for tournament in &mut state.tournaments {
        tournament
            .entrants
            .retain(|entrant| &entrant.team_id != team);

        for round in tournament
            .winners
            .iter_mut()
            .chain(tournament.losers.iter_mut())
            .chain(tournament.grand_final.iter_mut())
        {
            for game in &mut round.matches {
                if game.slot1.as_ref() == Some(team) {
                    game.slot1 = None;
                    game.slot1_manual = false;
                }
                if game.slot2.as_ref() == Some(team) {
                    game.slot2 = None;
                    game.slot2_manual = false;
                }
                if game.winner.as_ref() == Some(team) {
                    game.winner = None;
                    game.winner_manual = false;
                }
            }
        }

        for group in &mut tournament.groups {
            group.team_ids.retain(|id| id != team);
        }
        for game in &mut tournament.group_matches {
            if game.team1.as_ref() == Some(team) {
                game.team1 = None;
            }
            if game.team2.as_ref() == Some(team) {
                game.team2 = None;
            }
        }

        if tournament.champion.as_ref() == Some(team) {
            tournament.champion = None;
            tournament.champion_manual = false;
        }
    }

    if let Some(current) = &mut state.current_match {
        if current.team_a.as_ref() == Some(team) {
            current.team_a = None;
        }
        if current.team_b.as_ref() == Some(team) {
            current.team_b = None;
        }
    }
}

/// Clears every current-match assignment that points at `talent`.
pub fn clear_talent_references(state: &mut AppState, talent: &TalentId) {
    let Some(current) = &mut state.current_match else {
        return;
    };

    for slot in [
        &mut current.talent.caster1,
        &mut current.talent.caster2,
        &mut current.talent.panel1,
        &mut current.talent.panel2,
        &mut current.talent.panel3,
        &mut current.talent.interviewer,
    ] {
        if slot
            .as_ref()
            .is_some_and(|reference| &reference.talent_id == talent)
        {
            *slot = None;
        }
    }

    if current
        .talent
        .interviewee
        .as_ref()
        .is_some_and(|interviewee| interviewee.talent_id.as_ref() == Some(talent))
    {
        current.talent.interviewee = None;
    }
}

fn rounds_reference(rounds: &[Round], team: &TeamId) -> bool {
    rounds.iter().flat_map(|round| &round.matches).any(|game| {
        game.slot1.as_ref() == Some(team)
            || game.slot2.as_ref() == Some(team)
            || game.winner.as_ref() == Some(team)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Match, TalentId, TeamId, Tournament};
    use serde_json::json;

    fn tournament(value: serde_json::Value) -> Tournament {
        serde_json::from_value(value).expect("valid tournament fixture")
    }

    fn current_match(value: serde_json::Value) -> Match {
        serde_json::from_value(value).expect("valid match fixture")
    }

    #[test]
    fn unreferenced_teams_have_no_references() {
        let state = AppState::default();
        assert!(team_references(&state, &TeamId::from_raw("missing")).is_empty());
    }

    #[test]
    fn every_tournament_and_match_reference_is_reported() {
        let mut state = AppState::default();
        let team = TeamId::from_raw("team-1");
        state.tournaments.push(tournament(json!({
            "id": "tournament-1",
            "name": "Test Cup",
            "entrants": [{ "teamId": "team-1", "seed": 1 }],
            "winners": [{
                "name": "Round 1",
                "matches": [{ "id": "bracket-1", "slot1": "team-1", "winner": "team-1" }]
            }],
            "losers": [{
                "name": "Lower 1",
                "matches": [{ "id": "bracket-2", "slot2": "team-1" }]
            }],
            "grandFinal": [{
                "name": "Grand final",
                "matches": [{ "id": "bracket-3", "slot1": "team-1" }]
            }],
            "groups": [{ "id": "group-1", "name": "Group A", "teamIds": ["team-1"] }],
            "groupMatches": [{ "id": "group-match-1", "group": "group-1", "team1": "team-1" }],
            "champion": "team-1"
        })));
        state.current_match = Some(current_match(json!({
            "id": "match-1",
            "teamA": "team-1",
            "teamB": "team-1"
        })));

        let references = team_references(&state, &team);
        assert_eq!(references.len(), 2);
        assert_eq!(references[0].name, "Test Cup");
        assert_eq!(
            references[0].contexts,
            vec![
                "entrants",
                "winners bracket",
                "losers bracket",
                "grand final",
                "groups",
                "group fixtures",
                "champion",
            ]
        );
        assert_eq!(references[1].name, "Current match");
        assert_eq!(references[1].contexts, vec!["team A", "team B"]);
    }

    #[test]
    fn cleanup_removes_every_reference() {
        let mut state = AppState::default();
        let team = TeamId::from_raw("team-1");
        state.tournaments.push(tournament(json!({
            "id": "tournament-1",
            "name": "Test Cup",
            "entrants": [{ "teamId": "team-1", "seed": 1 }],
            "winners": [{
                "name": "Round 1",
                "matches": [{
                    "id": "bracket-1",
                    "slot1": "team-1",
                    "slot2": "team-1",
                    "winner": "team-1",
                    "slot1Manual": true,
                    "slot2Manual": true,
                    "winnerManual": true
                }]
            }],
            "groups": [{ "id": "group-1", "teamIds": ["team-1"] }],
            "groupMatches": [{ "id": "group-match-1", "group": "group-1", "team1": "team-1", "team2": "team-1" }],
            "champion": "team-1",
            "championManual": true
        })));
        state.current_match = Some(current_match(json!({
            "id": "match-1",
            "teamA": "team-1",
            "teamB": "team-1"
        })));

        remove_team_references(&mut state, &team);

        assert!(team_references(&state, &team).is_empty());
        let tournament = &state.tournaments[0];
        assert!(tournament.entrants.is_empty());
        assert!(tournament.groups[0].team_ids.is_empty());
        assert!(tournament.group_matches[0].team1.is_none());
        assert!(tournament.group_matches[0].team2.is_none());
        assert!(tournament.champion.is_none());
        assert!(!tournament.champion_manual);
        let game = &tournament.winners[0].matches[0];
        assert!(game.slot1.is_none());
        assert!(game.slot2.is_none());
        assert!(game.winner.is_none());
        assert!(!game.slot1_manual);
        assert!(!game.slot2_manual);
        assert!(!game.winner_manual);
        let current = state.current_match.as_ref().expect("current match");
        assert!(current.team_a.is_none());
        assert!(current.team_b.is_none());
    }

    #[test]
    fn deleting_a_talent_clears_match_assignments() {
        let mut state = AppState::default();
        let talent = TalentId::from_raw("talent-1");
        state.current_match = Some(current_match(json!({
            "id": "match-1",
            "talent": {
                "caster1": { "talentId": "talent-1" },
                "panel3": { "talentId": "talent-1" },
                "interviewee": { "talentId": "talent-1", "customName": "Guest" }
            }
        })));

        clear_talent_references(&mut state, &talent);

        let current = state.current_match.as_ref().expect("current match");
        assert!(current.talent.caster1.is_none());
        assert!(current.talent.panel3.is_none());
        assert!(current.talent.interviewee.is_none());
    }
}
