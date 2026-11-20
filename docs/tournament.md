# Tournament design — Phase 4

Status: **approved**. Replaces the original WIP tournament with a generation-preview workflow; structure is fixed after commit, while results and match details remain editable. Implements Q1–Q8.

---

## 1. Model

The bracket is the source of truth. Generation is previewed and committed as a whole; afterwards results and match details stay editable, but structure is fixed.

```rust
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub format: BracketFormat,          // SingleElim | DoubleElim | RoundRobin | MultiStage
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

Generation produces a valid starting bracket. It is previewed first, then committed in one save; results and match details can be changed afterwards, but the generated structure cannot.

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
- Standings are recomputed from completed fixtures. A fixture's effective winner
  is the manual `winner` when `winner_manual` is true and the team is still in one
  of the two slots, otherwise the higher score. Head-to-head ties use the same
  effective winner.
- A fixture's **First to** (1–4) and **Winner** override are editable in its
  details dialog, matching the bracket editor. Choosing a winner marks the
  fixture complete, survives later score edits, and is cleared by returning to
  **Derived from scores**.
- Optional playoff: top `playoff_qualifiers` per group advance into a generated single- or double-elim bracket with cross-group seeding (`A1 vs B2`, `B1 vs A2`). With one group, the top N seed straight in.

### Multi-stage

- `BracketFormat::MultiStage` serialises as `"multi-stage"` and shows
  **Setup**, **Groups**, **Bracket** and **Settings** in that order.
- `generate_multistage(group_count)` creates the groups and their fixtures only.
  The playoff stays empty and the Bracket tab shows a locked state until the
  group stage is complete.
- `group_stage_complete()` checks that every expected round-robin pairing in
  every group has a completed `GroupMatch`.
- `set_group_result()` detects the incomplete → complete transition and calls
  `generate_playoff()` once, seeding `playoff_qualifiers` per group with the
  existing cross-group order (`A1 v B2`, `B1 v A2`). Later manual bracket edits
  are preserved. `set_group_winner()` uses the same transition rule, so a manual
  winner that completes the last fixture also seeds the playoff once. The result
  is written even when seeding fails: `set_group_result()` swallows the error and
  the group fixture dialog never reports it. `set_group_result_with_playoff()`
  still returns `NotEnoughQualifiers` for callers that need the typed error, and
  the **Bracket** tab reports why a playoff cannot be generated; the group result
  editor does not.
- `generate_playoff()` returns `GroupStageIncomplete` before the group stage is
  finished. `generate_round_robin()` sets the format to round robin;
  `generate_playoff()` leaves the format unchanged, so adding a playoff to a
  round-robin event does not silently convert it to multi-stage.
- Saved multi-stage events with a stale playoff while their groups are incomplete
  are repaired by clearing the unresolved bracket and returning to `Setup`.

### Byes

A bye is a match with one empty slot marked `completed`, `winner = the present team`, and `score1/score2 = 0`. The auto-advance pass fills the next slot immediately. Byes can be removed or edited like any other match.

---

## 3. Editing

### 3.1 Generation preview

Every generation action opens a preview before anything is saved. **Groups**
offers **Preview group stage** / **Preview round robin** from its heading action
row, and **Bracket** offers **Preview … bracket** for elimination formats and
**Preview playoff** for a round-robin event. Setup has no generation panel or
button at all. For a multi-stage event the playoff preview/generate is on
**Bracket**, never on **Groups**. Group generation creates two groups; there is
no group-count selector in the preview and no manual **+ Group** action. The
preview works on a clone of the tournament and shows only the details being
committed:

- group-stage previews show **Seeding** and **Group distribution**;
- bracket and playoff previews show **Seeding** and **Bracket placements** using
  the live traditional bracket cards and connectors.

A multi-stage group-stage preview does not show a provisional bracket. The real
playoff stays locked and is seeded from group standings after the last group
fixture is complete.

Pointer dragging is supported in the preview alone:

- group chips move between groups and regenerate that preview's fixtures;
- bracket slots swap only with another slot in the same section and round;
- bracket seeds can be edited by number or reordered by their handles, which
  regenerates the previewed placements;
- an animated drag ghost follows the pointer and highlights the drop target.

**Cancel** closes the preview without changing saved state and returns focus to
the button that opened it. **Generate** commits the previewed tournament in one
mutation. No generation result is written to `app.json` before **Generate**.

### 3.2 Structure

Before generation, an event may be arranged by hand: groups, group teams and
fixtures can be added and removed, and the bracket can receive rounds and
matches. Once an event has groups, fixtures or a bracket, its structure is
fixed: entrants, group membership, fixtures, rounds, matches and placements are
no longer editable. Results and match details stay editable.

There are no match move buttons and no drag/drop in the live Groups or Bracket
views. Live bracket slots are read-only; a slot can be cleared from a match's
details dialog, and a cleared slot is marked manual so auto-advance will not
refill it. A swap or move that would put the same team on both sides of a match
is rejected.

**Regenerate** and **reset** are separate actions. Regenerate always opens a
preview and commits only on **Generate**. Reset clears only the selected stage
(groups or bracket) after a confirmation dialog; entrants and tiebreakers are
kept, and a group reset also drops the playoff seeded from those standings.
Groups and Bracket expose their own stage actions in one horizontal row beside
the heading: `▷` Preview in accent orange, `↻` Regenerate in amber, `+` Add in
green and `⌫` Reset in red. Each action keeps a visible label and matching
`aria-label`/`title`; there is no stage-settings dropdown and no Setup
**Generate** panel.

### 3.3 Results and auto-advance (Q3)

- Completing a match derives the winner from scores (higher score, first to `first_to`).
- An explicit winner override is available and shown with a badge; `winner_manual = true`.
- On completion, the winner is written into `next.slot` **unless** that slot is `slotN_manual`.
- The loser is written into `loser_next.slot` under the same rule (double elim).
- Clearing a result clears every downstream slot that this match filled and that is still auto (`!slotN_manual`), recursively.
- Editing a score after completion re-runs the same propagation.
- Group fixtures follow the same model: `score_winner()` derives the winner from
  the higher score, and **Advanced settings → Winner** stores a manual override
  in `GroupMatch.winner`/`winner_manual`. A manual winner survives later score
  edits while both teams remain in the fixture, marks the fixture complete and
  is reflected in standings and head-to-head. **First to** is a persisted 1–4
  per-fixture override; the dialog's read-only **Effective winner** label shows
  the winner actually in force. Group result edits never surface a playoff
  seeding error.

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

- **Placements** are arranged in the generation preview, not on the live board. The live board has no slot selection, drag handles or match move buttons.
- **Slot details** are available from each match's pencil/details dialog; completed results auto-advance the bracket under the normal rules.
- **Board layout:** centred CSS grid, one column per round, horizontal scroll, zoom control (50–150%). Winners, losers and grand final are separate labelled sections.
- Solid connectors follow each stored winner link, so lower-bracket rounds join
  the correct next-round cards without crossing. Upper-bracket loser drops are
  dotted stubs because they lead into the separate lower-bracket section.

### 3.7 Push to match

Group fixtures and bracket matches expose an icon-only ↗ **Push to match**
action. Selecting it asks **Replace the current Match Controller match with this
fixture?**; confirming loads that fixture into `AppState.current_match` through
the normal `AppStore` mutation path, then forces a projection and broadcasts to
the overlays. **Cancel** leaves the current match untouched.

- Teams come from the fixture's two slots; empty slots stay empty.
- Scores are stored as `Scoring::Manual { a, b }` so the pushed result is exact.
- Status maps `Pending → Pre`, `Live → Live` and `Completed → Post`.
- The action does not change the tournament board, and it does not create a new
  saved match in the match library.
- The tooltip and confirmation explicitly distinguish this from the planned
  tournament graphic overlay, which is not part of v1.

---

## 4. Groups, standings and tiebreakers

### 4.1 Group structure (Q5)

- Group distribution is previewed before generation; teams can be moved in that preview only. Moving a team regenerates the preview's fixtures.
- Before a group stage exists, groups, group teams and fixtures can be arranged by hand. After generation they are fixed; regenerate to change them.
- Result and match details autosave from the fixture details dialog. Status is a
  two-state toggle with only **Upcoming** and **Complete**; there is no results
  Save button and no inline score/status editors in fixture rows.
- Standings recompute after every result; played results are never silently discarded.

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
- UI: up and down controls reorder the list in the tournament settings; a reset-to-default button.
- `HeadToHead` compares only the tied teams' mutual results. It is re-evaluated inside every subgroup it creates, so the mini-table is rebuilt for each tie; if the mini-table is still tied, the next criterion applies.
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
| Delete match | Hidden after generation; bracket match removal is not available in the live editor |

Validation: a completed match must have a winner; a winner must be one of the two slots; scores cannot be negative. Errors appear inline, not as a global toast.

---

## 6. Validation and repair

| Rule | Behaviour |
|---|---|
| Same team twice in one single-elim bracket section | Warn, allow (manual boards can be intentionally odd) |
| Same team on both sides of a match | Reject the drop |
| `next` points at itself or forms a cycle | Reject the link edit |
| A second link points at an already-fed slot | The new link takes over and the older link is cleared; load repair keeps the first link in board order |
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
| T4 | Placements, matches, rounds and links are previewed and adjustable before generation; live results and match details stay editable. |
| T5 | Auto-advance fills the next slot and loser slot on completion, and never overwrites a manually placed slot. |
| T6 | Clearing or editing a result recursively clears the auto-filled downstream slots it produced. |
| T7 | Grand-final reset honours `grand_finals_reset` and produces the correct champion; both can be overridden manually. |
| T8 | Champion is set automatically from the final unless manually overridden. |
| T9 | Round-robin tiebreakers are configurable, ordered, persisted and correctly applied, including head-to-head mini-tables. |
| T10 | Optional playoff generation from group qualifiers, with cross-group seeding and a preview before commit. |
| T11 | Match modal supports scores, status, first-to, winner override, link editing and reset. |
| T12 | Full persistence/restore; standings recomputed; malformed data repaired rather than crashing. |
| T13 | Unit tests for generation, propagation, override, reset, tiebreakers and repair. |
| T14 | Keyboard-accessible match details and a usable, centred board at 1080p with 32 teams. |
| T15 | Multi-stage events keep groups and playoff brackets, restore `MultiStage` on seeding failure, and push selected fixtures to the current match. |

---

## 9. Out of scope for v1

- Bracket overlay output (backlog P4).
- Forfeits/disqualifications as a first-class result type (winner override covers the common case).
- More than one group stage and one playoff in a single event.
- Custom point values per win/draw/loss.
- Match history and per-team tournament statistics.
- Automatic import from external tournament sites.
