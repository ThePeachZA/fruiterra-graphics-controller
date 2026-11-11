# Manual QA runbook

This runbook is the step-by-step companion to
[`docs/manual-qa.md`](manual-qa.md). It tells a tester exactly what to do, what
to look at, and what counts as a pass or a failure. It covers the release gates
R1–R9 and the highest-risk interactive checks from M1–M8.

Automated checks are recorded in `docs/manual-qa.md`; they do not replace the
tests below.

---

## 1. How to record a result

For every test, record:

| Field | Value |
|---|---|
| Test ID | For example `R1` or `A21` |
| Result | **Pass**, **Fail**, **Blocked** or **Not run** |
| Date and version | For example `2026-11-12`, `1.0.0` |
| Environment | Windows build, OBS version, WebView2/CEF version, display scaling |
| Evidence | Screenshot, log excerpt, `/health` output, or a short note |
| Notes | What differed, and whether it is accepted |

A test is **Pass** only when every pass condition is met. If one condition
fails, record **Fail** and open an issue with the evidence. Do not mark a test
pass because "it mostly worked".

Update the status and add a row to the sign-off log in
[`docs/manual-qa.md`](manual-qa.md) when a gate is complete.

---

## 2. What you need

### Builds

- `fruiterra-gc-<version>-windows-x64-setup.exe`
- `fruiterra-gc-<version>-windows-x64-portable.zip`
- `SHA256SUMS.txt`
- The same version built locally, if you are testing before the first tag.

### Machines and accounts

| Environment | Why |
|---|---|
| Windows 10 or 11 x64 test PC | Main interactive pass |
| Clean Windows VM, snapshot taken before install | R5, R6, R7 |
| Windows VM without WebView2 | R8 |
| Second PC on the same LAN | A5 |
| Network you can disconnect | R2, A15 |
| Standard (non-admin) Windows account | Installer and uninstall |
| Display scaling at 100%, 125% and 150% | A12 |

### OBS and overlays

- OBS Studio. Record the exact version.
- The seven Browser Sources at 1920×1080.
- A copy of an **original pre-rebuild overlay HTML** file for R3. If you do not
  have one, mark R3 **Blocked**; a rebuilt overlay cannot prove drop-in
  compatibility.
- A third-party timer overlay, or the test page in R4.
- A browser with developer tools (Edge or Chrome) for the `show` and WebSocket
  checks.

### Test images

Prepare a folder with:

- 2 square PNG logos.
- 6 square PNG portraits.
- 5 map images (any aspect ratio).
- 6 hero images.
- 1 large PNG close to but under 20 MB.
- 1 file over 20 MB.
- 1 safe SVG.
- 1 unsafe SVG containing a `<script>` tag or an external `href`.

The exact pictures do not matter. File names do: use names such as
`logo-a.png`, `portrait-1.png`, `map-ilios.png`, `hero-ana.png`,
`unsafe.svg`.

### Recording tools

- `Get-FileHash` in PowerShell for checksums.
- A screenshot tool.
- A text editor for the save-file tests.
- Optional: `curl.exe` for `/health`.

---

## 3. Build the test fixture

Create this project once on the test PC. Reuse it for every test unless a test
says otherwise.

### 3.1 Teams

Create four teams on **Teams → + New team**:

| Team | Name | Tag | Record | Colours |
|---|---|---|---|---|
| A | `Northwind Sentinels` | `NWS` | `12-3` | Background `#1f6feb`, text `#ffffff`, logo background `#090909` |
| B | `王小明战队` | `WXM` | `9-6` | Background `#d29922`, text `#111111`, logo background `#090909` |
| C | `Long Team Name That Should Never Overflow The Scoreboard` | `LONG` | `0-0` | Any |
| D | `ДМИТРИЙ ВОЛКОВ` | `ДВ` | `3-1` | Any |

Use `logo-a.png` for A and B. Use any logo for C and D.

### 3.2 Rosters

Add four players to team A and four to team B.

Team A:

| Player | Role | Position | Pronouns |
|---|---|---|---|
| `SØREN` | Tank | Core | `he/him` |
| `ДМИТРИЙ ВОЛКОВ` | Damage | Core | `they/them` |
| `王小明` | Support | Core | `he/him` |
| `Longplayer Name That Should Not Overflow` | Damage | Sub | `she/her` |

Team B:

| Player | Role | Position | Pronouns |
|---|---|---|---|
| `Alex` | Tank | Core | `he/him` |
| `Priya` | Damage | Core | `she/her` |
| `Marco` | Support | Core | `they/them` |
| `Sam` | Support | Sub | `they/them` |

Give each player a portrait and a featured hero from the imported assets.

### 3.3 Talent

Create four talent members on **Talent → + Add talent**:

| Display name | Platform | Handle | Pronouns |
|---|---|---|---|
| `Arc` | Twitch | `@arc` | `he/him` |
| `Priya` | Twitter / X | `@priya` | `she/her` |
| `Marco` | YouTube | `@marco` | `he/him` |
| `Sam` | Instagram | `@sam` | `they/them` |

Give each one a portrait.

### 3.4 Assets

Import on **Assets**:

- 2 logos in **Logo**.
- 6 portraits in **Portrait**.
- 5 maps in **Map**: `Ilios`, `Nepal`, `King's Row`, `Dorado`,
  `Antarctic Peninsula`.
- 6 heroes in **Hero**: `Ana`, `Kiriko`, `Winston`, `Tracer`, `Sojourn`,
  `Lucio`.

### 3.5 Match

On **Match Controller → Create match**, set:

- **Title:** `Grand Final`
- **Subtitle:** `Upper Bracket`
- **Format:** First to 3
- **Status:** Live
- **Side indicator:** Attack / Defend
- **Loser picks:** On
- **Team A:** Northwind Sentinels
- **Team B:** 王小明战队

