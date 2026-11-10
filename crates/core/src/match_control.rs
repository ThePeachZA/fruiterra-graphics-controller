//! Match-controller helpers shared by the UI and the projection tests.
//!
//! The original Electron controller kept the current match in React state and
//! recalculated loser picks on every edit. These functions move that logic into
//! the domain crate so the same rules are exercised without a UI: eight empty
//! map slots by default, the first map picked by team A, and every later map
//! picked by the loser of the most recent map (or the previous picker after a
//! draw).

use crate::{MapHeroBans, MapSlot, Match, MatchId, MatchTalent, TalentRef, TeamSide};

/// Number of map slots created for a new match.
pub const DEFAULT_MAP_SLOTS: usize = 8;

/// Creates an empty map slot.
#[must_use]
pub fn empty_map_slot() -> MapSlot {
    MapSlot {
        name: String::new(),
        game_mode_name: String::new(),
        completed: false,
        score_a: 0,
        score_b: 0,
        chosen_by: None,
    }
}

/// Creates empty hero bans for one map.
#[must_use]
pub fn empty_hero_bans() -> MapHeroBans {
    MapHeroBans {
        first_ban_team: None,
        team_a: Vec::new(),
        team_b: Vec::new(),
    }
}

/// Creates the match used by the original "new match" workflow.
#[must_use]
pub fn new_match() -> Match {
    Match {
        id: MatchId::new(),
        title: String::new(),
        subtitle: String::new(),
        team_a: None,
        team_b: None,
        maps: (0..DEFAULT_MAP_SLOTS).map(|_| empty_map_slot()).collect(),
        hero_bans: (0..DEFAULT_MAP_SLOTS).map(|_| empty_hero_bans()).collect(),
        map_pool: Vec::new(),
        first_to: 3,
        status: crate::MatchStatus::Pre,
        current_map: None,
        swap_sides: false,
        scoring: crate::Scoring::Derived,
        side: crate::Side::Neutral,
        talent: MatchTalent::default(),
        loser_picks: true,
        show_player_portraits: true,
    }
}

/// Returns the losing side of a completed map, or `None` for a draw.
#[must_use]
pub fn losing_side(map: &MapSlot) -> Option<TeamSide> {
    match map.score_a.cmp(&map.score_b) {
        std::cmp::Ordering::Greater => Some(TeamSide::B),
        std::cmp::Ordering::Less => Some(TeamSide::A),
        std::cmp::Ordering::Equal => None,
    }
}

/// Returns the team that picks the map at `index`.
///
/// The first map is always picked by team A. Later maps are picked by the loser
/// of the closest preceding named map, or by that map's original picker when it
/// was a draw. Empty slots before the first named map fall back to team A.
#[must_use]
pub fn next_chooser(maps: &[MapSlot], index: usize) -> TeamSide {
    if index == 0 {
        return TeamSide::A;
    }

    let previous = maps[..index.min(maps.len())]
        .iter()
        .rev()
        .find(|map| !map.name.trim().is_empty());

    previous
        .and_then(|map| losing_side(map).or(map.chosen_by))
        .unwrap_or(TeamSide::A)
}

/// Recalculates `chosen_by` for every slot after loser picks is enabled.
///
/// Empty slots keep no picker; named slots follow [`next_chooser`].
pub fn recalculate_loser_picks(maps: &mut [MapSlot]) {
    for index in 0..maps.len() {
        let chosen_by = if maps[index].name.trim().is_empty() {
            None
        } else {
            Some(next_chooser(maps, index))
        };
        maps[index].chosen_by = chosen_by;
    }
}

/// Updates only the next map's picker after a score edit.
///
/// The original controller behaved this way so editing an earlier result does
/// not rewrite every later map.
pub fn update_next_chooser(maps: &mut [MapSlot], index: usize) {
    let Some(current) = maps.get(index) else {
        return;
    };
    let Some(loser) = losing_side(current).or(current.chosen_by) else {
        return;
    };
    if let Some(next) = maps.get_mut(index + 1)
        && !next.name.trim().is_empty()
    {
        next.chosen_by = Some(loser);
    }
}

