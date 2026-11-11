# Tournament design — Phase 4

Status: **approved**. Replaces the original WIP tournament with a generated-then-fully-editable board. Implements Q1–Q8.

---

## 1. Model

The board is the source of truth. Generation only populates it; every slot and link remains editable afterwards.

```rust
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub format: BracketFormat,          // SingleElim | DoubleElim | RoundRobin
    pub entrants: Vec<Entrant>,         // { team_id, seed }
    pub winners: Vec<Round>,
    pub losers: Vec<Round>,             // empty for single elim
    pub grand_final: Vec<Round>,        // 1 match, or 2 with reset
    pub groups: Vec<Group>,
    pub group_matches: Vec<GroupMatch>,
    pub standings: BTreeMap<GroupId, Vec<Standing>>,  // derived, cached for display
    pub tiebreakers: Vec<Tiebreaker>,
    pub playoff_qualifiers: u8,         // top N per group, default 2
    pub playoff_format: BracketFormat,  // SingleElim | DoubleElim, default SingleElim
    pub champion: Option<TeamId>,
    pub champion_manual: bool,
    pub status: TournamentStatus,       // Setup | InProgress | Completed
    pub default_first_to: u8,
    pub grand_finals_reset: bool,       // default true
}

pub struct BracketMatch {
    pub id: MatchId,
    pub round: u32,
    pub number: u32,
    pub slot1: Option<TeamId>,
    pub slot2: Option<TeamId>,
    pub slot1_manual: bool,             // slot was placed by hand
    pub slot2_manual: bool,
    pub score1: u16,
    pub score2: u16,
    pub winner: Option<TeamId>,
    pub winner_manual: bool,            // explicit override
    pub status: BracketMatchStatus,     // Pending | Live | Completed
    pub next: Option<MatchLink>,        // { match, slot }
    pub loser_next: Option<MatchLink>,
    pub first_to: u8,
}

pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub team_ids: Vec<TeamId>,
}

pub struct GroupMatch {
    pub id: MatchId,
    pub group: GroupId,
    pub team1: Option<TeamId>,
    pub team2: Option<TeamId>,
    pub score1: u16,
    pub score2: u16,
    pub status: BracketMatchStatus,
    pub first_to: u8,
}

pub enum Tiebreaker {
    Wins,
    HeadToHead,
    MapDifferential,
    MapsWon,
    PointsFor,
    PointsAgainst,
    Seed,
}
```

`standings` is derived from `group_matches` on every change and cached only for rendering; it is never hand-edited.

---

## 2. Generation (defaults, not rules)

Generation produces a valid starting board. Everything it creates can be changed.

### Single elimination

- Round up to the next power of two; byes go to the highest seeds.
- Standard seeding order: `1v8, 4v5, 3v6, 2v7` (extended recursively).
- Rounds named `Round 1`, `Quarterfinal`, `Semifinal`, `Final` where the size allows.
- Match numbering top-to-bottom within a round.
- `next` links point to the following round; the final has none.

### Double elimination

- Winners bracket as above.
- Losers bracket generated with standard drop mapping: losers of winners round *r* enter losers round `2r-2` (or `2r-1` depending on bracket size), with the two "drop rounds" per winners round.
- Grand final: one match, plus a second reset match when `grand_finals_reset` is on and the losers-bracket team wins the first.
- Losers-bracket matches carry `loser_next` where a further loss eliminates.

### Round robin

- Circle method per group; every team plays every other team once.
- Groups are created manually or split evenly from the entrant list.
- Standings computed from completed matches.
- Optional playoff: top `playoff_qualifiers` per group advance into a generated single- or double-elim bracket with cross-group seeding (`A1 vs B2`, `B1 vs A2`). With one group, the top N seed straight in.

### Byes

A bye is a match with one empty slot marked `completed`, `winner = the present team`, and `score1/score2 = 0`. The auto-advance pass fills the next slot immediately. Byes can be removed or edited like any other match.

---

## 3. Editing

### 3.1 Team slots

- Select a slot, then select another slot to **swap** the two teams. Clicking the selected slot again clears the selection.
- A swap sets `slotN_manual = true` for both affected slots.
- Slots can also be set or cleared from the match modal; a cleared slot is marked manual, so auto-advance will not refill it.
- A swap that would put the same team on both sides of a match, or duplicate a team in the same match, is rejected.

### 3.2 Structure

Full board editing (Q4):

- Add / remove matches in a round.
- Add / remove rounds, with automatic renumbering.
- Rename rounds (e.g. `Upper Final`, `Lower Round 3`).
- Reorder matches within a round by dragging.
- Edit links in the match modal: `next` and `loser_next` target match + slot, or none.

Removing a match clears any links that pointed at it and any slots it had filled, after a confirmation listing the affected matches.

### 3.3 Results and auto-advance (Q3)

- Completing a match derives the winner from scores (higher score, first to `first_to`).
- An explicit winner override is available and shown with a badge; `winner_manual = true`.
- On completion, the winner is written into `next.slot` **unless** that slot is `slotN_manual`.
- The loser is written into `loser_next.slot` under the same rule (double elim).
- Clearing a result clears every downstream slot that this match filled and that is still auto (`!slotN_manual`), recursively.
- Editing a score after completion re-runs the same propagation.

### 3.4 Champion

- Auto-set from the grand final winner (or reset winner when played), unless `champion_manual`.
- The tournament status flips to `Completed` when a champion exists; manual status override is allowed.
- A manual champion is never overwritten by auto-advance.

### 3.5 Grand final reset (Q1)

- `grand_finals_reset` defaults on.
- When the losers-bracket team wins the first grand final, the reset match is created (if absent) and populated with the same two teams. The winner of the reset is champion.
- If the winners-bracket team wins the first final, the tournament completes immediately; the reset match is removed or left empty and excluded.
- Both behaviours can be overridden by hand (add/remove the second final).

