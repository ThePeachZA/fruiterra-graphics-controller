# FAQ

### Is this the same as the old Google Sheets workflow?

It replaces it with a desktop app. The overlay protocol is unchanged, so overlay
HTML from the old setup still works. There is no automatic import of the old
spreadsheets; teams, talent, matches and assets are entered in the app.

### Can I keep my existing overlays?

Yes. The seven bundled overlays are drop-in compatible with the previous
protocol, and third-party overlays can connect with `?port=<port>`. If you edited
an overlay, put your copy in the **Overlays folder** shown in Settings.

### Can I write my own overlay?

Yes. Your HTML can connect to the same feed as the bundled overlays. See
[Custom overlays](Custom-Overlays) for the connection code, message types and
scoreboard fields. Version 1 has no in-app variable inspector or script
generator; those are separate post-v1 work.

### Is there a timer overlay?

Yes, as a separate simple source. The app still owns the timer controls, and a
standalone `timer.html` at `/overlays/timer.html` renders the broadcast time.
It uses the existing `matchTimer` ticks and applies the sent alignment,
leading-zeroes and show-hours settings; it does not run its own clock or appear
in the auto-overlay scene list. Turn **Show hours** off in Timer settings and
the overlay adds the hours to the minutes (`01:12:00` appears as `72:00`).
Third-party overlays still receive the same ticks and start, pause and reset
actions.

### What does the hero bans overlay show?

The banned heroes for the map that is currently selected, as two full-bleed
halves of the screen. The hero art fills each half, the team logo sits in a
hexagonal badge in the outer top corner, and the role badge and the hero's
name sit over the lower half of the art. Under the name the ban order is
written in that team's colour, so you can see at a glance which team banned
first.

A faint honeycomb wall sits behind the hero and runs along the outer edge of
each half, so the hexagon is the shape the whole screen is built from: the
team logo badge, the role badge and the backdrop are all the same shape. The
banner between the two halves carries both team colours.

Give a hero a **transparent cut-out** in the Assets tab and the overlay uses it
instead of the regular hero art, so the honeycomb wall shows through behind
the hero. Give the hero a **role** and the role badge appears above the name.
Both are optional; without them the overlay falls back to the regular art and
hides the role badge.

### Which overlay shows which people?

Three separate talent overlays, so you put only the row you need in OBS:

| Overlay | Shows |
|---|---|
| `row-casters.html` | The people assigned as **casters** |
| `row-hosts.html` | The **host** and the **analyst** panel, each with a role tag |
| `row-interviews.html` | The **interviewer** and the **guest** |

The on-air role is set per assignment in Match Controller → Talent, so the same
person can be a caster in one match and an analyst in the next. Each overlay
filters that list itself, and all three are separate Browser Sources.

### Can the game overlay show short team names?

Yes. Match Controller → Config → Overlay appearance has a **Team
abbreviations** switch. Turn it on and the game overlay shows each team's short
tag (`NWS`) instead of its full name. It is off by default, and it is saved
with your settings.

### Can I change the overlay colours?

Yes. Match Controller → Config → Overlay theme has a **Theme** picker with
**Night Broadcast** (the default dark theme) and **Light**. The change is
applied to every bundled overlay straight away, with no restart and no reload
of the OBS Browser Source.

You can also **Import theme file** to load a JSON file of your own. The file
may be a flat object of token names and values:

```json
{ "--fgc-surface": "#f4f6fb", "--fgc-text": "#11151d" }
```

or it may name itself and nest the tokens:

```json
{ "name": "Daylight", "tokens": { "--fgc-surface": "#f4f6fb" } }
```

Imported tokens are layered over the selected theme, so you only list the ones
you want to change. **Remove** clears them again.

### Is there a bracket overlay?

Not in version 1. The bracket board is for the operator; it is not a broadcast
overlay.

### Can I run more than one match or scene?

The Match Controller edits one current match at a time. You can save matches to
the match library, load them later, and export or import the library, but there
are no presets, multiple scenes or undo/redo in version 1.

### Can I use Stream Deck or an HTTP API?

No. Version 1 has no HTTP control API and no Stream Deck integration.

### Does LAN mode require a password?

No. There is no authentication in version 1. Use **Allow LAN connections** only
on a trusted network, and turn it off when you are done.

### Can I share overlays over Tailscale?

Not in the published v1.0.0 release. Post-v1 builds add a **Tailscale** switch
that uses your own client and generates `https://<dnsname>/fgc/overlays/…`
URLs. **Local network** is a separate switch in the same panel, so both access
paths can be on at once and **Browser sources** lists both URL sets. The tailnet
is the access boundary for Tailscale; the app does not install Tailscale or
manage ACLs. See [Remote overlays](Remote-Overlays).

### Can I share overlays publicly?

No. Funnel is still pending, and no build generates a public URL. Do not expose
the overlay port through a router or a third-party tunnel.

### Why does Windows warn about the app?

Version 1 is unsigned. Verify the download against `SHA256SUMS.txt`, then choose
**More info → Run anyway** if you trust the source.

### Does it update itself?

No. Download a newer release and install it over the old version. Your project
data is kept.

### Does it run on macOS or Linux?

No. Version 1 is Windows x64 only.

### Where is my data?

In `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\`. See
[Data and Backups](Data-and-Backups) for the folder layout and how to move it.

### How do I move to another PC?

Export a project bundle from **Assets**, copy the folder to the new PC, install
the app there and select **Import project…**.

### Why do the overlays use a different font?

Version 1 bundles Archivo under the neutral `FGC Display*` family names. It is
licensed for redistribution and served locally, so there are no runtime font
downloads. The layout and protocol are unchanged. The typeface is not Apotek;
Archivo was selected during v1 QA, and Saira remains bundled as a reversible
fallback.

### Can I add custom points or match history?

Not in version 1. These are recorded as post-v1 work. The current release covers
single- and double-elimination brackets, round-robin groups, tiebreakers,
multi-stage group-to-playoff events and pushing a fixture to the current match.

### Where do I report a bug?

Open an issue at
[github.com/ThePeachZA/fruiterra-graphics-controller/issues](https://github.com/ThePeachZA/fruiterra-graphics-controller/issues).
Include the app version, your Windows and OBS versions, the steps to reproduce,
and the newest log file.