/// Updates a map name and its mode, recalculating the picker when loser picks
/// is enabled.
pub fn set_map_name(match_state: &mut Match, index: usize, name: &str, mode: &str) {
    let empty = name.trim().is_empty();
    let chosen_by = if empty {
        None
    } else if match_state.loser_picks {
        Some(next_chooser(&match_state.maps, index))
    } else {
        match_state.maps.get(index).and_then(|map| map.chosen_by)
    };

    let Some(map) = match_state.maps.get_mut(index) else {
        return;
    };
    map.name = name.to_owned();
    map.game_mode_name = if empty {
        String::new()
    } else {
        mode.to_owned()
    };
    map.chosen_by = chosen_by;
}

/// Updates one map's score and, with loser picks enabled, the next map's picker.
pub fn set_map_scores(match_state: &mut Match, index: usize, score_a: u16, score_b: u16) {
    let Some(map) = match_state.maps.get_mut(index) else {
        return;
    };
    map.score_a = score_a;
    map.score_b = score_b;
    if match_state.loser_picks {
        update_next_chooser(&mut match_state.maps, index);
    }
}

/// Replaces every map slot and hero-ban row with empty defaults.
pub fn reset_maps(match_state: &mut Match) {
    match_state.maps = (0..DEFAULT_MAP_SLOTS).map(|_| empty_map_slot()).collect();
    match_state.hero_bans = (0..DEFAULT_MAP_SLOTS).map(|_| empty_hero_bans()).collect();
    match_state.current_map = None;
}

/// Clears hero bans for every map slot.
pub fn reset_hero_bans(match_state: &mut Match) {
    match_state.hero_bans = match_state.maps.iter().map(|_| empty_hero_bans()).collect();
}

/// Grows or shrinks the hero-ban list to match the map slots.
pub fn ensure_hero_bans(match_state: &mut Match) {
    match_state
        .hero_bans
        .resize_with(match_state.maps.len(), empty_hero_bans);
}

/// Inserts an empty map and its hero-ban row at `index`.
pub fn insert_map(match_state: &mut Match, index: usize) {
    let index = index.min(match_state.maps.len());
    let previous_len = match_state.maps.len();
    match_state.maps.insert(index, empty_map_slot());
    match_state
        .hero_bans
        .resize_with(previous_len, empty_hero_bans);
    match_state.hero_bans.insert(index, empty_hero_bans());
    match_state.hero_bans.truncate(match_state.maps.len());

    if let Some(current) = match_state.current_map
        && current >= index
    {
        match_state.current_map = Some(current + 1);
    }
}

/// Removes one map and its hero-ban row.
pub fn remove_map(match_state: &mut Match, index: usize) {
    if index >= match_state.maps.len() {
        return;
    }
    let previous_len = match_state.maps.len();
    match_state.maps.remove(index);
    match_state
        .hero_bans
        .resize_with(previous_len, empty_hero_bans);
    match_state.hero_bans.remove(index);
    match_state.hero_bans.truncate(match_state.maps.len());

    match_state.current_map = match_state.current_map.and_then(|current| {
        if current == index {
            None
        } else if current > index {
            Some(current - 1)
        } else {
            Some(current)
        }
    });
}

/// Moves a map by `delta` positions, keeping hero bans aligned.
///
/// Returns the new index, or `None` when the move would leave the list.
pub fn move_map(match_state: &mut Match, index: usize, delta: isize) -> Option<usize> {
    let target = index.checked_add_signed(delta)?;
    if index >= match_state.maps.len() || target >= match_state.maps.len() {
        return None;
    }
    if index == target {
        return Some(index);
    }

    match_state.maps.swap(index, target);
    ensure_hero_bans(match_state);
    if index < match_state.hero_bans.len() && target < match_state.hero_bans.len() {
        match_state.hero_bans.swap(index, target);
    }

    match_state.current_map = match_state.current_map.map(|current| {
        if current == index {
            target
        } else if current == target {
            index
        } else {
            current
        }
    });

    Some(target)
}

/// Adds or removes a map name from the pool, preserving insertion order.
pub fn toggle_map_pool(match_state: &mut Match, name: &str, included: bool) {
    if included {
        if !match_state.map_pool.iter().any(|item| item == name) {
            match_state.map_pool.push(name.to_owned());
        }
    } else {
        match_state.map_pool.retain(|item| item != name);
    }
}

/// Moves a map-pool entry by `delta` positions.
pub fn move_map_pool(match_state: &mut Match, index: usize, delta: isize) -> Option<usize> {
    let target = index.checked_add_signed(delta)?;
    if index >= match_state.map_pool.len() || target >= match_state.map_pool.len() {
        return None;
    }
    match_state.map_pool.swap(index, target);
    Some(target)
}

