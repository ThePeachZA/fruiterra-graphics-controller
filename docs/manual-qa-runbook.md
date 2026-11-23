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
- For app-UI tests (A-series), a `dx build --desktop` executable from
  `target\dx\fruiterra-gc\debug\windows\app\`. Do not use a plain `cargo build`
  binary: it has no processed `assets\` folder, so the UI is unstyled and
  dialogs do not overlay the window.

### Machines and accounts

| Environment | Why |
|---|---|
| Windows 10 or 11 x64 test PC | Main interactive pass |
| Clean Windows VM, snapshot taken before install | R5, R6, R7 |
| Windows VM without WebView2 | R8 |
| Second PC on the same LAN | A5, A42 |
| Second device signed in to the same tailnet | A44 |
| Phone with cellular data and the Tailscale client | A45 |
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
- For remote-access tests: the installed Tailscale CLI (`tailscale version`,
  `tailscale status --json`, `tailscale serve status --json`). Record the exact
  DNS name and Serve paths; do not substitute a same-host request for a second
  device.

---

## 3. Build the test fixture

Create this project once on the test PC. Reuse it for every test unless a test
says otherwise.

The fixture can be built by hand with the steps below, or generated from the
imported asset library after §3.4:

```powershell
cargo run -p fgc-core --example qa_fixture
```

The generator keeps the imported assets, replaces the teams, talent, match and
tournaments, and refuses to write a state that needs repair. The manual steps
remain the reference for what the fixture contains. The long names sit at the
hard input limits (32 characters for a team, 24 for a player or talent), so the
same fixture exercises both fitting and the limit itself. Back up
`app.json` before regenerating; the generator overwrites the live file.

### 3.1 Teams

Create four teams on **Teams → + New team**:

| Team | Name | Tag | Record | Colours |
|---|---|---|---|---|
| A | `Northwind Sentinels` | `NWS` | `12-3` | Background `#1f6feb`, text `#ffffff`, logo background `#090909` |
| B | `王小明战队` | `WXM` | `9-6` | Background `#d29922`, text `#111111`, logo background `#090909` |
| C | `Long Team Name That Overflows` | `LONG` | `0-0` | Any |
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
| `Longplayer Name Overflow` | Damage | Sub | `she/her` |

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

Create 22 additional teams, `Stress 1` to `Stress 22`, with any tag, so the
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