On **Map Selection**, create five maps:

| # | Map | Mode | Chosen by | Current | Complete | A | B |
|---|---|---|---|---|---|---|---|
| 1 | Ilios | Control | Team A | Yes | Yes | 2 | 1 |
| 2 | Nepal | Control | Team B | No | Yes | 1 | 2 |
| 3 | King's Row | Hybrid | Team A | No | Yes | 3 | 2 |
| 4 | Dorado | Escort | Team B | No | No | 0 | 0 |
| 5 | Antarctic Peninsula | Control | Team A | No | No | 0 | 0 |

The derived score should be **2–1**.

On **Hero Bans**, set:

| Map | First ban | Team A | Team B |
|---|---|---|---|
| Ilios | Team A | Ana | Kiriko |
| Nepal | Team B | Winston | Tracer |
| King's Row | Team A | Sojourn | Lucio |

On **Talent**, assign Arc, Priya, Marco, Sam to the caster, panel and interviewer
slots. Set the interviewee to `Sam`.

On **Map Pool**, tick all five maps in the order above.

On **Timer**, set **Mode** to Count down, **Minutes** to `5`, **Seconds** to
`0`, **Leading zeroes** on, **Standalone timer** off.

### 3.6 Tournament fixture

Create six more teams, `Team 1` to `Team 6`, with any tag.

Create a tournament named `Autumn Invitational`:

- **Format:** Double elimination
- **First to:** 3
- **Grand-final reset:** On

Add eight entrants in this order: A, B, Team 1, Team 2, Team 3, Team 4, Team 5,
Team 6. Generate the bracket.

Create a second tournament named `Group Stage`:

- **Format:** Round robin
- **Groups:** 2

Add the same eight entrants and generate the round robin.

### 3.7 Board stress fixture

Create one more tournament named `Board Stress`:

- **Format:** Single elimination
- **First to:** 3

Create 24 additional teams, `Stress 1` to `Stress 24`, with any tag, so the
project has 32 teams in total. Add all 32 teams as entrants and generate the
bracket.

This produces a five-round, 32-slot winners bracket. It is used only by A24 to
prove the board stays usable at full size; the other tests use `Autumn
Invitational`.

---

## 4. Release gates R1–R9

### R1 — OBS overlay pass

**Blocks:** `v1.0.0`  
**Covers:** M2-1, M3-4, M9-1, M9-6, overlay QA rows 1–14

**Precondition:** Fixture loaded, overlay server running on port 3001, seven
Browser Sources at 1920×1080 with transparent backgrounds and **Shutdown source
when not visible** turned off.

**Steps:**

1. Open each overlay URL in OBS and confirm it paints without a blank first
   frame.
2. Check each overlay against the table below.
3. Leave all seven sources open for at least 10 minutes and watch for flicker,
   disconnects or drift.
4. Record the OBS version, WebView2/CEF version and app version.

**Pass:**

| Overlay | Expected |
|---|---|
| Game overlay | Score `2 - 1`; tags `NWS` and `WXM`; title `Grand Final`; subtitle `Upper Bracket`; current map `Ilios`; first-to `3`; side indicator matches Attack / Defend. |
| Map select | Five maps in order; `Ilios` chosen by NWS; `Nepal` chosen by WXM; `King's Row` chosen by NWS; scores match the fixture. |
| Hero bans | Three map rows; first-ban order matches the fixture; Ana/Kiriko, Winston/Tracer, Sojourn/Lucio. |
| Map pool | Ilios, Nepal, King's Row, Dorado, Antarctic Peninsula, in that order, grouped by mode. |
| Home roster | Northwind Sentinels players in roster order, with roles, portraits and heroes. |
| Away roster | 王小明战队 players in roster order, with roles, portraits and heroes. |
| Casters row | Arc, Priya, Marco, Sam in assignment order, with the correct platform icons and handles. |
| All seven | No clipping at 1920×1080; transparent background; Saira renders; long names ellipsize or fit without overflowing the canvas. |
| All seven | No network request leaves `127.0.0.1:3001`. Only the overlay WebSocket and local `/assets/...` URLs load. |
| All seven | After 10 minutes, all seven remain connected and no visible lag appears when editing the match. |

**Fail:**

- Any overlay is blank, shows the wrong team or side, or shows stale data.
- Any overlay requests a remote font or image.
- Text overflows the canvas, is clipped, or renders in a fallback font for Latin
  text.
- A source disconnects, flickers or stops updating during the 10-minute watch.
- The 6-caster `row-casters` clipping is **known** (see M9-7). Record it if you
  see it, but do not fail R1 for that specific pre-existing case.

**Evidence:** One screenshot per overlay, the OBS source list, and the
`/health` output showing `overlays: 7`.

---

### R2 — Offline font check

**Blocks:** `v1.0.0`  
**Covers:** M9-2

**Steps:**

1. Load the fixture and start the overlay server.
2. Open all seven overlays in OBS and confirm they render.
3. Disconnect the network adapter, or disable Wi-Fi and unplug Ethernet.
4. Restart OBS.
5. Reload all seven Browser Sources.
6. Inspect the overlay text.

**Pass:**

- All seven overlays still render Saira.
- No font request fails.
- No overlay shows a system-font fallback for Latin text.
- The app UI still renders Inter.

**Fail:**

- Any overlay falls back to a default sans-serif for Latin text.
- Any font request fails or hangs.
- An overlay shows a missing-glyph box for Latin characters.

**Evidence:** Screenshot of each overlay offline, plus the network panel showing
no external requests.

---

### R3 — Drop-in overlay compatibility

**Blocks:** `v1.0.0`  
**Covers:** M2-1, M9-3

**Precondition:** An original pre-rebuild overlay HTML file.

**Steps:**

1. Serve the original overlay HTML from a local file or a simple local web
   server. Do not edit it.