/// Sets the team that placed the first ban for one map.
pub fn set_first_ban_team(match_state: &mut Match, index: usize, team: Option<TeamSide>) {
    ensure_hero_bans(match_state);
    if let Some(bans) = match_state.hero_bans.get_mut(index) {
        bans.first_ban_team = team;
    }
}

/// Replaces one team's hero ban for one map.
///
/// The overlays show one ban per team, so a new value replaces the previous
/// one and an empty value clears it.
pub fn set_hero_ban(match_state: &mut Match, index: usize, side: TeamSide, hero: &str) {
    ensure_hero_bans(match_state);
    let Some(bans) = match_state.hero_bans.get_mut(index) else {
        return;
    };
    let value = if hero.trim().is_empty() {
        Vec::new()
    } else {
        vec![hero.to_owned()]
    };
    match side {
        TeamSide::A => bans.team_a = value,
        TeamSide::B => bans.team_b = value,
    }
}

/// Derived match score: completed map wins, ignoring draws.
#[must_use]
pub fn derived_scores(match_state: &Match) -> (u16, u16) {
    let wins_a = match_state
        .maps
        .iter()
        .filter(|slot| slot.completed && slot.score_a > slot.score_b)
        .count();
    let wins_b = match_state
        .maps
        .iter()
        .filter(|slot| slot.completed && slot.score_b > slot.score_a)
        .count();
    let cap = usize::from(u16::MAX);
    (wins_a.min(cap) as u16, wins_b.min(cap) as u16)
}

/// The six talent slots that hold a [`TalentRef`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TalentSlot {
    /// First caster.
    Caster1,
    /// Second caster.
    Caster2,
    /// First panel member.
    Panel1,
    /// Second panel member.
    Panel2,
    /// Third panel member.
    Panel3,
    /// Interviewer.
    Interviewer,
}

impl TalentSlot {
    /// Every slot in display order.
    pub const ALL: [Self; 6] = [
        Self::Caster1,
        Self::Caster2,
        Self::Panel1,
        Self::Panel2,
        Self::Panel3,
        Self::Interviewer,
    ];

    /// Label used by the match controller.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Caster1 => "Caster 1",
            Self::Caster2 => "Caster 2",
            Self::Panel1 => "Panel 1",
            Self::Panel2 => "Panel 2",
            Self::Panel3 => "Panel 3",
            Self::Interviewer => "Interviewer",
        }
    }
}

impl MatchTalent {
    /// Returns the assignment for `slot`.
    #[must_use]
    pub fn slot(&self, slot: TalentSlot) -> Option<&TalentRef> {
        match slot {
            TalentSlot::Caster1 => self.caster1.as_ref(),
            TalentSlot::Caster2 => self.caster2.as_ref(),
            TalentSlot::Panel1 => self.panel1.as_ref(),
            TalentSlot::Panel2 => self.panel2.as_ref(),
            TalentSlot::Panel3 => self.panel3.as_ref(),
            TalentSlot::Interviewer => self.interviewer.as_ref(),
        }
    }

