# Match Controller

The Match Controller edits **one current match at a time**. Every change is
projected to the overlays as soon as you make it.

If no match exists, the page shows **No current match** with a **Create match**
button. Use **New match** in the toolbar to replace the current match with a
fresh one; this happens immediately and cannot be undone. **Force update**
re-sends the current state to every connected overlay.

The seven tabs are:

1. [Match](#match)
2. [Map Selection](#map-selection)
3. [Hero Bans](#hero-bans)
4. [Talent](#talent)
5. [Map Pool](#map-pool)
6. [Timer](#timer)
7. [Config](#config)

## Match

| Field | What it does |
|---|---|
| **Title** | The main match title, for example `Grand Finals`. |
| **Subtitle** | The second line, for example `Upper Bracket`. |
| **Format** | **First to 1** through **First to 4**. |
| **Status** | **Upcoming**, **Live** or **Complete**. |
| **Side indicator** | **Neutral**, **Attack / Defend** or **Defend / Attack**. |
| **Loser picks** | Each map after the first is picked by the previous map's loser. |
| **Team A** / **Team B** | The two teams on the scoreboard. The swap button exchanges them. |
| **Score** | The current score, either derived from completed maps or entered manually. |
| **Manual score override** | Enter the score directly instead of deriving it. |

## Map Selection

Select **+ Add map** for each map. The first map is picked by team A; later maps
follow the loser of the previous map when **Loser picks** is enabled. The eighth
slot is labelled **Tiebreaker**.

Each map has:

- **Map name** — choose a known map or type your own.
- **Game mode** — filled from the map asset when known; otherwise type it.
- **Chosen by** — **None**, **Team A** or **Team B**.
- **Current map** — marks the map shown by the game overlay.
- **Complete** — includes the map in the derived score.
- **Team A score** and **Team B score**
- **↑**, **↓** and **Remove**

**Reset all maps** clears every map slot immediately. There is no confirmation,
so use it carefully.

## Hero Bans

The overlays show one ban per team per map.

For each map, choose:

- **First ban** — **Team A** or **Team B**
- **Team A** — the hero banned by team A
- **Team B** — the hero banned by team B

**Reset all bans** clears every ban immediately. There is no confirmation, so
use it carefully.

## Talent

Assign **Caster 1**, **Caster 2**, **Panel 1**, **Panel 2**, **Panel 3** and
**Interviewer** from your talent library.

The **Interview** panel sets the **Interviewee** and, when no talent member fits,
a **Custom name** and **Custom subtext**.

## Map Pool

**Available maps** lists known maps grouped by game mode when map assets exist.
Tick the maps to include. You can also type a **Custom map name** and select
**Add**.

**Pool order** is the order the map-pool overlay uses. Use the up and down
controls to reorder, and **Remove** to take a map out.

## Timer

**Timer settings** control the clock:

- **Mode** — **Count up**, **Count down** or **Count to target**
- **Alignment** — **Left**, **Centre** or **Right**
- **Hours**, **Minutes**, **Seconds**
- **Target date and time** for **Count to target**
- **Leading zeroes**
- **Standalone timer** — hide the timer surround

The **Timer** panel shows the current value. Use **Start**, **Pause** and
**Reset**.

> [!NOTE]
> The timer is control-only in version 1. There is no bundled timer overlay, but
> third-party overlays receive the ticks and the start, pause and reset actions.

## Config

- **Overlay appearance**
  - **Team colours** — tint the overlays with each team's primary colour.
  - **Player portraits** — show portraits on the roster overlays.
- **Branding**
  - **Tournament logo**
  - **Watermark**
- **Overlay URLs** — the same seven URLs shown on the Overlay page, each with a
  **Copy URL** button.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Overlays in OBS](Overlays-in-OBS)
- [Tournament](Tournament)