2. Start the new overlay server with the fixture loaded.
3. Open the original overlay in OBS or a browser.
4. Confirm it paints immediately from `init`.
5. Change the match subtitle, score, current map and a map pick in the app.
6. Confirm the original overlay updates for each change.
7. Restart the overlay server and confirm the original overlay reconnects.

**Pass:**

- The original overlay paints without a manual reload.
- Every live change appears.
- Reconnect works within about two seconds.
- No JavaScript error appears in the developer console.

**Fail:**

- The overlay stays blank until a live change.
- A message is missing or has the wrong shape.
- The overlay throws a JavaScript error.
- Reconnect does not happen.

**Evidence:** Screenshots before and after each change, plus the console output.

---

### R4 — Third-party timer overlay

**Blocks:** `v1.0.0`  
**Covers:** M7-6, M9-4

**Steps:**

1. Save the following as `timer-test.html` and open it in Edge or Chrome while
   the overlay server is running.

```html
<!doctype html>
<meta charset="utf-8">
<title>FGC timer test</title>
<pre id="out">connecting...</pre>
<script>
  const out = document.getElementById('out');
  const ws = new WebSocket('ws://127.0.0.1:3001/');
  ws.onopen = () => {
    out.textContent = 'connected\n';
    ws.send(JSON.stringify({ type: 'requestInit' }));
  };
  ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (message.type === 'init') {
      out.textContent += 'init: ' + Object.keys(message.data).join(', ') + '\n';
    } else if (message.type === 'matchTimer') {
      out.textContent += 'matchTimer: ' + JSON.stringify(message.data) + '\n';
    } else {
      out.textContent += message.type + ': ' + JSON.stringify(message.data) + '\n';
    }
  };
  ws.onerror = () => { out.textContent += 'websocket error\n'; };
</script>
```

2. Confirm the page shows `connected` and an `init` line.
3. In the app, open **Match Controller → Timer**.
4. Select **Start**. Watch the page for `matchTimer` messages once per second.
5. Select **Pause**. Confirm the ticks stop and the action is reported.
6. Select **Reset**. Confirm the timer returns to its start value.
7. Change **Mode**, **Alignment**, **Leading zeroes** and **Standalone timer**
   and confirm the payload reflects each change.

**Pass:**

- The page receives `init` and `matchTimer`.
- Ticks arrive about once per second while running.
- Start, pause and reset are visible in the payload.
- Setting changes appear in later messages.

**Fail:**

- No connection, no `init`, or no `matchTimer`.
- Ticks stop while the timer is running.
- Actions are missing or the payload does not change with the settings.

**Evidence:** Copy the page output into the test record.

---

### R5 — Clean-VM installer

**Blocks:** `v1.0.0`  
**Covers:** M0-1, M10-3, M10-4

**Precondition:** A clean Windows 10/11 x64 VM snapshot with no previous install
and no WebView2 if you are also testing R8.

**Steps:**

1. Copy the installer to the VM and run it.
2. Record the product name, publisher and icon shown by the installer.
3. Complete the install as a standard user.
4. Confirm the Start Menu entry exists.
5. Launch the app. Confirm the main window opens and the sidebar shows the seven
   destinations.