    /// Replaces the assignment for `slot`.
    pub fn set_slot(&mut self, slot: TalentSlot, value: Option<TalentRef>) {
        match slot {
            TalentSlot::Caster1 => self.caster1 = value,
            TalentSlot::Caster2 => self.caster2 = value,
            TalentSlot::Panel1 => self.panel1 = value,
            TalentSlot::Panel2 => self.panel2 = value,
            TalentSlot::Panel3 => self.panel3 = value,
            TalentSlot::Interviewer => self.interviewer = value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MatchStatus, Scoring};

    fn named(name: &str, a: u16, b: u16, completed: bool, chosen_by: Option<TeamSide>) -> MapSlot {
        MapSlot {
            name: name.to_owned(),
            game_mode_name: "Control".to_owned(),
            completed,
            score_a: a,
            score_b: b,
            chosen_by,
        }
    }

    #[test]
    fn a_new_match_has_eight_empty_maps_and_bans() {
        let match_state = new_match();
        assert_eq!(match_state.maps.len(), DEFAULT_MAP_SLOTS);
        assert_eq!(match_state.hero_bans.len(), DEFAULT_MAP_SLOTS);
        assert!(match_state.maps.iter().all(|map| map.name.is_empty()));
        assert!(
            match_state
                .hero_bans
                .iter()
                .all(|bans| bans.team_a.is_empty())
        );
        assert_eq!(match_state.first_to, 3);
        assert_eq!(match_state.status, MatchStatus::Pre);
        assert!(match_state.loser_picks);
        assert!(match_state.show_player_portraits);
        assert_eq!(match_state.scoring, Scoring::Derived);
        assert!(match_state.current_map.is_none());
    }

    #[test]
    fn derived_scores_count_completed_wins_and_ignore_draws() {
        let mut match_state = new_match();
        match_state.maps[0] = named("Ilios", 3, 1, true, Some(TeamSide::A));
        match_state.maps[1] = named("Nepal", 1, 3, true, Some(TeamSide::B));
        match_state.maps[2] = named("Lijiang", 2, 2, true, Some(TeamSide::A));
        match_state.maps[3] = named("Oasis", 3, 0, false, Some(TeamSide::A));
        assert_eq!(derived_scores(&match_state), (1, 1));
    }

    #[test]
    fn the_first_map_is_picked_by_a() {
        let match_state = new_match();
        assert_eq!(next_chooser(&match_state.maps, 0), TeamSide::A);
    }

    #[test]
    fn later_maps_are_picked_by_the_previous_loser() {
        let mut maps = new_match().maps;
        maps[0] = named("Ilios", 3, 1, true, Some(TeamSide::A));
        assert_eq!(next_chooser(&maps, 1), TeamSide::B);

        maps[1] = named("Nepal", 0, 3, true, Some(TeamSide::B));
        assert_eq!(next_chooser(&maps, 2), TeamSide::A);
    }

    #[test]
    fn draws_keep_the_previous_picker() {
        let mut maps = new_match().maps;
        maps[0] = named("Ilios", 2, 2, true, Some(TeamSide::B));
        assert_eq!(next_chooser(&maps, 1), TeamSide::B);
    }

    #[test]
    fn chooser_lookups_skip_empty_slots_and_default_to_a() {
        let mut maps = new_match().maps;
        maps[3] = named("Nepal", 0, 3, true, Some(TeamSide::A));
        assert_eq!(next_chooser(&maps, 4), TeamSide::A);
        assert_eq!(next_chooser(&maps, 2), TeamSide::A);
    }

    #[test]
    fn recalculating_loser_picks_clears_empty_slots() {
        let mut maps = new_match().maps;
        maps[0] = named("Ilios", 3, 1, true, None);
        maps[1] = named("Nepal", 1, 3, true, None);
        maps[2].name = "Lijiang".to_owned();
        recalculate_loser_picks(&mut maps);
        assert_eq!(maps[0].chosen_by, Some(TeamSide::A));
        assert_eq!(maps[1].chosen_by, Some(TeamSide::B));
        assert_eq!(maps[2].chosen_by, Some(TeamSide::A));
        assert!(maps[3..].iter().all(|map| map.chosen_by.is_none()));
    }

    #[test]
    fn naming_a_map_updates_its_mode_and_picker() {
        let mut match_state = new_match();
        match_state.maps[0] = named("Ilios", 3, 1, true, None);
        set_map_name(&mut match_state, 1, "Nepal", "Control");
        assert_eq!(match_state.maps[1].name, "Nepal");
        assert_eq!(match_state.maps[1].game_mode_name, "Control");
        assert_eq!(match_state.maps[1].chosen_by, Some(TeamSide::B));

        set_map_name(&mut match_state, 1, "", "Control");
        assert_eq!(match_state.maps[1].game_mode_name, "");
        assert_eq!(match_state.maps[1].chosen_by, None);
    }

    #[test]
    fn manual_picker_survives_when_loser_picks_is_off() {
        let mut match_state = new_match();
        match_state.loser_picks = false;
        match_state.maps[0] = named("Ilios", 3, 1, true, Some(TeamSide::B));
        set_map_name(&mut match_state, 1, "Nepal", "Control");
        assert_eq!(match_state.maps[1].chosen_by, None);
    }

    #[test]
    fn score_edits_only_update_the_next_picker() {
        let mut match_state = new_match();
        match_state.maps[0] = named("Ilios", 3, 1, true, Some(TeamSide::A));
        match_state.maps[1] = named("Nepal", 0, 0, false, Some(TeamSide::B));
        match_state.maps[2] = named("Lijiang", 0, 0, false, Some(TeamSide::A));

        set_map_scores(&mut match_state, 0, 1, 3);
        assert_eq!(match_state.maps[1].chosen_by, Some(TeamSide::A));
        assert_eq!(match_state.maps[2].chosen_by, Some(TeamSide::A));
    }

    #[test]
    fn insert_remove_and_move_keep_bans_aligned() {
        let mut match_state = new_match();
        match_state.maps[0] = named("Ilios", 0, 0, false, None);
        match_state.hero_bans[0].team_a = vec!["Tracer".to_owned()];
        match_state.current_map = Some(0);

        insert_map(&mut match_state, 1);
        assert_eq!(match_state.maps.len(), 9);
        assert_eq!(match_state.hero_bans.len(), 9);
        assert_eq!(match_state.current_map, Some(0));
        assert_eq!(match_state.hero_bans[0].team_a, vec!["Tracer".to_owned()]);

        assert_eq!(move_map(&mut match_state, 0, 1), Some(1));
        assert_eq!(match_state.maps[1].name, "Ilios");
        assert_eq!(match_state.hero_bans[1].team_a, vec!["Tracer".to_owned()]);
        assert_eq!(match_state.current_map, Some(1));

        remove_map(&mut match_state, 1);
        assert_eq!(match_state.maps.len(), 8);
        assert_eq!(match_state.hero_bans.len(), 8);
        assert_eq!(match_state.current_map, None);
        assert!(match_state.hero_bans[1].team_a.is_empty());
    }

    #[test]
    fn resetting_maps_and_bans_restores_empty_defaults() {
        let mut match_state = new_match();
        match_state.maps.truncate(2);
        match_state.hero_bans[0].team_a = vec!["Tracer".to_owned()];
        match_state.current_map = Some(0);

        reset_hero_bans(&mut match_state);
        assert_eq!(match_state.hero_bans.len(), 2);
        assert!(match_state.hero_bans[0].team_a.is_empty());

        reset_maps(&mut match_state);
        assert_eq!(match_state.maps.len(), DEFAULT_MAP_SLOTS);
        assert_eq!(match_state.hero_bans.len(), DEFAULT_MAP_SLOTS);
        assert!(match_state.current_map.is_none());
    }

    #[test]
    fn hero_bans_replace_and_clear_per_team() {
        let mut match_state = new_match();
        set_first_ban_team(&mut match_state, 0, Some(TeamSide::B));
        set_hero_ban(&mut match_state, 0, TeamSide::A, "Tracer");
        set_hero_ban(&mut match_state, 0, TeamSide::B, "Winston");
        assert_eq!(match_state.hero_bans[0].first_ban_team, Some(TeamSide::B));
        assert_eq!(match_state.hero_bans[0].team_a, vec!["Tracer".to_owned()]);
        assert_eq!(match_state.hero_bans[0].team_b, vec!["Winston".to_owned()]);

        set_hero_ban(&mut match_state, 0, TeamSide::A, "");
        assert!(match_state.hero_bans[0].team_a.is_empty());
    }

    #[test]
    fn map_pool_preserves_order_and_supports_moves() {
        let mut match_state = new_match();
        toggle_map_pool(&mut match_state, "Ilios", true);
        toggle_map_pool(&mut match_state, "Nepal", true);
        toggle_map_pool(&mut match_state, "Ilios", true);
        assert_eq!(match_state.map_pool, vec!["Ilios", "Nepal"]);

        assert_eq!(move_map_pool(&mut match_state, 0, 1), Some(1));
        assert_eq!(match_state.map_pool, vec!["Nepal", "Ilios"]);

        toggle_map_pool(&mut match_state, "Nepal", false);
        assert_eq!(match_state.map_pool, vec!["Ilios"]);
    }

    #[test]
    fn talent_slots_read_and_write_the_right_fields() {
        let mut talent = MatchTalent::default();
        let reference = TalentRef {
            talent_id: crate::TalentId::from_raw("talent-1"),
        };
        talent.set_slot(TalentSlot::Caster1, Some(reference.clone()));
        assert_eq!(talent.slot(TalentSlot::Caster1), Some(&reference));
        assert!(talent.slot(TalentSlot::Panel1).is_none());

        talent.set_slot(TalentSlot::Caster1, None);
        assert!(talent.slot(TalentSlot::Caster1).is_none());
        assert_eq!(TalentSlot::ALL.len(), 6);
        assert_eq!(TalentSlot::Interviewer.label(), "Interviewer");
    }
}
