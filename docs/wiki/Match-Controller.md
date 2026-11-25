# Match Controller

The Match Controller edits **one current match at a time**. Every change is
projected to the overlays as soon as you make it.

If no match exists, the page shows **No current match** with a **Create match**
button. Use **New match** in the toolbar to replace the current match with a
fresh one; this happens immediately and cannot be undone. **Force update**
re-sends the current state to every connected overlay.

A timer widget sits above the status bar on every page. It shows the current
clock, icon **Start**, **Pause** and **Stop** buttons, and a settings button
that expands the timer settings popup. There is no separate Timer tab.

The five tabs are:

1. [Match](#match)
2. [Maps & Bans](#maps--bans)
3. [Talent](#talent)
4. [Map Pool](#map-pool)
5. [Config](#config)

## Match

| Field | What it does |
|---|---|
| **Title** | The main match title, for example `Grand Finals`. |
| **Subtitle** | The second line, for example `Upper Bracket`. |
| **Scheduled start** | Optional local date and time for the match. Saved with the match and shown in the match library. |
| **Format** | **First to 1** through **First to 4**. |
| **Status** | **Upcoming**, **Live** or **Complete**. |
| **Side indicator** | **Neutral**, **Attack / Defend** or **Defend / Attack**. |
| **Team A / Team B** | Choose the two teams from the roster. The tag shown on the map cards follows this choice. |
| **Tournament** | Optional. Choose a tournament to fill the broadcast records from its played group standings or entrant seeds. |
| **Team A record / Team B record** | Optional manual overrides for the broadcast record. Each field overrides its own derived value; clear both to use the tournament again. |

**Swap sides** swaps Team A and Team B, including their scores, per-map picks
and every completed map result.

**Scores** can be **Manual** or **Derived**. Derived scoring counts one point
for each completed map that one team wins and the other does not; draws add no
points. Turn manual scoring on to enter the score yourself. While manual scoring
is on, the derived counters still show the result of the completed maps. Turn
manual scoring off to return to the derived score.

### Match library

The **Match library** panel saves snapshots of the current match so you can load
them later. Each saved match keeps its map pool and scheduled start. The list is
collapsed under **Saved matches (n)** by default; expand it to load or delete a
saved match.

- **Save current** saves or updates the current match in the library.
- **Load** makes a saved match the current match.
- **Delete** removes a saved match from the library.
- **Export…** saves the whole match library as a portable JSON file.
- **Import…** adds matches from a match library file. Only new match IDs are
  added; teams, talent and assets are not replaced.

Import accepts the current match-library format. Newer library formats are
refused with an error.

## Maps & Bans

Each slot is a map card. The list is sized to the **Format** (for example, Best
of 5 shows five slots). The first map is picked by the team selected as Team A.
When **Loser picks** is off, every later map is picked by the team selected as
Team B. When **Loser picks** is on, each map after the first is picked by the
team that lost the previous map. The labels in this view use the selected teams'
display names, falling back to their tags and then to `Team A` or `Team B` only
when no team is selected.

**Loser picks** is configured on the [Config](#config) tab.

> [!NOTE]
> The cards follow the pick order. After the Team A side loses the previous map,
> the **Picked by** label on the next card shows **TBD** until that map's result
> is known, while the header text and the live `map-select` overlay already show
> the picking team.

On each card:

- **Current map** marks the map that is in progress. The **Current** button on a
  card selects it.
- **Picked by** chooses the team selected as Team A or Team B for that map,
  except the first map. The option text shows that team's display name.
- **Map** chooses the map from the custom **Map pool**. A map that is not in the
  pool appears as **Custom**.
- **Game mode** is the game mode for that map. It is filled from the map's
  **Game mode** in Assets and shows **From map** when empty. A mode typed by
  hand is kept until you choose a different map name, which re-derives it from
  the map's current game mode.
- **Swap** swaps the map and mode with the adjacent card.
- **Reset** clears the map's pick, result, sides and bans.

Set the result with **Completed**, **Not played** or **Draw**. A completed map
needs a winner and, when a side layout is selected, the sides for both teams. A
draw records the sides but awards no point and counts as no pick when **Loser
picks** is on.

**Reset all maps** clears every map card. The current scores, including a manual
score, stay in place.

When **Hero bans** is enabled on the [Config](#config) tab, each map card also
shows a collapsible **Hero bans** section with the banned hero pickers for the
two selected teams. The **First ban** option and both ban labels use the teams'
display names. Bans for unplayed maps do not appear on the hero-bans overlay
until that map is relevant.

## Talent

**Match assignments** links the casters row and panel to people in the Talent
library:

| Slot | Overlay role |
|---|---|
| **Caster 1**, **Caster 2** | The two casters in the row |
| **Panel 1** to **Panel 3** | Up to three panel members |

Each person slot also has a **Role** select: **Caster**, **Host**, **Analyst**,
**Interviewer** or **Other**. The role belongs to this match assignment, so the
same person can be a host in one match and an analyst in another.

### Interview

The **Interview** panel chooses one interview subject from three mutually
exclusive sources. The **Interviewee** select starts with **Custom…**, then lists
every Talent library entry as `Talent · <name>`, then lists every player on the
two teams currently set on the Match tab as `<team> · <player>`. Players from
teams outside this match are not offered. If neither team has a roster yet, the
panel says that you should choose **Custom…** instead.

Selecting a talent uses that person's portrait and pronoun subtext. Selecting a
player uses the player's portrait, falling back to the linked featured hero's
art, and composes the subtext from the team name, on-air role, position and
pronouns separated by ` · `. In player mode the custom fields are hidden and a
hint explains that the details come from the roster.

Selecting **Custom…** shows **Custom name**, **Custom subtext** and **Guest
image**. Type the on-air name and any short line you want beneath it, then choose
a portrait from the asset library. The **Quick upload…** action in that picker
imports a new image straight into the Portrait category and selects it, so a
one-off guest does not need a separate asset-library visit. The saved interview
reference keeps only the fields for the chosen source; changing source clears
the others.

## Map Pool

The pool is the list of maps offered in **Maps & Bans** and the order shown by
the map-pool overlay.

- **Reset** restores the pool to all bundled maps, grouped by game mode.
- **Use empty pool** clears the list. With an empty pool, **Maps & Bans**
  offers every map and the map-pool overlay shows **All maps**.
- Drag a map by its handle, or use the up and down arrows, to change the order.
- Select the remove control on a map to take it out of the pool.

## Timer

The timer is **control-only**. The app sends timer updates to the overlay server,
but it does not include a timer graphic. A third-party timer overlay can read the
same `matchTimer` messages.

The widget above the status bar is the only timer control. Select the sliders
icon to expand these settings:

| Setting | Meaning |
|---|---|
| **Mode** | **Count up**, **Count down** or **Count to target** |
| **Countdown hours / minutes / seconds** | The start value for count-down mode |
| **Target date and time** | The time to count to in target mode |
| **Alignment** | Left, centre or right on a third-party overlay |
| **Leading zeroes** | Show `01:05:00` instead of `1:05:00` |
| **Standalone timer** | Tell the overlay to treat the timer as its own element |

The icon buttons **Start**, **Pause** and **Stop** control the clock. Stop
returns it to zero, or to the count-down start. The clock keeps running while
you visit another page. Only a restart clears the target value; timer settings
persist and run state does not.

## Config

**Overlay appearance**

- **Team colours** — colour the overlay with each team's colours. Turn it off
  for a neutral look.
- **Player portraits** — show or hide portraits on the roster overlays.
- **Loser picks** — each map after the first is picked by the previous map's
  loser.
- **Hero bans** — show hero bans inside **Maps & Bans**.

**Branding**

- **Tournament logo** — an image from the Assets library.
- **Watermark** — a second image, typically a corner mark.

**Browser Source URLs** lists the URL for each overlay, with **Copy** and
**Open** actions. **Force update** is in the toolbar at the top of the page.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Assets](Assets)
- [Overlays in OBS](Overlays-in-OBS)
