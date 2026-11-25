# Tournament

The Tournament page builds and runs brackets and round-robin groups.

Select **+ New tournament** to create one. New tournaments start as a single
elimination event and open on **Setup**.

The **Tournaments** sidebar lists every created event. Use the chevron beside
**+ New tournament** to collapse or expand the list; collapsing it gives the
selected tournament's detail area more room. The toggle works with a mouse or
keyboard.

The tabs depend on the tournament format:

- **Single elimination** and **double elimination** show **Setup**, **Bracket**
  and **Settings**.
- **Round robin** shows **Setup**, **Groups** and **Settings**.
- **Multi-stage** shows **Setup**, **Groups**, **Bracket** and **Settings** in
  that order.

Only multi-stage events show **Bracket** and **Groups** together. The playoff is
locked until every group fixture is complete.

## Setup

### Tournament details

| Field | What it does |
|---|---|
| **Name** | The tournament name. |
| **Format** | **Single elimination**, **Double elimination**, **Round robin** or **Multi-stage**. |
| **First to** | The match length used by generated bracket matches. |
| **Status** | **Setup**, **In progress** or **Completed**. |
| **Grand-final reset** | Play a second grand final when the losers-bracket team wins the first. |

### Entrants

1. Choose a team in **Team to add** and select **Add entrant**.
2. Use **Move up** and **Move down** to set the seed order.
3. Use **Remove entrant** to take a team out.

Seeds run top to bottom and are renumbered after every add, remove or reorder.
Generation places byes against the highest seeds. Once the event has been
generated, these controls are hidden. To change the field, use the `↻`
**Regenerate** action beside the **Group stage** or **Bracket** heading; it opens
a preview and discards the old structure and results when confirmed. The `⌫`
**Reset group stage** and **Reset bracket** actions clear only that stage after a
confirmation and keep the entrant list. All stage actions sit in one horizontal,
colour-coded row beside the heading.

### Generating

Setup has no **Generate** card or button. Every generation action starts from
the stage heading row on **Groups** or **Bracket**, opens a preview first, and
saves nothing until you select **Generate** in that preview.

- Elimination formats offer **Preview … bracket** in the **Bracket** heading
  row. A double-elimination bracket needs at least two entrants.
- Round robin and multi-stage events offer **Preview round robin** or
  **Preview group stage** in the **Groups** heading row. Multi-stage creates only
  the groups and fixtures. Once every group fixture is complete, the playoff is
  seeded automatically from the group standings.
- A multi-stage event's playoff preview and generate action are on **Bracket**,
  not **Groups**. The `▷` **Preview playoff** action shows the group-completion
  error until all group fixtures are complete.

Group generation splits the entrants into two groups. The preview shows only
what is being generated: group-stage previews show seeding and group
distribution, while bracket and playoff previews show seeding and bracket
placements in the same traditional bracket layout as the Bracket tab. There is
no group-count selector and no manual **+ Group** action.
Drag a team chip to another group to change the distribution, drag a bracket
slot onto another slot in the same round to swap participants, or edit/reorder
seeds in a bracket preview to regenerate its placements. A visible drag ghost
highlights what is being moved and where it will drop. **Cancel** leaves the
saved event unchanged; **Generate** commits the previewed result in one step.
A multi-stage group-stage preview does not show a bracket; the real playoff is
seeded only after every group result is complete.

Group fixtures and bracket matches have an icon-only ↗ **Push to match** action.
Selecting it asks **Replace the current Match Controller match with this
fixture?**; **Cancel** leaves everything unchanged. Confirming it loads the two
teams, scores and status into the current Match Controller match so the overlays
show that fixture immediately. The tournament bracket is unchanged. A separate
tournament graphic overlay is not available in v1; **Push to match** never pushes
one.

## Bracket

The Bracket tab is the source of truth for bracket play. It is available for
single elimination, double elimination and multi-stage tournaments. For
multi-stage events, it stays locked until the group stage is complete, then
shows a centred traditional bracket. Solid connectors follow the stored winner
links, so two adjacent matches in a round reach the correct next-round match
without crossing. Upper-bracket loser drops are dotted stubs leading into the
separate lower bracket.

