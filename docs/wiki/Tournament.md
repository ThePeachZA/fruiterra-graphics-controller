# Tournament

The Tournament page builds and runs brackets and round-robin groups. It has four
tabs: **Setup**, **Board**, **Groups** and **Settings**.

Select **+ New tournament** to create one. New tournaments start as a single
elimination event and open on **Setup**.

## Setup

### Tournament details

| Field | What it does |
|---|---|
| **Name** | The tournament name. |
| **Format** | **Single elimination**, **Double elimination** or **Round robin**. |
| **First to** | The match length used by generated bracket matches. |
| **Status** | **Setup**, **In progress** or **Completed**. |
| **Grand-final reset** | Play a second grand final when the losers-bracket team wins the first. |

### Entrants

1. Choose a team in **Team to add** and select **Add entrant**.
2. Use **Move up** and **Move down** to set the seed order.
3. Use **Remove entrant** to take a team out.

Seeds run top to bottom and are renumbered after every add, remove or reorder.
Generation places byes against the highest seeds.

### Generate

Generation **replaces the current board**. Everything it creates can still be
edited afterwards.

- For single or double elimination, select **Generate single elimination
  bracket** or **Generate double elimination bracket**.
- For round robin, choose the number of **Groups** and select **Generate round
  robin**.

At least two entrants are required.

## Board

The board appears once the tournament has rounds. It shows the **Winners
bracket**, **Losers bracket** and **Grand final**, with a **Zoom** control from
50% to 150%.

### Move a team between slots

You can move a team by click or by keyboard:

- **Click:** select one slot, then select another slot. The two slots swap. Click
  the same slot again to cancel.
- **Keyboard:** focus a slot with **Tab**, press **Enter** or **Space** to pick
  it up, use the **arrow keys** to move to another slot, then press **Enter** or
  **Space** to drop it. Press **Escape** to cancel.

### Edit a match

Select **Edit** on a match card. The dialog lets you set:

- **Slot 1** and **Slot 2**
- **Score 1** and **Score 2**
- **Status** — Pending, Live or Completed
- **First to**
- **Winner** — a manual override; clearing it falls back to the scores
- **Winner advances to** — target match and slot
- **Loser drops to** — target match and slot

A completed match needs a winner. Set the scores or choose a winner, or the app
shows an error. **Reset result** clears the result and closes the dialog.

Match cards also have **Move match up** and **Move match down** controls. A
match with a manual winner shows a **Manual winner** badge.

### Rounds

- **Add round** adds a round to that bracket section.
- The round name is editable.
- **+ Match** adds a match to that round.
- **Remove round** deletes the round and its matches immediately and clears any
  links that pointed into it. It is unavailable when only one round remains.
  There is no confirmation step, so use it carefully.

## Groups

The Groups tab manages round-robin groups and their fixtures.

1. Choose the number of **Groups** and select **Generate round robin**.
2. Rename a group by editing its name at the top of the group.
3. Add a team with the group's team picker and **Add**. Remove one with the
   **×** on its chip.
4. The standings table updates from completed fixtures. Columns are **Team**,
   **P**, **W**, **L**, **D**, **PF**, **PA**, **Diff** and **Pts**.
5. Add fixtures with the two team pickers and **Add fixture**.
6. For each fixture, enter the scores, choose the **Match status** and select
   **Save**. Use the remove control to delete it.

**Delete group** requires a second confirmation. It removes the group, its
fixtures and its results.

### Generate the playoff

The **Playoff** panel sets **Qualifiers per group** (1–8) and the bracket
**Format** (single or double elimination). It appears on both **Groups** and
**Settings**.

Once the standings are final, return to **Groups** and select **Generate
playoff**. Qualifiers are seeded from every group into a new bracket, replacing
the current bracket. If there are no groups or not enough qualifiers, the app
explains what is missing.

## Settings

### Tiebreakers

Tiebreakers apply in order and form the full standings sort. The default order
starts with **Wins** (one point per win), so the later criteria break ties on
points. Available criteria:

- **Wins**
- **Head-to-head** (compares the results between the teams still level)
- **Map differential**
- **Maps won**
- **Points for**
- **Points against**
- **Seed**

Use the up and down controls to reorder, **×** to remove, and **Add** to include
a criterion. **Reset to default** restores the standard order.

### Champion

The champion is set automatically from the final. You can override it with the
**Champion** picker, or choose **Derived from the bracket** to clear the
override. The **Current** field shows the resulting champion, or **No champion
yet**.

## Data safety

Tournament data is saved with the rest of the project. On load, malformed
tournament data is repaired rather than crashing the app. A completed bracket
match without a valid winner is reset to Pending.

Deleting a team that is still used by a tournament is blocked. The Teams
deletion dialog lists the references and can clear them for you.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Match Controller](Match-Controller)
- [Data and Backups](Data-and-Backups)