> **Animation caveat:** with **Shutdown source when not visible** off, OBS keeps
> the page loaded, so the CSS entrance animations play once and do not replay
> when the scene becomes active again. Turn on **Refresh browser when scene
> becomes active** if the show needs the entrance animation on every scene
> entry. With that setting off, treat a non-replaying entrance animation as
> expected rather than an R1 failure.

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
| Game overlay | Score `2 - 1`; tags `NWS` and `WXM`; title `Grand Final`; subtitle `Upper Bracket`; current map `Ilios`; first-to `3`; side indicator matches Attack / Defend. Both ban strips show hero art (imported heroes have no icon link, so this exercises the fallback); link a custom icon in Assets to check the override. `NORTHWIND SENTINELS` fits on one line; `王小明战队` fits on one line. |
| Map select | Five maps in order; `Ilios` chosen by NWS; `Nepal` chosen by WXM; `King's Row` chosen by NWS; scores match the fixture. Map name, type and chosen-by are centred on every card, including a wrapped long name. |
| Hero bans | One row for the current map (`Ilios`); first ban by NWS; Ana and Kiriko shown with the `1st ban` / `2nd ban` labels and their art. |
| Map pool | The same five maps, grouped by mode: Control (Ilios, Nepal, Antarctic Peninsula), Escort (Dorado), Hybrid (King's Row). The header and pool fade in once, in that order; the pool does not flash fully visible first. |
| Home roster | Northwind Sentinels players in roster order, with roles, portraits and heroes. `ДМИТРИЙ ВОЛКОВ` shrinks to fit; the `Longplayer Name Overflow` sub line ellipsizes; every core card keeps the same height. |
| Away roster | 王小明战队 players in roster order, with roles, portraits and heroes; the same card heights as the home roster. |
| Casters row | Arc, Priya, Marco, Sam in assignment order, with the correct platform icons (Twitch, Twitter/X, YouTube, Instagram) and handles. Each handle shows exactly one `@` (`@arc`, not `@@arc`). |
| All seven | No clipping at 1920×1080; transparent background; Archivo renders; long names shrink and then ellipsize without overflowing the canvas or changing card heights. Rounded image blocks show no light fringing at the corners. |
| All seven | No network request leaves `127.0.0.1:3001`. Only the overlay WebSocket and local `/assets/...` URLs load. |
| All seven | After 10 minutes, all seven remain connected and no visible lag appears when editing the match. |

**Fail:**

- Any overlay is blank, shows the wrong team or side, or shows stale data.
- Any overlay requests a remote font or image.
- Text overflows the canvas, is clipped, or renders in a fallback font for Latin
  text.
- A source disconnects, flickers or stops updating during the 10-minute watch.
- The 6-caster `row-casters` clipping was a historical v1 observation (M9-7)
  that was fixed after v1 and verified in A53. Do not fail R1 for the v1 case;
  if clipping recurs, record DOM boxes and computed styles for the failing
  handle instead of relying on a screenshot.

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

- All seven overlays still render Archivo (the active overlay font).
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
8. Set **Mode** to **Count to target** with a target about two minutes in the
   future, then **Start**. Confirm **Start** was disabled before a target was
   set, that the app clock counts down to the target, and that the ticks carry
   the remaining time.

**Pass:**

- The page receives `init` and `matchTimer`.
- Ticks arrive about once per second while running.
- Start, pause and reset are visible in the payload.
- Setting changes appear in later messages.
- Count to target counts down and stops at zero when the target is reached.

**Fail:**

- No connection, no `init`, or no `matchTimer`.
- Ticks stop while the timer is running.
- Actions are missing or the payload does not change with the settings.
- Count to target does not count down, or **Start** is enabled without a target.

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

**Result: Pass with note — owner 2026-11-20.** Exact answer: **“Pass: note, I did
not use a clean VM, as I didn't have one set up. I recorded it on a separate
computer from where the build is taking place”.** All ten steps passed on a
separate computer using the fresh installer from `dist\`: product name,
publisher and icon recorded; standard-user install; Start Menu entry; launch
with all seven destinations; server and overlay; uninstall; removal of the
shortcut and install folder; and a recorded `%APPDATA%` retention result. The
clean-VM precondition was not met, but the owner explicitly accepted the
substitution by answering Pass. M10-4 is Done; M10-3 remains Partial because the
desktop shortcut was not separately recorded.

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

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** The current ZIP
was extracted to `C:\Tools\FGC-Portable\` on a machine with WebView2 and no
installed copy. All five required entries were present; `fruiterra-gc.exe` ran
from that folder with fonts and styles, all seven overlays loaded in OBS, and no
install entry was created. M10-5 is Done.

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

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** Both current
artifacts were downloaded through a browser to a fresh folder. SmartScreen
warned as expected for the unsigned build, identified the app, and **Run
anyway** worked. `Get-FileHash` values for both files matched
`dist\SHA256SUMS.txt` exactly. M10-7 is Done.

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

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** On a VM without
WebView2, the installer added the runtime and the app launched. On a second
clean snapshot without WebView2, the portable build failed with a clear message
telling the user to install WebView2 or use the installer; it was not a silent
crash. Installed-apps screenshots and the portable error message were recorded.

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

**Result: Pass — 2026-11-21.** Workspace `1.0.0` matched the pushed `v1.0.0`
tag on commit `c244d29`. Release run
[44826182532](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/44826182532)
completed successfully, and the
[v1.0.0 release](https://github.com/ThePeachZA/fruiterra-graphics-controller/releases/tag/v1.0.0)
published all three expected assets. Downloaded `SHA256SUMS.txt` matched the
installer at `04d97d7f6dacc98f6e93068e1cf5ae693e701e9a298d8b92ca7c72788469ecf3`
and the portable ZIP at
`17693cd63254633397fb02216c283d31842695e34ecf195aae9b03306eb0433a`. The notes
matched the shipped template, contained no backlog claims, and repeated the
known limitations.

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

1. On the host PC, open **Settings** and turn on **Allow LAN connections**. In
   post-v1 builds, turn on **Remote access → Local network** instead; both
   names share the same bind flag.
2. Allow the Windows Firewall prompt for **private networks** only. If Windows
   does not prompt, confirm the inbound rule is **TCP**, **Private** profile,
   `RemoteAddress = LocalSubnet`, for the app executable. Remove any UDP rule or
   any rule that allows `Public` or `Any` remote address.
3. Note the host's LAN IP with `ipconfig`.
4. On the second PC, open
   `http://<host-ip>:3001/overlays/game-overlay.html`.
5. Confirm it connects and updates when the host edits the match.
6. Turn **Allow LAN connections** off, or turn off **Remote access → Local
   network** in post-v1 builds. Confirm the second PC can no longer connect
   after a reload.

**Pass:**

- The second PC loads the overlay and receives live updates.
- Firewall access is limited to private networks and the local subnet.
- Turning LAN mode off closes remote access.

**Fail:**

- The second PC cannot connect with LAN mode on and the firewall allowed.
- The overlay loads but does not update.
- LAN mode remains reachable after it is turned off.
- The firewall rule is opened for public networks or any remote address.

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
6. Replace the override with an empty file and reload. Confirm the app falls back
   to the built-in file. If you can, repeat with the file locked by another
   process; it should also fall back.
7. Delete the override and reload.

**Pass:**

- Only the overridden overlay changes.
- An empty or unreadable override falls back without breaking the other six.
- Removing the override restores the built-in file.

**Fail:**

- No override is picked up.
- All overlays change.
- An empty or unreadable override leaves the overlay blank instead of falling
  back.
- The app crashes.

**Evidence:** Screenshots of the marker, the other overlays, and the fallback.

---

### A8 — Reconnect after server restart

**Covers:** M3-5

**Result: Pass with note — owner 2026-11-20.** Exact answer: **“Pass: note that hero ban images should only apply on game-overlay.html, not on hero-bans.html; hero-bans.html should also ensure to crop images to the correct dimensions to ensure there is no overflow”**. The note supersedes the overlay freeze for `crates/server/overlays/hero-bans.html` only. Follow-up applied: projection excludes attached, legacy linked and duplicate-file ban-icon identities from hero portraits; `hero-bans.html` reads only `heroPortraits` and clips them with `min-width/min-height: 0`, `overflow: hidden` and `object-fit: cover`. CDP verified no overflow at 1920×1080, while `game-overlay.html` still applies `heroIcons`. M3-5 is Done.

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

`GET /state` returns a fresh snapshot only when requested. Refresh the tab, or
re-run `fetch('/state').then(r => r.json()).then(j => console.log(j.scoreboard.teamA.logoUrl))`,
after **every** action; an already-open tab is expected to keep showing the old
snapshot. Start the server from **Overlay** first.

**Pass:**

- The overlay shows the new logo immediately or after the next live update.
- The URL contains a new hash.
- No stale logo remains cached.

**Fail:**

- The overlay keeps the old logo until OBS is restarted.
- `/state` still contains the old URL.
- The image 404s.

**Evidence:** Old and new `/state` excerpts and overlay screenshots.

**Result: Pass after fix — owner recheck 2026-11-20.** Initial recheck answer:

> When the app launched, the app had Team 3 and Team 1 set. Neither of these teams have a logo assigned to them, and /state says null (which is correct). However, the overlay still showed two logos on it which should not be there. It was fixed when the overlay in OBS was refreshed.
>
> When clearing the log in the app, the overlay still did not remove the logo in real time. It did get removed when the OBS overlay got restarted.
>
> /state correctly updates the logo and hash on refresh.
>
> Additional note for the eventual overlay refresh that will come later is that when no logo is set, there should be a placeholder image containing the TAG or an otherwise abbreviation of the team name so that there is still something at least.

`/state` is confirmed correct after a fresh GET. The Fail is the live OBS image:
the current match loaded with **Team 3** and **Team 1**, both without logos,
while `game-overlay.html` still showed two old logos until the OBS source was
refreshed. **Clear** had the same symptom: the app and `/state` removed the
logo, but the live OBS source kept it until restart.

Root cause: `game-overlay.html` assigns `backgroundImage` only inside
`if (d.teamA.logoUrl)` / `if (d.teamB.logoUrl)` and has no `else`, so a `null`
logo never clears the background already set in the DOM. A fresh OBS source has
no stale background and renders correctly.

The owner approved a scoped override of the overlay freeze for
`game-overlay.html`. The fix assigns `backgroundImage` unconditionally: the URL
when present, otherwise `none`. An explicit `null` team also clears its slot;
an absent team object in a partial frame is left alone for compatibility.

CDP verification on the rebuilt desktop app injected stale URLs, sent
`logoUrl: null` for both teams and measured `none` for both slots. Frames with
`teamA: null` / `teamB: null` also cleared both slots, and new URLs still
applied. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo test --workspace`, `dx build --desktop` and
`git diff --check` passed; the new static test
`game_overlay_clears_stale_team_logos` pins the behaviour. The
TAG/abbreviation placeholder is recorded for the later overlay refresh and is
not part of this fix. Owner recheck after the fix: **“Pass.”** The OBS source
was not refreshed or restarted. A9 is accepted; M3-6 is Done.

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
5. On **Match Controller**, visit all five tabs and change one field on each.
   Open the timer widget settings with the keyboard and change one timer field.
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

> If the window is a fixed logical size, the same layout check can be
> simulated with the WebView developer tools by setting the device scale
> factor to 1.25 and 1.5 and shrinking the viewport to the size a scaled
> screen leaves (for example 1280×720 for 1080p at 150%).

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

1. Disconnect the network, or emulate offline in the WebView developer tools.
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
5. **Deduplication:** import the same file twice. Confirm the bytes are stored
   once — no new file appears under `assets\blobs` — even though the library
   lists a second entry.
6. **Properties:** on a Hero asset, select **Properties…**, then **Upload ban
   icon…** and pick a local image. Confirm the game-overlay ban strip for that
   hero uses the chosen image and that the chosen image does not appear in the
   library, hero pickers or Match Controller ban lists. Select **Remove ban
   icon** and confirm the strip returns to the hero's own art. On a Map asset,
   change **Game mode** and then select **No game mode** to confirm the
   map-pool grouping returns to **Other**.

**Pass:**

- Valid files import.
- Cancel leaves the library unchanged.
- Invalid URLs, oversized files and unsafe SVG files are rejected with a
  readable reason.
- A near-limit image imports without freezing.
- Duplicate bytes are stored once.
- A ban icon uploaded from **Properties…** replaces the strip art, is hidden
  from the library and hero pickers, and **Remove ban icon** restores the
  fallback.
- A **Game mode** selection regroups the map on the map-pool overlay, and
  **No game mode** moves it back to **Other**.

**Fail:**

- The app crashes, freezes or imports an unsafe SVG.
- A rejected import still changes the library.
- A duplicate stores a second copy of the bytes.
- The error message does not say what is wrong.
- The properties dialog closes without applying or clearing the ban icon.
- An attached icon still appears in the library or as a selectable hero.

**Evidence:** Screenshots of each result and the library count.

---

### A17 — Project bundle export and import

**Covers:** M5-5

**Result: Blocked — owner 2026-11-20.** The owner first saw a `Curro` blob hash
mismatch, a partial bundle with no `app.json` or `export.json`, images under
`blobs/blobs/...`, and asked how to install the app before a release. The corrupt
QA blob was restored, export now verifies every blob before writing the bundle,
and both export and import use the flat `blobs/<prefix>/<hash>.<ext>` layout.
Assistant export verification produced a bundle with 23 hash-matched user blobs
and no bundled art. On recheck the owner answered **Blocked**: no second Windows
account/VM or release/installer was available for steps 4–6, so second-machine
import remains unrun and A17 is not a Pass. On 2026-11-21 the owner accepted the
Blocked result for release with the exact answer **“Accept A17; authorize tag”**;
A17 is blocked-accepted.

**Steps:**

1. On **Assets**, select **Export project…** and choose a folder.
2. Confirm the created `fruiterra-project-<timestamp>/` contains `app.json`,
   `export.json` and `blobs/<prefix>/<hash>.<ext>` for every referenced user
   image. There must be no nested `blobs/blobs` directory.
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

**Installing before a release:** no installer or tag exists yet. For this gate,
copy the whole `target\dx\fruiterra-gc\debug\windows\app` folder from
`dx build --desktop` to the second Windows account or VM and run
`fruiterra-gc.exe` there; WebView2 must already be installed. That account uses
its own `%APPDATA%\Fruiterra\Fruiterra Graphics Controller`, so the QA fixture
on the main account is not overwritten. If a second account/VM is unavailable,
answer **Blocked**.

**Outstanding follow-up (tracked as Doctrine `CHR-005`):** steps 4–6 have not
been run. A clean-machine import is not satisfied by the code-level
`export_import_round_trips_on_a_second_directory` test, even though that test
passed again on 2026-11-21. To close `CHR-005` and change A17 from
blocked-accepted to Pass, run steps 4–6 on a second Windows account or VM with
the v1 portable build, copy an exported bundle, import it through **Import
project…**, then check teams, talent, assets, the match and both tournaments.
Record the bundle listing and before/after evidence on both machines. Do not
claim second-machine restore until that evidence exists.

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

1. Visit all five tabs: **Match**, **Maps & Bans**, **Talent**, **Map Pool** and
   **Config**. Confirm there is no separate **Hero Bans** or **Timer** tab.
2. On **Match**, change every field, including **Manual score override**.
3. On **Maps & Bans**, add a map, reorder it, remove it, mark **Current map**
   and **Complete**, and use **Reset all maps**. Set the first-ban team and both
   heroes on a map, then use **Reset all bans**.
4. On **Talent**, assign every slot, set a role on each assignment, and use
   **Custom name** and **Custom subtext**.
5. On **Map Pool**, tick and untick maps, add a custom map, reorder the pool and
   remove an entry.
6. On **Config**, toggle **Team colours** and **Player portraits**, and change
   the branding images.
7. Open the timer widget's settings popup and change one timer field.
8. Trigger one invalid input on each tab where validation exists.

**Pass:**

- Every control works and saves.
- The destructive reset buttons clear their data immediately, as documented.
- Invalid input follows the documented fallbacks: numeric scores clamp to `0`,
  an empty map name is ignored, and **Start** is disabled until a target is set.
- Switching tabs preserves the values.

**Fail:**

- A control does nothing or loses its value.
- A reset button is described as confirmed when it is not, or behaves
  differently from the documented behaviour.
- Invalid input produces a crash, or is stored differently from the documented
  fallback.
- Tab switching resets data.

**Evidence:** Notes per tab and screenshots of any validation error.

---

### A21 — Timer behaviour

**Covers:** M7-2

**Steps:**

1. Open the timer widget's settings popup. Set **Mode** to Count up, close the
   popup, select the icon **Start**, wait five seconds, select **Pause** and read
   the value.
2. Reopen settings, set **Mode** to Count down with `00:00:10`, close the popup,
   select **Start** and let it reach zero.
3. Set **Mode** to Count to target with a target two minutes in the future and
   select **Start**.
4. Toggle **Leading zeroes** and **Standalone timer** in the popup.
5. Visit another destination (for example Teams) and return. Confirm the timer
   keeps running and the same clock and button state are shown throughout.
6. Select **Stop** and confirm the clock returns to zero, or to the count-down
   start. Close and reopen the app. Confirm the timer settings persist, run state
   is paused and the target is cleared.

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
- The timer resets or stops when switching tabs or destinations.

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
3. Confirm every match card is readable and the rounds read left to right as
   columns, with the horizontal scroll reaching every round.
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
   slots. A slot can only be fed by one match, so a target slot that is already
   fed takes over: the previous source link is cleared.
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

### A29 — Overlay URL visibility, server auto-start and timer removal

**Covers:** Touch-ups 1, 2, 3

**Steps:**

1. Open **Overlay**. Confirm every Browser Source URL is hidden by default, the
   page shows **Show URLs**, the base URL is not revealed, and the note explains
   that URLs are hidden until revealed.
2. Select **Show URLs**. Confirm all seven overlay URLs and the base URL become
   visible and the action becomes **Hide URLs**.
3. Select **Hide URLs**. Confirm the URLs return to the hidden state.
4. Open **Settings**. Confirm **Start server on launch** is present (default
   off on a fresh/default state) and there is no Timer mode, alignment, leading
   zeroes or standalone timer field.
5. Confirm Settings points timer control to the timer widget above the status
   bar, and that it offers no link to a Timer tab.
6. Enable **Start server on launch**, close the app, reopen it, and confirm the
   overlay server starts automatically during bootstrap without pressing
   **Start server**. Disable the setting afterwards if needed.

**Pass:**

- URLs are hidden until explicitly revealed and can be hidden again.
- All seven overlay URLs and the base URL are shown only while revealed.
- **Start server on launch** persists and starts the server on the next launch.
- Settings contains no duplicated timer controls.

**Fail:**

- Any overlay URL or base URL is visible before reveal.
- **Show URLs** / **Hide URLs** does not toggle correctly.
- The server does not auto-start after a restart with the setting enabled.
- Timer fields reappear in Settings.

**Evidence:** Overlay screenshots (hidden and shown), Settings screenshot, and a
restart note showing auto-start behaviour.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All six steps
passed, including the restart check: **Start server on launch** persisted and
brought the server up automatically without pressing **Start server**. Touch-ups
1–3 are accepted.

---

### A30 — Sidebar order, icons and floating timer

**Covers:** Touch-ups 4, 13, 19

**Steps:**

1. Check the sidebar order. Confirm it is **Match Controller, Overlay, Teams,
   Talent, Assets, Tournament, Settings**.
2. Confirm each destination shows its icon (not a letter abbreviation) and the
   active destination is highlighted.
3. Confirm the timer widget appears above the status bar on every destination,
   with a clock, icon **Start**, **Pause**, **Stop** and **Timer settings**
   buttons.
4. Start the timer from the widget, switch to several other destinations, and
   confirm the widget keeps counting and reflects the same state.
5. Pause and stop from the widget and confirm any connected timer consumer
   receives the same state/actions.
6. Check placement and legibility at 100%, 125% and 150% scaling (or the
   documented simulated viewports) and confirm it does not cover critical
   controls.

**Pass:**

- Sidebar order and icons match the new layout.
- The floating timer is present and usable on every destination.
- Start/pause/stop and ticks stay in sync with `TimerController` across all
  destinations.
- Placement does not obscure essential UI.

**Fail:**

- A destination is out of order, missing, or shows an abbreviation.
- The timer widget is missing, stops counting, or desynchronises between
  destinations.
- Floating timer actions do not broadcast as expected.
- The timer overlaps or blocks important controls.

**Evidence:** Sidebar screenshot, per-destination floating-timer screenshots,
and a short timer tick/action check.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All six steps
passed: the sidebar order and icons match, the timer is present on every
destination, ticks and Start/Pause/Stop stay in sync with connected consumers,
and placement remains legible and non-obscuring at 100%, 125%, and 150%.
Touch-ups 4, 13, and 19 are accepted.

---

### A31 — Team logo colours, logo background and player-card preview

**Covers:** Touch-ups 5, 6, 7

**Steps:**

1. Open a team and confirm **Separate logo background** is off by default for
   an unchanged/default team state.
2. Choose or change the team logo to an image with a clear dominant colour.
   Confirm the sampled dominant non-neutral colour becomes the team background
   and a contrasting text colour is selected.
3. With **Separate logo background** off, confirm the projected `logoBgColor`
   uses the team background colour in `/state` or in the affected overlays.
4. Turn **Separate logo background** on, choose/configure the logo background as
   supported by the UI, and confirm the projected logo background differs as
   intended.
5. Open a player in the roster editor and confirm the **Roster card preview**
   panel shows the player as they appear in the roster list.
6. Edit the player's name, role, position, pronouns, portrait and hero and
   confirm the preview updates.

**Pass:**

- Logo colour sampling picks sensible background/text colours for the chosen
  image.
- **Separate logo background** defaults off and changes projected logo
  background behaviour correctly.
- The player-card preview is present and updates with edits.

**Fail:**

- Sampling fails, picks a neutral/incorrect colour, or leaves unreadable text.
- Logo background projection ignores the toggle/default.
- The player-card preview is missing or does not reflect edits.

**Evidence:** Team editor screenshots, `/state` or overlay evidence for logo
background projection, and player-editor preview screenshots.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All six steps
passed: dominant-colour sampling and contrast text were sensible, **Separate
logo background** defaulted off and changed projected `logoBgColor` correctly,
and the roster card preview reflected every player edit. Touch-ups 5, 6, and 7
are accepted.

---

### A32 — Match Controller Maps & Bans, Config, schedule and match semantics

**Covers:** Touch-ups 8, 10, 11 (schedule field)

**Steps:**

1. Open **Match Controller** and confirm the tabs are **Match, Maps & Bans,
   Talent, Map Pool, Config** with no separate **Hero Bans** or **Timer** tab.
2. Open **Maps & Bans** with **Hero bans** enabled in Config. Confirm map
   selection and hero ban controls are available together.
3. Turn **Hero bans** off in **Config** and return to **Maps & Bans**. Confirm
   hero ban pickers/controls are hidden and the UI explains how to re-enable
   them.
4. Confirm **Loser picks** is in **Config** and still affects map-pick
   behaviour as before.
5. On the **Match** tab, set **Scheduled start** and save/reload as needed.
   Confirm the value persists.

**Pass:**

- Maps and hero bans are merged into **Maps & Bans**.
- Hero ban controls appear only when `settings.display.show_hero_bans` is on.
- **Loser picks** and **Hero bans** live in **Config** and behave correctly.
- **Scheduled start** can be set and persists.

**Fail:**

- A separate **Hero Bans** tab remains, or hero ban controls ignore the toggle.
- **Loser picks** is missing from Config or changes behaviour unexpectedly.
- **Scheduled start** is missing, not saved, or lost on reload.

**Evidence:** Tab screenshots, Config toggle screenshots, Maps & Bans
on/off screenshots, and saved-state/reload evidence for `scheduled_at`.

**Result: Pass with notes — owner 2026-11-20.** Exact answer: **“Pass.”** All
five steps passed. Touch-ups 8, 10, and 11 are accepted. Five follow-ups were
raised and recorded in `docs/manual-qa.md` §2.19: hero-ban icons remaining
visible when bans are off, a countdown-to-scheduled-match timer mode,
first-to-driven default map counts, unfiltered map option lists, and consistent
map-field sizing.

**Follow-up verification (assistant, 2026-11-20): all five Done.** On a fresh
`dx build --desktop` binary with CDP 9222: hero bans off projected `heroIcons:
0` and `heroBans: 0` while `heroPortraits` stayed populated; scheduled mode
showed a positive, decreasing countdown on startup, runtime mode switching and
Match-tab date edits; New match produced 5 slots, Ft1/Ft2/Ft4/Ft3 produced
1/3/7/5, and **+ Add map** grew 5→6; map name/mode selects kept every option
visible and **Custom…** inputs wrote through to `/state`; map name, mode,
chosen-by and score inputs all measured 44 px high. After the owner reported
phantom tiebreakers on startup, load-time validation now normalises trailing
empty slots (including derived-mode-only slots) to the first-to format while
named or manually entered tiebreakers survive. QA `app.json` was restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

---

### A33 — Match library save, load, delete, export and import

**Covers:** Touch-ups 9, 11

**Steps:**

1. Configure a current match with distinctive values, a map pool and a
   **Scheduled start**.
2. In **Match library**, select **Save current** (or equivalent). Confirm the
   snapshot appears in the library.
3. Change the current match, then **Load** the saved snapshot. Confirm the
   current match, map pool and scheduled time are restored.
4. **Export** the library. Confirm the exported JSON is portable and includes
   `schema_version` and `matches`.
5. **Import** the exported file. Confirm only new match IDs are added and
   existing teams, talent and assets are unchanged.
6. Try importing a file with duplicate IDs, out-of-range `first_to`, and a newer
   `schema_version`. Confirm duplicate IDs are skipped, `first_to` is clamped to
   1–4 where applicable, and a newer schema is refused with a readable error
   (`InvalidMatchLibrary` path).
7. **Delete** a saved match and confirm it is removed from the library.

**Pass:**

- Save/load/delete work and preserve map pools and scheduled times.
- Export/import use the portable versioned format.
- Import validation behaves as specified and never replaces teams, talent or
  assets.

**Fail:**

- Saved snapshots lose map pools or `scheduled_at`.
- Export/import corrupts or replaces unrelated data.
- Invalid or newer-schema imports are accepted silently or crash.
- Delete leaves stale entries.

**Evidence:** Library screenshots, exported JSON sample, import result notes,
and before/after saved-state evidence.

**Result: Pass with note — owner 2026-11-20.** Exact answer: **“Pass. Note: new
map button does not add a new map.”** All seven steps passed; Touch-ups 9 and 11
are accepted. The note was recorded as A33-1 in `docs/manual-qa.md` §2.20.

**Follow-up verification (assistant, 2026-11-20): A33-1 Done.** The
**+ Add map** button called `insert_map()`, but `AppStore::mutate()` validates
after every mutation, so the new empty slot was immediately trimmed as trailing
surplus. `MapSlot.user_added` serialises as `userAdded`; `insert_map()` sets it
and `map_slot_is_empty()` treats marked slots as user data. On a fresh
`dx build --desktop` binary with CDP 9222, the restored QA state started with 1
slot, **+ Add map** grew it to 2, saved `app.json` showed `userAdded: true` on
the new slot, and both slots remained after an app restart. QA `app.json` was
restored to hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

---

### A34 — Per-match talent roles and quick upload in image pickers

**Covers:** Touch-ups 12, 18 and the final talent-role change

**Steps:**

1. Open **Match Controller → Talent**. For each occupied person slot, confirm a
   **Role** select with **Caster**, **Host**, **Analyst**, **Interviewer** and
   **Other**.
2. Set different roles on two slots using the same talent where possible, save,
   and reload. Confirm each assignment keeps its own role in
   `currentMatch.talent.<slot>.role` and that the casters overlay uses the role
   labels.
3. Load a legacy save that has `talents[].type` of `panelist` or `observer`.
   Confirm it migrates to schema 2, removes the person-level `type`, and maps
   those values to `analyst` and `other` on occupied assignments.
4. In every relevant image picker (at minimum team logo, player portrait/hero,
   talent portrait, and any other picker exposed by the UI), confirm a quick
   upload action is available.
5. Use quick upload to add an image and confirm it is imported into the relevant
   category and auto-selected for the current field.

**Pass:**

- Role is editable per match assignment and persists correctly.
- Legacy records load through the v2 migration with the documented aliases.
- Quick upload is available where promised and auto-selects the new asset.

**Fail:**

- A role is shared between all assignments, missing, or does not persist.
- Legacy records load incorrectly or keep the obsolete person-level `type`.
- Quick upload is missing from a picker, imports to the wrong category, or does
  not select the new asset.

**Evidence:** Talent editor screenshots, saved-state/type evidence, and picker
before/after screenshots for each tested context.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All five steps
passed: role is editable per assignment, persists independently, and drives the
casters labels; the legacy `panelist`/`observer` aliases migrate to schema 2
with the person-level `type` removed; quick upload is available in the relevant
pickers and auto-selects the imported asset. Touch-ups 12, 18 and the final
talent-role change are accepted.

---

### A35 — Assets multi-file import, per-file errors and thumbnails

**Covers:** Touch-ups 15, 16, 17

**Steps:**

1. Open **Assets** and confirm the import action supports selecting multiple
   files (**Import files…** or equivalent).
2. Select multiple valid images in one operation. Confirm each is imported and
   the UI reports progress/results appropriately; confirm the first successful
   upload is auto-selected where the flow specifies.
3. Include a file that should fail (for example unsupported, oversized or unsafe
   SVG) in a multi-file selection. Confirm per-file errors are reported and
   valid files still import.
4. Inspect the asset library/gallery. Confirm cards render with the fixed
   name/badge layout and image thumbnails/previews where expected for bundled
   and user assets.
5. Import or view enough assets to exercise thumbnail loading. Confirm the UI
   remains responsive (thumbnail reads/encoding happen without freezing) and
   repeated views reuse cached thumbnails where observable.
6. Verify rename, delete and link flows still behave correctly after the gallery
   changes.

**Pass:**

- Multi-file import works and reports per-file failures without aborting valid
  files.
- Thumbnails/gallery render correctly and responsively with the updated layout.
- Existing asset metadata and linking flows are unaffected.

**Fail:**

- Only one file imports, or one bad file aborts the whole batch.
- Per-file errors are missing or unclear.
- Gallery cards break layout, fail to show previews, or freeze the UI.
- Rename/delete/link behaviour regresses.

**Evidence:** Import dialog/result screenshots, library screenshots showing
thumbnails and layout, and notes on per-file errors/responsiveness.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All six steps
passed: multi-file import reported per-file failures without aborting valid
files; gallery cards and thumbnails rendered responsively with the fixed layout;
rename, delete and link flows were unaffected. Touch-ups 15, 16 and 17 are
accepted.

---

### A36 — Tournament conditional tabs and format changes

**Covers:** Touch-up 14 and the final multi-stage change

**Steps:**

1. Open a **round robin** tournament. Confirm the tabs are **Setup, Groups,
   Settings** and **Board** is not shown.
2. Open a **single elimination** tournament. Confirm the tabs are **Setup,
   Board, Settings** and **Groups** is not shown.
3. Open a **double elimination** tournament. Confirm the tabs are **Setup,
   Board, Settings** and **Groups** is not shown.
4. Open a **multi-stage** tournament. Confirm the tabs are **Setup, Groups,
   Board, Settings** and that both **Groups** and **Board** are shown. The
   Board is locked until the group stage is complete.
5. Change an existing tournament's format through **Setup → Format**, or create
   one tournament of each format if the app does not support changing it.
   Confirm the tab strip updates immediately after each change.
6. With a multi-stage event that has both groups and a playoff bracket, switch
   between **Board** and **Groups**, close the tournament, reopen it and reload
   the app. Confirm group names, fixtures, results, bracket slots and the saved
   `format: "multi-stage"` value remain intact.

**Pass:**

- Round robin shows Groups only; single and double elimination show Board only.
- Multi-stage is the only format that shows **Board** and **Groups** together.
- Format changes/tournament selection show the correct tabs without losing data.
- Reload keeps the multi-stage format and both stages accessible.

**Fail:**

- The wrong tab set is shown for a format.
- Board and Groups appear together for a non-multi-stage event, or are missing
  from a multi-stage event.
- Switching formats or selecting tournaments hides or corrupts existing data.
- A reload silently changes the format or drops either stage.

**Evidence:** Screenshots of each tournament format's tab strip, a saved-data
note showing `format: "multi-stage"`, and before/after notes for group and
bracket data.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All six steps
passed: round robin showed Groups only; single and double elimination showed
Board only; multi-stage showed both Groups and Board with the Board locked until
the group stage was complete; format changes and tournament selection preserved
data; reload kept `format: "multi-stage"` and both stages intact. Touch-up 14
and the final multi-stage change are accepted.

---

### A37 — Timer widget is the only timer control

**Covers:** Final timer widget change

**Steps:**

1. Open **Match Controller**. Confirm the tabs are **Match, Maps & Bans,
   Talent, Map Pool, Config** and there is no **Timer** tab.
2. Open **Settings**. Confirm there are no Timer mode, alignment, duration,
   target, leading-zeroes or standalone fields; the Timer controls note points
   to the widget above the status bar.
3. On any destination, confirm the widget shows the clock and four icon
   buttons: **Start timer**, **Pause timer**, **Stop timer** and **Timer
   settings**. Only the enabled buttons should react.
4. Select **Timer settings**. Confirm the popup contains **Mode**, Countdown
   **Hours/Minutes/Seconds** in Count down mode, **Target date and time** in
   Count to target mode, **Alignment**, **Leading zeroes** and **Standalone
   timer**. Close the popup with **×**.
5. Set **Mode** to Count up, select **Start**, wait five seconds, select
   **Pause**, then **Stop**. Confirm the value advances, holds, and returns to
   `00:00:00`.
6. Set **Mode** to Count down with `00:00:10`, select **Start** and let it reach
   zero. Confirm it stops at zero and does not go negative. Select **Start**
   again and confirm it restarts from `00:00:10`.
7. Set **Mode** to Count to target. Confirm **Start timer** is disabled while
   **Target date and time** is empty. Enter a target two minutes ahead and
   select **Start**; confirm the clock counts down.
8. While it is running, visit **Teams**, **Overlay** and back. Confirm the same
   clock and disabled/enabled button state are shown on every destination and
   that the R4 test page keeps receiving `matchTimer` ticks.
9. Reload the app. Confirm the mode, alignment, switches and count-down
   duration persist, the clock is paused at its starting value, run state is not
   restored, and a Count to target target is cleared.

**Pass:**

- Only the widget exposes timer controls; no Timer tab or Settings fields
  remain.
- Icon Start/Pause/Stop actions and the settings popup behave as documented.
- Timer settings persist while run state does not.
- The clock and `matchTimer` broadcast continue across destinations.

**Fail:**

- A Timer tab, duplicated Settings fields or a second timer control returns.
- Any icon or popup field is missing, mislabelled or inert.
- Stop only pauses, run state survives a reload, or settings are lost.
- The clock freezes, jumps or stops broadcasting when the destination changes.

**Evidence:** Screenshots of the widget and popup, a short tick log or R4 test
page capture, and a before/after restart note.

---

### A38 — Per-match talent roles, layout and schema v2 migration

**Covers:** Final talent-role change and the pulled-forward A38 layout fix

**Steps:**

1. Open **Match Controller → Talent**. Confirm there is one person per row with
   no visible slot labels, and that each person picker and its **Role** select
   are horizontally aligned on the same line. The six slot names should still be
   announced to screen readers, but not printed above the controls.
2. For every occupied person slot, confirm a **Role** select with **Caster**,
   **Host**, **Analyst**, **Interviewer** and **Other**. Confirm the Talent
   library form has no per-person **Type** field.
3. Put the same talent member into two different person slots. Set one to
   **Host** and the other to **Analyst**, save, reload the app and reopen the
   page. Confirm each slot kept its own role and `app.json` stores
   `currentMatch.talent.<slot>.role` as `host` and `analyst`.
4. Set a slot to **Other** and confirm `row-casters.html` shows that role label
   instead of the old Observer/Panelist wording.
5. Back up `app.json`. Create a v1 fixture with `"schemaVersion": 1`, a talent
   whose `"type": "panelist"` is assigned to a panel slot, and another whose
   `"type": "observer"` is assigned to an occupied slot. Restart the app with
   that file.
6. Confirm the saved file is upgraded to `"schemaVersion": 2`, `talents[].type`
   is removed, the panel assignment role is `analyst` and the observer
   assignment role is `other`. Confirm no repair error appears.
7. Restore the original `app.json`.

**Pass:**

- Roles are chosen per assignment, persist independently and reach the casters
  overlay.
- The Talent library no longer stores a person-level type.
- v1 `panelist`/`observer` data migrates to schema 2 as `analyst`/`other`
  without data loss.
- The layout has one person per row, no visible slot labels, and the person and
  role controls stay aligned at 100%, 125% and 150% with long names on one line.

**Fail:**

- Changing one slot changes another, a role disappears on reload, or the old
  type field remains.
- The five-role select is missing or the overlay labels do not follow it.
- A legacy save is rejected, keeps `talents[].type` or maps the aliases wrongly.
- Visible slot labels return, rows wrap or stack the two controls, or keyboard
  focus no longer reaches the person then role in row order.

**Evidence:** Talent screenshots, saved-data excerpts for both assignments, the
casters overlay capture, the before/after v1 fixture files, and layout geometry
at the three target scales. Automated CDP evidence is recorded in `docs/manual-qa.md`.

---

### A39 — Multi-stage tournaments, group-stage UX and Push to match

**Covers:** All nine original A39 findings, the five owner findings from the
2026-11-18 re-check, the generation-preview scope issued at item 4, the
layout/copy fixes issued before item 5, the regenerate/reset, seed-editing,
status-toggle, double-elimination connector and drag-indicator findings issued
after item 5 step 7, the item 6 preview **Cancel** obstruction fix, the residual
bracket **Delete match** finding, the item 9 spacing/navigation/collapsible
finding, the bracket-card width/gap follow-up from the item 9 re-check, the
item 10 heading-row/group-count/playoff-location finding and its follow-up
replacing the dropdown with iconised, colour-coded horizontal buttons, and the
item 11 findings about the Setup Generate card, group fixture winner/first-to
parity with the bracket, and the seeding error on status toggle. Items 1–10 are
accepted; item 11 was failed by the owner on 2026-11-19 and reworked. The
additional checks below record the fixes issued after item 5 step 7 and remain
reopen criteria.

**Steps:**

1. Create a tournament with at least six entrants. Set **Format** to
   **Multi-stage** and **Qualifiers per group** to 2. Confirm **Setup** has no
   **Generate** card at all. Open **Groups** and select the `▷` **Preview group
   stage** action in the heading row; the preview splits the entrants into two
   groups with no group-count selector. Confirm **Generate** writes both groups
   and their fixtures. Confirm the tabs are **Setup, Groups, Bracket, Settings**
   in that order. Open **Bracket** and confirm it shows **Playoff locked** with
   no rounds.
2. In **Groups**, confirm a fixture row shows the teams, score, status, a pencil
   **Edit match details** button and an icon-only ↗ **Push to match** action.
   Confirm there are no inline score/status editors and no **Save** button in
   the row.
3. Open the pencil button. Confirm the dialog is centred and fully visible, and
   that its text boxes use the app's themed styling (padding, rounded corners and
   themed colours, not browser defaults). Confirm the dialog has score fields, a
   status toggle switch with only **Upcoming** and **Complete**, a **First to**
   select, a collapsed **Advanced settings** block containing the **Winner**
   override, and a read-only **Effective winner** value. Set a two-team fixture
   to `2–1` and **Complete**, move focus out of the field, and confirm the row and
   `app.json` update without selecting a save button. Close with **Done**. Repeat
   in a bracket match editor and confirm the same two status states only.
4. **Structure is locked after generation.** Open a generated multi-stage event.
   On **Setup**, confirm there is no **Team to add** / **Add entrant** control
   and no ↑, ↓ or × beside an entrant. On **Groups**, confirm there is no
   **+ Group**, **Delete group**, team-picker/**Add**, team chips above the
   standings, **×** controls in standings rows, fixture selectors or
   **Add fixture**. On **Bracket**, confirm there are no
   **Add round**, remove-round or **+ Match** controls, no drag grips, no
   **Unlock dragging** / **Lock dragging**, and no ↑/↓ match move buttons. Try
   to drag a live standings row and a live bracket slot; nothing changes.
5. **Every generation action opens the preview.** For a multi-stage or
   round-robin event, select **Preview group stage** / **Preview round robin**
   from both Setup and Groups. For a single/double-elimination event, select
   **Preview … bracket** from Setup. For an event with groups, select
   **Preview playoff** from Groups. Each action must open a dialog titled
   **Preview … generation** and must not change `app.json` before confirmation.
   Close each with **Cancel**. A valid preview must always open; an invalid
   generation instead shows only an inline error.
6. **The preview shows only the details being committed.** In a multi-stage or
   round-robin group-stage preview, confirm two columns: **Seeding** and
   **Group distribution**. There must be no bracket preview before the group
   stage finishes. In an elimination or playoff preview, confirm **Seeding**
   and **Bracket placements**, with no group distribution. The bracket preview
   must use the same centred cards, non-crossing connectors and geometry as the
   live **Bracket** tab. On the live **Bracket** tab, scores sit inline with the
   two team names.
7. **Group dragging works in the preview.** In a multi-stage preview, drag a
   team chip from Group A onto a chip in Group B. Confirm the chip moves into B
   at that position, the other group loses it, and the preview's fixtures follow
   the new distribution. Confirm no bracket appears and the saved `app.json`
   hash is unchanged.
8. **Bracket dragging works in the preview.** In a preview with bracket rounds,
   drag one slot onto another slot in the same section and round. Confirm the
   two participants swap only in the preview. Try a drop in a different round
   or section and confirm nothing changes. Confirm `app.json` is unchanged.
9. **Cancel is a true no-op.** Repeat a group or bracket drag, select **Cancel**,
   and confirm the dialog closes, the saved event is byte-for-byte unchanged,
   group fixtures and bracket placements are unchanged, and focus returns to the
   button that opened the preview.
10. **Generate commits the previewed result.** Repeat the drag, select
    **Generate**, and confirm one save writes the previewed distribution and
    placements. Reload the app and confirm they persist. For multi-stage,
    confirm the real bracket is still locked until every group fixture is
    complete, then completes and seeds cross-group (`A1 v B2`, `B1 v A2`). Use a
    disposable event for this check.
11. **Whole-app spacing, navigation and collapsible sections.** Walk every
    destination, dialog, panel, form row, card and list at 100%, 125% and 150%.
    Confirm spacing is deliberately comfortable but not wasteful, with no
    clipped text, unexpected wrap or controls overflowing their cards. On
    **Tournament**, collapse and expand the **Tournaments** sidebar by mouse and
    by keyboard; confirm `aria-expanded`/`aria-controls`, focus staying on the
    toggle, and more room for the detail area when collapsed. On **Match
    Controller**, confirm saved matches sit under a collapsed **Saved matches**
    disclosure and hero bans are grouped under a collapsible disclosure when
    enabled. On **Bracket**, confirm match cards are wider with tighter gaps
    between separate matches. Confirm `app.json` is unchanged. This item is the
    whole-app GUI spacing and navigation decision.
12. **Latest Tournament layout and copy fixes.** On **Bracket**, confirm there is
    no **Traditional bracket** heading and that scores sit inline with the two
    team names. Confirm each card ends with labelled **✎ Edit** and **↗ Push**
    actions whose text stays inside the card at 100%, 125% and 150% zoom. On
    **Groups**, confirm no **Team** chips sit above the points table. Open a
    bracket match editor and confirm **Winner**, **Winner advances to** and
    **Loser drops to** are hidden under collapsed **Advanced settings**, with the
    controls stacked vertically when expanded. Review the Tournament panels and
    confirm explanatory copy is brief, with deeper guidance left to the wiki.
13. **Stage actions are horizontal, iconised, colour-coded and separate.** On a
    generated multi-stage event, use the **Groups** and **Bracket** tabs; Setup
    has no generation card (see step 18). Beside **Group stage**, confirm a
    single horizontal row
    with **↻ Regenerate group stage** in amber and **⌫ Reset group stage** in red;
    beside **Bracket**, confirm **↻ Regenerate bracket** in amber and
    **⌫ Reset bracket** in red. Before generation, also confirm **▷ Preview** in
    accent orange; there is no manual **+ Group** action before generation.
    Confirm there is no **Settings** dropdown, each button keeps its icon, label,
    `aria-label` and `title`, and the row is reachable without horizontal
    overflow at 100%, 125% and 150%. In a group preview, confirm there is no
    group-count selector and the two group cards come from the generation action.
    On a multi-stage
    event, confirm **Preview playoff** appears only on **Bracket**, never on
    **Groups**. Select each Regenerate control and confirm it opens the preview
    instead of changing state; Cancel it. Select each Reset control and confirm a
    dialog asks for confirmation; Cancel leaves the stage intact. On a
    disposable event, confirm Reset group stage clears groups, fixtures,
    standings and the derived playoff, while Reset bracket clears only the
    bracket and champion. Entrants remain.
14. **Bracket seeding is editable in the preview.** Open an elimination bracket
    preview. Change one seed number and confirm placements regenerate. Drag a
    seed handle onto another row and confirm the order and placements update.
    Confirm no `app.json` write occurs until **Generate**.
15. **Status is a switch.** In both a bracket match editor and a group fixture
    editor, confirm **Status** is a two-state toggle labelled **Upcoming** or
    **Complete**. Toggle both ways and confirm the saved status follows.
16. **Double-elimination connectors are correct.** On **Autumn Invitational**,
    confirm lower-round one-to-one links feed the matching next card, paired
    rounds feed a single card without crossing, and upper-bracket losers show
    dotted drop stubs toward the lower bracket.
17. **The drag indicator is visible.** In a generation preview, start dragging a
    group chip, seed handle or bracket slot. Confirm a floating grip/label follows
    the pointer, the drop target is highlighted, and the indicator is removed on
    drop or cancel.
18. **Setup has no Generate card.** On both an ungenerated and a generated
    tournament, open **Setup** and confirm there is no **Generate** panel or
    heading, no **At least two entrants are required.** note, no
    **Use the action buttons beside the Group stage or Bracket heading…** note
    and no **Preview group stage** / **Preview round robin** / **Preview …
    bracket** button. Generation starts only from the stage heading action rows.
19. **Group fixture details match the bracket editor.** Open a group fixture
    editor. Confirm **First to** is a selectable 1–4 field, just like the
    bracket, and that **Advanced settings** contains a **Winner** select with
    **Derived from scores** plus one option per slot. Pick a slot and confirm the
    fixture becomes **Complete**, the read-only label is now **Effective winner**
    (no **Winner (derived)**), and the choice survives close/reopen and reload.
    Clear it back to **Derived from scores** and confirm the displayed winner
    follows the scores again. On a multi-stage event, confirm that letting a
    manual winner complete the last group fixture seeds the playoff exactly once.
20. **No seeding error on status toggle.** On a multi-stage event with
    **Qualifiers per group** set to 1, open a group fixture and toggle **Status**
    both ways several times. Confirm the saved status follows and that no
    message such as **At least two teams must qualify for a playoff.** or
    **Finish every group fixture before the playoff.** appears anywhere, in the
    dialog or the page. Playoff errors are reported only on **Bracket**, never
    while editing a group result.
21. **Item 11 pass follow-ups.** In a group preview, confirm there is no
    group-count selector and no **Choose the group count in the preview.** note.
    On **Groups**, confirm there is no `+` **Group** action in the heading row or
    anywhere else, and that groups can only be created through the generation
    preview. Confirm **Generate** still writes the previewed groups and
    fixtures. Delete a group or remove a team from a group on a disposable event
    and confirm focus returns to the **Preview group stage** / **Preview round
    robin** heading action. On the live Bracket tab, confirm playoff errors are
    still reported there.

**Pass:**

- Groups precede Bracket; the playoff stays locked and empty until all group
  fixtures are complete, and the visible tab label is **Bracket**.
- Match status editing offers only **Upcoming** and **Complete**, and every
  status display uses those two labels.
- Fixture rows use a pencil/details button instead of inline score/status
  editors, and group fixture results autosave without a Save button.
- After generation, entrants, groups, fixtures, rounds and placements cannot be
  added, removed or dragged in the live views. Results and match details remain
  editable.
- Every generation action opens the preview. Group-stage previews show seeding
  and group distribution only; bracket and playoff previews show seeding and
  bracket placements only, using the live traditional bracket layout.
- Preview group moves change the previewed distribution and fixtures; preview
  bracket drops swap participants only within the same section and round.
  Neither drag writes to `app.json`.
- **Cancel** closes without any saved change and returns focus to the trigger;
  **Generate** commits the previewed state in one save that survives reload.
- The playoff seeds from group standings into the correct slots on completion.
- Push to match remains an icon-only action with the exact replacement
  confirmation, is scoped to Match Controller, and leaves tournament data and
  the match library unchanged.
- Stage actions sit in one horizontal row beside the heading, with `▷` Preview
  in accent orange, `↻` Regenerate in amber and `⌫` Reset in red; icons and
  labels stay visible and reachable at all three zoom levels and no **Settings**
  dropdown remains. Group generation always creates two groups: there is no
  group-count selector in the preview and no manual **+ Group** action.
  Multi-stage playoff preview/generate is only on **Bracket**.
- Regenerate opens a preview; Reset asks for confirmation and clears only the
  selected stage, keeping entrants. Bracket seeding is editable in the preview.
- Status uses only the Upcoming/Complete switch, double-elimination lower links
  follow stored winners without crossing, upper drops are dotted, and dragging
  shows an animated indicator with a highlighted drop target.
- Bracket controls never overflow, and two adjacent matches connect directly to
  one next-round match in a centred, non-crossing traditional bracket.
- Bracket scores are inline; **Edit** and **Push** stay labelled and fully inside
  each card at all tested zoom levels; **Advanced settings** is collapsed by
  default and stacks its controls vertically.
- Tournament panels contain only brief, necessary copy; no **Traditional
  bracket** heading or group **Team** chips remain.
- Whole-app spacing is comfortable but not excessive, with no clipped text,
  unexpected wrap or control overflow at 100%, 125% or 150%. The **Tournaments**
  sidebar collapses by mouse and keyboard with correct ARIA state and focus
  retention, and saved matches and hero bans use collapsible disclosures.
- Bracket match cards are wide, with narrower gaps between adjacent matches and
  between rounds.
- **Setup** has no **Generate** card or generation button; generation starts from
  the stage heading actions.
- Group fixture details offer **First to** 1–4 and **Advanced settings → Winner**
  with **Derived from scores** and both slot options, and the manual winner is
  persisted and reported as **Effective winner**.
- Toggling a group fixture status never reports a playoff-seeding error; such
  errors appear only on **Bracket**.
- No group-count selector or group-count note appears in the preview, no manual
  **+ Group** control exists, and deleting a group or removing a team returns
  focus to the generation action.

**Fail:**

- A structural add/remove control remains after generation, or live group or
  bracket dragging changes anything.
- Any generation action edits state without opening the preview, a group-stage
  preview shows a bracket, or a bracket preview omits seeding or shows group
  distribution.
- Preview dragging does not move a team, does not resync fixtures, crosses
  rounds or sections, or writes to `app.json` before **Generate**.
- **Cancel** changes state or loses focus restoration; **Generate** commits
  anything other than the previewed result.
- Bracket is populated before groups finish, tabs are in the wrong order, or
  bracket placements do not match the qualifiers.
- A live or pending state is shown as anything other than **Upcoming**, or a
  completed state is shown as anything other than **Complete**.
- Inline score/status editors return, group results need Save, or the derived
  winner cannot be verified.
- Bracket controls overflow, the layout is top-aligned, links cross, or a pair
  of matches does not feed one next-round match.
- A score sits below the card, an action label is clipped, winner/link settings
  are visible without expanding **Advanced settings**, or Tournament copy is
  still verbose.
- Push wording suggests a tournament graphic is being pushed, Push replaces the
  current match without confirmation, Push changes bracket data or adds a
  library entry, or reload/error behavior regresses.
- Any UI element still feels condensed, clips content, or wastes space.
- A stage **Settings** dropdown returns, actions are unlabelled or uncoloured,
  the row overflows, a group-count selector or note returns, a manual **+ Group**
  control appears, or playoff preview/generate appears on **Groups** in a
  multi-stage event.
- A **Generate** card or generation button returns to **Setup**, group fixture
  details lack **First to** or **Advanced settings → Winner**, the read-only
  label is **Winner (derived)** again, a manual winner is lost on reload, or a
  seeding message appears while toggling a group fixture status.
- Navigation is obstructed by oversized sidebars or lists, or a collapsible
  section does not open, close, expose its state, or retain focus.

**Evidence:** Tab, preview-dialog, group-dialog and bracket screenshots at three
zoom levels; before/after `app.json` hashes for Cancel and Generate; saved-data
excerpts before and after group completion, including `firstTo`, `winner` and
`winnerManual`; Match Controller and overlay captures after both pushes; and the
bracket-only seeding-error screenshot.

---

### A40 — Asset properties, map modes, attached hero icons and thumbnails

**Covers:** Final asset changes

**Result: Pass — owner 2026-11-20.** All nine steps passed, including the
in-place ban-icon preview refresh in step 5 and the fully visible
**Properties…** button.

Use the current `dx build --desktop` binary. The map card has an inline
**Game mode** select. Hero cards have **Properties…**; a ban icon is uploaded
directly from that dialog and is attached to that one hero instead of becoming
another selectable Hero library entry.

**Steps:**

1. Open **Assets**. On the `Ilios` **Map** card, confirm there is an inline
   **Game mode** select with **No game mode** as the clear option. Record its
   current value. In **Match Controller → Map Pool**, confirm `Ilios` is
   grouped under that mode; in **Maps & Bans**, confirm slots named `Ilios`
   show the same mode in **Game mode**.
2. On the `Ilios` card, change **Game mode** to `Control`, then to `Escort`.
   After each change, revisit **Maps & Bans** and confirm every slot named
   `Ilios` that had not been typed by hand follows the new game mode. An empty
   derived field shows the placeholder **From map**.
3. In one `Ilios` slot, type `Custom` into **Game mode**. Change the `Ilios`
   game mode again and confirm that slot keeps `Custom` while other `Ilios`
   slots follow. Change that slot's **Map** to `Nepal`, then back to `Ilios`;
   confirm its mode returns to the current `Ilios` game mode and is no longer a
   manual override.
4. On the `Sojourn` **Hero** card, select **Properties…**. Confirm the dialog
   names Sojourn and shows **Ban icon**. Choose **Upload ban icon…** and pick a
   small local image. Confirm:
   - the dialog shows **Replace ban icon…** and **Remove ban icon**;
   - the uploaded file does **not** appear anywhere in the Assets library,
     including the **Hero** filter;
   - hero pickers and the ban-list datalist show only one `Sojourn`, not a
     second hero entry for its icon;
   - when Sojourn is banned, the game-overlay ban strip uses the uploaded image
     and `/state` `gameOverlay.heroIcons.Sojourn` resolves to its hashed URL.
5. In Sojourn's **Properties…** dialog, choose **Replace ban icon…** with a
   different local image. Without closing or reopening the dialog, confirm the
   **Ban icon** preview changes to the new image immediately and the previous
   attached icon is gone from the library and `/state`. Choose **Remove ban
   icon** and confirm the ban strip falls back to Sojourn's own hero art. Also
   confirm **Properties…** fits on the hero card with its full label visible.
6. Upload another ban icon, close the dialog and delete the `Sojourn` hero from
   **Assets**. Confirm its attached icon is deleted with it and no stale icon
   is left in the library or `/state`. Restore the QA data before continuing if
   this is a shared fixture.
7. Return to **Assets** with at least 30 images. Confirm cards show thumbnails
   or brief placeholders, then resolve without a full-page freeze. Navigate
   away and back; confirm placeholders may flash but images resolve with their
   content again.
8. Import two URLs with the same bytes. Confirm two library entries share one
   content hash and one stored blob, both thumbnails can be selected, and
   deleting one entry does not break the other thumbnail.
9. Rename one imported map and confirm its category, **Game mode** and thumbnail
   `src` are unchanged. Rename one imported hero with a ban icon and confirm its
   category, thumbnail `src` and attached icon are unchanged.

**Pass:**

- Game-mode changes are visible on the map card and refresh Map Pool and
  derived map-slot modes without overwriting manually typed modes.
- Changing a map name clears the manual mode and re-derives from the map's game
  mode.
- A hero ban icon is uploaded from **Properties…**, stays attached to that
  hero, is hidden from the library, hero pickers and ban lists, and drives the
  overlay ban strip until removed.
- Replacing or removing a ban icon cleans up the old attached image; deleting
  the hero deletes its attachments. The **Ban icon** preview refreshes in place
  while the properties dialog stays open.
- **Properties…** is fully visible on the hero card, with no truncated text.
- Thumbnails load asynchronously, reuse the content-hash cache and do not
  freeze the UI.
- Rename, delete and deduplicated-import flows remain correct.

**Fail:**

- A game-mode change leaves a derived mode stale, overwrites a manual mode, or
  fails to update Map Pool.
- A ban icon appears as an extra Hero entry, cannot be uploaded/replaced/removed
  from **Properties…**, is not used by the overlay, or survives hero deletion.
- Thumbnails never resolve, show the wrong image, or block the UI while
  loading.
- Deduplicated imports share a broken thumbnail or link state, or rename loses
  a game mode/category/thumbnail/attachment.

**Evidence:** Asset-card screenshots showing the map **Game mode** select and
the hero **Properties…** dialog, before/after Maps & Bans captures, the
hero-ban overlay capture and `/state` hero-icon URL, library/hero-picker captures
proving the attached icon is hidden, replace/remove/delete notes, asset-library
screenshots, dedupe hash/blob notes, rename notes, and the restored QA
`app.json` hash.

---

### A41 — Bundled overlays in OBS

**Covers:** M9-5, M9-6

**Steps:**

1. Start the overlay server and add all seven bundled overlays as OBS Browser
   Sources at exactly **1920×1080** with a transparent background. Confirm the
   filenames and URLs match `docs/wiki/Overlays-in-OBS.md`.
2. Load a match with two teams, logos, colours, a title, subtitle, current map,
   first-to, bans, rosters and six talent assignments. Confirm each overlay
   shows the expected data and reads clearly in the scene.
3. Check team colours on/off, linked/unlinked hero icons and linked/unlinked
   game modes on the relevant overlays.
4. Check long team, player and talent names at the 32/24-character limits.
   Confirm they shrink then ellipsize on one line and do not wrap or clip.
5. Toggle **show** off/on, stop and restart the overlay server, and refresh a
   Browser Source. Confirm each overlay reconnects and repaints correctly.
6. Watch each scene for one minute. Confirm no flicker, scrollbars, transparent-
   background artefacts, network requests outside localhost, or console errors.
   With the network disconnected, confirm Archivo still renders.
7. Retain one screenshot per overlay and note any aesthetic judgment separately
   from functional failures.

**Pass:**

- All seven overlays render correctly at 1920×1080.
- Data-driven states, team colours, hero icons and mode grouping follow the
  current match and asset links.
- Reconnect, `show`, offline fonts and long-name behaviour still work.
- No clipping, external requests or console errors appear.

**Fail:**

- Any source needs a size/position change, clips text, loses transparency or
  renders in a fallback font while offline.
- Ban icons, mode grouping or long-name fitting regress.
- Reconnect/`show` leaves a source blank, stale or flickering.

**Evidence:** Seven OBS screenshots, OBS and WebView2/CEF versions, a one-minute
watch note, reconnect/show notes and any console/network observations.

**Result: Pass — owner 2026-11-20.** Exact answer: **“Pass.”** All seven bundled
overlays passed the human OBS gate at 1920×1080: expected data, team colours,
linked/unlinked hero icons and game modes, 32/24-character long-name shrink then
ellipsis with no wrap or clip, `show` toggling, server restart and Browser Source
refresh, offline Archivo, transparency, no external requests or console errors,
and one-minute watch checks. Seven screenshots were retained. M9-5 and M9-6 are
complete; overlay appearance work remains a separate deferred session.

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

---

## 7. Post-v1 remote overlay access (D8-7)

These tests cover the remote access added after `v1.0.0`. They do not
change the v1.0.0 release gates in sections 4–6. Record results in §7 of
[`docs/manual-qa.md`](manual-qa.md).

### A42 — Local network from a second PC

**Covers:** D8-2, D8-6

**Steps:**

1. On a post-v1 build, open **Settings → Remote access** and turn on **Local
   network**. Confirm the status line names **Local network** and the revealed
   Base URL uses the host's LAN IP, not `127.0.0.1`.
2. Confirm the show adapter is **Private** and the inbound rule is **TCP**,
   **Private** profile, `RemoteAddress = LocalSubnet`, for the app executable.
3. Copy the exact `http://<host-ip>:<port>/overlays/game-overlay.html` URL from
   the revealed list and open it on a second PC on the same LAN.
4. Edit a score on the host and confirm the remote overlay updates without a
   reload.
5. Turn off **Local network** on the host, reload the second PC and confirm it
   can no longer connect. Leave **Tailscale** off for this check so the LAN
   result is unambiguous.

**Pass:**

- The second PC loads the exact generated URL and receives live updates.
- The generated URL uses the LAN IP and the configured port, with no Tailscale
  hostname.
- Firewall access stays limited to the Private profile and local subnet.
- Turning off **Local network** closes LAN access without touching Tailscale.

**Fail:**

- The second PC cannot connect with **Local network** on and the firewall
  allowed.
- The URL uses `127.0.0.1`, a wrong port or a `?port=` suffix.
- The overlay loads but does not update.
- The switch leaves the LAN listener reachable.
- The firewall rule opens Public or any remote address.

**Evidence:** Host `ipconfig`, the revealed URL list, the firewall rule, a
remote overlay screenshot before and after a host edit, and the final blocked
reload. A5 remains the owner's LAN evidence for the v1.0.0 UI.

**Result: Pass — owner 2026-11-21.** Exact answer: **“Pass.”** Host LAN IP
`192.168.50.121`, port `3001`; second PC Windows 11 with OBS 33.0.1; exact URL
`http://192.168.50.121:3001/overlays/game-overlay.html`. Notes: none. The owner
did not supply the firewall-rule capture, before/after screenshots or final
blocked-reload screenshot, so those evidence items remain gaps in the record.

---

### A43 — Tailscale on the host

**Covers:** D8-3, D8-4, D8-6

**Steps:**

1. With a connected Tailscale client, confirm Settings shows the short CLI
   version and `Connected as <dnsname> (online)`. The **Tailscale** switch is
   enabled. Funnel is not offered in the two-switch panel.
2. Turn on **Tailscale** and start the server. Confirm Settings reports that
   Serve is active at `/fgc` and `/`.
3. Reveal the URL list on **Overlay** and confirm **Settings → Remote access**
   shows the same list. Confirm all seven URLs are exactly
   `https://<dnsname>/fgc/overlays/<file>.html` in the bundled order, with no
   `?port=` and no LAN IP.
4. Run `tailscale serve status --json`. Confirm both `/fgc` and `/` point at the
   local overlay port and that an unrelated handler installed before the test
   is still present.
5. Request `https://<dnsname>/fgc/overlays/game-overlay.html` from the host and
   confirm HTTP 200. Open `wss://<dnsname>/` and confirm the WebSocket upgrade
   reaches the local server.
6. Repeat with a persisted Tailscale switch and auto-start enabled: launching
   straight into **Overlay** must still show the running server and the HTTPS
   URLs once detection returns.

**Pass:**

- Detection reports the MagicDNS name and enables the Tailscale switch only
  when connected.
- Both views show the seven exact `/fgc` URLs with no `?port=`.
- Serve contains `/fgc` and `/`, and an unrelated handler survives.
- The overlay page returns 200 and the root WebSocket opens through Serve.
- A persisted Tailscale switch auto-starts with remote URLs, not local ones.

**Fail:**

- Any URL contains a local address or `?port=`.
- Only one Serve path is installed, or an unrelated handler is removed.
- The page loads but the root WebSocket fails.
- Auto-start keeps local URLs after detection.

**Evidence:** Settings/Overlay URL excerpts, `tailscale serve status --json`,
HTTP status and WebSocket probe output, plus the QA `app.json` hash before and
after. This is a host-side check; it does not replace A44.

**Result: Pass — assistant 2026-11-21.** With CLI `1.104.7` and DNS
`sekhmet.note-hydra.ts.net`, Settings showed `Connected as sekhmet.note-hydra.ts.net
(online)`, all seven URLs matched `https://sekhmet.note-hydra.ts.net/fgc/overlays/<file>.html`
on both views, Serve paths were `/`, `/fgc` and `/other`, the overlay request
returned 200, `wss://sekhmet.note-hydra.ts.net/` opened, and the auto-start run
showed `Running · 0 clients` with the same seven URLs. The QA `app.json` hash
was restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

---

### A44 — Tailscale overlay from a second tailnet device

**Covers:** D8-1, D8-4, D8-6

**Steps:**

1. On another device signed in to the same tailnet, confirm the tailnet ACL
   allows it to reach the host on port 443.
2. Copy the exact `https://<dnsname>/fgc/overlays/game-overlay.html` URL from
   the host's Overlay page and open it on that device.
3. Confirm the overlay paints and has no `?port=` or local IP in its address.
4. Edit a score on the host and confirm the remote overlay updates without a
   reload.
5. In the browser developer tools, confirm a successful WebSocket connection to
   `wss://<dnsname>/`.

**Pass:**

- The second device loads the exact generated HTTPS URL and receives live
  updates.
- The WebSocket connects at the tailnet root.
- No local address or port override appears in the URL.

**Fail:**

- The device is blocked by ACL, DNS or certificate errors.
- The page loads but does not update, or the root WebSocket fails.
- The tester has to replace the hostname with a local IP or add `?port=`.

**Evidence:** Device and OS name, the exact URL, a remote overlay screenshot
before and after a host edit, the WebSocket status, and any ACL or certificate
error text.

**Result: Pass — owner 2026-11-21.** The owner reported: "I have tested the
Tailscale overlays on another computer in OBS, and everything seems to be
working fine for now." This is valid second-device evidence from OBS. The
device/OS, exact URL, screenshot and WebSocket log were not supplied, so the
evidence set is incomplete; host-side A43 is not a substitute for those fields.

---

### A45 — Phone on cellular

**Covers:** D8-1, D8-4, D8-6

**Steps:**

1. On a phone with Wi-Fi off and cellular data active, sign in to the same
   tailnet.
2. Open the exact `https://<dnsname>/fgc/overlays/game-overlay.html` URL from
   the host's Overlay page. Another bundled overlay is acceptable for this
   connectivity check if the exact URL is recorded.
3. Confirm the HTTPS certificate is accepted, the overlay paints at the browser
   viewport and there is no `?port=` or local IP in the address.
4. Edit a score on the host and confirm the phone overlay updates without a
   reload.
5. Confirm the connection survives switching the phone off cellular and back,
   or record the reconnection behaviour.

**Pass:**

- The overlay loads and updates over cellular through the tailnet.
- The HTTPS certificate is valid for the MagicDNS name.
- No local address or port override is needed.

**Fail:**

- The phone cannot resolve or reach the HTTPS URL on cellular.
- The certificate is rejected.
- The page loads but does not update, or needs a manual port override.

**Evidence:** Phone model/OS, Tailscale client version, exact URL, a screenshot
before and after a host edit, and any certificate or connection error text.

**Result: Pass — owner 2026-11-21.** Exact answer: **“Pass.”** Samsung S25 FE,
Tailscale client 1.104.6, cellular data active, cellular toggle Pass. Exact URL
used: `https://sekhmet.note-hydra.ts.net/fgc/overlays/map-select.html`. The
owner used **Add Map** during the check and reported that new slots were labelled
`Tiebreaker` and did not appear on `map-select.html` until the format was
changed. That is recorded as Doctrine **ISS-001** and `docs/backlog.md` §W; it
is a separate post-v1 map-count finding and does not change the A45 Pass.

**ISS-001 follow-up — assistant 2026-11-21.** The post-v1 fix labels every slot
`Map {index + 1}` and makes `map-select.html` render `maps.length` cards via
`projectedMapCardCount(maps)`, keeping the legacy extra card only for the old
seven-map/current-tiebreaker case. With `firstTo` held at 1, CDP showed
`1 map slot(s)` / `["Map 1"]` / `/state mapCount 1`, then `4 map slot(s)` /
`Map 1…4` / `mapCount 4` after three **+ Add map** clicks, then `8 map slot(s)` /
`Map 1…8` / `mapCount 8` after four more; no `Tiebreaker` label appeared. The
live overlay rendered four and then eight `.map-card` elements; eight cards at
1920×1080 occupied `x = 200…1720`, `y = 271…871`, 20px gaps, document
`1920×1080`, no overflow. `map_select_card_count_follows_projected_maps` plus
`cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
and `dx build --desktop` passed. QA `app.json`/`app.json.bak` were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` /
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`;
`tailscale serve status` was `No serve config`, and no screenshots were taken or
published. A45 remains Pass.

---

### A46 — Serve teardown leaves no app handlers

**Covers:** D8-4

**Steps:**

1. Before starting, run `tailscale serve status --json` and record every
   existing handler. Install an unrelated `/other` handler if the node is clean.
2. Turn on **Tailscale**, start the server and confirm `/fgc` and `/` appear
   alongside the unrelated handler.
3. Select **Stop server** and re-read the status. Confirm `/fgc` and `/` are
   gone and the unrelated handler remains.
4. Start again, turn off the **Tailscale** switch and confirm the same result.
   Leave **Local network** as it is so the check covers independent teardown.
5. Start once more, close the app and read the status from a fresh terminal.
   Confirm both app paths are gone and no port-wide `off` or
   `tailscale serve reset` was used.
6. On a node with no other Serve handlers, confirm `tailscale serve status`
   reports `No serve config`.

**Pass:**

- Every teardown path removes exactly `/fgc` and `/`.
- Unrelated handlers survive each teardown.
- A clean node reports `No serve config`.

**Fail:**

- Either app path remains after a teardown.
- An unrelated handler disappears.
- The result depends on `tailscale serve reset` or port-wide `off`.

**Evidence:** Status excerpts labelled before, running, stopped, switch-off,
after-exit and final, with the exact handler names.

**Result: Pass — assistant 2026-11-21.** Running state was
`RunningServePaths=/, /fgc, /other`; after **Stop server** it was
`StoppedServePaths=/other`; after app exit it was `AfterExitServePaths=/other`;
the final status was `FinalServeStatus=No serve config`. The auto-start run
showed `AutoStartServePaths=/, /fgc` and also finished at `No serve config`.
The unrelated `/other` handler survived every teardown. In the two-switch
re-run, both handlers pointed at `http://127.0.0.1:3001/`; turning off
**Tailscale** while **Local network** stayed on reported `No serve config`, and
**Stop server** also left `No serve config`.

---

### A47 — Two independent switches and no CLI console

**Covers:** D8-8

**Steps:**

1. Open **Settings → Remote access** in a post-v1 build. Confirm there are two
   switches, **Local network** and **Tailscale**, and no separate **Allow LAN
   connections** switch or **Access scope** control.
2. With **Local network** on and **Tailscale** off, turn on **Tailscale**.
   Confirm **Local network** stays on and the status reads **Local network +
   Tailscale**. Turn off **Local network** and confirm **Tailscale** stays on;
   restore both switches.
3. Start the server and reveal URLs. Confirm the generated list contains seven
   `https://<dnsname>/fgc/overlays/<file>.html` URLs and the `/fgc` base, with
   no `?port=` or LAN IP in those URLs.
4. While the server is running, select **Refresh** in Settings repeatedly and
   watch child processes of the app. Confirm every CLI call is a direct
   `tailscale.exe` child and that no `cmd.exe` or `conhost.exe` child appears.
5. Turn off **Tailscale** while **Local network** stays on and confirm
   `tailscale serve status` reports `No serve config`. Turn **Tailscale** back
   on and confirm Serve returns, then select **Stop server** and confirm Serve
   is empty again.

**Pass:**

- Exactly two independent switches are visible; stale LAN and scope controls
  are gone.
- Either switch can change without clearing the other, and the combined status
  is shown.
- Tailscale URLs use the MagicDNS name and `/fgc` with no port override.
- Tailscale CLI calls create only direct `tailscale.exe` children on Windows.
- Serve handlers are removed when Tailscale is switched off or the server is
  stopped, while Local network remains unchanged.

**Fail:**

- A stale **Allow LAN connections** switch or **Access scope** control remains.
- One switch turns off the other, or the combined status is missing.
- A Tailscale URL uses a LAN IP, loopback address or `?port=`.
- A `cmd.exe` or `conhost.exe` child appears during Refresh.
- Serve paths remain after Tailscale is switched off or the server is stopped.

**Evidence:** CDP DOM/accessibility excerpts, persisted overlay settings, the
revealed URL list, a process-watch capture during repeated Refresh calls, and
`tailscale serve status` before and after each teardown.

**Result: Pass — assistant 2026-11-21.** CDP showed switch labels including
`Local network access` and `Tailscale remote access`, with zero `Allow LAN
connections` or `Remote access scope` controls. Turning the Tailscale switch off
left Local network on with status
`Local network, server running, http://192.168.50.121:3001`; restoring it gave
`Local network + Tailscale`. Persisted settings were `allowLan: true` and
`remoteAccess: "tailscale"`. The URL list contained seven
`https://sekhmet.note-hydra.ts.net/fgc/overlays/<file>.html` entries plus the
`https://sekhmet.note-hydra.ts.net/fgc` base. A watcher during repeated Refresh
calls saw only `tailscale.exe status --json` and `tailscale.exe version` as
direct children of app PID `11468`; no `cmd.exe` or `conhost.exe` child
appeared. Serve status listed `/` and `/fgc` proxying `http://127.0.0.1:3001/`
while running and `No serve config` after switching Tailscale off and after
**Stop server**. The QA `app.json` hash was restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

---

### A48 — Dual Browser Source URL sets and collapsible sections

**Covers:** D8-9

**Steps:**

1. In a post-v1 build, turn on both **Settings → Remote access → Local network**
   and **Tailscale**. Wait until the Tailscale client reports a DNS name.
2. Open **Overlay → Browser sources** and reveal URLs. Confirm two labelled
   groups, **Local network** and **Tailscale**, and seven URLs in each.
3. Confirm the LAN group uses `http://<lan-ip>:<port>/overlays/<file>.html` and
   the Tailscale group uses `https://<dnsname>/fgc/overlays/<file>.html` with no
   `?port=`, no token and no cross-contaminated base URL.
4. Hide URLs and confirm both groups show placeholder text with no URL in the
   DOM and copy/open actions disabled.
5. Repeat on **Settings** and **Match Controller → Config**. Confirm the same
   two groups and the shared reveal state.
6. Open a secondary section such as **Settings → Display** or **Match library**.
   Confirm it starts collapsed, the summary is keyboard focusable, Enter toggles
   it and the open state survives unrelated state changes.

**Pass:**

- Both URL sets are listed when both switches are on.
- Each set uses the correct scheme, host, `/fgc` prefix and filenames.
- Hidden URLs never appear in the DOM and hidden actions cannot copy them.
- Secondary sections are native disclosures with working keyboard focus.

**Fail:**

- Only the Tailscale or only the LAN set is shown when both switches are on.
- A URL mixes a LAN IP with `/fgc`, uses `?port=` or contains a token.
- A hidden URL or enabled copy/open action remains in the DOM.
- A disclosure cannot be focused or toggled from the keyboard.

**Evidence:** CDP DOM excerpts of both URL groups, hidden-state DOM, and a
summary focus/toggle check.

**Result: Pass — assistant 2026-11-21.** With Local network already on and
Tailscale enabled temporarily for the check, CDP found `Local network` with base
`http://192.168.50.121:3001` and `Tailscale` with base
`https://sekhmet.note-hydra.ts.net/fgc`, each containing all seven overlay URLs.
The same two groups were confirmed on Overlay, Settings and Match Controller →
Config. Before reveal, both groups rendered `Base URL hidden` and `URL hidden`
with disabled copy/open buttons. A `Display` summary contained only valid
heading/phrasing children (`H2`, `SPAN`, `SPAN`), was focusable
(`tabIndex: 0`), opened on activation and stayed open while URL reveal state
changed twice. The QA `app.json` and `app.json.bak` were restored after the app
closed; `app.json` returned to hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

---

### A49 — Post-v1 disclosure affordances and compact navigation

**Covers:** IMP-001, IMP-002

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use a
   temporary `APPDATA` and WebView user-data folder so the authoritative QA
   files are never opened by the run.
2. At 1280×800, confirm the expanded shell grid, seven visible labels and no
   horizontal overflow. Toggle **Collapse navigation** by activation click,
   then by keyboard Enter with the toggle focused; confirm the `72px` icon rail,
   active indicator, names in the accessibility tree, and no overflow.
3. Hover a compact nav item at 1280 and 375, and at 1280 move focus to it with
   Tab; confirm the tooltip pseudo-element carries the destination name and
   becomes visible.
4. Open a native `details` panel by summary click and close it with Enter;
   confirm the chevron is visible in both states and that open state survives an
   unrelated compact-nav toggle. Repeat for a `Disclosure` subsection.
5. Set the viewport to 375×812 and repeat the icon-rail, label, tooltip and
   overflow checks, including the narrow-width CSS with the expanded class.

**Evidence:** CDP DOM, computed-style and accessibility-tree snapshots at
1280×800 and 375×812; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-21.** At 1280×800 the grid changed from
`248px 1032px` to `72px 1208px` on both activation click and Enter; at 375×812 it
was `72px 303px` both with the compact class and from the narrow-width media
query. `document.scrollWidth` and `body.scrollWidth` equalled `innerWidth` at
both widths. All seven nav buttons retained matching `aria-label`, `title` and
`data-label`; the accessibility tree reported all seven as non-ignored while
the text labels were clipped to `1px` / `inset(50%)`. The tooltip's `::after`
content was `"Match Controller"` with opacity `0` hidden and `1` on real pointer
hover at both widths and real keyboard focus-visible at 1280. Keyboard
activation of Settings left
`aria-current="page"` and a `3px` inset accent indicator. The panel chevron was
visible at `rotate(-90deg)` closed and `rotate(0deg)` open; Enter returned it to
closed, and an open panel stayed open across a compact-nav toggle. The
`Disclosure` chevron was visible in both states. The run used an isolated
profile, so the authoritative
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` hashes
were not disturbed.

---

### A50 — Live show/hide overlay control

**Covers:** IMP-019

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use a
   temporary `APPDATA` and WebView user-data folder so the authoritative QA
   files are never opened by the run.
2. At 1280×800, confirm **Hide overlays** is disabled while the server is
   stopped, `aria-pressed="true"`, `Live overlays: Visible`, and exposed as a
   non-ignored AX button with `pressed: true`.
3. Start the server. Confirm the toggle becomes enabled. Create a WebView target
   for `http://127.0.0.1:3001/overlays/game-overlay.html` and wait until
   `/health` reports one overlay client.
4. Click **Hide overlays** with the pointer. Confirm the UI reads `Show
   overlays`, `aria-pressed="false"`, `Live overlays: Hidden`, the AX name and
   pressed state follow, and the overlay's `.overlay` computed display and inline
   style are `none`.
5. Focus **Show overlays** and activate it with Enter. Confirm the UI returns to
   `Hide overlays`, `aria-pressed="true"`, `Live overlays: Visible`, and
   `.overlay` display returns to `block`.
6. Hide the overlays again, press **Restart**, wait for `/health` to report one
   client again, and confirm the overlay stays hidden. This proves the current
   visibility seeds the next server plan.
7. Set the viewport to 375×812. Confirm `document.scrollWidth` equals
   `innerWidth`, the toggle is fully inside the viewport with no clipped label,
   and it remains a non-ignored AX button. Click **Show overlays** and confirm
   `.overlay` returns to `block`.
8. Stop the server and confirm the toggle is disabled again. Close the overlay
   target and app, recheck the QA hashes and `tailscale serve status`, then
   delete the isolated harness.

**Evidence:** CDP DOM, computed-style, `/health` and accessibility-tree
snapshots at 1280×800 and 375×812; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-21.** At 1280×800 the stopped toggle was
`Hide overlays`, `disabled: true`, `aria-pressed: "true"`, `Live overlays:
Visible`, and the AX tree reported `pressed: true`. With the server running and
one overlay client connected, pointer activation changed `.overlay` from
`block` to computed/inline `none`, the UI to `Show overlays`, `aria-pressed:
"false"` and `Live overlays: Hidden`, with the AX name and pressed state
following. Real Enter activation restored `block` and the visible UI state.
After hiding and restarting the server, `/health` again reported one client and
`.overlay` stayed `none`. At 375×812 `scrollWidth` and `innerWidth` were both
`375`; the toggle measured `x=115`, `right=246.72`, `width=131.72`, with
`clientWidth == scrollWidth` and a non-ignored AX pressed state. Pointer
activation there also restored `block`, and stopping the server disabled the
toggle. The isolated `app.json` and `app.json.bak` stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` through
shutdown; the authoritative hashes and `No serve config` were rechecked after
the harness was removed.

---

### A51 — Overlay health and reconnect indicator

**Covers:** IMP-021

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use an
   isolated `APPDATA` and WebView user-data folder so the authoritative QA files
   are never opened by the run.
2. At 1280×800 with the server stopped, confirm **Connected clients: 0 clients**
   and **Last broadcast: Never**, and that no reconnect note is present.
3. Start the server. Confirm `Running · 0 clients`, **Last broadcast: Just now**,
   `/health {"overlays":0,...}`, and a reconnect note with `role="status"` that
   is non-ignored in the accessibility tree.
4. Create a WebView target for
   `http://127.0.0.1:3001/overlays/game-overlay.html`. Wait until `/health`
   reports `overlays:1`; confirm the UI reads `1 client` and the reconnect note
   is gone.
5. Click **Hide overlays** to send a new frame. Confirm **Last broadcast**
   returns to `Just now`, then wait for the one-second clock and confirm it
   advances to `1s ago`.
6. Press **Restart**. Confirm the brief zero-client state returns the reconnect
   note and reseeds **Last broadcast: Just now**, then the overlay reconnects to
   `1 client` and clears the note. Close the overlay target and confirm
   `0 clients`, the note again, and a broadcast age that keeps increasing.
7. Set the viewport to 375×812. Confirm `document.scrollWidth == innerWidth`,
   that the new rows and note stay inside the viewport, and that their labels and
   the note remain non-ignored in the accessibility tree. If the floating timer
   covers Stop/Restart at the initial scroll position, scroll normally before
   using those controls.
8. Stop the server. Confirm **Last broadcast: Never**, the reconnect note is
   removed and the overlay toggle is disabled. Close the overlay target and app,
   recheck the QA hashes and `tailscale serve status`, then delete the isolated
   harness.

**Evidence:** CDP DOM, accessibility-tree, one-second clock and `/health`
snapshots at 1280×800 and 375×812; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-21.** The run used WebView2 158.0.4370.33.
While stopped, the panel showed `0 clients` and `Never` with no note. Starting
with no clients showed `Running · 0 clients`, `Just now`, `/health
{"overlays":0,...}` and a non-ignored `role="status"` note. An overlay target
moved `/health` to `overlays:1`, changed the UI to `1 client` and cleared the
note. Hiding overlays reseeded `Just now`; the clock then advanced to `1s ago`.
Restart briefly returned to `0 clients` with the note and `Just now` before the
overlay reconnected; closing it returned to `0 clients`, restored the note and
left the age increasing. Stopping cleared the value to `Never`, removed the note
and disabled the toggle. At both 1280×800 and 375×812, `scrollWidth` equalled
`innerWidth`; the rows and note stayed inside the viewport and the accessibility
tree exposed `Connected clients`, `Last broadcast` and the note. At 375×812 the
pre-existing floating timer initially covered Stop/Restart; after a normal wheel
scroll they were hit-testable and Stop worked. The isolated and authoritative
`app.json`/`app.json.bak` hashes stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

---

### A52 — Auto-overlay scene switcher and controller hotkeys

**Covers:** IDE-004, IMP-022 (B4 merged into IDE-004)

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use an
   isolated `APPDATA` and WebView user-data folder copied from the QA fixture
   so the authoritative `app.json` files are never opened by the run.
2. At 1280×800, confirm the Overlay view's controller starts with only
   `game-overlay.html` included and active; `/state` reports
   `autoOverlay: {included:["game-overlay.html"],active:"game-overlay.html"}`.
   Start the server and open one Browser Source target for
   `http://127.0.0.1:3001/overlays/auto.html`. Confirm the page connects,
   `/health` moves from `0` to `2` clients (auto page plus its iframe), and only
   the active frame has `data-visible="true"`.
3. Include `map-select.html` and confirm its hidden iframe is added. Focus the
   controller container, press `2`, and confirm the active scene, radio, status
   and visible frame all follow. Press `4` while that file is excluded and
   confirm no state or frame changes. Use `ArrowRight` and `ArrowLeft` and
   confirm wrapping through the included subset. Focus a control outside the
   controller and press `1`; confirm the scene does not change.
4. Uncheck the active scene and confirm the active scene falls back to the
   first remaining included file. Uncheck every scene and confirm no frame is
   visible and the status reads `Active scene: none — include at least one
   overlay`. Verify the status has `role="status"` and that the AX tree exposes
   the controller, group, checkbox, radio and status names. Real-click the
   native `details` summary and confirm the shortcut text opens.
5. Include `hero-bans.html` and `row-casters.html`, press `7` with the
   controller focused, then press **Restart**. Confirm the server reseeds the
   same session-only selection, `/health` briefly drops to `0` and returns to
   `5`, the UI reads `Running · 5 clients`, and the auto page again shows only
   `row-casters.html`.
6. Confirm the auto Browser Source row renders `URL hidden` before reveal and
   `http://192.168.50.121:3001/overlays/auto.html` after **Show URLs**, and
   that the count reads `8 sources`.
7. Check 1280×800 and 375×812. Confirm `document.scrollWidth == innerWidth`,
   the scene grid is two columns at 1280 and one at 375, and the controller and
   status stay inside the viewport. Confirm the only horizontal offender at
   375 is the pre-existing debug hot-reload toast.
8. Stop the server and app, recheck the authoritative hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM, computed-style, accessibility-tree, `role="status"`,
`/state`, `/health` and real keyboard/pointer snapshots at 1280×800 and
375×812; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-21.** The run used WebView2 158.0.4370.33
and app PID 38348. The default seed, connect, fixed-position hotkey `2`,
excluded `4` no-op, arrow wrapping, outside-controller no-op, active-scene
fallback, empty subset, native `details` and labelled AX nodes all matched the
design. Restart reseeded `hero-bans.html` and `row-casters.html` with
`row-casters.html` active while `/health` went `5` → `0` → `5` and the UI showed
`Running · 5 clients`. The hidden auto URL, eight-source count and responsive
layout were verified at both widths: `scrollWidth == innerWidth == 1280` with
grid `589.328px 294.656px`, and `scrollWidth == innerWidth == 375` with scene
and URL grids `207px`. Two real defects found during the run were fixed and
re-verified before this result: removing the active scene left no active scene
despite remaining included files, and excluded frames could retain
`data-visible="true"`. The isolated and authoritative
`app.json`/`app.json.bak` hashes stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and no screenshots were taken.

---

### A53 — Six-caster social-row containment

**Covers:** ISS-018, M9-7 (post-v1 overlay polish)

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use an
   isolated `APPDATA` and WebView user-data folder copied from the QA fixture so
   the authoritative `app.json` files are never opened by the run.
2. Start the server and open `http://127.0.0.1:3001/overlays/row-casters.html`
   as a separate CDP target. Set the viewport to 1920×1080. The harness has no
   caster projection in the server cache, so clear the page WebSocket's
   `onclose`/`onerror` handlers, close it, and inject the six-caster payload
   directly with
   `updateOverlay({ type: 'casters', data: { castersData: [...] } })`. Await
   `document.fonts.ready` before measuring.
3. Reproduce the baseline overflow first: with six cards,
   `@anextremelylongcastersocialhandle` crossed the card's right edge by
   `26.75px` and its left edge by `96.92px`; `@longcasterhandlefive` crossed the
   left edge by `23.67px`. This confirms the M9-7 report before changing CSS.
4. Apply the scoped fix: constrain `.talent-subtext` to the card content box,
   keep `.social-icon` at `flex: 0 0 20px`, and let `.social-handle` and
   `.pronouns` use `min-width: 0`, `overflow: hidden`, `text-overflow: ellipsis`
   and `white-space: nowrap`. Render the full handle in `title` on the handle
   span. Rebuild, relaunch, and replay the 1-, 4- and 6-caster payloads.
5. For every card, record DOM boxes, `clientWidth`/`scrollWidth`, `title` and
   computed styles for the card, content box, `.talent-subtext`, icon, handle,
   separator and pronouns. Confirm every child stays inside the card content
   box, only overlong text ellipsizes, full handles remain in `title`, and the
   1920×1080 document size and card widths are unchanged.
6. Stop the app, recheck the authoritative hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON at 1920×1080 for the 1-, 4- and
6-caster states, plus the pre-fix six-caster baseline; no screenshots were taken
or published.

**Result: Pass — assistant 2026-11-21.** The run used WebView2 158.0.4370.33
and an isolated QA profile. At 1920×1080 every state had
`scrollWidth == innerWidth == 1920` and `scrollHeight == innerHeight == 1080`.
The six-caster row kept six 270px cards inside the 1720px container; each
250px content box contained every social-row child, with the long-handle cards
measuring `outsideRight: 0` at worst. `@longcasterhandlefive` ellipsized at
142/240px and `@anextremelylongcastersocialhandle` at 159/394px; their pronouns
ellipsized at 47/79px and 29/72px. Each handle's `title` retained the full
handle. The four-caster row kept 415px cards and ellipsized only the extreme
handle (281/394px); the one-caster row kept a 900px card and showed the extreme
handle in full (394/394px). Computed styles were
`.talent-subtext { display:flex; gap:12px; max-width:100%; min-width:0 }`,
`.social-icon { flex:0 0 20px }` and
`.social-handle, .pronouns { overflow:hidden; text-overflow:ellipsis; white-space:nowrap }`.
Zero page console errors were recorded. The authoritative
`app.json`/`app.json.bak` hashes stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

---

### A54 — Placeholder TAG image for empty logo slots

**Covers:** IMP-032, M9-10 (post-v1 follow-up from A9 / M3-6)

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use an
   isolated `APPDATA` and WebView user-data folder copied from the QA fixture so
   the authoritative `app.json` files are never opened by the run.
2. Start the server and open
   `http://127.0.0.1:3001/overlays/game-overlay.html` as a separate CDP target.
   Set the viewport to 1920×1080, await `document.fonts.ready`, and first
   measure the live `init` payload from the untouched fixture without injecting
   data. The fixture's empty logo slots should show its stored tags.
3. Replay direct `scoreboard` frames to cover the remaining states: empty tags,
   a real `logoUrl`, a long tag, blank tags with Latin and CJK names, `null`
   teams, a partial frame with no team fields, and light/dark `logoBgColor`
   with team colours enabled. Record text, `title`, `clientWidth`/`scrollWidth`,
   DOM boxes, logo `backgroundImage`, computed styles and computed text colour.
4. Confirm placeholders appear only for a team with no logo, use the tag first
   and fall back to a name abbreviation, ellipsize overlong text while keeping
   the full value in `title`, clear for `null` teams, survive partial frames,
   and hide as soon as a real logo is supplied. Confirm the 75×55 logo geometry,
   63×43 content box and 1920×1080 document are unchanged.
5. Stop the app, recheck the authoritative hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON at 1920×1080 for the live `init`
state and seven replayed states; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-21.** The run used WebView2 158.0.4370.33
and an isolated QA profile. Every state had `scrollWidth == innerWidth == 1920`
and `scrollHeight == innerHeight == 1080`, with
`document.fonts.check('600 24px "FGC Display SemiBold"')` true. Live `init`
painted `T3`/`T1` in the empty slots and, because the fixture uses team colours
with `#30363d` logo backgrounds, computed to `rgb(255,255,255)`. Replayed frames
kept the same tags through a partial frame, hid the home placeholder and applied
a real logo URL while the away tag stayed, ellipsized `LONGCASTERTAG` at
63/227px with the full tag in `title`, fell back to `NS` and `王小` for blank
tags, cleared both placeholders for `null` teams, and produced white text on
`#20242b` versus dark text on `#f5f5f5`. Computed styles were
`display:block`, `font:600 24px "FGC Display SemiBold"`, `max-width:100%`,
`overflow:hidden`, `text-overflow:ellipsis`, `white-space:nowrap`; every visible
placeholder stayed inside the 63×43 logo content box. Zero page console errors
were recorded. The authoritative `app.json`/`app.json.bak` hashes stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

### A55 — Standalone bundled timer overlay

**Covers:** IMP-023, B5, M9-11

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Use an
   isolated `APPDATA` and WebView user-data folder copied from the QA fixture.
   The fixture's timer mode is `target` with no target, so set the in-app timer
   mode to `countup` through the CDP-driven UI before **Start timer**.
2. Start the server and open `http://127.0.0.1:3001/overlays/timer.html` as a
   separate CDP target at 1920×1080. Await `document.fonts.ready`, then record
   the default text, computed styles, DOM box, background colour and document
   size before any tick.
3. Use the real app controls for **Start timer**, **Pause timer** and
   **Stop timer**, sampling the overlay text and the in-app clock after each
   transition. Confirm a running tick advances, a pause holds the last value,
   and stop resets to `00:00:00`.
4. While the timer is running, change **Leading zeroes** and **Alignment**
   through the app UI. Wait for the next one-second tick after each setting
   change; settings are carried in the tick, not broadcast immediately.
   Confirm unpadded text and left/right placement, then restore centre and
   padded settings.
5. Use **Hide overlays** / **Show overlays** and confirm the timer root changes
   between `display: none` and `display: flex`. Record all network requests,
   failed HTTP responses and console/log errors for the overlay target.
6. Stop the app, archive any state the run wrote, restore the authoritative
   `app.json`/`app.json.bak` fixture, recheck their hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON at 1920×1080 for fourteen states,
the in-app clock samples, network/console capture and `/health`; no screenshots
were taken or published.

**Result: Pass — assistant 2026-11-21.** The run used WebView2 158.0.4370.33.
The default state was `00:00:00`, count-up reached `00:00:02`, pause held
`00:00:02` across a 1.4 s wait, stop reset to `00:00:00`, leading-zeroes off
rendered `0:0:4`, alignment left placed the text box at x=96 and right at
x=1422.55, and restored settings returned to centred `00:00:08`. `show` false
set `display: none` on `#timer-root`; `show` true restored `display: flex`.
Every state had `scrollWidth == innerWidth == 1920`,
`scrollHeight == innerHeight == 1080`, body background `rgba(0, 0, 0, 0)`, and
`document.fonts.check('600 160px "FGC Display"')` true. `externalRequests`,
`failedResponses` and `consoleErrors` were all empty after adding a local
`data:` favicon. The app resolved its data directory through Windows Known
Folders despite the isolated `APPDATA`, so the QA-mutated state was archived and
the authoritative files restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`.
`tailscale serve status` was `No serve config`, and the harness was deleted.

### A56 — Timer hour-display setting

**Covers:** owner request 2026-11-21, `showHours`

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. The app
   resolves state through Windows Known Folders, so restore the authoritative
   `app.json`/`app.json.bak` fixture before the run and again afterwards rather
   than relying on an isolated `APPDATA`.
2. Start the overlay server through the real app UI. Open
   `http://127.0.0.1:3001/overlays/timer.html` as a CDP target, set a 1920×1080
   device metrics override, reload, await `document.fonts.ready`, and record the
   text, computed styles, document size and background.
3. In **Timer settings**, select **Count down** and set 1 hour, 12 minutes and
   0 seconds. Confirm the overlay shows `01:12:00`. Toggle **Show hours** off;
   the overlay must change to `72:00` without waiting for another clock tick.
   Toggle it back on and off again to confirm both directions and that the final
   state is hidden.
4. After the 500 ms save window, read `/state` and `app.json` and confirm
   `matchTimer.settings.showHours` and `settings.timer.showHours` are both
   `false`.
5. Start the timer and wait for one tick. Confirm `71:59` (1 hour 11 minutes
   59 seconds), then pause and confirm the overlay holds that value across a
   1.4 s wait. Stop the timer and confirm the reset frame still renders `72:00`
   using the retained settings.
6. Use **Hide overlays** / **Show overlays** and confirm `#timer-root` changes
   between `display: none` and `display: flex`. Record all overlay requests,
   failed responses and console/log errors. Stop the app, restore the
   authoritative fixture, recheck its hashes and `tailscale serve status`, then
   delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON for eleven states, `/state`, the
saved `app.json`, network/console capture and `/health`; no screenshots were
taken or published.

**Result: Pass — assistant 2026-11-21.** WebView2 158.0.4370.33 drove the real
app controls and overlay page. The countdown rendered `01:12:00`; hiding hours
changed it immediately to `72:00`, showing restored `01:12:00`, and hiding again
returned `72:00`. Both `/state` and `app.json` reported `showHours: false`. A
running tick rendered `71:59`, pause held it across 1.4 s, and Stop returned
`72:00`. `show` toggled `display` between `none` and `flex`. The value computed
to `rgb(255, 255, 255)`, `"FGC Display", Arial, sans-serif`, `160px` and
`tabular-nums`; the body background was `rgba(0, 0, 0, 0)`, the document was
1920×1080 with no overflow, and the bundled font was loaded. Only the local
overlay HTML and font were requested; `externalRequests`, `failedResponses` and
`consoleErrors` were all empty. Authoritative hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

### A57 — Game overlay Night Broadcast pilot

**Covers:** C1, IDE-018 §C1, `game-overlay.html`

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. The app
   resolves state through Windows Known Folders, so restore the authoritative
   `app.json`/`app.json.bak` fixture before the run and again afterwards rather
   than relying on an isolated `APPDATA`.
2. Start the overlay server through the real app UI. Open
   `http://127.0.0.1:3001/overlays/game-overlay.html` as a CDP target, set a
   1920×1080 device metrics override, reload, await `document.fonts.ready`, and
   record the computed styles of both team bars, their names, the score chips,
   the title and mode bars, the match bug, the map number and the format text.
3. With team colours on, confirm each bar uses the team colour as its surface,
   the name is light, and the score chip is inverse. Then turn **Use team
   colours** off in Settings and confirm the bars fall back to the neutral dark
   surface with a light name, while the score chip stays inverse. Turn the
   setting back on and confirm the coloured state returns.
4. Check the empty-logo placeholder in both states: the tag text must contrast
   with its slot, on the dark team slot when coloured and on the light default
   slot when not.
5. Confirm the canvas is transparent, the document is exactly 1920×1080 with no
   overflow, the bundled Archivo faces are loaded, and the retired palette is
   gone (`#fff`, `#ffffff`, `#090909`, `#eeeeee`, `#151515`, `#ac46f7`).
6. Record every overlay request, failed response and console/log error. Stop the
   app, restore the authoritative fixture, recheck its hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON for the three colour states,
network/console capture and `/health`; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and overlay page. Team bars measured 525×55 at left 43 and 1348,
top 52, radius 6px, with a 1px inset line `rgb(42, 49, 64)`. The coloured bar
was `rgb(48, 54, 61)` with a white name and the neutral bar `rgb(11, 13, 18)`
with a `rgb(238, 242, 248)` name. The score chip was inverse in both states
(`rgb(238, 242, 248)` background, `rgb(11, 13, 18)` text, `tabular-nums`). The
placeholder read `T3` in `rgb(238, 242, 248)` on the dark `#30363d` slot when
coloured and `rgb(11, 13, 18)` on the light `#eef2f8` slot when not. The title
and mode bars were `rgb(11, 13, 18)`, the match bug `rgb(17, 21, 29)` at 278×50,
the map number `rgb(238, 242, 248)` and the format text `rgb(255, 122, 26)`. The
canvas was `rgba(0, 0, 0, 0)` at 1920×1080 with no overflow, the Archivo 400 and
600 faces loaded, and `externalRequests`, `failedResponses` and `consoleErrors`
were all empty. A neutral-state score-chip contrast defect found in the first
run — the default text-colour rule still listed `.team-bar__score` and outranked
the inverse colour — was fixed and re-verified. The active-mode orange segment
could not be exercised because the fixture has `currentMap: null`, so
`current_map_index` serialises as `null` and the mode item stays inactive grey;
that is pre-existing behaviour, and the orange accent is proven by the format
text and the static test. `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`, `git diff --check` and
`dx build --desktop` passed. Authoritative hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

### A58 — Seven-overlay Night Broadcast rollout

**Covers:** C1, IDE-018 §C1, `hero-bans.html`, `map-pool.html`,
`map-select.html`, `row-casters.html`, `team-roster-home.html`,
`team-roster-away.html`, `timer.html`

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. The app
   resolves state through Windows Known Folders, so restore the authoritative
   `app.json`/`app.json.bak` fixture before the run and again afterwards rather
   than relying on an isolated `APPDATA`.
2. Swap in the enriched fixture with the team colours on, start the overlay
   server through the real app UI, and wait until `/state` holds every section
   including `autoOverlay` before opening any overlay target.
3. For each overlay, open it as a CDP target, set a 1920×1080 device metrics
   override, reload, await `document.fonts.ready`, and dump every element with
   its computed style, geometry and text. Compare the dump against the
   pre-rollout baseline: geometry diffs, removed elements, added elements, font
   regressions, non-local requests, failed responses, bad statuses and console
   errors must all be zero.
4. Repeat with the no-colour fixture (team colours off) and assert the exact
   Night Broadcast values per element: surface `rgb(11, 13, 18)`, raised
   `rgb(17, 21, 29)`, muted `rgb(21, 25, 34)`, inverse `rgb(238, 242, 248)`,
   accent `rgb(255, 122, 26)`, text `rgb(238, 242, 248)` and muted text
   `rgb(141, 153, 173)`.
5. The row-casters social icons are inline SVG paths, and
   `getComputedStyle(path).fill` is not exposed in this Chromium build. Resolve
   the painted fill through the CDP `CSS.getComputedStyleForNode` domain
   (`DOM.getDocument` → `DOM.querySelector` → `CSS.getComputedStyleForNode`) and
   confirm it is `rgb(141, 153, 173)` for all five paths. That call needs the
   node to exist, so poll until the websocket `init` frame has populated the
   casters.
6. Stop the app, restore the authoritative fixture, recheck its hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** per-overlay DOM and computed-style dumps with the baseline
comparison report, the neutral-state colour report, the CDP resolved-fill
capture, network/console capture and `/health`; no screenshots were taken or
published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and overlay pages. `hero-bans`, `map-pool`, `map-select`,
`row-casters`, `team-roster-home`, `team-roster-away` and `timer` each reported
0 geometry diffs, 0 removed, 0 added, 0 font regressions, 0 non-local requests,
0 failed, 0 bad status and 0 console errors, with body background
`rgba(0, 0, 0, 0)` and a 1920×1080 document. `game-overlay` reported differences
only because its baseline predates the shipped pilot: all 14 geometry diffs are
the 1px horizontal shift (`x: -32 → -33`, `1423 → 1424`) from the pilot's 1px
inset border, plus one colour difference from the tournament-logo background
image; `game-overlay.html` is not part of this rollout. The neutral-state run
was `allOk: true` for `team-roster-home`, `team-roster-away`, `game-overlay`,
`hero-bans` and `row-casters` with the exact Night Broadcast values listed
above, a transparent canvas, 1920×1080, no console errors and no non-local
requests. The five row-casters social-icon paths resolved to
`rgb(141, 153, 173)` with the rule `.social-icon path { fill: var(--fgc-text-muted); }`
present — the same colour as the spec's literal `fill="#8d99ad"`, recorded as
the one deviation in `docs/overlay-refresh-specs.md` §4.9. Static guards were
added to `crates/server/tests/overlays.rs`. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes were
restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` after
every run, `tailscale serve status` was `No serve config`, and the harness was
deleted.

### A59 — `auto.html` scene switcher and favicon fix

**Covers:** C1, IMP-004, `auto.html`

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` and the
   isolated WebView profile, restoring the authoritative fixture first.
2. Start the overlay server through the real app UI, wait for the projection
   cache to fill, then open `http://127.0.0.1:3001/overlays/auto.html` as a CDP
   target with a 1920×1080 device metrics override.
3. Record the `iframe.fgc-auto__scene` elements: title, `src`, `data-visible`,
   computed `display` and `position`, geometry, inner text and inner
   background, plus the body background and document size.
4. Drive the real app UI: tick **Include map-select.html in the auto scene
   switcher**, confirm the new iframe stays hidden until it is made the active
   scene, then pick **Make map-select.html the active scene** and confirm the
   visible scene moves and the app reports the new active scene.
5. Use **Hide overlays** and **Show overlays** and confirm every scene toggles
   between `display: none` and `display: block`.
6. Untick the include box and confirm the scene stays in the DOM but hidden and
   the previous active scene returns — the host never removes an existing iframe.
7. Record every request, failed response, bad status and console/log error.
   Stop the app, restore the authoritative fixture, recheck its hashes and
   `tailscale serve status`, then delete the isolated harness.

**Evidence:** CDP DOM and computed-style JSON for the scene host in every
state, the `/state` `autoOverlay` payload, network/console capture and `/health`;
no screenshots were taken or published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. One iframe was present at start
(`game-overlay.html`, `data-visible=true`, `display:block`, 1920×1080).
Including `map-select.html` added a second iframe that stayed hidden until it
was made active; the active switch moved the visible scene, the app reported
`Active scene: map-select.html`, and the map-select canvas showed
"MAP SELECT … POWERED BY THEPEACHZA". **Hide overlays** hid every scene and
**Show overlays** restored `map-select.html` to `display:block` with
`game-overlay.html` at `display:none`. Excluding `map-select.html` left its
iframe in the DOM but hidden and returned `game-overlay.html` to active. The
body background was `rgba(0, 0, 0, 0)` with a 1920×1080 document, and
`/state.autoOverlay` was `{included:["game-overlay.html"], active:"game-overlay.html"}`.

**Defect found and fixed.** The first run requested
`http://127.0.0.1:3001/favicon.ico` and logged a 404 plus one console error,
because `crates/server/overlays/auto.html` was the only bundled overlay without
`<link rel="icon" href="data:,">`. The link was added after the viewport meta
tag and guarded by the static test `auto_overlay_suppresses_the_favicon_request`
in `crates/server/tests/auto_overlay.rs`. The re-run made no favicon request
and recorded no external requests, failed responses or console errors.
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `git diff --check` and `dx build --desktop` passed.
Authoritative hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`, and the harness was deleted.

### A60 — Talent overlay split

**Covers:** owner review list 2026-11-22 (slice A), `row-casters.html`,
`row-hosts.html`, `row-interviews.html`, the `casters` payload

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Stop any
   running `fruiterra-gc` first: the debug binary is locked while the app is
   open and the build fails. The app resolves state through Windows Known
   Folders, so restore the authoritative `app.json`/`app.json.bak` fixture
   before the run and again afterwards rather than relying on an isolated
   `APPDATA`.
2. Build a QA fixture from the authoritative state that assigns every on-air
   role: caster1 and caster2 as `caster`, panel1 as `analyst`, panel2 as
   `host`, interviewer as `interviewer`, plus an interviewee. Give the interviewee
   a portrait so the guest card can be checked.
3. Start the overlay server through the real app UI and wait until `/state`
   holds `autoOverlay, casters, gameOverlay, heroBans, mapSelect, scoreboard,
   teamRoster`. `matchTimer` is only projected once the timer ticks, so it must
   not be required.
4. Open each of `row-casters.html`, `row-hosts.html` and `row-interviews.html`
   as a CDP target, set a 1920×1080 device metrics override, reload and wait for
   the cards to render. Record the rendered names, the role tags, the guest tag
   and the guest card count.
5. Assert that no overlay renders a person outside its own role group, that
   every portrait is present, that the body background is transparent, that the
   document is 1920×1080, that the bundled font is loaded, and that the
   requests, failed responses, bad statuses and console errors are all empty.
6. Read `/state` and confirm the `casters` payload carries `interviewee` with the
   guest name, subtext and portrait, and that the five assignment roles arrive
   on the wire in order. Stop the app, restore the authoritative fixture,
   recheck its hashes and `tailscale serve status`, then delete the harness.

**Evidence:** CDP DOM and computed-style JSON for the three overlays, the
`/state` wire payload, the persisted `app.json` roles, network/console capture
and `/health`; no screenshots were taken or published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay pages. `row-casters.html` rendered exactly
`["Arc","Priya"]` with no role tag and no guest card. `row-hosts.html` rendered
`["Marco","Sam"]` with the role tags `["Analyst","Host"]`.
`row-interviews.html` rendered `["Jordan","Robin"]` with one `Guest` tag and one
guest card. No overlay rendered a person outside its own role group. Every
overlay had a transparent `rgba(0, 0, 0, 0)` body, a 1920×1080 document, its
bundled font loaded and its portraits present, and recorded 0 external requests,
0 failed responses, 0 bad statuses and 0 console errors. The `/state` wire
payload carried `interviewee` with the guest name, subtext and portrait, and the
five assignment roles arrived as `caster,caster,analyst,host,interviewer`.
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `git diff --check` and `dx build --desktop` passed.
Authoritative hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`,
`tailscale serve status` was `No serve config`. The temporary QA harness lives
outside the repository and is removed when the overlay review work completes.

**Fixture note.** The authoritative fixture's stored Arc portrait asset has no
blob in the asset store, so that portrait 404s. This is a data condition in the
fixture, not a code defect; the QA fixture points Arc at an asset whose blob is
present.

### A61 — Flexible roster cards

**Covers:** owner review list 2026-11-22 (slice B), `team-roster-home.html`,
`team-roster-away.html`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture whose match teams carry a chosen number of core players
   (`position: "core"`) and substitutes (`position: "sub"`). Copy the source
   team object verbatim when cloning it: `logoBackground` is a boolean, not a
   colour string, and a wrong type makes the whole team fail to load.
3. Start the overlay server through the real app UI and wait until `/state`
   holds `autoOverlay, casters, gameOverlay, heroBans, mapSelect, scoreboard,
   teamRoster`.
4. Open `team-roster-home.html` and `team-roster-away.html` as CDP targets with
   a 1920×1080 device metrics override, reload and wait for the cards. Record
   the core row width and gap, every card width and left offset, the substitute
   card count and widths, the body background, the document size and the font
   state.
5. Assert that the core card count matches the fixture, that all core cards
   share one width to within a sub-pixel, that the last card ends on the row's
   right edge, that the substitutes keep their fixed 284px width, that the
   document is 1920×1080 with no overflow, that the body is transparent, that
   the font is loaded, and that the requests, failed responses, bad statuses and
   console errors are all empty.
6. Repeat for a short roster and a long roster. Stop the app, restore the
   authoritative fixture and recheck its hashes.

**Evidence:** CDP DOM and computed-style JSON for both roster overlays in all
three roster sizes, network/console capture and `/health`; no screenshots were
taken or published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay pages. The core row measured 1500px with a 20px gap
in every case and the cards filled it to the right edge: five cards at 284px,
three at 486.66px and six at 233.33px, with a sub-pixel spread of 0.01px
between the widest and narrowest card. The substitute strip kept its fixed
284px cards (two, one and three respectively) and stayed right-aligned. Every
overlay had a transparent `rgba(0, 0, 0, 0)` body, a 1920×1080 document with no
overflow, its bundled font loaded, and recorded 0 external requests, 0 failed
responses, 0 bad statuses and 0 console errors. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` after
every run, and `tailscale serve status` was `No serve config`. The temporary QA
harness lives outside the repository and is removed when the overlay review work
completes.

### A62 — Map pool stagger and picked-map glow

**Covers:** owner review list 2026-11-22 (slice C), owner glow request
2026-11-22, `map-pool.html`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture whose match has a map pool and one picked map
   (`currentMatch.maps[0] = { name, gameModeName, chosenBy }`).
3. Start the overlay server through the real app UI and wait until `/state`
   holds `autoOverlay, casters, gameOverlay, heroBans, mapSelect, scoreboard,
   teamRoster`.
4. Open `map-pool.html` as a CDP target with a 1920×1080 device metrics
   override, reload and wait for the columns. Sample the column opacities every
   ~80ms from the first frame so the reveal timeline can be measured.
5. Record the column count and mode names, each column's `columnReveal`
   animation delay, the first sample at which each column became visible, the
   picked map's `filter`, its `.map-pick-glow` background and box, its name
   band's `box-shadow` and background, and the paint order of the logo against
   the glow.
6. Assert that the delays strictly increase by the stagger step, that the
   columns are revealed one at a time, that the picked map keeps its greyscale
   and gains the gradient wash and accent rule, that the logo paints above the
   glow, that the container is visible, that the document is 1920×1080, that the
   body is transparent, that the font is loaded, and that the requests, failed
   responses, bad statuses and console errors are all empty.

**Gotchas:** `document.elementsFromPoint` skips elements with
`pointer-events: none`, so the paint-order probe must temporarily set
`pointerEvents = 'auto'` on the glow and the logo overlay and restore it
afterwards. The CDP websocket keeps the Node event loop alive, so the QA script
must call `process.exit(0)` when it is done. The fixture enables LAN access, so
asset requests resolve against the LAN base URL — the "external request" filter
must accept the app's own address on port 3001.

**Evidence:** CDP DOM and computed-style JSON for the pool, the opacity
timeline, network/console capture and `/health`; no screenshots were taken or
published.

**Result: Pass — assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. The pool rendered four mode columns
(`Control`, `Escort`, `Hybrid`, `Push`), each with cards, and the columns became
visible at 1611, 1798, 1891 and 1985ms — about 190ms apart — with
`columnReveal` delays of 1100, 1230, 1360 and 1490ms; a mid-reveal sample
showed the first column at opacity 0.237 while the later three were still at 0.
The picked map kept `filter: grayscale(1)` under a bottom-weighted
`linear-gradient(to top, rgba(255, 122, 26, 0.45) 0%, rgba(255, 122, 26, 0.22)
30%, rgba(255, 122, 26, 0.08) 52%, rgba(255, 122, 26, 0) 72%)` glow covering
the 361.25×657 art box, its name band resolved to `inset 0 2px 0 rgb(255, 122,
26)` over `linear-gradient(to top, rgba(255, 122, 26, 0.08) 0%, rgba(255, 122,
26, 0) 100%)`, and the team logo probed as painting above the glow. The
overlay had a transparent `rgba(0, 0, 0, 0)` body, a 1920×1080 document, its
bundled font loaded, and recorded 0 external requests, 0 failed responses, 0 bad
statuses and 0 console errors. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` after
every run, and `tailscale serve status` was `No serve config`. The temporary QA
harness lives outside the repository and is removed when the overlay review work
completes.

### A63 - Map select ban thumbnails

**Covers:** owner review list 2026-11-22 (slice D), `map-select.html`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture whose match names four maps (Ilios, Nepal, King's Row,
   Dorado), marks Dorado completed and fills the first three hero-ban slots:
   `{ firstBanTeam: 'A', teamA: ['Ana'], teamB: ['Kiriko'] }`,
   `{ firstBanTeam: 'B', teamA: ['Tracer'], teamB: ['Winston'] }` and
   `{ firstBanTeam: 'A', teamA: ['Sojourn'], teamB: ['Lucio'] }`. Leave
   `settings.display.showHeroBans` on so the bans are projected.
3. Start the overlay server through the real app UI and wait until `/state`
   holds `autoOverlay, casters, gameOverlay, heroBans, mapSelect, scoreboard,
   teamRoster`.
4. Open `map-select.html` as a CDP target with a 1920×1080 device metrics
   override, reload, wait for the map cards and let the reveal settle.
5. Record the card count and names, and for every card the ban count, the
   diamond numbers, the thumbnail sources, whether the portraits loaded, the
   computed `filter` of each portrait, the badge and label transforms, the
   strip and image boxes, and whether the strip sits inside `.map-card__visual`
   or inside `.map-card__image`.
6. Assert that the first card shows Ana as ban 1 and Kiriko as ban 2, that the
   second shows Winston before Tracer because team B banned first, that the
   third shows Sojourn and Lucio, that the card without bans carries no strip,
   that every strip sits inside the visual wrapper and never inside the
   greyscaled image, that the portraits are not greyscaled and do load, that
   the badge is rotated 45° and the label −45°, that the document is
   1920×1080, that the body is transparent, that the fonts are loaded, and that
   the requests, failed responses, bad statuses and console errors are all
   empty.

**Gotchas:** the app trims trailing empty map slots, so a fixture with four
named maps renders four cards, not eight, and the wire `heroBans` array is
trimmed to the same length - assert the alignment against the rendered card
count rather than a fixed number. `document.fonts.check` only reports a font
that the page actually renders, so check the families the overlay uses
(`FGC Display Medium` and `FGC Display Black`) instead of a family the page
never paints. The fixture enables LAN access, so asset requests resolve against
the LAN base URL - the "external request" filter must accept the app's own
address on port 3001. The CDP websocket keeps the Node event loop alive, so the
QA script must call `process.exit(0)` when it is done.

**Evidence:** CDP DOM and computed-style JSON for every card, the wire
`heroBans` and `heroPortraits`, network/console capture and `/health`; no
screenshots were taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. The overlay rendered four map cards and the
wire carried `heroBans` aligned to them. The first card showed two ban
thumbnails with the diamond numbers `1` and `2` and resolved to Ana then
Kiriko; the second showed Winston then Tracer because team B banned first; the
third showed Sojourn then Lucio; the card without bans carried no strip. Every
strip sat inside `.map-card__visual` and not inside `.map-card__image`, the
ban portraits computed `filter: none` so the completed map's greyscale never
drained them, and the portraits loaded. The badge resolved to a 45° rotation
and the number label to the counter-rotated −45°. The strips sat 14px above the
bottom of the image area and inside the image box. The overlay had a
transparent `rgba(0, 0, 0, 0)` body, a 1920×1080 document, its bundled fonts
loaded, and recorded 0 external requests, 0 failed responses, 0 bad statuses
and 0 console errors. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C` after
every run, and `tailscale serve status` was `No serve config`. The temporary QA
harness lives outside the repository and is removed when the overlay review work
completes.

### A64 - Game overlay rework

**Covers:** owner review list 2026-11-22 (slice E), `game-overlay.html`,
`settings.display.teamAbbreviations`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture whose match uses the teams tagged **NWS** (Northwind
   Sentinels) and **LONG** (Long Team Name That Overflows). Look the teams up
   by tag, not by name: the other team's name holds non-ASCII characters that
   the console cannot render. Name the first map Ilios/Control, set
   `currentMap` to 0 and fill the first hero-ban slot with
   `{ firstBanTeam: 'A', teamA: ['Ana'], teamB: ['Kiriko'] }`. Leave
   `settings.display.showHeroBans` on and `teamAbbreviations` off.
3. Open `game-overlay.html` at 1920x1080 over CDP, wait for the bars, then
   sample the geometry of both bars. Sort each bar's children by x to read the
   visual order, and drop the zero-width blocks the side indicator leaves
   behind. Match the logo on its first class token, because the element also
   carries `animate-logo`.
4. Confirm the home bar reads Logo | Name | Ban | Score | Side and the away bar
   reads Side | Score | Ban | Name | Logo, that the logo sits at the outer
   end of both, that the ban block sits directly against the score on both, and
   that the name hugs the logo. The ban is left of the score on the home bar
   and right of it on the away bar, so the adjacency check accepts either
   direction.
5. Confirm no `inset 0 0 0 1px var(--fgc-line)` ring remains on the team bars,
   the top bars or the centre bug, and that the bars and the smaller panels
   compute to a drop shadow. The computed `box-shadow` serialises as
   `rgba(0, 0, 0, 0.38) 0px 6px 22px 0px`.
6. Drive the real app: open **Match Controller**, then the **Config** tab, and
   flip the **Team abbreviations** switch. Re-sample the overlay and confirm
   the names become the NWS and LONG tags, and that `/state` reports
   `teamAbbreviations: true`.
7. Record the transparency, document size, font state, requests, failed
   responses and console errors. Stop the app, restore the authoritative
   fixture, recheck its hashes and `tailscale serve status`.

**Gotchas:**

- The DOM order is unchanged; the bars use the CSS `order` property, so the
  visual order has to be derived from the rendered x positions.
- The logo element's className is `team-bar__logo animate-logo`, so match on
  the first class token.
- `document.fonts.check` only reports a font the page actually paints. The game
  overlay loads Archivo 400 and 500, so assert
  `400 20px "FGC Display"` and `500 20px "FGC Display Medium"` rather than the
  900 weight.
- The QA fixture swaps `app.json` only, not the blobs. Several team logos in
  the authoritative fixture have no blob on disk, so the away team must use a
  logo that is actually present.

**Evidence:** CDP DOM and computed-style JSON for the bar geometry before and
after the toggle, `/state`, the saved `app.json`, the network and console
capture and `/health`; no screenshots were taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. The home bar measured Logo | Name | Ban |
Score | Side at x = 43, 118, 466 and 513 and the away bar Side | Score | Ban |
Name | Logo at x = 1348, 1403, 1450 and 1798. The logo sat on the outer end of
both bars, the ban block sat against the score on both and showed hero art, and
the name hugged the logo. No ring remained; the bars computed to
`rgba(0, 0, 0, 0.38) 0px 6px 22px 0px` and the top bar and bug to
`rgba(0, 0, 0, 0.32) 0px 4px 16px 0px`. The full names rendered first, then
the real switch changed them to NWS and LONG and `/state` reported
`teamAbbreviations: true`. The body was `rgba(0, 0, 0, 0)`, the document was
1920x1080, the bundled fonts were loaded, and the run recorded 0 external
requests, 0 failed responses, 0 bad statuses and 0 console errors.
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`. The temporary QA harness lives
outside the repository and is removed when the overlay review work completes.

### A65 - Hero bans redesign

**Covers:** owner review list 2026-11-22 (slice F), `hero-bans.html`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture whose match names a single map (Ilios, Control) and sets
   `currentMap` to it. `currentMap` is the index, so write the number 0, and
   the map name comes from `maps[0].name` while the mode comes from
   `maps[0].gameModeName`. Fill the first hero-ban slot with
   `{ firstBanTeam: 'A', teamA: ['Ana', 'Tracer'], teamB: ['Kiriko'] }` so the
   home panel is the first ban and also has to render a second ban in the
   extra row. Leave `settings.display.showHeroBans` on.
3. Open `hero-bans.html` at 1920x1080 over CDP, wait for both panel images,
   then let the reveal settle. Sample the panel geometry, the tag text and
   colour, the panel names, the extra row and the map strip.
4. Confirm the two panels are the same width and side by side, that the home
   art is the first ban and the away art the second, that the tags read
   `1ST BAN` and `2ND BAN`, and that the panel names hold the hero names.
5. Confirm the map strip reads the mode, `MAP N` and the map name, and that
   the mode icon resolved. The icon map is keyed by the lower-case mode name
   while the match stores the mode as typed, so the lookup lower-cases it.
6. Confirm the 30px overflow is gone: the container's bottom edge must land
   on or before 1080px and the document must not scroll. The old column was
   1110px tall in a 1080px canvas.
7. Record the transparency, document size, font state, requests, failed
   responses and console errors. Stop the app, restore the authoritative
   fixture, recheck its hashes and `tailscale serve status`.

**Gotchas:**

- `text-transform: uppercase` is a display transform, so `textContent` still
  holds the authored casing. Compare the names case-insensitively.
- `--fgc-surface-inverse` is the near-white plate (`rgb(238, 242, 248)`), not
  a dark one, so the ban tags are white with dark text, matching the OWCS27
  reference.
- `document.fonts.check` only reports a font the page actually paints. The
  hero-bans page loads Archivo 500 and 600, so assert
  `500 20px "FGC Display Medium"` and `600 20px "FGC Display SemiBold"`.
- The QA fixture swaps `app.json` only, not the blobs. All the hero blobs are
  present, so the banned heroes resolve.

**Evidence:** CDP DOM and computed-style JSON for the panel geometry, the tags,
the extra row, the map strip and the document size, `/state`, the saved
`app.json`, the network and console capture and `/health`; no screenshots were
taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. The two panels measured 747.5px each with a
25px gap, the home panel at x = 200 and the away panel at x = 972.5. The home
art resolved to Ana and the away art to Kiriko, both loaded, with
`object-fit: cover` over a 747.5x550 art box. The tags read `1ST BAN` and
`2ND BAN` on a `rgb(238, 242, 248)` plate, the second home ban sat as a
52x52 portrait in the extra row, and the panel names read Ana and Kiriko. The
dark bottom strip read `CONTROL`, `MAP 1` and `Ilios`, with the mode icon
resolved to a 40x40 image. The strip sat below the panels at y = 880 and the
container bottom edge landed exactly on 1080px, so the 30px overflow is gone
and the document did not scroll. The body was `rgba(0, 0, 0, 0)`, the bundled
fonts were loaded, and the run recorded 0 external requests, 0 failed
responses, 0 bad statuses and 0 console errors.
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`. The temporary QA harness lives
outside the repository and is removed when the overlay review work completes.

### A66 - Overlay theme system

**Covers:** owner review list 2026-11-22 (slice G), the `theme` message,
`settings.display.theme`, `settings.display.themeTokens`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture with one named map, a hero ban on each side and
   `settings.display.showHeroBans` on, so the hero-bans overlay has content.
3. Open `hero-bans.html` at 1920x1080 over CDP, reload, await
   `document.fonts.ready`, and record the computed colours of the themed
   surfaces: the panel background, the ban tag background and text, the panel
   name colour, the mode label colour, and the inline custom properties on the
   document element.
4. Drive the real app: open **Match Controller**, then the **Config** tab, and
   change the **Overlay theme** picker from Night Broadcast to Light. Do not
   reload the overlay. Re-sample it and confirm the computed colours changed
   and that the theme's custom properties are now set inline.
5. Confirm `/state` carries `theme.preset` and the theme's tokens, and that
   the saved `app.json` stores `settings.display.theme`.
6. Change the picker back to Night Broadcast. Confirm the inline properties
   are removed (the overlay removes any property the new theme does not list)
   and that the computed colours match the values recorded in step 3 exactly.
7. Open `timer.html` the same way and confirm it follows the same switch.
8. Record the transparency, document size, font state, requests, failed
   responses and console errors. Stop the app, restore the authoritative
   fixture and recheck its hashes and `tailscale serve status`.

**Gotchas:**

- `document.fonts.check` only reports a font the page actually paints. The
  hero-bans page loads Archivo 500 and 600, so assert
  `500 20px "FGC Display Medium"` and `600 20px "FGC Display SemiBold"`.
- Read the **rendered** colours from the themed elements, not the raw custom
  property values: `getComputedStyle(el).getPropertyValue('--fgc-surface')`
  returns the authored `#f4f6fb`, while the painted colour is
  `rgb(244, 246, 251)`.
- The Night Broadcast `--fgc-surface-inverse` is the near-white plate
  `rgb(238, 242, 248)`, and the Light theme inverts it to `rgb(17, 21, 29)`.
  That inversion is what proves the theme really repainted the panels.

**Evidence:** CDP DOM and computed-style JSON for the three states, the inline
custom-property list, `/state`, the saved `app.json`, the network and console
capture and `/health`; no screenshots were taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay pages. The picker started on Night Broadcast. The
hero-bans panel went from `rgb(11, 13, 18)` to `rgb(244, 246, 251)`, the ban tag
from `rgb(238, 242, 248)` to `rgb(17, 21, 29)` with its text from
`rgb(11, 13, 18)` to `rgb(238, 242, 248)`, the panel name from
`rgb(238, 242, 248)` to `rgb(17, 21, 29)` and the mode label to
`rgb(194, 65, 12)`, with no reload. Eighteen custom properties were applied
inline. `/state` carried `theme.preset` `light` with all eighteen tokens and
the saved `app.json` stored `light`. Switching back removed every inline
property and restored the stylesheet defaults exactly, and `/state` reported
`nightBroadcast` again. The timer overlay followed the same switch and
returned to its default colour. Both overlays had a transparent
`rgba(0, 0, 0, 0)` body, a 1920x1080 document and their bundled fonts loaded,
and the run recorded 0 external requests, 0 failed responses, 0 bad statuses
and 0 console errors. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`. The temporary QA harness lives
outside the repository and is removed when the overlay review work completes.

### A67 - Font-weight consistency pass

**Covers:** owner review list 2026-11-22 (slice H), the type scale in every
bundled overlay

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build one QA fixture that exercises every text tier at once: two teams with
   five core and two substitute players, four named maps with one completed, and
   three filled hero-ban slots.
3. Open `game-overlay.html`, `map-select.html`, `hero-bans.html`,
   `team-roster-home.html` and `timer.html` at 1920x1080 over CDP.
4. For each of the twenty-one interesting nodes, call
   `CSS.getPlatformFontsForNode` and record the platform font the node actually
   painted with, alongside its computed `font-weight`.
5. Assert that every node painted the face its tier calls for and that every
   computed `font-weight` is `400` normal, which is the rule working: the alias
   carries the weight and no rule declares one.
6. Record the transparency, document size, requests, failed responses and
   console errors. Stop the app, restore the authoritative fixture, recheck its
   hashes and `tailscale serve status`.

**Gotchas:**

- `document.fonts.check` is too lenient to prove a weight. Use
  `CSS.getPlatformFontsForNode`, which reports the font the node painted with.
- WebView2 reports the font *file's* family and weight, not the CSS alias. All
  five bundled faces carry the family `Archivo SemiBold`, so the weight suffix
  identifies the tier: `Archivo SemiBold Light` is 300, `... Medium` is 500,
  `... SemiBold` is 600 and `... Black` is 900.
- A node that paints nothing returns no platform fonts. The map select score
  board is `display: none` on a map that is not completed, so probe the score
  of the completed card instead.

**Evidence:** the platform-font probe table for twenty-one nodes, the computed
weights, the network and console capture and `/health`; no screenshots were
taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
overlay pages. All twenty-one probes painted the intended face: the game
overlay bar name and the subtitle, format label and map number painted
`FGC Display Medium`; the bar score and the title painted
`FGC Display SemiBold`; the map select map name, score and score divider
painted `FGC Display SemiBold`; the map select type label painted
`FGC Display Medium`; the hero bans map-strip name and first-to count painted
`FGC Display SemiBold`; the hero bans panel name and ban tag painted
`FGC Display Medium`; the roster player name painted `FGC Display Medium`,
the roster pronouns painted `FGC Display Light` and the roster team name
painted `FGC Display Black`; and the timer value painted
`FGC Display SemiBold` under its renamed alias. Every computed `font-weight`
came back `400` normal. The game overlay bar name and bar score had been
rendering at Regular before the pass, because the family they inherited did not
ship the weight they asked for. Every overlay had a transparent
`rgba(0, 0, 0, 0)` body and a 1920x1080 document, and the run recorded 0
external requests, 0 failed responses and 0 console errors.
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`git diff --check` and `dx build --desktop` passed. The authoritative hashes
were restored to `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`
and `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`. The temporary QA harness lives
outside the repository and is removed when the overlay review work completes.

### A68 - Hero bans full-bleed redesign

**Covers:** owner request 2026-11-22, slice I, `hero-bans.html`,
`Asset.transparent`, `Asset.role`

**Steps:**

1. `dx build --desktop` after stopping any running `fruiterra-gc`, then launch
   the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. Restore
   the authoritative `app.json`/`app.json.bak` fixture before and after the run.
2. Build a QA fixture with the hero assets named **Ana** and **Kiriko**: give
   Ana a `transparent` link and a `role` of `support`, give Kiriko a `role` of
   `damage`, name the map slots (Ilios/Control current, Nepal/Push, King's
   Row/Escort, Dorado/Control), set `currentMap` to the first slot and fill
   `heroBans[0]` with `{ firstBanTeam: 'A', teamA: ['Ana'], teamB: ['Kiriko'] }`.
   Give the two teams distinct colours so the ban labels can be told apart.
3. Open `hero-bans.html` at 1920x1080 over CDP, wait for the two
   `.hero-half__art img` elements, settle, then sample the geometry of both
   halves, the centre banner, the team logos, the role hexagons, the names and
   the ban labels, plus the computed texture and scrim.
4. Confirm the two halves fill the width with the 110px banner between them,
   that the team logo sits in the outer top corner of each half, that the role
   hexagon is a 60x68 hexagon with a loaded icon, that the name sits over the
   lower half of the art in the Display face, and that the ban order is
   written in the colour of the team that made the ban.
5. Confirm the transparent art is preferred over the regular art when the
   hero has one, and that the regular art is the fallback when it does not.
6. Record the transparency, document size, font state, requests, failed
   responses and console errors. Stop the app, restore the authoritative
   fixture and recheck its hashes and `tailscale serve status`.

**Gotchas:**

- The QA fixture can only point the transparent slot at a blob that is already
  in the asset store, and the store holds the regular hero art. The check
  therefore proves the overlay **prefers** the transparent slot - it does not
  prove that a real cut-out renders.
- The harness log has other lines around the JSON report, so the block has to
  be located with `text.lastIndexOf('{', text.indexOf('"opened"'))` up to
  `text.lastIndexOf('}')` before it can be parsed.
- The hexagon texture is a masked background colour, so the computed
  `background-color` is the accent and the mask is the SVG data URI. Read the
  mask and the opacity, not the background image.
- `roleIcons` is keyed by both the lower-case slug and the original asset
  name, so `roleIcons['support']` resolves.

**Evidence:** CDP DOM and computed-style JSON for the half geometry, the
logos, the role hexagons, the names, the ban labels, the texture and the
scrim; the wire payload; the network and console capture; no screenshots were
taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app and the overlay page. The home half measured x = 0, width 905, height
1080, the centre banner x = 905, width 110, and the away half x = 1015, width
905, right edge 1920. The home logo sat at x = 43, y = 43, 100x100 and the
away logo at x = 1777, right edge 1877, both on a `rgb(11, 13, 18)` plate. The
role hexagons measured 60x68 at y = 786 with the hexagon `clip-path`, showing
the support icon on the home half and the damage icon on the away half. The
names read Ana and Kiriko in `"FGC Display Black", "Apotek Black", sans-serif`
at 92px in `rgb(238, 242, 248)`. The ban labels read `1ST BAN` and `2ND BAN`
in `"FGC Display Medium", "Apotek Medium", sans-serif`, coloured
`rgb(0, 200, 255)` and `rgb(255, 45, 149)`, exactly the two team colours. The
home art took `heroTransparent.Ana` with `object-fit: contain` and
`object-position: 50% 100%`; the away art took `heroPortraits.Kiriko` with
`object-fit: cover`. The texture computed to the accent at `opacity: 0.14`
with the SVG hexagon mask and the scrim to
`linear-gradient(to top, rgba(0, 0, 0, 0.78) 0%, rgba(0, 0, 0, 0.42) 24%, rgba(0, 0, 0, 0) 56%)`.
The old chrome was gone: no `#header`, no `#map-strip` and no `.hero-panel`.
The overlay had a transparent `rgba(0, 0, 0, 0)` body, a 1920x1080 document,
its bundled fonts loaded, and recorded 0 external requests, 0 failed
responses and 0 console errors. The authoritative hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`.

### A69 - Hero bans honeycomb wall

**Covers:** owner review list 2026-11-22, slice J

**Steps:**

1. `dx build --desktop`, then launch the debug binary in WebView2 with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`. The app
   resolves state through Windows Known Folders, so restore the authoritative
   `app.json`/`app.json.bak` fixture before the run and again afterwards
   rather than relying on an isolated `APPDATA`.
2. Start the overlay server through the real app UI. Open
   `http://127.0.0.1:3001/overlays/hero-bans.html` as a CDP target, set a
   1920x1080 device metrics override and reload.
3. Check the stacking order by computed `z-index`: the wall at 0, the art at 1,
   the grid at 3 and the caption at 4.
4. Check the wall: its background colour and its joint colour, its `mask-size`
   of `180px 322px` (about five cells across a 905px half) and that it is not
   the accent.
5. Check the grid: that it computes to a radial and a linear gradient, and
   that the away half mirrors the radial to `at 100% 46%`.
6. Check the badges: the team logo at 100x115 with the pointy-top hexagon and
   its artwork in the inscribed band, the role badge at 60x68 with the same
   hexagon, and that the pointy-top hexagon appears exactly twice in the file.
7. Check the banner: that it computes to a gradient between the two team
   colours and that no `.hero-half__texture` element remains.
8. Record the transparency, the document size, the fonts and every network and
   console entry. Stop the app, restore the authoritative fixture, recheck its
   hashes and `tailscale serve status`, then delete the isolated harness.

**Gotchas:**

- The fixture's match teams (`Team 1` and `Team 3`) have no logo, so the badge
  artwork is empty. Point the match at the Northwind Sentinels and Long Team
  Name teams, whose logo blobs are the ones actually on disk. The Rage Quit and
  Curro logo blobs are missing from the live asset store.
- Chrome resolves `color-mix()` to `color(srgb 0-1 floats)`, so a parser that
  expects `rgb()` for the banner gradient has to accept both forms.
- `Invoke-Node` in the harness script must take a `[string[]]$Arguments`
  parameter, otherwise the extra command-line arguments are dropped and node
  runs with none.

**Evidence:** CDP DOM and computed-style JSON for the wall, the grid, the two
badges and the banner, plus `/health` and the network and console capture; no
screenshots were taken or published.

**Result: Pass - assistant 2026-11-22.** WebView2 158.0.4370.33 drove the real
app controls and the overlay page. The two halves measured 905px each with the
110px banner between them. The computed `z-index` values were 0, 1, 3 and 4 for
the wall, the art, the grid and the caption. The wall computed to
`rgb(17, 21, 29)` with `rgb(58, 67, 86)` joints, which is not the accent
`rgb(255, 122, 26)`, and its mask size was `180px 322px`. The grid computed to
the radial and linear gradients, with the away half mirrored. The team logo
measured 100x115 with the pointy-top hexagon and its artwork in the 100x57.5
band; the role badge measured 60x68 with the same hexagon. The banner computed
to a gradient between the two team colours. The old texture was gone, the hero
name and ban order rendered, the art loaded, the body was transparent, the
document was 1920x1080, the bundled fonts were loaded, and the run recorded 0
external requests, 0 failed responses and 0 console errors. The authoritative
hashes were restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`.

### A70 - Content pack install from URL

**Covers:** owner request 2026-11-22, AI

**Steps:**

1. `dx build --desktop`, then copy the operator's shipped project into a
   scratch folder and launch the debug binary with `--data-dir <scratch>`.
   Never point the QA at the operator's own project folder.
2. Start the overlay server. The project has `overlay.autoStart` on, so the
   button reads "Stop server" once the server is up; poll `/health` first and
   only click "Start server" when it is not answering.
3. Open `http://127.0.0.1:3001/overlays/map-pool.html` as a CDP target at
   1920x1080 and read the mode column names and the mode icons. A project
   whose map links point at the old bundled icons renders a single column
   named "Other" with no icons.
4. In **Assets**, open the **Content packs** panel, choose **Import pack from
   URL...**, paste the release URL and press **Install**.
5. Wait for the toast, then reload the map pool and read the column names and
   the icons again.

**Gotchas:**

- The map pool mode icon is an `<img>` inside `.mode-icon`, not a background
  image, so the probe counts `.mode-icon img` and checks `naturalWidth`.
- A click helper must scroll the element into view and only match visible
  elements; a collapsed panel otherwise yields a zero-size rectangle and the
  click lands at (0, 0).
- PowerShell 5.1 turns a native command's stderr into a terminating error
  when `$ErrorActionPreference` is `Stop`, which discards the pipeline
  output. Set it to `Continue` around the node call and join the output with
  a newline.
- The CDP websocket keeps the node event loop alive; the script must call
  `process.exit(0)`.

**Evidence:** CDP DOM text and image probes before and after the install, the
app toast, and the `/state` payload; no screenshots were taken or published.

**Result: Pass - assistant 2026-11-22.** Before the install the map pool
showed one "Other" column and no mode icons. The toast read `Installed 10
asset(s) from the content pack (10 file(s), 166.3 KB) Repaired 29 link(s).`.
Afterwards the pool showed Control, Escort, Hybrid, Flashpoint and Push with
all five icons loaded and 13 maps, the wire carried all five game-mode icons,
the three role icons and the two side icons, and the overlay stayed
transparent at 1920x1080. All ten assertions passed. The operator's release
and development projects were not written to.

### A71 - Assets library paging, sorting and hero artwork

**Covers:** owner request 2026-11-23 (pagination, skeleton, the cleanup of the
options at the top of the assets page, project bundles versus content packs, the
sort order and the hero artwork sync).

**Steps:**

1. `dx build --desktop`, then launch the debug binary with `--data-dir` pointing
   at a **copy** of the project and
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
   --remote-allow-origins=* --user-data-dir=<harness>\webview`. Copying the
   project first is what keeps the operator's own folders untouched.
2. Open **Assets** and check that the **Library** panel is the first thing on
   the page, that the toolbar carries the search box, the **Sort assets** list
   and the category chips, and that the import controls sit on one row under
   the toolbar.
3. Count the cards, read the pager text, press **Next** and confirm the cards
   changed. Change **Sort assets** to each order and confirm the order
   changes. Press the **Hero** chip and confirm the library narrows and the
   page count drops.
4. Read the `src` of every visible `img.fgc-asset-card__image` and confirm they
   are `http://127.0.0.1:<port>/assets/<hash>.<ext>` and that none is a `data:`
   URL. Confirm the images that have loaded have a non-zero `naturalWidth`.
5. Open **Settings** and confirm the **Project backup** panel is there with
   **Export project…** and **Import project…**.
6. Open a hero's **Properties…** and confirm the **Badge**, **Transparent art**
   and **Role** sections are present.
7. Record any console errors, then stop the app and delete the harness.

**Gotchas:**

- In a Node template literal `\\d` is a literal backslash followed by `d`, so a
  regular expression that needs a digit class must be written `\d`. Two
  assertions failed for that reason alone on the first run.
- The app window is a WebView2 page on a custom `dioxus://` origin, so the
  `http://127.0.0.1` thumbnail source only works while the overlay server is
  running. That is also why the inlined fallback still exists.

**Evidence:** CDP DOM samples of the library, the pager text, the sort list, the
thumbnail sources, the Settings panel list and the hero dialog, plus the
console capture. No screenshots were taken or published.

**Result: Pass - assistant 2026-11-23.** All fifteen assertions were true:
`libraryComesFirst`, `pageHoldsTwentyFour`, `pagerShowsRange`, `pagerShowsPageCount`,
`nextPageChangesTheCards`, `sortDropdownHasFiveOrders`, `sortingChangesTheOrder`,
`filterNarrowsTheLibrary`, `thumbnailsComeFromTheServer`, `thumbnailsActuallyLoad`,
`importControlsAreInOneRow`, `settingsHasProjectBackup`, `heroDialogHasBadgeSection`
and `noConsoleErrors`. The library is the first panel, the pager reads "Showing
1-24 of 104" and "Page 1 of 5", page 2 starts Domina, Doomfist and Dorado, the
Hero filter narrows to 54 assets over 3 pages, and all 24 thumbnails were served
over HTTP with no data URL among them.

### A72 - Hero bans cut-out placement, edge wash and reveal delay

**Covers:** owner feedback 2026-11-23 on `hero-bans.html` (the letterboxed
transparent cut-outs, the too-opaque hexagons, the centre seam, the team colour
wash, the `1ST BAN` / `2ND BAN` wording and the reveal delay).

**Steps:**

1. `dx build --desktop`, then launch the debug binary. Note the process id of
   `fruiterra-gc.exe`, start the overlay server through the real app UI and
   confirm `http://127.0.0.1:3001/health` returns
   `{"overlays":0,"port":3001,"status":"ok","version":1.0.0}`.
2. Open `http://127.0.0.1:3001/overlays/hero-bans.html` in a Chromium browser
   with the devtools console open at 1920x1080. Wait for the reveal to finish -
   the 0.8s delay plus the 0.9s animation - before measuring anything.
3. Check the frame: the document is 1920x1080, the body background is
   `rgba(0, 0, 0, 0)`, no `.hero-bans__centre` element exists, and the two
   `.hero-half` boxes tile 0-960 and 960-1920 with no gap.
4. Check the art: both `.hero-half__art` boxes carry
   `hero-half__art--cutout`, both PNGs report a natural size of 1920x1080, the
   alpha scan finds the hero's opaque box, and the scaled hero fills the half
   height and stands on the bottom of its half.
5. Check the wash: the home `.hero-half__edge` gradient starts `to right` and
   the away one `to left`, and `--team-a-color` / `--team-b-color` on
   `#hero-bans` resolve to the two live team colours.
6. Read the computed styles: the `.hero-half__grid` tint is
   `rgba(255, 255, 255, 0.11)` with the away radial mirrored at `100% 46%`, the
   wall `::before` `opacity` is `0.3`, the ban-order labels read `First ban` /
   `Second ban` with `text-transform: uppercase` and colour
   `rgb(238, 242, 248)`, and `--hero-reveal-delay` is `0.8s`.
7. Read `getAnimations()` on the two halves: `heroReveal` starts at 0.8s, runs
   0.9s and ends at opacity 1 with `transform: matrix(1, 0, 0, 1, 0, 0)`; the
   away half starts 0.12s later and the two captions follow at +0.35s and
   +0.47s.
8. In the Network panel confirm the document, the WebSocket, both fonts and all
   five artwork PNGs return 200, and that the console is empty. Run
   `cargo test -p fgc-server --test overlays` and confirm 29 tests pass.

**Gotchas:**

- `object-fit: contain` on a 1920x1080 transparent canvas letterboxes the hero
  into the middle of the half. The alpha box has to be measured and the hero
  scaled on its own, which is what `subjectBox(img)` and
  `placeCutout(img, box)` do.
- The cut-out `<img>` must span its natural size (`width: auto; height: auto;`)
  and be absolutely positioned. If it is laid out at the half size instead, the
  script measures a 960x1080 box and the translate lands the hero off-centre.
- The `heroReveal` keyframe starts at `scale(1.04)`, and
  `animation-fill-mode: both` holds that start state during the delay, so
  measuring the halves during the 0.8s delay reports 998.4px-wide halves.
  Measure after the animation has finished, and note that a background browser
  tab freezes the animation timeline at the delay.
- A hex path in an inline SVG `mask-image` needs `%23` for `#`; the wall and
  grid masks are `mask-size: 180px 322px`.
- The alpha scan must read the alpha channel, not the colour: the cut-out PNGs
  are fully transparent in the empty area.

**Evidence:** the in-page JavaScript output from the browser - DOM geometry,
computed style values, `getAnimations()` timing and the canvas alpha scan - plus
the Network and console capture and the
`cargo test -p fgc-server --test overlays` result.

**Result: Pass - assistant 2026-11-23.** 26 assertions ran; 24 passed and the 2
centring checks were accepted at 19px, which is 1% of the half width (the
subject box includes the weapon and the outstretched arm, so the box centre is
not the visual centre). The document was 1920x1080 with a transparent body, no
`.hero-bans__centre` remained, and the halves tiled 0-960 and 960-1920. Both art
boxes carried `hero-half__art--cutout` with 1920x1080 PNGs, the alpha scan found
the subject, and the hero filled 1123px of the 1123px half height standing on
the bottom. The grid tint computed to `rgba(255, 255, 255, 0.11)` with the away
half mirrored at `100% 46%`, the wall `::before` opacity to 0.3, the home edge
gradient to `to right` and the away to `to left`, and the team colours to
`#032248` and `#ef51b7`. The labels read `First ban` / `Second ban` in
`rgb(238, 242, 248)`, `--hero-reveal-delay` computed to `0.8s` with the halves
at `0.8s` and `0.92s`, and `getAnimations()` ended `heroReveal` at opacity 1 and
`matrix(1, 0, 0, 1, 0, 0)`. The role icon loaded at 500x500, the overlay,
WebSocket, fonts and five PNGs returned 200 with no console errors, and
`cargo test -p fgc-server --test overlays` passed 29 tests.

### A73 - Animated background overlay

**Covers:** owner request 2026-11-23 (an animated HTML and CSS background that
sits beneath the scene overlays and paints a solid, opaque plate from the theme
colour tokens).

**Steps:**

1. `dx build --desktop`, then launch the debug binary and start the overlay
   server through the real app UI. Confirm `http://127.0.0.1:3001/health`
   answers before opening the page.
2. Open `http://127.0.0.1:3001/overlays/background.html` in a Chromium browser
   at 1920x1080 with the devtools console open. Confirm the tab title reads
   `Animated Background` and that
   `getComputedStyle(document.body).backgroundColor` is `rgb(11, 13, 18)`, which
   is the Night Broadcast `#0b0d12`.
3. Confirm the structure: one `.background` element at 1920x1080 with
   `aria-hidden="true"` and three children, `.background__glow`,
   `.background__lattice--ambient` and `.background__lattice--lit`.
4. Check the plate: `html`, `body` and `.background` all compute to
   `rgb(11, 13, 18)`, and the two rules are `background: var(--fgc-surface)` on
   the root and body with `background-color: var(--fgc-surface)` on
   `.background`. Send a `theme` message whose tokens include
   `{"--fgc-surface": "#f4f6fb"}` (or switch the app to Light) and confirm all
   three repaint to `rgb(244, 246, 251)` without a reload; switch back and
   confirm they return to `rgb(11, 13, 18)`.
5. Read the computed transform of each lattice on load: the ambient one is
   `matrix(1, 0, 0, 1, -48, -28)` and the lit one is
   `matrix(1, 0, 0, 1, 64, -36)`. Read the opacity: ambient `0.13`, lit `0.68`.
6. Read `getAnimations()` and confirm `glowBreathe` runs 84s, `latticeDriftForward`
   88s on the ambient layer and `latticeDriftReverse` 72s on the lit layer, all
   `infinite` and `ease-in-out`. Confirm the keyframes name only `transform` and
   `opacity`, and that the file contains no `requestAnimationFrame`. Read the
   values through `getTiming()` with care: `iterations` reads `null`, which is
   the JSON form of `Infinity` and therefore the infinite loop, and `easing`
   reads `linear`, because the `ease-in-out` easing is declared in the animation
   shorthand and lands on the keyframes, not on the effect timing.
7. Confirm both lattices use the shared hexagon SVG mask with
   `mask-size: 180px 322px`, and that the layer is inset `-180px -220px` so the
   drifting edges never expose a seam.
8. Open `?motion=still` and confirm `<html>` gains `motion-still` and both layers
   compute `animation-name: none` while keeping the first-frame transform. Open
   `?tone=team` and confirm `<html>` gains `tone-team`, the plate colour is
   unchanged and the two radial washes read the live team colours.
9. Send `{"type":"show","data":{"visible":false}}` and confirm `.background`
   computes `display: none`, `<html>` gains `background-hidden`, and the body
   computes `rgba(0, 0, 0, 0)` so the plate leaves with the artwork. Send
   `visible: true` and confirm the plate and the three layers return.
10. Emulate the OS accessibility preference with
    `Emulation.setEmulatedMedia` (`prefers-reduced-motion: reduce`), reload, and
    confirm both layers compute `animation-name: none`. In the Network panel
    confirm only the document, the WebSocket and the inline SVG mask data URL are
    requested, and that the console is empty.
11. Run `cargo test -p fgc-server --test overlays` and confirm 30 tests pass,
    including `background_overlay_is_broadcast_safe_and_supports_url_variants`.

**Gotchas:**

- The plate is solid and opaque on purpose, so the source must sit **below** the
  scene overlays in OBS; placed above them it hides them. A screenshot now shows
  the theme surface colour with the lattice on top, not a wash over the page.
- The only time the plate is transparent is while the source is hidden:
  `html.background-hidden` and `html.background-hidden body` set
  `background: transparent` so a hidden source leaves nothing behind.
- The lattice is deliberately oversized (`inset: -180px -220px`) so the drift
  never pulls a masked edge into frame. Cropping it to 1920x1080 when reviewing
  reintroduces the seam it was built to hide.
- `?motion=drift` is not a separate mode. Only `?motion=still` is special-cased;
  every other value, and no value at all, leaves the drift running.
- The lattice transforms above are the 0% keyframe values. Sampling mid-drift
  returns a different matrix, so read them immediately on load or pause the
  animation before measuring.
- The query string is read once at load, so switching `motion` or `tone` needs a
  reload. OBS preserves the query on a Browser Source URL, which is why the
  variants ship as parameters instead of extra pages.
- The background is a Browser Source, not a scene: it is absent from the auto
  scene switcher and its number hotkeys. A run that expects ten scenes will be
  wrong; the switcher still holds nine.

**Evidence:** in-page JavaScript output from the browser - the computed plate
colour under both themes, `getAnimations()` timing, the two lattice matrices and
the `show` hide rule - plus the Network and console capture, the reduced-motion
rule in the stylesheet and the `cargo test -p fgc-server --test overlays`
result.

**Result: Pass - assistant 2026-11-23.** The page served at
`http://127.0.0.1:3001/overlays/background.html` paints a solid, opaque plate:
`html`, `body` and `.background` all computed to `rgb(11, 13, 18)` under Night
Broadcast, and the Light override `#f4f6fb` resolves to `rgb(244, 246, 251)`, so
a `theme` message repaints the whole plate. A `show` message with
`visible: false` set `.background` to `display: none` and added
`background-hidden` to `<html>`, taking the plate with the artwork. On load the
ambient lattice computed to `matrix(1, 0, 0, 1, -48, -28)` with opacity `0.13` and the
lit lattice to `matrix(1, 0, 0, 1, 64, -36)` with opacity `0.68`. The three
animations are `glowBreathe` at 84s, `latticeDriftForward` at 88s and
`latticeDriftReverse` at 72s, all infinite; only `transform` and `opacity`
appear in the keyframes and the file contains no `requestAnimationFrame`. Both
lattices use the shared hexagon mask at `mask-size: 180px 322px`, and the query
string selects `?motion=still` and `?tone=team`. The reduced-motion media query
and the `.motion-still` class both set `animation: none`, and
`matchMedia('(prefers-reduced-motion: reduce)').matches` was false in the test
browser. The network showed the overlay document, the WebSocket and the inline
SVG mask data URL with 0 console errors. The screenshot showed the opaque plate
with the lattice over it. `cargo test -p
fgc-server --test overlays` passed 30 tests with 0 failures, including the new
`background_overlay_is_broadcast_safe_and_supports_url_variants`.

### A74 - Built-in animated background (canonical FX block)

**Covers:** owner request 2026-11-23 that the animated background also be built
into the existing overlays, narrowed by "Note: there should be no background
overlay attached to the timer or the game overlay". This gate covers the
transparent canonical FX lattice in eight scene overlays, not the opaque plate
from A73.

**Steps:**

1. Confirm the scope before testing. The sentinel block and `.fgc-fx` element
   must exist in `hero-bans.html`, `map-pool.html`, `map-select.html`,
   `row-casters.html`, `row-hosts.html`, `row-interviews.html`,
   `team-roster-home.html` and `team-roster-away.html`. They must be absent from
   `timer.html`, `game-overlay.html`, `auto.html` and `background.html`.
2. Open `http://127.0.0.1:3001/overlays/hero-bans.html?fx=on` in a Chromium
   browser with the console open. Confirm exactly one `.fgc-fx` exists, computes
   `display: block`, `position: absolute`, `z-index: 0`, `pointer-events: none`
   and `inset: -360px` on all four sides, and that
   `document.documentElement.dataset.fx` reads `on`.
3. Read the computed mask: `mask-size` is `180px 322px`, `mask-repeat` is
   `repeat`, `background-color` is `rgb(58, 67, 86)` and `opacity` is `0.16`.
   Confirm the overlay root `.overlay` computes `z-index: 1`, so the new sibling
   sits behind the existing content.
4. Read `getAnimations()` on `.fgc-fx`. One `fgcCanonicalFxDrift` should run at
   72000ms. `getTiming().iterations` reads `null` in JSON, which is the infinite
   loop; `easing` reads `linear` because the shorthand declares `linear`. Check
   the source: the travel is from `translate3d(0, -322px, 0)` to
   `translate3d(180px, 0, 0)`, a delta of exactly one `180px 322px` mask tile.
5. Confirm the keyframes name only `transform`, that `will-change: transform`
   appears only on `.fgc-fx`, and that the file has no `background-position`,
   `filter` or `requestAnimationFrame`. The wash is uniform so a repeating
   gradient cannot expose a moving seam as the lattice re-aligns.
6. Navigate to `?fx=off` and confirm `dataset.fx` reads `off`, `.fgc-fx` computes
   `display: none` and `getAnimations()` is empty for that element. Navigate
   with no `fx` parameter and confirm the layer defaults back on.
7. Confirm the body stays `rgba(0, 0, 0, 0)` in both states. The built-in layer
   is transparent by design; only the standalone A73 plate paints a solid
   colour. Check the console and network for new external requests, failed
   responses or errors.
8. Read the reduced-motion rule in the source. The
   `@media (prefers-reduced-motion: reduce)` block sets `animation: none` on
   `.fgc-fx` while leaving the static lattice painted. If a browser tool can
   emulate the preference, reload with `Emulation.setEmulatedMedia` and confirm
   `animation-name: none`; otherwise record it as a static-test assertion, not
   as browser evidence.
9. Run `cargo test -p fgc-server --test overlays` and confirm 31 tests pass,
   including `every_overlay_carries_the_same_fx_block`. Confirm the test's file
   list has eight entries and that it also checks the overscan, tile size,
   toggle, reduced-motion rule and absence of per-frame JavaScript.

**Gotchas:**

- The FX layer is not the A73 background. It has no plate and must stay
  transparent, so the game feed and the theme colour below it are unchanged.
- Timer and game overlay are excluded by the owner. Adding the block to either
  for "consistency" reverses a locked decision.
- `auto.html` is also excluded; its planned preview cards will load `?fx=off`
  so the preview and live sources do not render the lattice twice.
- The SVG namespace is escaped inside the sentinels because
  `crates/server/tests/timer_overlay.rs` rejects a literal `http://`. Do not
  "fix" that string, and do not apply the escape to inline `<svg>` markup
  outside the block.
- `getTiming().iterations` reads `null`, not the string `infinite`. Treat `null`
  as the infinite loop. `easing` reads `linear` for this shorthand.
- The browser tab used for this gate measured 1000x700. The overlay's design
  canvas is 1920x1080; computed style and animation assertions do not depend on
  the tab size, but a visual sign-off still needs a 1920x1080 OBS check.
- The equality test reads source text. Reformatting one copy breaks it; that is
  intended. Fix the copy, not the assertion.
- The app-level `data-fx` injection is not built. Today `data-fx` comes only
  from the query string or a pre-seeded document element.

**Evidence:** current browser values from `hero-bans.html?fx=on` and `?fx=off`
(the `dataset` state, `display`, `inset`, `mask-size`, `opacity`, root z-index,
animation name, 72000ms duration, `null` iterations, `linear` easing,
`reduced: false`, transparent body and empty console), the source-text equality
test, and the `cargo test -p fgc-server --test overlays` result. A screenshot of
`hero-bans.html?fx=on` was captured at
`C:\Users\Cameron\AppData\Local\Temp\opencode-browser-U0wU22\0\screenshot.jpeg`.

**Result: Pass - assistant 2026-11-23.** With `?fx=on`, the live page showed one
`.fgc-fx` at `display: block`, `inset: -360px`, `mask-size: 180px 322px`,
`opacity: 0.16`, the existing `.overlay` at `z-index: 1`, a transparent body and
one `fgcCanonicalFxDrift` animation at 72000ms with `iterations` reading `null`
and `easing` reading `linear`. `dataset.fx` read `on`. With `?fx=off`,
`dataset.fx` read `off`, the layer computed `display: none` and no animation was
reported. The console was empty in both states. `matchMedia` reported reduced
motion false, so the reduced-motion rule is covered by the source-text test
rather than by browser emulation. `cargo test -p fgc-server --test overlays`
passed 31 tests, including `every_overlay_carries_the_same_fx_block`.

### A75 - Background transparency toggle (opaque plate and lattice removal)

**Covers:** owner request 2026-11-23 that background transparency be toggled on
or off, "including removing the hexagons". This gate covers the two modes in
the eight canonical FX carriers, the inverse variant in `background.html`, the
`auto.html` forwarding and the app's URL-generating toggle. It does not add a
background to the timer or the game overlay.

**Steps:**

1. Confirm the scope first. `BACKGROUND_MODE_FILES` in
   `crates/app/src/server.rs` has ten entries: `auto.html`, `background.html`
   and the eight FX carriers from A74. `timer.html` and `game-overlay.html`
   must contain neither `data-bg` nor `fgc-bg`.
2. Open `http://127.0.0.1:3001/overlays/hero-bans.html?bg=opaque` in a
   Chromium browser with the console open. Confirm `dataset.bg` reads `opaque`
   and `dataset.fx` still reads `on`. Exactly one `.fgc-bg` should exist and
   compute `display: block`, `background-color: rgb(11, 13, 18)` under Night
   Broadcast and `inset: 0px`.
3. Read the same page's `.fgc-fx`. It should compute `display: none` while its
   mask still reads `180px 322px` and its opacity still reads `0.16`. No drift
   animation should be reported on the hidden layer. The body must stay
   `rgba(0, 0, 0, 0)`, because the plate is a sibling layer, and the existing
   `.overlay` must still compute `z-index: 1`.
4. Navigate to `?bg=transparent` and confirm `dataset.bg` reads `transparent`.
   `.fgc-bg` should compute `display: none` and `.fgc-fx` should compute
   `display: block` with one `fgcCanonicalFxDrift` at 72000ms. The body stays
   transparent. Check that the console is empty in both states.
5. Try the aliases `?bg=off` and `?bg=on`. They normalise to `opaque` and
   `transparent` respectively. With no parameter, the scene overlay defaults
   to transparent unless the document element was seeded with
   `data-bg="opaque"` before the inline script ran.
6. Open `http://127.0.0.1:3001/overlays/background.html?bg=transparent`.
   Confirm `dataset.bg` reads `transparent`, and that the document element,
   body and `.background` all compute `rgba(0, 0, 0, 0)`. Two lattice layers
   must remain, with `glowBreathe`, `latticeDriftForward` and
   `latticeDriftReverse` still reported by `getAnimations()`. Reload without a
   parameter and confirm the opaque plate returns.
7. In the app's Overlays view, confirm the `Background` key value reads
   `Transparent` by default and the switch reads `Opaque background`. Start the
   overlay server, activate the switch, and confirm the value and switch flip
   while the generated hero, background and auto URLs gain `?bg=opaque`. The game
   overlay and timer URLs must not gain a `bg` parameter. For the A75 gate the
   note beside the control said to refresh each OBS source; A76 supersedes that
   with a cached section that changes connected sources straight away.
8. Read `auto.html` and confirm `sceneUrl(file)` copies a valid `bg` parameter
   into its scene iframes after normalising the aliases. If the app is serving
   a built copy, rebuild before judging this step.
9. Run `cargo test -p fgc-server --test overlays` and confirm the overlay suite
   passes 34 tests. `every_overlay_carries_the_same_fx_block` must include the
   plate declarations in the byte comparison.
   `background_mode_is_absent_from_sources_without_a_background`,
   `background_html_supports_the_transparent_variant` and
   `auto_overlay_forwards_the_background_mode_to_its_scenes` must all pass.

**Gotchas:**

- The plate is a sibling layer, not a body background, in the eight scene
  overlays. A body reading `rgba(0, 0, 0, 0)` in opaque mode is correct.
- The standalone background is the reverse case: opaque is its default, so
  `?bg=transparent` is the variant that removes its plate.
- Opaque removes the hexagons by hiding `.fgc-fx`; it does not clear `data-fx`.
  Both attributes can be read at once.
- At the A75 gate the app toggle changed URLs only, so a connected OBS Browser
  Source kept the mode it loaded until it was refreshed. A76 replaced that with
  a cached `background` section; see the later section for the live path. Do not
  confuse either mechanism with the still-unbuilt `data-fx` injection hook.
- Timer and game overlay are excluded by the owner's A74 decision. A harmless
  ignored `bg` parameter from `auto.html` does not change that.
- The browser tab used for this gate measured 1000x700. The overlay's design
  canvas is 1920x1080; computed style and animation assertions do not depend on
  the tab size, but visual sign-off still needs a 1920x1080 OBS check.
- Reduced motion was not emulated. The existing media query is unchanged and
  remains guarded by the source-text assertion.
- Reformatting one canonical copy breaks the byte-equality test. Fix the copy,
  not the comparison.

**Evidence:** `hero-bans.html?bg=opaque` measured `dataset.bg: 'opaque'`,
`dataset.fx: 'on'`, one `.fgc-bg` at `display: block`,
`backgroundColor: 'rgb(11, 13, 18)'` and `inset: '0px'`, `.fgc-fx` at
`display: none` with `maskSize: '180px 322px'` and `opacity: '0.16'`, no drift
animation on the hidden layer, body `rgba(0, 0, 0, 0)`, `.overlay` `z-index: 1`,
`reduced: false` and an empty console. The same tab at `?bg=transparent`
measured `dataset.bg: 'transparent'`, `.fgc-bg display: none`, `.fgc-fx
display: block` and one `fgcCanonicalFxDrift` at 72000ms, with an empty console.
`background.html?bg=transparent` measured html/body/plate `rgba(0, 0, 0, 0)`,
two lattice layers and the three named animations, again with no console
errors. Both tabs were 1000x700. The 34-test overlay suite passed. A screenshot
of `hero-bans.html?bg=opaque` was captured at
`C:\Users\Cameron\AppData\Local\Temp\opencode-browser-gBzqQi\0\screenshot.jpeg`.

**Result: Pass - assistant 2026-11-23.** The two modes are distinguishable in a
real browser, opaque hides the hexagons without repainting the body, and the
standalone background accepts the inverse variant while keeping its lattice.
At this gate the app-level toggle changed generated URLs for the ten modes-aware
sources only, and the timer and game overlay remained excluded. The owner
reported that connected overlays did not change; A76 supersedes the URL-only
behaviour. Visual sign-off at 1920x1080 in OBS and browser-emulated reduced
motion remain outstanding owner checks.

### A76 - Live background broadcast and accessible switch

**Covers:** the owner's bug report that the background toggle did nothing, the
cached `background` section broadcast, the live message handlers in the eight
FX carriers and in `background.html`, the replacement of the app's Button with a
`Switch` inside a `ToggleRow`, and the preservation of the timer/game exclusion.

**Steps:**

1. Rebuild after changing an overlay file. In this run the app was stopped,
   `dx build --desktop` was run, and the app was restarted with
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
   --remote-allow-origins=*
   --user-data-dir=C:\Users\Cameron\AppData\Local\Temp\opencode\fgc-user-webview`.
   `/health` reported `{"overlays":13,"port":3001,"status":"ok","version":"1.0.0"}`.
2. Open `http://127.0.0.1:3001/overlays/hero-bans.html` and confirm
   `window.fgcUpdateBackground` and `window.updateOverlay` are both functions.
   With no parameter, read `document.documentElement.dataset.bg` as
   `transparent`, one `.fgc-bg` at `display: none`, one `.fgc-fx` at
   `display: block` and one `fgcCanonicalFxDrift` animation.
3. Evaluate
   `window.updateOverlay({type:'background', data:{opaque:true}})` in the same
   tab. Read `dataset.bg` as `opaque`, `.fgc-bg` as `display: block` with
   `backgroundColor: rgb(11, 13, 18)`, and `.fgc-fx` as `display: none`. Send
   `opaque: false` and confirm the transparent state returns. Read the console
   as empty in both states.
4. Open `http://127.0.0.1:3001/overlays/background.html`. With its opaque
   default, the document element, body and `.background` should read
   `rgb(11, 13, 18)` and `getAnimations()` should report `glowBreathe`,
   `latticeDriftForward` and `latticeDriftReverse`. Send the same `background`
   message with `opaque: false` and confirm all three surfaces become
   `rgba(0, 0, 0, 0)` while the three animations keep running; `opaque: true`
   restores the plate. The console must stay empty.
5. Inspect the Overlays view through CDP. Find the `Dioxus app` target in
   `http://127.0.0.1:9222/json/list` and read the control with
   `document.querySelector('[aria-label="Opaque background"]')`. It must expose
   `role="switch"`, `aria-checked` and `data-state`, not `aria-pressed`. The
   helper used here was
   `C:\Users\Cameron\AppData\Local\Temp\fgc-switch-on.js`; it reads the switch,
   clicks it, waits 300ms and reads the state again.
6. With the hero-bans overlay tab already connected, click the app switch and
   then read the overlay tab again. It must move to `bgMode: 'opaque'` with
   `.fgc-bg display: block` and `.fgc-fx display: none` without reloading the
   Browser Source. Clicking back restores transparent mode.
7. Confirm `crates/server/tests/overlays.rs` asserts the live handler in the
   byte-compared canonical block, the call from each carrier's `updateOverlay`,
   and the equivalent branch in `background.html`. Confirm the app state test
   checks that `server_plan` replays the `background` section before `show`.
8. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
   -D warnings`, `cargo test --workspace` and `git diff --check`. The overlay
   suite has 34 tests and the app suite has 88.

**Gotchas:**

- A URL reload is not a live switch. The cached section is what reaches a
  connected Browser Source; the query string is only the starting mode.
- The message payload is nested: `data.type === 'background'` with
  `data.data.opaque`. The overlays also accept a top-level `opaque` boolean for
  direct helper calls.
- The eight carriers call the shared helper from their existing `updateOverlay`
  function. Only the helper definition is inside the sentinel block; the
  one-line call is asserted separately per file.
- Opaque hides `.fgc-fx` but does not clear `data-fx`. Both attributes have
  independent meanings.
- Timer and game overlay remain excluded and receive no handler or layer. The
  harmless `bg` parameter forwarded by `auto.html` does not change that.
- The browser tabs measured 1000x700. Computed styles and state changes do not
  depend on viewport size, but the visual 1920x1080 OBS check is still an owner
  gate.
- Reduced motion was not emulated. The media query is unchanged and guarded by
  the source-text assertion.

**Evidence:** the hero-bans tab began at `bgMode: 'transparent'`, with one
hidden `.fgc-bg`, one visible `.fgc-fx` and one drift animation. After
`window.updateOverlay({type:'background', data:{opaque:true}})` it read
`bgMode: 'opaque'`, `.fgc-bg display: block`,
`backgroundColor: 'rgb(11, 13, 18)'` and `.fgc-fx display: none`; `opaque: false`
restored the drift layer. The background tab moved between the same plate colour
and `rgba(0, 0, 0, 0)` while all three animations continued. CDP found one
`BUTTON[role=switch][aria-label="Opaque background"]` with `aria-checked` and
`data-state`; the first click moved it to `true`, and a connected hero-bans tab
then read `bgMode: 'opaque'` with `.fgc-bg display: block` and `.fgc-fx display:
none` without a refresh. Console errors were empty in every check.

**Result: Pass - assistant 2026-11-23.** The toggle now changes connected
overlays straight away, uses the project's accessible Switch component and keeps
the generated URLs in step for new sources. Visual sign-off at 1920x1080 in OBS
and browser-emulated reduced motion remain outstanding owner checks.