- **Zoom** changes the bracket scale from 50% to 150%.
- The heading row offers `↻` **Regenerate bracket**, or `▷` **Preview playoff**
  for a locked multi-stage playoff. **Reset bracket** clears the bracket after
  confirmation; entrants and groups stay in place.
- Placements are fixed on the live bracket. There are no drag handles, slot
  selection, up/down match move buttons or lock controls. Use the generation
  preview to rearrange participants before committing.
- The pencil button opens the match details. The ↗ **Push to match** action asks
  for confirmation, then loads that match into Match Controller; it does not
  push a tournament graphic.

Select the pencil on a match card to open **Edit match**:

| Control | What it does |
|---|---|
| **Slot 1 / Slot 2** | Set or change the teams in the match. |
| **Score 1 / Score 2** | Enter the map score. |
| **Status** | Toggle switch for **Upcoming** or **Complete** only. |
| **First to** | The match length for that match. |
| **Winner override** | Force a winner instead of deriving it from the score. |
| **Winner advances to** | Route the winner to another slot. Choosing a slot replaces whichever match fed it before. |
| **Loser drops to** | Route the loser to another slot in a double-elimination bracket. Choosing a slot replaces whichever match fed it before. |

Select **Save result** to apply the edit, or **Reset result** to clear the match.

Before generation you can add rounds and matches, and rename or remove rounds.
After generation the structure is fixed; only results and match details remain
editable. Removing a round that feeds another match clears the dependent links.

> [!NOTE]
> The tournament bracket is not a broadcast overlay in version 1.

## Groups

Groups are available for round robin and multi-stage tournaments.

1. In the **Groups** heading row, select `▷` **Preview group stage** /
   **Preview round robin** to preview the two-group split. Adjust the preview if
   needed, then select **Generate**.
2. After generation, group teams and fixtures are fixed. Use the `↻`
   **Regenerate group stage** action to open a preview, or `⌫` **Reset group
   stage** to clear it after confirmation. A reset also drops any playoff seeded
   from those standings.
3. Select the pencil on a fixture to open its details.

In fixture details, change the score or toggle the status; the result saves
automatically. **First to** sets that fixture's match length. Expand
**Advanced settings** to force a winner instead of deriving it from the score:
choose **Derived from scores** to use the higher score, or pick one of the two
teams. Forcing a winner marks the fixture complete and survives later score
edits. The read-only **Effective winner** value shows the winner actually in
force, or that a completed fixture is a draw. There is no Save button and no
seeding error appears here.
Fixture rows only show the teams, score, status, pencil and the ↗ **Push to
match** action.

**Tiebreakers** are applied in the order shown. Move a tiebreaker up or down to
change priority. Use **Reset to default** to restore the default order.

Before generation, select **Delete group** and confirm to remove a group. After
generation that control is hidden because the fixtures and results are fixed.

The current match can select a tournament on **Match Controller → Match**. Its
broadcast record then comes from the team's played group win-loss record where
one exists, otherwise from `Seed n`. A manual record typed on the Match tab wins
over that derived value; clearing both record fields restores it. This is only a
broadcast label and does not change standings, fixtures or seeds.

## Settings

**Playoff bracket** can follow a group stage:

1. Set **Qualifiers per group**.
2. Choose the playoff format.
3. Complete every group fixture. A multi-stage event seeds its playoff
   automatically. To regenerate it manually, use `▷` **Preview playoff** on the
   **Bracket** tab and confirm **Generate**; there is no playoff preview on
   **Groups**.

Attempting to seed before the group stage is complete reports an error on the
**Bracket** tab and leaves the groups intact. If fewer than two teams qualify,
the final group result still saves without any message; **Bracket** reports
**At least two teams must qualify for a playoff.** and stays locked. Group
fixture status and score edits never show that message.

**Champion** shows the tournament winner. You can override the champion or
return to the derived winner.

Select **Delete tournament** to remove the tournament and all of its bracket and
group data.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Match Controller](Match-Controller)
- [Data and Backups](Data-and-Backups)