### 3.6 Interaction mechanics

- **Selection-based swap** (click/click or keyboard) instead of a pointer-drag ghost. WebView2 pointer capture is inconsistent, and selection gives precise targets on a dense board; keyboard parity is identical.
- **Keyboard alternative:** focus a slot, press Enter (or Space) to select, move focus with the arrow keys, press Enter to swap, Escape to cancel. Required for accessibility and for precision on large boards.
- **Board layout:** CSS grid, one column per round, horizontal scroll, zoom control (50–150%). Winners, losers and grand final are separate labelled sections.
- The selected slot is highlighted; an invalid swap (same match, duplicate team) is rejected with a toast.

---

## 4. Groups, standings and tiebreakers

### 4.1 Editable after play starts (Q5)

- Add/remove groups, rename them, drag teams between groups.
- Adding a team to a group regenerates missing fixtures; removing a team removes its unplayed fixtures and keeps played ones as records with an empty slot.
- Group fixtures can be added, removed and edited by hand at any time.
- Standings recompute after every change; played results are never silently discarded.

### 4.2 Standings

Per team: `played`, `wins`, `losses`, `draws`, `points_for`, `points_against`, `map_differential`, `points`.

- `points_for` / `points_against` are map scores summed across completed fixtures.
- `map_differential = points_for - points_against`.
- A completed fixture with equal scores is a draw (only possible when first-to is not enforced, e.g. a manual result).
- Points are `wins` in v1 (no custom point values; that stays a backlog item).

### 4.3 Tiebreakers (Q7)

- Configurable ordered list, persisted per tournament.
- Available criteria: `Wins`, `HeadToHead`, `MapDifferential`, `MapsWon`, `PointsFor`, `PointsAgainst`, `Seed`.
- Default order: `Wins → HeadToHead → MapDifferential → PointsFor → Seed`.
- UI: drag to reorder in the tournament settings; a reset-to-default button.
- `HeadToHead` compares only the tied teams' mutual results; if more than two are tied and the mini-table is still tied, the next criterion applies.
- If all criteria are exhausted, teams are ordered by seed, then name (stable, deterministic).

---

## 5. Match modal (Q6)

Fields:

| Field | Behaviour |
|---|---|
| Slot 1 / Slot 2 | Team selectors with clear; also drop targets |
| Score 1 / Score 2 | Integers, 0..=99 |
| First to | Per-match override, default tournament value |
| Status | Pending / Live / Completed |
| Winner | Derived from scores; explicit override dropdown with a visible badge |
| Next match / slot | Editable link |
| Loser next match / slot | Editable link (double elim) |
| Reset | Clears scores, winner, status and auto-filled downstream slots |
| Delete match | Removes it and clears dependent links (confirmation) |

Validation: a completed match must have a winner; a winner must be one of the two slots; scores cannot be negative. Errors appear inline, not as a global toast.

---

## 6. Validation and repair

| Rule | Behaviour |
|---|---|
| Same team twice in one single-elim bracket section | Warn, allow (manual boards can be intentionally odd) |
| Same team on both sides of a match | Reject the drop |
| `next` points at itself or forms a cycle | Reject the link edit |
| `next` points at a removed match | Clear the link |
| Completed match with no winner | Reset to Pending on load, log a repair |
| Champion not present in the tournament | Clear `champion`, log a repair |
| Group fixture with a team outside the group | Move the team into the group or clear the slot; log |
| Duplicate seeds | Keep first, reassign the rest |

---

## 7. Persistence

- The whole tournament (including links, manual flags and tiebreaker order) is stored in `app.json` under `tournaments`.
- Standings are **not** persisted; they are recomputed on load. This avoids stale derived data.
- `schemaVersion` covers the tournament shape like the rest of the model.
- Deleting a team used in any tournament is blocked with a reference list and an explicit “remove from tournaments and delete” action (Q8).

---

## 8. Acceptance criteria (replaces TR1–TR10)

| ID | Requirement |
|---|---|
| T1 | Generate a valid single-elimination bracket for any team count, with byes to top seeds and standard seeding. |
| T2 | Generate a valid double-elimination board: winners, losers, drop mapping and grand final. |
| T3 | Generate round-robin fixtures per group and compute standings. |
| T4 | Every generated slot, match, round and link is editable; teams can be moved between slots with swap semantics. |
| T5 | Auto-advance fills the next slot and loser slot on completion, and never overwrites a manually placed slot. |
| T6 | Clearing or editing a result recursively clears the auto-filled downstream slots it produced. |
| T7 | Grand-final reset honours `grand_finals_reset` and produces the correct champion; both can be overridden manually. |
| T8 | Champion is set automatically from the final unless manually overridden. |
| T9 | Round-robin tiebreakers are configurable, ordered, persisted and correctly applied, including head-to-head mini-tables. |
| T10 | Optional playoff generation from group qualifiers, with cross-group seeding, then fully editable. |
| T11 | Match modal supports scores, status, first-to, winner override, link editing and reset. |
| T12 | Full persistence/restore; standings recomputed; malformed data repaired rather than crashing. |
| T13 | Unit tests for generation, propagation, override, reset, tiebreakers and repair. |
| T14 | Keyboard-accessible slot movement and a usable board at 1080p with 32 teams. |

---

## 9. Out of scope for v1

- Bracket overlay output (backlog P4).
- Forfeits/disqualifications as a first-class result type (winner override covers the common case).
- Multi-stage playoffs beyond one group stage + one playoff.
- Custom point values per win/draw/loss.
- Match history and per-team tournament statistics.
- Automatic import from external tournament sites.