6. Start the overlay server and load one overlay in OBS.
7. Close the app.
8. Uninstall from **Settings → Apps**.
9. Confirm the Start Menu entry and install folder are gone.
10. Confirm whether `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\` remains.
    Record the result; do not delete it yet.

**Pass:**

- The installer opens with **Fruiterra Graphics Controller** and the correct
  icon.
- The install completes without administrator rights.
- The app launches, starts the server and serves an overlay.
- Uninstall removes the app and shortcuts.
- The data-retention result is recorded clearly.

**Fail:**

- Wrong product name, publisher or icon.
- Install or launch fails.
- Uninstall leaves shortcuts or the install folder behind.
- The app cannot serve an overlay after a clean install.

**Evidence:** Installer screenshots, Start Menu screenshot, uninstall
screenshot, and the data-folder result.

---

### R6 — Portable ZIP

**Blocks:** `v1.0.0`  
**Covers:** M10-5

**Precondition:** A machine with WebView2 and no installed copy of the app.

**Steps:**

1. Extract the ZIP to `C:\Tools\FGC-Portable\`.
2. Confirm the folder contains `fruiterra-gc.exe`, `assets\`, `README.md`,
   `LICENSE` and `THIRD_PARTY_NOTICES.md`.
3. Run `fruiterra-gc.exe` from that folder.
4. Confirm the UI loads with its fonts and styles.
5. Start the overlay server and load all seven overlays in OBS.
6. Close the app and confirm no install entry was created.

**Pass:**

- The app runs with no install.
- The UI assets and all seven overlays load.
- No registry keys or Start Menu entries are created.

**Fail:**

- The app fails to start because `assets\` is missing.
- The UI is unstyled or fonts are missing.
- An overlay 404s.
- The portable build creates an install entry.

**Evidence:** Folder listing, app screenshot, overlay screenshot, and the
Installed apps list before and after.

---

### R7 — SmartScreen and checksum

**Blocks:** `v1.0.0`  
**Covers:** M0-2, M10-7

**Steps:**

1. Download both artifacts through a browser to a fresh folder.
2. Run the installer. Confirm SmartScreen appears.
3. Choose **More info**, confirm the app name, then **Run anyway**.
4. Run `Get-FileHash` on both files.
5. Compare both hashes with `SHA256SUMS.txt`.

**Pass:**

- SmartScreen warns, as expected for an unsigned build.
- The warning identifies the app, and **Run anyway** works.
- Both hashes match exactly.

**Fail:**

- No warning appears on a fresh download (record it; this may be a reputation
  cache, not a build defect).
- A hash does not match.
- **Run anyway** is unavailable or the installer still refuses to run.

**Evidence:** SmartScreen screenshot and the two hash values.

---

### R8 — WebView2 bootstrap

**Blocks:** `v1.0.0`  
**Covers:** M10-6

**Precondition:** A VM snapshot without the WebView2 runtime. Confirm it is
absent in **Installed apps** and that `fruiterra-gc.exe` cannot start.

**Steps:**

1. Run the installer on the VM. Confirm it installs WebView2.
2. Launch the app and confirm the UI loads.
3. On a second clean snapshot without WebView2, run the portable build.
4. Record the error message.

**Pass:**

- The installer adds WebView2 and the app launches.
- The portable build fails with a clear message that tells the user to install
  WebView2 or use the installer.
- The failure is not a silent crash.

**Fail:**

- The installer does not add WebView2.
- The app still cannot launch after the installer.
- The portable build crashes silently or shows an unreadable error.

**Evidence:** Installed apps screenshots and the portable error message.

---

### R9 — Tagged release publish

**Blocks:** `v1.0.0`  
**Covers:** M10-1, M10-6, M10-8

**Precondition:** R1–R8 pass and the working tree is clean.

**Steps:**

1. Confirm `Cargo.toml` version is `1.0.0`.
2. Create and push the tag:

```powershell
git tag -a v1.0.0 -m "Fruiterra Graphics Controller v1.0.0"
git push origin v1.0.0
```

3. Open the **Release** workflow run for the tag.
4. Confirm every job passes.
5. Open the published GitHub release.
6. Download all three assets.
7. Verify `SHA256SUMS.txt` against both artifacts.
8. Read the release notes and compare them with the shipped feature set.

**Pass:**

- The workflow passes.
- The release contains the installer, portable ZIP and `SHA256SUMS.txt`.
- Both hashes match.
- The notes describe only shipped features and repeat the known limitations.
- The tag version matches the workspace version.

**Fail:**

- The workflow fails or publishes only some assets.
- A hash mismatches.
- The notes claim a backlog feature.
- The tag and workspace versions differ.

**Evidence:** Workflow URL, release URL, hash output and a notes excerpt.

---

## 5. High-risk application tests

### A1 — Save, restart, corrupt save and newer schema

**Covers:** M1-1, M1-2, M1-4, M4-6

**Steps:**

1. With the fixture loaded, make a small change, such as editing the match
   subtitle to `Save test`.
2. Wait two seconds, then close the app normally.
3. Reopen the app. Confirm every team, player, talent, match, map, asset and
   tournament is present and the subtitle is `Save test`.
4. Close the app. Back up `data\app.json`.
5. Replace `data\app.json` with the text `{ not valid json`.
6. Start the app. Confirm it loads the previous good save from
   `data\app.json.bak` and the project is intact.
7. Close the app. Restore the good `app.json` from your backup.
8. Back up `app.json`, then rename or delete `app.json.bak` so there is no
   usable backup.
9. Edit `app.json` and set `schemaVersion` to a number higher than the current
   one. Start the app.
10. Confirm the app reports a load error and does not silently replace the file.
11. Close the app without making changes and confirm the edited file is
    unchanged.

**Pass:**

- A change made immediately before closing is saved.
- The backup is used when `app.json` is corrupt, with no data loss.
- A newer schema with no usable backup is refused with a readable message.
- The newer file is not overwritten before the user changes something.

**Fail:**

- The last change is lost.
- Corrupt `app.json` and a good backup produce an empty project.
- A newer schema is accepted, crashes the app, or is overwritten immediately.
- The error message exposes an internal panic or path instead of a readable
  explanation.

**Note:** if a valid `app.json.bak` still exists, the app loads that backup
instead of showing the error. The refusal applies only when no usable backup
remains.

**Evidence:** Before/after screenshots, the corrupt-file error, and the log
excerpt.

---

### A2 — Logs

**Covers:** M1-3

**Steps:**

1. Start the app and perform a few actions.
2. Open `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\`.
3. Confirm a log file for today exists and contains entries.
4. If the build has been used for several days, confirm no more than seven daily
   files remain.

**Pass:**

- Today's log exists and records startup and user actions.
- The folder is writable and the file is plain text.
- Rotation keeps at most seven daily files.

**Fail:**

- No log file appears.
- The log is empty or unreadable.
- More than seven daily files accumulate.

**Evidence:** Folder listing and the first 20 lines of today's log.

---

### A3 — Third-party overlay with `?port=`

**Covers:** M2-2

**Steps:**

1. Change the overlay port to `3010` in **Settings** and select **Apply**.
2. Serve a copy of any overlay from a different local origin, for example
   `python -m http.server 8080` in a folder containing the file.
3. Open `http://127.0.0.1:8080/game-overlay.html?port=3010`.
4. Confirm it connects and paints.
5. Change the score in the app and confirm the page updates.

**Pass:**

- The overlay connects to port 3010, not the default 3001.
- It paints from `init` and updates live.

**Fail:**

- It tries port 3001, stays blank, or throws a WebSocket error.
- It connects but does not update.

**Evidence:** Browser console output and a screenshot.

---

### A4 — Inline data URL asset mode

**Covers:** M2-3

**Steps:**

1. Open **Settings → Asset URLs** and choose **Inline data URLs**.
2. Open the game overlay and a roster overlay.
3. Confirm logos and portraits still render.
4. Inspect a WebSocket frame or `/state` and confirm image URLs are
   `data:image/...` values.
5. Switch back to **Served URLs (/assets/…)** and confirm the overlays still
   render.

**Pass:**

- Both modes render every image.
- Inline mode uses `data:` URLs; served mode uses `/assets/...` URLs.
- Switching modes does not require an app restart.

**Fail:**

- An image disappears in either mode.
- Inline mode still sends HTTP asset URLs.
- Switching modes requires a restart or leaves stale images.

**Evidence:** One `/state` excerpt per mode and a screenshot of each overlay.

---

### A5 — LAN access

**Covers:** M3-1

**Steps:**

1. On the host PC, open **Settings** and turn on **Allow LAN connections**.
2. Allow the Windows Firewall prompt for **private networks** only.
3. Note the host's LAN IP with `ipconfig`.
4. On the second PC, open
   `http://<host-ip>:3001/overlays/game-overlay.html`.
5. Confirm it connects and updates when the host edits the match.
6. Turn **Allow LAN connections** off. Confirm the second PC can no longer
   connect after a reload.

**Pass:**

- The second PC loads the overlay and receives live updates.
- Firewall access is limited to private networks.
- Turning LAN mode off closes remote access.

**Fail:**

- The second PC cannot connect with LAN mode on and the firewall allowed.
- The overlay loads but does not update.
- LAN mode remains reachable after it is turned off.
- The firewall rule is opened for public networks.

**Evidence:** Host `ipconfig`, firewall prompt screenshot, remote overlay
screenshot, and the result after turning LAN mode off.

---

### A6 — Port already in use

**Covers:** M3-2

**Steps:**

1. Stop the overlay server.
2. Occupy port 3001 with another process, for example:
   `python -m http.server 3001`.
3. In the app, select **Start server**.
4. Read the error message.
5. Free the port and start the server again.

**Pass:**

- The app shows a clear bind error and does not silently switch ports.
- The status is not **Running** while the port is occupied.
- Starting again after freeing the port works.

**Fail:**

- The app reports success but nothing is listening.
- The app silently changes port.
- The app crashes or shows an unreadable error.

**Evidence:** Error screenshot and the process list showing the port owner.

---

### A7 — Per-file overlay override

**Covers:** M3-3

**Steps:**

1. Open **Settings → Overlay overrides** and select **Open overlays folder**.
2. Copy `game-overlay.html` from the app's embedded overlays into that folder.
3. Add a visible marker, for example a red `OVERRIDE` label in the HTML.
4. Reload the game overlay in OBS. Confirm the marker appears.
5. Confirm the other six overlays are unchanged.
6. Replace the override with broken HTML and reload. Confirm the app falls back
   to the built-in file.
7. Delete the override and reload.

**Pass:**

- Only the overridden overlay changes.
- A broken override falls back without breaking the other six.
- Removing the override restores the built-in file.

**Fail:**

- No override is picked up.
- All overlays change.
- Broken HTML leaves the overlay blank instead of falling back.
- The app crashes.

**Evidence:** Screenshots of the marker, the other overlays, and the fallback.

---

### A8 — Reconnect after server restart

**Covers:** M3-5

**Steps:**

1. With all seven overlays open in OBS, select **Restart** on the **Overlay**
   page.
2. Watch the OBS sources.
3. Confirm each repaints from the current state.
4. Check `/health` and confirm the connected count returns to seven.

**Pass:**

- Every source reconnects within about two seconds.
- Every source repaints the current score, maps, bans, rosters and casters.
- `/health` reports `overlays: 7`.

**Fail:**

- A source stays blank or requires a manual reload.
- A source shows stale data.
- `/health` stays below seven after 10 seconds.

**Evidence:** Screen recording or screenshots before and after, plus `/health`.

---

### A9 — Replacing an image updates the overlay

**Covers:** M3-6

**Steps:**

1. Note the current logo URL in `/state` for team A.
2. On **Assets**, delete the current team A logo and import a visibly different
   logo.
3. On **Teams**, set team A's logo to the new image.
4. Watch the game overlay without reloading OBS.
5. Check `/state` and confirm the logo URL hash changed.

**Pass:**

- The overlay shows the new logo immediately or after the next live update.
- The URL contains a new hash.
- No stale logo remains cached.

**Fail:**

- The overlay keeps the old logo until OBS is restarted.
- `/state` still contains the old URL.
- The image 404s.

**Evidence:** Old and new `/state` excerpts and overlay screenshots.

---

### A10 — `show` hide and show

**Covers:** M3-7

**Steps:**

1. Open the game overlay URL directly in Edge or Chrome.
2. Open the developer console.
3. Run `updateOverlay({ type: 'show', visible: false })`.
4. Confirm the overlay content is hidden.
5. Run `updateOverlay({ type: 'show', visible: true })`.
6. Confirm it is visible again.
7. Repeat on one roster overlay.

**Pass:**

- `visible: false` hides the overlay.
- `visible: true` shows it again.
- No console error appears.

**Fail:**

- The overlay ignores the message.
- It hides but does not come back.
- A console error appears.

**Evidence:** Console commands and before/after screenshots.

---

### A11 — Keyboard-only pass

**Covers:** M4-1

**Steps:**

1. Disconnect the mouse.
2. Start at the sidebar. Use **Tab** and **Enter** to visit all seven
   destinations.
3. On **Teams**, create a team, edit its name, open **Players**, add a player
   and edit each field.
4. On **Talent**, create and edit a talent member.
5. On **Match Controller**, visit all seven tabs and change one field on each.
6. On **Tournament**, create a tournament, add an entrant, open **Board**, pick
   up a slot with **Enter**, move with the arrow keys and drop with **Enter**.
   Press **Escape** to cancel a second pick-up.
7. Open and close every dialog with the keyboard.
8. Watch the focus ring at every step.

**Pass:**

- Every control is reachable and operable.
- Focus is always visible and follows a logical order.
- Dialogs trap focus and return it to the trigger when closed.
- No control requires a mouse.

**Fail:**

- A control is unreachable or needs a mouse.
- Focus disappears, jumps to the page start, or is trapped.
- A dialog cannot be closed with **Escape** or the keyboard.
- A button has no visible focus indicator.

**Evidence:** Notes per page, plus screenshots of the focus ring and any
failure.

---

### A12 — Display scaling

**Covers:** M4-2

**Steps:**

1. Set Windows scaling to 100%. Open every destination and one dialog per
   destination.
2. Repeat at 125%.
3. Repeat at 150%.
4. At each setting, check for clipped labels, overlapping controls, cut-off
   tables, hidden buttons and unreadable text.

**Pass:**

- Every destination is legible and unclipped at all three settings.
- Tables and the bracket board scroll rather than clip.
- Dialogs fit on screen.

**Fail:**

- A control is cut off or overlaps another.
- Text is unreadable or truncated without a tooltip.
- A dialog cannot be scrolled to its actions.

**Evidence:** Screenshot of any problem with the scaling value recorded.

---

### A13 — Error toasts and inline errors

**Covers:** M4-3

**Steps:**

1. Trigger a port bind error with A6.
2. Trigger an invalid asset URL import.
3. Trigger an invalid tournament result: mark a bracket match **Completed**
   without scores or a winner.
4. Trigger an invalid port in **Settings**: enter `0` and select **Apply**.
5. Read each message.

**Pass:**

- Every error is readable and says what to fix.
- The message does not include a panic, a stack trace or an internal path.
- The app remains usable and the error can be dismissed.

**Fail:**

- The app crashes or freezes.
- The message is empty, technical, or leaks an internal path.
- The error cannot be dismissed.

**Evidence:** Screenshot or text of each message.

---

### A14 — Second instance

**Covers:** M4-4

**Steps:**

1. Start the app and leave it open.
2. Start `fruiterra-gc.exe` again.
3. Observe what happens.
4. Check Task Manager for the number of `fruiterra-gc.exe` processes.
5. Confirm only one overlay server is listening.

**Pass:**

- The second launch exits immediately without opening a second window.
- Only one app window and one server remain.
- No second server is started.

**Fail:**

- Two windows run independently.
- Two servers fight over the port.
- The second launch crashes the first instance.

**Evidence:** Task Manager screenshot and the window behaviour.

---

### A15 — Local Inter font

**Covers:** M4-5

**Steps:**

1. Disconnect the network.
2. Start the app.
3. Open the developer tools for the WebView if available, or inspect the app's
   network activity with a proxy.
4. Confirm the UI text renders in Inter.
5. Confirm no request goes to Google Fonts or any remote font host.

**Pass:**

- The UI renders in Inter offline.
- All font requests are local.

**Fail:**

- The UI falls back to a system font offline.
- A remote font request is attempted.

**Evidence:** Screenshot and network log.

---

### A16 — Asset import paths

**Covers:** M5-1, M5-2, M5-3, M5-4

**Steps:**

1. **File picker:** import `logo-a.png` as a **Logo**. Confirm it appears.
   Cancel a second import and confirm nothing changes.
2. **URL:** import a direct image URL. Confirm it appears. Try an invalid URL, a
   non-image URL and an image over 20 MB. Confirm each is rejected with a clear
   message.
3. **Unsafe SVG:** import `unsafe.svg` containing a `<script>` tag. Confirm it is
   rejected.
4. **Large image:** import the large PNG under 20 MB. Confirm the UI stays
   responsive and the image appears.
5. **Deduplication:** import the same file twice. Confirm the library still has
   one copy.

**Pass:**

- Valid files import.
- Cancel leaves the library unchanged.
- Invalid URLs, oversized files and unsafe SVG files are rejected with a
  readable reason.
- A near-limit image imports without freezing.
- Duplicate bytes are stored once.

**Fail:**

- The app crashes, freezes or imports an unsafe SVG.
- A rejected import still changes the library.
- A duplicate creates a second asset.
- The error message does not say what is wrong.

**Evidence:** Screenshots of each result and the library count.

---

### A17 — Project bundle export and import

**Covers:** M5-5

**Steps:**

1. On **Assets**, select **Export project…** and choose a folder.
2. Confirm the created `fruiterra-project-<timestamp>/` contains `app.json`,
   `export.json` and the referenced user images.
3. Confirm bundled art is not copied.
4. On a second Windows account or VM, install the app and select **Import
   project…**.
5. Choose the bundle and confirm the replacement.
6. Check teams, talent, assets, the match and both tournaments.

**Pass:**

- The bundle contains the save and every referenced user image.
- Import restores the project on the second machine.
- Bundled art is not duplicated.

**Fail:**

- The bundle is missing an image or `export.json`.
- Import fails or leaves a partial project.
- Bundled art is copied unnecessarily.

**Evidence:** Bundle folder listing and before/after screenshots on both
machines.

---

### A18 — Asset cleanup and overlays folder

**Covers:** M5-6, M5-7

**Steps:**

1. Note the **Unused asset data** count in **Settings**.
2. Import an image, then delete it from the library.
3. Wait for a save, or select **Clean up now**.
4. Confirm the unused count returns to its previous value.
5. Confirm a referenced image and a bundled asset are still present.
6. Select **Open overlays folder** and confirm Explorer opens at
   `...\overlays\`.

**Pass:**

- Only unreferenced user images are removed.
- Referenced and bundled assets remain.
- Explorer opens the correct folder.

**Fail:**

- A referenced or bundled asset is removed.
- Cleanup does nothing or reports an error.
- Explorer opens the wrong folder or fails.

**Evidence:** Unused counts before and after, and a screenshot of Explorer.

---

### A19 — Teams, talent and deletion guards

**Covers:** M6-1, M6-2, M6-3, M6-4

**Steps:**

1. Create a team, edit every field, add a player, edit every player field and
   delete the player.
2. Create a talent member, edit every field and delete them.
3. Assign that talent member to the current match, then delete them. Confirm the
   assignment is cleared.
4. Try to delete team A while it is used by the match and both tournaments.
5. Confirm the dialog lists every reference.
6. Select **Cancel** and confirm nothing changed.
7. Select **Delete team** and then **Remove references and delete**. Confirm the
   team and its references are gone.
8. Check that long, accented, Cyrillic and CJK names render in the UI and on the
   overlays.

**Pass:**

- CRUD works for every field.
- Deleting talent clears match assignments.
- A referenced team cannot be deleted silently.
- The reference list is complete and accurate.
- Cancel is non-destructive; the explicit cleanup deletes the team.
- All test scripts render.

**Fail:**

- A field does not save.
- Deleting talent leaves a broken assignment.
- A referenced team is deleted without the reference list.
- The reference list misses a tournament or match.
- Cancel changes data.
- A name renders as boxes or overflows.

**Evidence:** Reference-list screenshot, before/after data, and name screenshots.

---

### A20 — Match Controller tabs

**Covers:** M7-1

**Steps:**

1. Visit all seven tabs: **Match**, **Map Selection**, **Hero Bans**, **Talent**,
   **Map Pool**, **Timer**, **Config**.
2. On **Match**, change every field, including **Manual score override**.
3. On **Map Selection**, add a map, reorder it, remove it, mark **Current map**
   and **Complete**, and use **Reset all maps**.
4. On **Hero Bans**, set the first-ban team and both heroes, then **Reset all
   bans**.
5. On **Talent**, assign every slot and use **Custom name** and **Custom
   subtext**.
6. On **Map Pool**, tick and untick maps, add a custom map, reorder the pool and
   remove an entry.
7. On **Config**, toggle **Team colours** and **Player portraits**, and change
   the branding images.
8. Trigger one invalid input on each tab where validation exists.

**Pass:**

- Every control works and saves.
- The destructive reset buttons clear their data immediately, as documented.
- Invalid input is rejected inline with a readable message.
- Switching tabs preserves the values.

**Fail:**

- A control does nothing or loses its value.
- A reset button is described as confirmed when it is not, or behaves
  differently from the documented behaviour.
- Invalid input is accepted or produces a crash.
- Tab switching resets data.

**Evidence:** Notes per tab and screenshots of any validation error.

---

### A21 — Timer behaviour

**Covers:** M7-2

**Steps:**

1. Set **Mode** to Count up, **Start**, wait five seconds, **Pause** and read
   the value.
2. Set **Mode** to Count down with `00:00:10`, **Start** and let it reach zero.
3. Set **Mode** to Count to target with a target two minutes in the future and
   **Start**.
4. Toggle **Leading zeroes** and **Standalone timer**.
5. Switch to another tab and back. Confirm the timer keeps running.
6. Close and reopen the app. Confirm the timer settings persist.

**Pass:**

- Count up, count down and target modes behave correctly.
- Start, pause and reset work at each step.
- The clock updates about once per second.
- Settings persist across tabs and restarts.
- The standalone setting changes the surround as described.

**Fail:**

- The clock drifts, stops or double-counts.
- Count down passes zero or target mode is wrong.
- A setting is ignored or lost.
- The timer resets when switching tabs.

**Evidence:** A short screen recording or timestamped notes.

---

### A22 — Force update, copy URL and live updates

**Covers:** M7-3, M7-4

**Steps:**

1. Open the game overlay and one roster overlay in OBS.
2. On **Match Controller**, change the subtitle. Confirm both update.
3. Change the current map, a map pick and a hero ban. Confirm the relevant
   overlays update.
4. Change a caster assignment. Confirm the casters overlay updates.
5. Select **Force update** and confirm the overlays repaint without a reload.
6. On **Config → Overlay URLs**, select **Copy URL** for the game overlay and
   paste it into a text editor.
7. Confirm the URL matches the configured port and filename.

**Pass:**

- Every edit appears live.
- **Force update** repaints all connected overlays.
- The copied URL is correct.

**Fail:**

- An edit does not reach the overlay.
- **Force update** does nothing.
- The copied URL has the wrong port or filename.

**Evidence:** Screenshots and the copied URL text.

---

### A23 — Scores and match rules

**Covers:** M7-5

**Steps:**

1. With the fixture, confirm the derived score is `2–1`.
2. Mark Dorado **Complete** with a `2–0` score for team B. Confirm the score
   becomes `2–2`.
3. Enter a draw on a completed map. Confirm the draw does not change the score.
4. Turn on **Manual score override**, enter `3–1` and confirm the overlay shows
   `3–1`.
5. Turn the override off and confirm the derived score returns.
6. Change **First to** and confirm the overlay updates.
7. Change **Side indicator** and **Loser picks** and confirm the overlays
   reflect the change.
8. Swap the teams and confirm the score and sides follow the swap.

**Pass:**

- Completed maps drive the score.
- Draws are ignored.
- Manual override wins while enabled and reverts cleanly.
- First-to, side and loser-pick changes appear on the overlays.
- Swapping teams keeps the data consistent.

**Fail:**

- The score is wrong, double-counted or stuck.
- A draw changes the score.
- Manual override cannot be cleared.
- Swapping teams mismatches names, scores or sides.

**Evidence:** Score before/after each step and overlay screenshots.

---

### A24 — Tournament board usability and keyboard swap

**Covers:** M8-1, M8-2

**Steps:**

1. Open the `Board Stress` tournament and select **Board**. Confirm the winners
   bracket has five rounds and 32 slots.
2. Set zoom to 50%, 100% and 150%. Scroll in both directions.
3. Confirm every match card is readable and the connectors still make sense.
4. Click one slot, then another. Confirm they swap.
5. Click a slot twice. Confirm the selection cancels.
6. Use **Tab** to focus a slot. Press **Enter**, use the arrow keys and press
   **Enter** to drop. Repeat with **Space**.
7. Press **Escape** during a pick-up. Confirm nothing swaps.
8. Confirm a team that is already in the bracket can be moved without creating
   a duplicate.

**Pass:**

- The 32-slot board is usable at 1080p at every zoom level.
- Click and keyboard swaps both work.
- Cancel paths leave the bracket unchanged.
- No duplicate team appears in two slots.

**Fail:**

- A card is unreadable or unreachable at a supported zoom.
- A swap silently drops a team or duplicates one.
- The keyboard cannot complete a swap.
- **Escape** does not cancel.

**Evidence:** Screenshots at each zoom and a note of the keyboard sequence.

---

### A25 — Tournament match dialog and rounds

**Covers:** M8-3, M8-4

**Steps:**

1. Select **Edit** on a bracket match.
2. Set both slots, both scores, **Status**, **First to** and a manual
   **Winner**.
3. Set **Winner advances to** and **Loser drops to** to specific matches and
   slots.
4. Save and confirm the board reflects the result and the **Manual winner**
   badge.
5. Reopen the match, clear the manual winner and confirm the score-derived
   winner returns.
6. Mark a match **Completed** with no scores and no winner. Confirm it is
   rejected with an inline error.
7. Select **Reset result** and confirm the match returns to Pending.
8. Add a round, rename it, add a match with **+ Match**, then select **×** on
   the round. Confirm the round and its matches are removed and that links into
   the removed round are cleared.
9. Confirm **Remove round** is unavailable when only one round remains in a
   section.

**Pass:**

- Every field saves.
- Links move the winner and loser to the chosen match and slot.
- Invalid completed matches are rejected.
- Reset clears the result.
- Removing a round deletes the round and its matches and clears dependent links.
- The last remaining round cannot be removed.

**Fail:**

- A field does not save or shows the wrong value.
- A link sends a team to the wrong match or slot.
- An invalid completed match is accepted.
- A round removal leaves a link pointing at the removed round.
- The last remaining round can be removed.

**Evidence:** Dialog screenshots, board before/after, and the error message.

---

### A26 — Groups and tiebreakers

**Covers:** M8-5

**Steps:**

1. Open the `Group Stage` tournament and select **Groups**.
2. Rename a group, add a team, remove a team and confirm the standings update.
3. Add a fixture, enter a score, set **Match status** to Completed and select
   **Save**.
4. Confirm **P**, **W**, **L**, **D**, **PF**, **PA**, **Diff** and **Pts** match
   the result.
5. Enter a draw and confirm it is counted as a draw.
6. Create two teams with identical league points, then change the tiebreaker
   order on **Settings**. Confirm the standings reorder.
7. Set **Head-to-head** first and confirm the order follows the head-to-head
   result.
8. Try **Delete group** and confirm it needs a second confirmation.

**Pass:**

- Fixtures save and drive the standings.
- Draws are counted correctly.
- Reordering tiebreakers changes the standings as documented.
- Head-to-head works.
- Group deletion requires confirmation and removes its fixtures.

**Fail:**

- A fixture does not save or the standings are wrong.
- A draw is counted as a win or loss.
- Tiebreaker order has no effect.
- Group deletion happens on the first click.

**Evidence:** Standings screenshots before/after each change and the tiebreaker
order.

---

### A27 — Playoff, champion and grand-final reset

**Covers:** M8-6

**Steps:**

1. On the `Group Stage` tournament, open **Settings** and set **Qualifiers per
   group** to 2 and **Format** to Single elimination.
2. Return to **Groups** and select **Generate playoff**.
3. Confirm the bracket contains the top two from each group in seed order.
4. Change the champion with the **Champion** picker. Confirm **Current** updates.
5. Clear the override with **Derived from the bracket**.
6. On the double-elimination tournament, complete the winners and losers
   brackets so the losers-bracket team wins the first grand final.
7. With **Grand-final reset** on, confirm a second grand final is created.
8. Turn the reset off and repeat on a fresh bracket. Confirm no second final is
   created.

**Pass:**

- Playoff generation uses the configured qualifier count and format.
- The bracket replaces the old one and seeds correctly.
- Champion override and derivation both work.
- Grand-final reset behaves according to the toggle.

**Fail:**

- The wrong teams qualify or the seed order is wrong.
- Generation fails without an explanation.
- The champion cannot be overridden or cleared.
- The reset creates the wrong number of finals.

**Evidence:** Bracket screenshots and the qualifier list.

---

### A28 — Tournament reload, repair and team deletion guard

**Covers:** M8-7, M8-8

**Steps:**

1. Make a tournament change, close the app and reopen it. Confirm the change
   survives.
2. Close the app. Back up `app.json`.
3. Edit `app.json` to break one tournament field, for example set a completed
   match's winner to a team that is not in the slot.
4. Start the app. Confirm it repairs the data instead of crashing.
5. Check the affected match and confirm it is reset to Pending with an
   explanation in the log.
6. Try to delete a team that is a tournament entrant, a bracket slot, a group
   member and a champion. Confirm the reference list names every place.
7. Select **Remove references and delete** and confirm the team is removed from
   every tournament structure.

**Pass:**

- Tournament data survives a normal reload.
- Malformed data is repaired without a crash.
- The reference list covers entrants, slots, groups and the champion.
- Cleanup leaves the tournaments valid.

**Fail:**

- The app crashes or shows a blank tournament.
- Repaired data is silently lost without a log entry.
- The reference list misses a place the team is used.
- Cleanup leaves a broken slot, group or champion.

**Evidence:** Repaired-match screenshot, log excerpt, reference list and
before/after tournament data.

---

## 6. Final sign-off

A release is ready to tag only when:

- R1–R8 are **Pass**, or a failure is explicitly accepted and recorded.
- R9 is run after tagging.
- Every **Fail** in section 5 has either been fixed or accepted with a written
  reason.
- `docs/manual-qa.md` has a completed sign-off row with the date, version,
  environment and tester.
- `docs/overlay-qa.md` has all 15 rows ticked.
- `docs/build.md` §7 items 1–12 are complete.

If a test cannot be run, record **Blocked** and say what is missing. Do not
substitute an automated check for a human gate without recording the
substitution.
