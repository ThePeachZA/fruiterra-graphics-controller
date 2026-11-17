# Manual QA and intervention register

This register collects every check that cannot be completed by CI or by an
automated agent. It covers the work shipped in M0–M9 and the release gates in
M10. The two authoritative checklists remain [`docs/overlay-qa.md`](overlay-qa.md)
and [`docs/build.md`](build.md) §7; this file records what is still outstanding
and why it needs a human.

The step-by-step instructions, exact expected results and failure conditions are
in [`docs/manual-qa-runbook.md`](manual-qa-runbook.md). Use the runbook to run a
test and this register to record its status.

Status values:

- **Pending** — not run, or no signed-off record exists.
- **Partial** — an automated, WebView2 or UIAutomation smoke check passed, but
  the real-environment check remains.
- **Blocked** — the test cannot be run because a required input or environment
  does not exist. Record why in the status cell.
- **Done** — run and recorded with a date/version.

Update the status and add a row to the sign-off log when an item is completed.

## 1. Environments a human must provide

| Environment | Used for |
|---|---|
| OBS Studio (record the version) | Overlay rendering, Browser Source behaviour, reconnect, `show` |
| Clean Windows 10/11 x64 VM | Installer, portable ZIP, uninstall, SmartScreen, WebView2 bootstrap |
| VM without WebView2 | Installer bootstrapper and portable-build failure message |
| Second PC on the same LAN | `Allow LAN connections`, Windows Firewall, remote overlay loading |
| Offline network | Local font and asset loading with no internet |
| Interactive Windows desktop | Native file/folder dialogs, Explorer reveal, SmartScreen |
| 100%, 125% and 150% display scaling | UI legibility and clipping |
| Standard (non-admin) Windows account | Per-user NSIS install and uninstall |

### 1.1 Recorded test environment (2026-11-14)

The machine used for the R1 re-run and the local release checks:

| Item | Value |
|---|---|
| OS | Windows 11 Pro, version 10.0.26200, build 26200 |
| CPU / RAM | AMD Ryzen 9 5900X, 32 GB |
| Display | 1440p at 180 Hz, 100% scaling |
| OBS Studio | 31.0.4 |
| WebView2 runtime | 157.0.4341.57 (registered under `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients`; Microsoft Edge itself is not installed) |

Other environments in the table above (clean VM, no-WebView2 VM, second LAN
PC, offline network, non-admin account) still need to be provided for R5–R8.

App-UI checks (A-series) must run on a `dx build --desktop` executable. A plain
`cargo build` binary is not usable for UI checks: without the `dx` asset pass the
window renders unstyled and dialogs lay out in document flow.

## 2. Release-blocking gates

| ID | Gate | Milestone | What to do | Status |
|---|---|---|---|---|
| R1 | OBS overlay pass | M9 | Complete all 15 rows in `docs/overlay-qa.md` with seven Browser Sources at 1920×1080 and a loaded match. Record OBS, WebView2/CEF and app versions. | **Pass with notes** — 2026-11-14 re-run on the 1.0.0 fixture; no disconnects or flickers. Notes in §2.2. Rows 1–14 signed in `docs/overlay-qa.md`; row 15 is covered by R4. |
| R2 | Offline font check | M9 | Disconnect the network, restart OBS and confirm every overlay still renders the active overlay font (Archivo) from `/assets/fonts/`. | **Done** — 2026-11-15, repository owner. All seven overlays rendered correctly in Archivo with the network disconnected. |
| R3 | Drop-in overlay compatibility | M2/M9 | Point an original pre-rebuild overlay HTML at the new server. Confirm `init` paints immediately and live messages still update it. | **Blocked** — 2026-11-15, repository owner: no original pre-rebuild overlay HTML exists to test with. The wire protocol is covered by the M2 projection and server tests; drop-in rendering with an old file cannot be demonstrated. |
| R4 | Third-party timer overlay | M7/M9 | Connect a timer overlay and confirm `matchTimer` ticks and start/pause/reset actions arrive. | **Done** — 2026-11-15, repository owner. Count up, count down, count to target, start, pause, reset, alignment, leading zeroes and standalone all passed. R4-1 and R4-2 fixed in `e939e6b` (§2.3). |
| R5 | Clean-VM installer | M10 | Install, launch, start the server, load one overlay in OBS, close and uninstall on a clean VM. Confirm no installer or shortcut leftovers. | Pending |
| R6 | Portable ZIP | M10 | Extract on a machine with no install, run `fruiterra-gc.exe`, confirm the UI assets and all seven overlays load. | Pending |
| R7 | SmartScreen and checksum | M10 | Run the unsigned installer from a downloaded copy. Confirm the warning is expected and `SHA256SUMS.txt` verifies both artifacts. | Pending |
| R8 | WebView2 bootstrap | M10 | On a VM without WebView2, confirm the installer installs the runtime. Confirm the portable build fails with a clear message instead. | Pending |
| R9 | Tagged release publish | M10 | After R1–R8 (R3 accepted as Blocked), tag `v1.0.0`. Confirm the workflow publishes the installer, portable ZIP, checksums and release notes. | Pending |

## 2.1 R1 findings (2026-11-12)

First OBS pass on the fixture. Result: **Fail**. Root cause confirmed by
measuring the served overlays at 1920×1080.

| ID | Overlay | Finding | Cause | Disposition |
|---|---|---|---|---|
| R1-1 | Game overlay | Hero-ban strips render empty on both sides | `heroIcons` only includes heroes with an `icon` link; imported heroes have none, so the map is empty and the overlay draws nothing | **Fixed** — `heroIcons` falls back to the hero's own art or `portrait` link, and skips another hero's icon; quirks test added |
| R1-2 | Game overlay | `NORTHWIND SENTINELS` wraps to two lines and overflows the 55px bar | Name box is 293px wide with 30px text and no fitting rule | **Fixed** — single-line box with `fitText()`; fixture name now renders at 21px, `scrollWidth == clientWidth` |
| R1-3 | Game overlay | Team logos sit tight in the 75px box | Background fills the whole box | **Fixed** — `padding: 6px` + `background-origin: content-box` |
| R1-4 | Map select | Second line of `ANTARCTIC PENINSULA` is left-aligned, so the card looks off-centre | The name shrinks to content for one line but fills 208px for two, and inherits left alignment | **Fixed** — name, type and chosen-by are centred; name `line-height: 1.15` |
| R1-5 | Map select | Overlay appears off-centre | Canvas is exactly 1920×1080 and the maps row is centred; the enlarged current card shifts the visual weight | **Expected** — no change; measured canvas and row centre |
| R1-6 | Map select | Watermark looks stretched | Text measures 367.6px + 80px padding; no transform or letter-spacing is applied | **Expected** — font metrics, no change |
| R1-7 | Hero bans | Shows only map 1 bans | The overlay is current-map only by design | **Expected** — runbook expectation corrected |
| R1-8 | Casters | Handle renders as `@@arc` | Overlay prepends `@` to a stored handle that already includes it | **Fixed** — `@` is added only when missing; fixture shows `@arc`, `@priya`, `@marco`, `@sam` |
| R1-9 | Map pool | Pool is visible immediately, then fades a second time at 1.2s | A second render sets inline `opacity: 1` before the delayed animation class lands | **Fixed** — CSS-driven reveal; no inline opacity; delays 800ms/1200ms, `fill: forwards` |
| R1-10 | Rosters | Long player names wrap and change card height; substitutes differ | Player name has no fitting rule and the info block is not fixed height | **Fixed** — shared `fitText()`; core info block fixed at 106px; `ДМИТРИЙ ВОЛКОВ` shrinks to 25px; long sub name ellipsizes at 14px |
| R1-11 | All | Saira does not read like Apotek | Font substitution decision from M9 | **Trial** — all seven overlays now load Archivo; Saira retained for A/B review (`docs/fonts.md`); M9-5 re-run |
| R1-12 | All | Long names need a hard limit, not just visual fitting | Raised during the pass; fitting alone leaves unbounded input | **Fixed** — team 32, player 24, talent 24 characters; `maxlength` in the UI, `RepairKind::NameTooLong` on load |

Design changes raised during the pass that are **not** v1 fixes (map-mode icons
on map select, hero-ban redesign, merged Map Selection/Hero Bans tab, caster
camera/audio integration) are tracked in `docs/backlog.md`.

### 2.2 R1 re-run (2026-11-14)

Re-ran the full pass on the fixture in the environment recorded in §1.1 after
the R1 fix batch (`0e1de62`). Result: **Pass with notes** on every overlay. All
seven Browser Sources stayed connected for the 10-minute watch with no visible
flicker or lag.

| Overlay | Result | Notes |
|---|---|---|
| Game overlay | Pass | Ban strips render. Custom hero icons can now be linked in Assets (R1-15); the current fallback still shows hero art until an icon is linked. |
| Map select | Pass | |
| Hero bans | Pass | Current-map only, by design (R1-7). |
| Map pool | Pass | Per-mode row animation requested for later; backlog D5. |
| Home / away rosters | Pass | Player cards autofill request for later; backlog D6. |
| Casters | Pass | Handles render with one `@`; YouTube, Instagram and TikTok icons now supported (R1-13). |
| All seven | Pass | Corner fringing fixed (R1-14); the OBS animation caveat is recorded in the runbook. |

| ID | Overlay | Finding | Cause | Disposition |
|---|---|---|---|---|
| R1-13 | Casters | No platform symbol for the YouTube and Instagram handles | `getSocialIcon()` only mapped `twitter` and `twitch`; an unmapped platform still reserved an empty 20px icon slot | **Fixed** — all five `SocialPlatform` variants have SVGs; an unmapped platform no longer leaves an empty slot |
| R1-14 | All | Small light fringes at the corners of rounded image blocks | Square image layers inside a rounded, clipped card; Chromium anti-aliases the two shapes slightly differently | **Fixed** — `clip-path: inset(0 round 6px)` on the card plus a matching top radius on the image layer (rosters, casters, map pool) |
| R1-15 | Game overlay | No way to use a custom hero icon in a ban strip | The Assets view could import and rename only; `Asset.icon` had no editor | **Fixed** — Hero assets get a **Ban icon…** link editor; linked icons are excluded from hero pickers and ban lists |
| R1-16 | Map pool | No way to link an imported map to a game mode (C14) | Same gap; `Asset.game_mode` had no editor | **Fixed** — Map assets get a **Game mode…** link editor; unlinked maps stay under **Other** |

OBS caveat: with **Shutdown source when not visible** off, OBS keeps the page
loaded, so CSS entrance animations play once and do not replay when the scene
becomes active again. Turn on **Refresh browser when scene becomes active** if
the show needs the entrance animation on every scene entry. The runbook and the
OBS wiki page now record this.

### 2.3 R4 findings (2026-11-15)

First third-party timer run. Count up, count down, start, pause, reset,
alignment, leading zeroes and standalone all worked. **Count to target failed:**
the timer stayed at `00:00:00` and never ticked.

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| R4-1 | Timer | Count to target never runs; the app clock stays at `00:00:00` and no tick is broadcast | The countdown was evaluated with `document::eval` and a trailing expression. Dioxus runs a script as an async function body, so the script returned `undefined`; the result failed to deserialize into `i64`, the error was discarded, and the zero remaining time stopped the timer immediately | **Fixed** — the script is built by `target_countdown_script` and returns explicitly; an eval failure is logged instead of discarded; a unit test pins the `return` |
| R4-2 | Timer | No feedback when Count to target has no target set | **Start** ran with an empty target, which computes zero and stops at once | **Fixed** — **Start** is disabled while Count to target has no target, and the field carries a short hint |

Verified on the `dx build --desktop` binary: with a target four minutes ahead the
clock counted down second by second (for example `69:03:17` → `69:03:13` for a
later typed target), and **Start** was disabled until a target existed. No
`timer target eval failed` warning appeared in the log.

Re-check on 2026-11-15: **Pass**. The third-party page received `matchTimer`
once per second while counting down to a target about five hours ahead
(`"hours":5,"minutes":4,"seconds":26` → `19`), and the same run passed count up,
count down, start, pause, reset, alignment, leading zeroes and standalone. One
tick was delivered one second late (a `22` → `20` gap); the payload still carried
the absolute remaining time, so nothing accumulated. Gate R4 is closed.

### 2.4 App-UI test results (A-series)

The A-series runbook is §5 of
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md). Tests run on this machine
use the `dx build --desktop` executable and the recorded environment in §1.1.
Results are recorded here as each test completes.

| ID | Test | Covers | Result | Evidence |
|---|---|---|---|---|
| A1 | Save, restart, corrupt save and newer schema | M1-1, M1-2, M1-4, M4-6 | **Pass** — 2026-11-15, run by the assistant at the repository owner's request | Subtitle edit reached `app.json` within two seconds (`"subtitle":"Save test"`, mtime 18:30:39). Restart preserved `Save test`, `Grand Final`, 32 teams, 4 talents, 20 assets and 3 tournaments. A corrupt `app.json` loaded `app.json.bak` and logged `state file was unreadable; loaded the backup`. With `app.json.bak` removed and `schemaVersion` raised to 2, the app showed `Could not load saved state: state schema version 2 is newer than the supported version 1` with a **Dismiss error** action, and the file hash `89C0F4FF99CB744B` was unchanged after closing. |
| A2 | Logs | M1-3 | **Pass** — 2026-11-15, repository owner | `logs\fruiterra-gc.log.2026-11-15` exists, is plain ASCII text, and records every start (`INFO fruiterra_gc: starting application version="1.0.0"`) plus real events such as the A1 backup warning. Five daily files remain (2026-11-11 to 2026-11-15), under the seven-file cap; the cap itself is not yet exercised. |
| A3 | Third-party overlay with `?port=` | M2-2 | **Pass** — 2026-11-15, repository owner | App server on `127.0.0.1:3010` (`/health` reported `port: 3010`). The overlay was served from a second origin (`tailscale serve` → a local Node server over `crates\server`) at `.../overlays/game-overlay.html?port=3010`. It painted `NORTHWIND SENTINELS 2 – 1 王小明战队`, `GRAND FINAL` and `SAVE TEST`, fetched logos from `127.0.0.1:3010` (not 3001), logged no WebSocket errors, and updated after a score change. Fonts loaded from the same second origin. Port restored to 3001 and serve config removed afterwards. |
| A4 | Inline data URL asset mode | M2-3 | **Pass** — 2026-11-16, run by the assistant at the repository owner's request | **Settings → Asset URLs → Inline data URLs** applied live (no restart). The game overlay painted 12 logos and icons as CSS `background-image: url("data:image/png;base64,…")`; the home roster rendered 4 player portraits as `<img src="data:image/png;base64,…">`; all loaded with 0 broken. `/state` contained 334 `data:image/` URLs and no `/assets/` URLs. Switching back to **Served URLs (/assets/.)** re-projected both pages without a restart: the game overlay used 12 `http://127.0.0.1:3001/assets/…` backgrounds and the roster 4 `/assets/…` images, all loaded, 0 broken; `/state` returned to 52,121 bytes. **Note:** inline `/state`/`init` was 337,873,351 bytes (~322 MiB) for this 20-image (18.6 MB) fixture, so inline mode is heavy; the mode is session-only and the app returns to served mode on restart. Left on **Served URLs**. |
| A5 | LAN access and firewall | M3-1 | **Pass** — 2026-11-16, repository owner | **Allow LAN connections** bound `0.0.0.0:3001` and the Ethernet profile was set to **Private**. A second PC on `192.168.50.0/24` painted `NORTHWIND SENTINELS 2 – 1 王小明战队`, `GRAND FINAL` and `SAVE TEST`, and updated live after host score edits. This exposed D8-1: all seven overlays hardcoded `ws://localhost:<port>`, so the fix was pulled forward into v1 (origin-aware resolver, recorded in `docs/decisions.md`). Firewall was then reduced to one inbound rule — TCP, **Private** profile, `RemoteAddress=LocalSubnet`, program `fruiterra-gc.exe`; the UDP rule and the old auto-generated TCP rule (`Private, Public`, any address) were removed. The second PC still loaded and updated after the change. Turning LAN mode off returned the listener to `127.0.0.1:3001` and the LAN IP stopped answering. |
| A6 | Port already in use | M3-2 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | With the app server stopped, a Node HTTP server held `0.0.0.0:3001` (pid 28024). **Start server** showed `Server failed — failed to bind the overlay server to 0.0.0.0:3001: Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)` and the status stayed **Server failed**, not Running. The port was not changed: the base URL and error both stayed on 3001. After the Node process was stopped, **Start server** reported **Listening on** and `/health` returned `{"overlays":0,"port":3001,"status":"ok","version":"1.0.0"}`. |
| A7 | Per-file overlay override | M3-3 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | With the server running, `overlays\game-overlay.html` was replaced by a copy carrying a fixed `QA OVERRIDE` label. `/overlays/game-overlay.html` returned the marker (30,229 bytes) while `/overlays/map-select.html` did not, so only that file changed. An empty override and a non-UTF-8 override both fell back to the embedded page (30,069 bytes) and logged `overlay override is empty` / `overlay override is not valid UTF-8; falling back to the embedded file`; a file locked by another process also fell back (`os error 32`). A readable but malformed file was served as-is (62 bytes), matching the documented rule that a malformed override affects only that file. Deleting the override restored the embedded page. **A7-1:** the runbook expected malformed HTML to fall back, which the server cannot detect without parsing; added a guard for empty and non-UTF-8 overrides and corrected the runbook and wiki wording to **missing, empty or unreadable**. |
| A10 | `show` hide and show | M3-7 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | Ran in the in-app Chromium browser (no Edge/Chrome devtools needed). On the game overlay, `.overlay` was `display: block` with `Grand Final` and `Northwind Sentinels` painted; `updateOverlay({ type: 'show', visible: false })` set it to `display: none` and `visible: false`; `visible: true` restored `display: block`. Repeated on `team-roster-home.html` with the same results. No console errors on either page. |
| A13 | Error toasts and inline errors | M4-3 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | Every error was readable and named the fix. Port bind (A6): `failed to bind the overlay server to 0.0.0.0:3001: … (os error 10048)`. Invalid asset URL `not-a-url`: inline **That is not a valid URL.** Non-image URL: toast **Could not import the image: unsupported asset type; expected PNG, JPEG, WebP, GIF or SVG**. A 21.9 MB PNG URL: toast **The image is larger than the 20.0 MB limit.** A completed bracket match with 0–0 and a derived winner: inline **A completed match needs a winner. Set the scores or override the winner.** Port `0` + **Apply**: inline **Enter a port between 1 and 65535.** No message showed a panic, stack trace or internal path, every message could be dismissed, and the app stayed usable. |
| A14 | Second instance | M4-4 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | With the app open as pid 30868 and the server listening on `0.0.0.0:3001`, a second launch (pid 38824) exited on its own without opening a window. The process list stayed at one `fruiterra-gc.exe` (30868), the listener stayed on 30868, and the first instance kept running and serving. |
| A18 | Asset cleanup and overlays folder | M5-6, M5-7 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | Baseline **Unused asset data** was `0 file(s), 0 B` with 24 user blobs on disk. `logo-a.png` (196,957 B) was imported from `http://127.0.0.1:8099/logo-a.png`; the library went 34 → 35, **Logo** 5 → 6, the toast read `Imported "logo-a".`, and the blob landed at `assets\blobs\03\038be9a0….png`. Deleting it from the card (**Delete** → confirmation **Delete**) returned the library to 34 and Logo to 5 with `Asset deleted. Its data is cleaned up after the next save.`; after the debounced save the blob folder held 23 files (24 before import) with `03\038be9a0….png` gone. **Unused asset data** was back to `0 file(s), 0 B`. The referenced `Ilios` blob (`55\5525cfb7….png`) was still present and the bundled `Peach` logo still listed. **Open overlays folder** opened Explorer at `C:\Users\Cameron\AppData\Roaming\Fruiterra\Fruiterra Graphics Controller\overlays`. |
| A16 | Asset import paths | M5-1, M5-2, M5-3, M5-4 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | **File picker:** `logo-a.png` (196,957 B) imported as **Logo** (`Imported "logo-a".`); a second picker cancelled with **Cancel** left the library unchanged, cleared the busy state and raised no error. `safe.jpg` (1,261 B), `safe.webp` (30,320 B) and `safe.svg` (161 B) each imported and listed as **JPG**, **WEBP** and **SVG**. **URL:** `http://127.0.0.1:8099/logo-a.png` imported (library 39 → 40, **Logo** 9 → 10). `not a url` produced the inline **That is not a valid URL.** with the dialog still open; `…/not-an-image.txt` produced the toast **Could not import the image: unsupported asset type; expected PNG, JPEG, WebP, GIF or SVG**; `…/over-20mb.png` (21,876,098 B) produced **The image is larger than the 20.0 MB limit.** Neither rejection changed the library. **Unsafe SVG:** `unsafe.svg` was rejected with **Could not import the image: unsafe SVG: <script> elements are not allowed**. **Large image:** `large-under-20mb.png` (18,755,423 B) imported as **Logo**; a DOM query issued immediately after submit returned the **Working…** state, so the UI thread stayed responsive while the file was copied. **Deduplication:** importing the same PNG twice added one blob file only (`assets\blobs\03\038be9a0….png`, blob folder 24 → 25 for both imports) but listed a second library entry with the same content. **Links:** **Sojourn → Ashe** changed `/state` `heroIcons.Sojourn` to Ashe's URL, removed `Ashe` from `heroIcons` and from the Match Controller `fgc-hero-names` datalist, and repainted Sojourn's `.team-bar__ban-icon` on `game-overlay.html` to Ashe's art without a reload; **Use hero art** restored Sojourn's own art, Ashe's entry and the datalist. Nepal → **No game mode** moved it into an **Other** group on `map-pool.html`; **Control** regrouped it with Ilios and Antarctic Peninsula. Every test asset was deleted afterwards; the library returned to 34, the blob folder to 23 files and **Unused asset data** to `0 file(s), 0 B`. **Notes:** the runbook asked for "one copy" in the library, which is stricter than the documented byte-level dedup (AS1) — see §2.6; the fixture holds two hero assets named `Ana`, so `heroIcons.Ana` resolves to the later entry and the link test used unique-named heroes to avoid that ambiguity. |
| A22 | Force update, copy URL and live updates | M7-3, M7-4 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Steps 1–5 ran in the in-app Chromium browser on all five match overlays instead of OBS (R1 already covers OBS), with a `window.__calls` counter wrapping `updateOverlay`. A subtitle edit to `A22 LIVE CHECK` reached the game, roster and casters pages (6 messages each); **Force update** doubled every counter from 6 to 12 with no reload; the current map moved Ilios → Nepal and `map-card--current` followed; Map 1 **Chosen by** Team A → Team B changed `NWS'S PICK` → `WXM'S PICK`; the Map 2 Team B ban changed Tracer → Sojourn in the hero-bans `alt`; caster 1 → Priya repainted the casters row as `Priya, Priya, Marco, Sam`. Steps 6–7: **Config → Overlay URLs → Copy URL** on the game overlay put `http://127.0.0.1:3001/overlays/game-overlay.html` on the clipboard, matching the configured port and filename. **Note:** the run exposed saved values missing from dynamically generated `<select>` options; fixed in this pass (§2.5). |
| A23 | Scores and match rules | M7-5 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Fixture: Grand Final, NWS vs WXM, first to 3, Ilios 2–1, Nepal 1–2 and King's Row 3–2 complete. With **Manual score override** off the derived score was `2–1`; marking Dorado **Complete** at `0–2` made it `2–2`; a `1–1` draw on the completed Dorado was ignored (`2–1`), and restoring `0–2` returned `2–2`. Override on with `3–1` showed `3 – 1` in `/state` and the game overlay; override off returned the derived `2–2`. **First to 4** changed the overlay footer to `FIRST TO 4`, restored to `3`. **Side indicator** Neutral set `/state` `side: 0` and added `hidden` to both `.team-bar__side` elements; Defend/Attack (`side: 2`) swapped the attack and defend icon URLs between the bars; Attack/Defend (`side: 1`) restored the original pairing. **Loser picks** off set `loserPicks: false` and made the unplayed Antarctic Peninsula card show `NWS's Pick` on `map-select.html`; back on hid it again. With a temporary `3–1` override, **Swap sides** moved `.team-bar--home` from `43px` to `1348px` and `.team-bar--away` the other way, added `swapped` to both, so the left bar showed WXM `1` and the right NWS `3`; turning it off and clearing the override restored NWS `2` on the left and WXM `2` on the right. Fixture restored to derived `2–2`, side 1, loser picks on, swap off, first to 3. **Note:** overlay checks ran in the in-app Chromium browser rather than OBS (R1 already covers OBS). |
| A20 | Match Controller tabs | M7-1 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | All seven tabs exercised on the fixture. **Match:** title `A20 TITLE`, subtitle `A20 SUBTITLE`, first-to 4, status `post`, side 2, loser picks off, swap sides on and a manual `5–4`; `game-overlay.html` showed both texts, `LONG 5 – 4 WXM`, both bars `swapped` and `FIRST TO 4`. **Map Selection:** a sixth map (`A20 MAP`, `Push`, chosen by A, `1–0`) was marked **Complete** and **Current map**, then reordered, removed and replaced as current; marking Ilios complete again and moving **Current map** to King's Row kept `currentMap` and the `map-card--current` class in step; **Reset all maps** restored 8 empty slots and `currentMap: null`, cleared every ban and kept the `5–4` manual score, and the overlay fell back to seven `0 - 0 TBC` cards. **Hero Bans:** first ban Team A with Ana and Tracer showed `FIRST Ana … Tracer SECOND` and both portraits on `hero-bans.html`; **Reset all bans** cleared all eight rows and the overlay showed `-`. **Talent:** every slot assigned (Arc, Priya, Marco, Sam, Arc, Priya plus interviewee custom name/subtext); `row-casters.html` showed the six expected names. **Map Pool:** unticking and re-ticking Dorado removed and re-appended it, a custom `A20 POOL MAP` was added, moved up and removed, and an empty **Add** did nothing; `map-pool.html` grouped Control/Escort/Hybrid but never showed the custom name (A20-1). **Timer:** **Count to target** with no target disabled **Start** and showed the hint; a `2027-01-15T20:30` target enabled it, running showed `1414:18:50`, **Pause** stopped it and **Reset** returned `00:00:00`. **Config:** **Team colours** off removed `team-colored` from both bars; **Player portraits** off switched the roster `.hero-image` from the portrait (`a404e7e9…`) to hero art (`e9c02444…`); **Choose logo…** set the tournament logo to Peach and the watermark to Curro and both `.match-bug__logo` backgrounds changed. **Invalid input:** manual score `abc` was rejected and stored `0`, an empty pool name was a no-op, and an empty timer target disabled **Start**; no other inline validation exists on this page. Switching tabs preserved every value. **Note:** overlay checks ran in the in-app Chromium browser rather than OBS (R1 covers OBS); the interviewee custom name and subtext are stored but no v1 overlay shows them (A20-2). |
| A24 | Tournament board usability and keyboard swap | M8-1, M8-2 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | **Board Stress** generated a single-elimination bracket from 32 entrants: `Round 1` (16 matches), `Round 2` (8), `Quarterfinal` (4), `Semifinal` (2) and `Final` (1), 62 slot buttons across 31 matches and 32 entrant slots in Round 1. **Zoom:** at 50% the grid was 680 px in a 680 px scroller (no horizontal overflow) with 8 px slot names; at 100% the grid was 1128 px with 460 px of horizontal scroll and 16 px names; at 150% the grid was 1692 px with 691 px of scroll and 24 px names. The page content scrolled vertically (1623 px in a 721 px viewport) at every zoom, and one long name was ellipsised rather than wrapped. Rounds render as columns with no connector lines, matching the approved layout. **Click/click:** Northwind Sentinels (Round 1 M1 slot 1) swapped with Stress 6 (M2 slot 1), the selection cleared and the swap reached `app.json`. **Click twice:** the selection on Team 4 (M3 slot 1) cancelled and the slot kept its team. **Keyboard:** Tab reached the slots; Enter picked up `9Team 5` (M4 slot 1), ArrowDown moved focus to M5 slot 1, Enter dropped and the two slots swapped; Space repeated the same sequence between M6 and M7. **Escape:** Enter on M8 slot 1 then Escape cleared the selection and left both labels unchanged. **No duplicates:** swapping two already-bracketed teams left 32 filled slots, 32 unique teams and no duplicate. **Note:** the runbook's step 3 still mentioned connectors; it now describes the column layout. |
| A25 | Tournament match dialog and rounds | M8-3, M8-4 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Opened **Edit** on winners Round 1 M1 and set Slot 1 `Northwind Sentinels`, Slot 2 `王小明战队`, scores `2–1`, status Completed, first to 3 and a manual **Winner** (slot 1); set **Winner advances to** `Winners bracket · Semifinal · M1` slot 2 and **Loser drops to** `Winners bracket · Final · M1` slot 1; saved. The board showed `M1 Completed 2 – 1` with the **Manual winner** badge and `app.json` stored the scores, status, `winnerManual`, `next` and `loserNext`. The target slots first stayed empty because both chosen slots were already fed by another match (A25-1); after the fix the Semifinal M1 slot 2 held Northwind Sentinels and Final M1 slot 1 held 王小明战队 on load and after save. Clearing the manual winner switched the picker to **Derived from scores**, removed the badge, kept the 2–1 winner and kept both target slots filled. Marking Round 1 M2 **Completed** at `0–0` with no winner showed the inline **A completed match needs a winner. Set the scores or override the winner.** and left the dialog open and M2 Pending. **Reset result** on M1 returned it to `Pending 0 – 0` and cleared the Semifinal and Final slots it had filled. Added a round (`Round 6`), renamed it `Consolation`, added a match with **+ Match**, linked Round 1 M3's winner into it, then **×** removed the round: the round and its match disappeared and M3's link was cleared. In **Autumn Invitational** the Grand final section has one round and its **Remove round** button was disabled, while every multi-round section's button was enabled. **Note:** the propagation defect found in step 4 is fixed (A25-1). |
| A26 | Groups and tiebreakers | M8-5 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Renamed **Group A** → **Group Alpha**; the card heading and saved `name` changed. Added **Stress 1** to the group (4 → 5 teams, fixtures regenerated) and removed it again (5 → 4 teams, its fixtures removed). Added a **Team 5–Team 1** fixture and saved it **Completed 3–1**: the standings showed Team 5 `P2 W1 D1 L0 PF5 PA3 Diff+2 Pts1` and Team 1 `P2 W1 D0 L1 PF6 PA3 Diff+3 Pts1`; Northwind Sentinels 2–2 Team 5 counted as a draw (`D1`, `Pts0`). With both teams on 1 point, moving **Map differential** above **Head-to-head** put Team 1 first (Diff +3 vs +2); moving **Head-to-head** back to first put Team 5 above Team 1 because Team 5 won their fixture. **Delete group** on Group B showed the second confirmation **Deleting this group also removes its 6 fixture(s) and results. This cannot be undone.** with **Cancel** and **Delete group**; Cancel restored the card. **Notes:** A26-1 and A26-2 were fixed during the run. |
| A27 | Playoff, champion and grand-final reset | M8-6 | **Pass** — 2026-11-17, run by the assistant at the repository owner's request | On **Group Stage**, **Qualifiers per group 2** and **Single elimination** produced Semifinal M1 `A1 Team 5 vs B2 Team 2` and M2 `B1 王小明战队 vs A2 Team 1` (cross-group seeding). The **Champion** picker set Team 5 and **Current** showed `Team 5` with the **Manual** badge (`championManual: true`); **Derived from the bracket** cleared it to **No champion yet** (`champion: null`, `championManual: false`). On **Autumn Invitational** (reset on, first to 3) the completed winners/losers brackets put 王小明战队 in Grand final slot 1 and Team 5 in slot 2; completing it 2–3 (the losers-bracket team wins) created a **Reset** round with the same two teams. Turning **Grand-final reset** off truncated that auto-filled Reset round. After regenerating the bracket and repeating every result with the toggle off, the 2–3 grand final produced only one **Grand final** round, no Reset, and the derived champion was Team 5. No second final was created. |
| A28 | Tournament reload, repair and team deletion guard | M8-7, M8-8 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Renamed **Group B** → **Group Beta**, closed the app, reopened it and the name survived (`groups` held `Group Alpha` and `Group Beta`). Backed up `app.json`, then set the Autumn Invitational grand final's `winner` to Team 3 (not in either slot) with `winnerManual: true`. On start the app repaired instead of crashing and logged `repaired state kind=BracketCompletedWithoutWinner 1 completed match(es) without a valid winner were reset`; the board showed **Grand final** `M1 Pending 2 – 3` with the winner cleared and the scores kept. **Delete team** on Team 5 listed `Autumn Invitational — entrants, winners bracket, losers bracket, grand final, champion`, `Group Stage — entrants, winners bracket, groups, group fixtures` and `Board Stress — entrants, winners bracket`. **Remove references and delete** took the library from 32 to 31 teams and left no field pointing at the id: the Autumn champion became null and its grand final Pending `2 – 3` with slot 2 empty, its five Team 5 wins were reset (four stayed Pending with their scores, one became a `0 – 0` structural bye), Group Alpha went from 4 to 3 teams with four fixtures keeping an empty side, Group Stage Semifinal M1 slot 1 and Board Stress Round 1 M5 slot 1 were cleared, and no match ended Completed without a valid winner or with two identical slots. The delete logged `repaired invalid state … 5 completed match(es) without a valid winner were reset`. The backup was restored afterwards: 32 teams, both groups, champion Team 5, grand final Completed `2 – 3`, and the next start logged no repair. **Notes:** A28-1 (the repair is logged twice) and A28-2 (deleting a team discards the bracket results it took part in). |
| A11 | Keyboard-only pass | M4-1 | **Pass with notes** — 2026-11-17, run by the assistant at the repository owner's request | Every step ran with the keyboard alone. Sidebar **Tab**/`Enter` visited all seven destinations with an accent focus ring on nav, tabs, selects and buttons. **Teams:** created and renamed a team, opened **Players**, added a player and edited every field including the role and position selects; the portrait picker trapped focus, **Escape** restored it and **Enter** selected; **Done**, delete-player and delete-team dialogs focused **Cancel**, trapped focus and restored the trigger. **Talent:** added and edited a member including the platform select and handle. **Match Controller:** all seven tabs with one field changed per tab (`firstTo` 3→4, map 1 `Ilios A11`, ban row 1 `Winston`, caster 1 `Priya`, map pool unticked `Ilios`, timer alignment right, `showPlayerPortraits: false`). **Tournament:** created and renamed a tournament (name commits on blur), added an entrant, picked up a board slot with **Enter**, moved with the arrow keys and dropped with **Enter**, and cancelled a second pick-up with **Escape**. The match dialog trapped focus (13 Tabs to **Save result**, Shift+Tab to **Reset result**), its delete confirmation was operable, and **Escape**/**Close** returned focus to **Edit**; the delete-tournament dialog returned focus to **Delete**. **Assets:** the category select and import-from-URL dialog were keyboard operable (URL field focused, Tab wrapped, **Escape** restored **Import from URL…**). **Notes:** A11-1 to A11-4 were found and fixed during the run; the live fixture was restored afterwards. |

### 2.5 A22 findings (2026-11-17)

The pass exposed one UI bug in saved-value selects. It did not affect the
overlay checks; the control UI showed the wrong option until the user picked a
value again.

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A22-1 | All views | A `<select>` whose saved value came from a list (Team A/Team B, first-to, talent slots, role, position, social platform, tournament format, status, groups, qualifiers, entrants, fixtures, bracket slots, links) showed its first option on first render instead of the saved one | `select.value` is a volatile attribute, so the interpreter writes it as a property before dynamic `<option>` children exist; the browser discards it | **Fixed** — every dynamically generated `<option>` now sets `selected: <bool>`, which is written after the option exists; static-option selects unchanged. Root cause and the rule are in `docs/decisions.md` |

Re-checked on the `dx build --desktop` binary: Match Controller **Team A**
`NWS — Northwind Sentinels` and **Team B** `WXM — 王小明战队`,
first-to `3`, talent `Priya`, `Priya`, `Marco`, `Sam`, interviewee `Sam`; Teams
role `damage` and position `sub`; Talent social platform `Instagram`; Tournament
format `round robin` and `double elimination`, first-to `3`, status `Setup`,
groups `2`, qualifiers `2`; bracket match dialog Slot 1 `Northwind Sentinels`,
Slot 2 `Team 6`, status `Pending`, first-to `3`, winner `Derived from scores`,
winner target `Winners bracket · Semifinal · M1`, loser target
`Losers bracket · Lower Round 1 · M1`. Every control opened on the saved value.

### 2.6 A16 findings (2026-11-17)

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A16-1 | Assets | The runbook expected a second import of identical bytes to leave one library entry; the library listed two entries with the same content. | Deduplication is byte-level by design (`docs/assets.md` AS1): one blob per unique content, but each import gets its own metadata entry so a file can be reused under another name, category or link. | **No code change** — runbook step 5 and its fail criterion now describe byte-level dedup. A duplicate-entry warning is optional and not in the v1 backlog. |
| A16-2 | Assets | With two hero assets both named `Ana`, linking one as a ban icon left `heroIcons.Ana` on the other asset's art. | `hero_icons` and `hero_portraits` are keyed by hero name, so the last asset with that name wins; the fixture stores hero art and a separate icon under the same name. | **No code change** — matches the name-keyed overlay contract. Recorded so link checks use unique hero names; a duplicate-name guard is not required for v1. |

### 2.7 A20 findings (2026-11-17)

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A20-1 | Map Pool | A custom map-pool name with no matching Map asset is stored in the pool but never appears on `map-pool.html`; the **Available maps** panel and the wiki implied it would. | `map_pool_data` drops pool entries it cannot match to a map asset — a deliberate parity quirk pinned in `docs/decisions.md` (§M2 quirks) and `crates/protocol/tests/quirks.rs`. | **No code change** — corrected the **Available maps** panel copy and `docs/wiki/Match-Controller.md` §Map Pool. A placeholder card for unknown names is a post-v1 option. |
| A20-2 | Talent | The interviewee custom name and subtext are saved in `currentMatch.talent.interviewee` but no overlay shows them. | v1 has no interview overlay; the original had none either. | **No code change** — the values persist for future use. Recorded so the empty result is not read as a regression. |

### 2.8 A25 findings (2026-11-17)

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A25-1 | Tournament board | Setting a manual winner with valid **Winner advances to** / **Loser drops to** targets left the target slots empty, even after a refresh. Links, scores, status and the manual winner persisted. | `set_link` allowed a second link into an already-fed `(match, slot)`. `propagate_once` iterates matches in board order, so a pending competing source wrote `None` over the completed source's team; `fill_structural_byes` had the same ambiguity through its `(match, slot)` map. | **Fixed** — `set_link` now takes over the slot by clearing the link that fed it before, `repair_links` keeps the first link in board order and clears later duplicates, and the match modal says that choosing a target slot replaces its previous source. Regression tests `linking_into_an_occupied_slot_takes_it_over_and_propagates` and `refresh_keeps_one_link_per_target_slot`; recorded in `docs/decisions.md` and `docs/tournament.md` §3.2/§6. |

### 2.9 A26 findings (2026-11-17)

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A26-1 | Groups / standings | With **Head-to-head** first, a team that won the head-to-head could still rank below its rival when both were level on league points. | `sort_rows_recursive` passed only `criteria[index + 1..]` into each tie it created, so head-to-head was not recomputed for the subgroup; the top-level mini-table result carried through. | **Fixed** — when the criterion is `HeadToHead`, recursion keeps `&criteria[index..]`, rebuilding the mini-table for every subgroup. New core test `head_to_head_first_recomputes_the_mini_table_per_tie`; recorded in `docs/decisions.md` and `docs/tournament.md` §4.3. |
| A26-2 | Groups / standings | A group member that was not a tournament entrant showed its raw GUID in the standings team column. | The row name looked only in `entrant_names`; non-entrant members fell through to `TeamId::to_string()`. | **Fixed** — falls back to the full `teams` list, the same lookup the member chips already used. |

### 2.10 A28 findings (2026-11-17)

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A28-1 | Logs / load | A repaired save file logs the same repair twice at startup. | The startup unused-asset scan (`refresh_unused`) loads the saved state again, so `migrate` → `validate_and_repair` runs a second time and logs the same repair. | **No code change** — the second load is read-only, repair is idempotent and nothing is persisted from it. Recorded so a duplicated warning is not read as two separate repairs. |
| A28-2 | Teams / tournaments | Deleting a team that had played bracket matches removed those results: the matches it won were reset, and one became a completed `0 – 0` bye. | `remove_team_references` clears the team's slots, winner and champion, then repair resets any Completed match without a valid winner; `fill_structural_byes` then awards the remaining side a bye and zeroes the score. | **No code change** — this is the documented **Remove references and delete** path and the reset count is logged. Refusing deletion once results exist is a post-v1 option. |

### 2.11 A11 findings (2026-11-17)

The keyboard-only pass found four focus defects. All four were fixed during the
run and re-checked on a fresh `dx build --desktop` binary.

| ID | Area | Finding | Cause | Disposition |
|---|---|---|---|---|
| A11-1 | All views / focus rings | Text fields showed no focus ring when reached with **Tab**. | The vendored `.dx-input:focus-visible { outline: none }` rule (CSS-module hash, specificity 0,2,0) beat the theme ring. | **Fixed** — `crates/app/assets/theme.css` adds `input[class^="dx-input"]:focus-visible` and `textarea[class^="dx-textarea"]:focus-visible` (specificity 0,2,1); the vendored component CSS is untouched. Re-checked: the team-name field showed `2px solid rgb(255, 122, 26)`. |
| A11-2 | Teams / roster | Focus fell to `BODY` when the inline player editor opened (after **+ Add player** or clicking a player card) and when **Done** closed it. | The editor replaces the player list, so the focused trigger is removed; nothing moved focus into the editor or back to the card. | **Fixed** — opening the editor focuses **Player name**; **Done** focuses the player's card. Verified for both the add and edit paths. |
| A11-3 | Tournament / confirmations and dialog | Inline confirmations (**Delete group**, **Delete match**) moved focus to `BODY`, ignored **Escape**, and their **Cancel** was reachable only by tabbing. The match **Edit** dialog also left focus on `BODY` on **Close**/**Escape** because `MatchDialog` is unmounted with `open: true`, so the vendored `FocusTrap::remove()` never runs. | The confirmation replaces its trigger in the DOM and the dialog unmounts without a focus-restore path. | **Fixed** — a shared `focus_element_later` helper polls on animation frames for the target id. Confirmations focus **Cancel**; **Escape** cancels the group confirmation and returns focus to **Delete group**; **Cancel** returns to the trigger; confirming focuses **+ Group** (group) or the **Board** tab (match). Closing or cancelling the match dialog focuses its **Edit** button. **Escape** in the match dialog closes the dialog and cancels a pending delete. |
| A11-4 | Tournament / board | Bracket slot buttons had no context in their accessible name: a screen reader heard only the seed and team. | `BracketSlotButton` rendered the visible seed and name but no `aria-label`. | **Fixed** — `slot_aria_label` builds e.g. `Winners bracket · M1 · slot 1: Northwind Sentinels`; empty slots read `… slot 1: —`. All 62 Board Stress slots expose it, and `aria-pressed` still tracks the pick-up. |


## 3. Milestone register

### M0 — Workspace and guardrails

| ID | Check | Why manual | Status |
|---|---|---|---|
| M0-1 | The NSIS installer opens with the approved display name and icon. | Installer UI is not rendered in CI. | Pending |
| M0-2 | Unsigned binaries trigger the documented SmartScreen warning. | Requires a downloaded binary and a real desktop. | Pending |

### M1 — Core domain, store and migration

| ID | Check | Why manual | Status |
|---|---|---|---|
| M1-1 | Corrupt `app.json` falls back to `app.json.bak` in the real app-data folder without losing the last good save. | Store tests use temp folders; real permissions and paths are not exercised. | Done — 2026-11-15, gate A1 (§2.4). |
| M1-2 | A change made immediately before closing the app is saved on shutdown. | Requires the real 500 ms saver loop and window close. | Done — 2026-11-15, gate A1 (§2.4). |
| M1-3 | Daily logs appear under `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\` and stop at seven files. | File-system side effect over several days. | Done — 2026-11-15, gate A2 (§2.4). Cap not yet reached (five files). |
| M1-4 | A newer `schemaVersion` is refused with a readable error when no usable backup exists, rather than silently overwriting data. | Requires editing the real save file and removing `app.json.bak`. | Done — 2026-11-15, gate A1 (§2.4). |

### M2 — Protocol and projection

| ID | Check | Why manual | Status |
|---|---|---|---|
| M2-1 | An unmodified original overlay renders from the new server's `init` and live frames. | Covered by R3; needs the original HTML and OBS. | Pending |
| M2-2 | `?port=<port>` connects a third-party overlay served from another origin. | Needs a second HTTP origin and a browser. | Done — 2026-11-15, gate A3 (§2.4). |
| M2-3 | `data_url` asset mode inlines images for a third-party overlay. | Needs the Settings toggle and a real overlay. | Done — 2026-11-16, gate A4 (§2.4). |

### M3 — Server

| ID | Check | Why manual | Status |
|---|---|---|---|
| M3-1 | `Allow LAN connections` binds `0.0.0.0` and a second PC can load an overlay after allowing Windows Firewall access on private networks. | Firewall and second-machine behaviour cannot run in CI. | Done — 2026-11-16, gate A5 (§2.4). |
| M3-2 | A port already in use shows a clear bind error and does not silently fall back. | Requires occupying the port with another process. | Done — 2026-11-17, gate A6 (§2.4). |
| M3-3 | A per-file override copied into `%APPDATA%\...\overlays\` changes only that overlay after an OBS reload. | Needs the real app-data folder and OBS cache behaviour. | Done — 2026-11-17, gate A7 (§2.4). |
| M3-4 | Seven simultaneous Browser Sources connect and stay connected without visible lag. | OBS/CEF resource behaviour. | Pending |
| M3-5 | Restarting the server while OBS is open reconnects and repaints every source. | Partially covered by the WebView2 harness; OBS remains. | Partial |
| M3-6 | Replacing a referenced image shows the new hash immediately, with no stale OBS cache. | Content-addressed URLs are tested; OBS caching is not. | Pending |
| M3-7 | `show` hides and shows each overlay when driven by a test page. | The app does not send `show`; needs the WebSocket test page. | Done — 2026-11-17, gate A10 (§2.4). |

### M4 — App shell and settings

| ID | Check | Why manual | Status |
|---|---|---|---|
| M4-1 | Every destination and dialog is usable with the keyboard alone; focus order and focus rings are correct. | UIAutomation smoke does not prove the whole interaction model. | Done — 2026-11-17, gate A11 (§2.4). A11-1 to A11-4 fixed. |
| M4-2 | The UI is legible and unclipped at 100%, 125% and 150% Windows scaling. | Requires display scaling changes and visual judgment. | Pending |
| M4-3 | Error toasts and inline errors are readable and do not leak internal paths. | Requires triggering real failures. | Done — 2026-11-17, gate A13 (§2.4). Bind, invalid URL, non-image, oversized image, invalid completion and invalid port errors were readable and dismissible. |
| M4-4 | Launching a second instance exits immediately instead of running a second window and server. | Needs two real process launches. | Done — 2026-11-17, gate A14 (§2.4). |
| M4-5 | Inter renders locally in the UI with no Google Fonts request. | Network inspection in the packaged app. | Partial |
| M4-6 | State survives an app restart, including settings, teams, talent, match and tournament data. | Needs the real app-data directory. | Done — 2026-11-15, gate A1 (§2.4). |

### M5 — Assets

| ID | Check | Why manual | Status |
|---|---|---|---|
| M5-1 | Native file picker imports PNG, JPG, WebP and safe SVG; cancelling leaves state unchanged. | Native dialogs cannot run in CI. | Done — 2026-11-17, gate A16 (§2.4). PNG, JPEG, WebP and a safe SVG imported; **Cancel** left the library unchanged. |
| M5-2 | URL import succeeds for a valid image and rejects invalid, oversized or unsupported URLs with a clear message. | Requires real network and UI feedback. | Done — 2026-11-17, gate A16 (§2.4). Valid URL imported; malformed, non-image and 21.9 MB URLs each produced a readable message and no library change. |
| M5-3 | A malicious SVG (script/event handler/external reference) is rejected in the UI. | Unit-tested; the user-facing path is not. | Done — 2026-11-17, gate A16 (§2.4). `unsafe.svg` was rejected with `unsafe SVG: <script> elements are not allowed`. |
| M5-4 | A large image near the size cap imports without freezing the UI. | Requires a real file and interactive timing. | Done — 2026-11-17, gate A16 (§2.4). An 18.8 MB PNG imported while the UI stayed responsive (`Working…` state answered a DOM query). |
| M5-5 | Export creates `fruiterra-project-<unix>/` with `app.json`, `export.json` and referenced blobs; import on a second machine restores it. | Cross-machine filesystem test. | Pending |
| M5-6 | Cleanup removes only unreferenced user blobs and keeps bundled and referenced assets. | Unit-tested; verify through the UI. | Done — 2026-11-17, gate A18 (§2.4). Deleting an imported logo removed its blob after the save; the referenced Ilios blob and bundled Peach logo remained. |
| M5-7 | "Open overlays folder" opens Explorer at the override directory. | Native shell integration. | Done — 2026-11-17, gate A18 (§2.4). Explorer opened at `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays`. |

### M6 — Teams and Talent

| ID | Check | Why manual | Status |
|---|---|---|---|
| M6-1 | Team, player and talent CRUD, roster reorder, colours and asset pickers work end to end. | Full interactive UI pass. | Pending |
| M6-2 | Deleting a referenced team lists every tournament and match reference; cancel leaves data intact and the explicit delete clears it. | Multi-view destructive flow. | Pending |
| M6-3 | Deleting talent clears current-match caster, panel and interviewer assignments. | Cross-view side effect. | Pending |
| M6-4 | Long, accented, Cyrillic and CJK names render in the UI and overlays. | Partially covered in the WebView2 overlay check. | Partial |

### M7 — Match Controller

| ID | Check | Why manual | Status |
|---|---|---|---|
| M7-1 | All seven tabs work interactively, and invalid input is handled as documented (clamped, ignored or blocked). | Full UI pass. | Done — 2026-11-17, gate A20 (§2.4). Every tab's controls saved and repainted the overlays; invalid input is clamped, ignored or blocked rather than shown inline. |
| M7-2 | Timer start/pause/reset, count-up/count-down/target, settings persistence and one-second ticks behave correctly. | Requires a running clock and tab switching. | Done — 2026-11-15, gate R4 (§2.3). |
| M7-3 | Force Update and copy-URL work, and the URL matches the configured port. | Clipboard and live-server interaction. | Done — 2026-11-17, gate A22 (§2.4). |
| M7-4 | Map, hero-ban, map-pool and talent edits update the overlays live. | WebView2 harness covered a subtitle edit; the remaining tabs need OBS. | Done — 2026-11-17, gate A22 (§2.4). Map selection, map pick, hero ban, caster and Force update repainted the in-app overlay pages. |
| M7-5 | Derived vs manual scores, draws ignored, first-to and side/swap/loser-pick rules behave as documented. | UI-level verification of tested rules. | Done — 2026-11-17, gate A23 (§2.4). Derived and manual scores, ignored draws, first-to, side indicator, loser picks and swap sides all matched `/state` and the overlay. |
| M7-6 | The timer is clearly control-only and a third-party overlay receives ticks/actions. | Covered by R4. | Pending |

### M8 — Tournament

| ID | Check | Why manual | Status |
|---|---|---|---|
| M8-1 | A 32-team double-elimination board is usable at 1080p with 50-150% zoom and scrolling. | Usability and visual density need a human. | Done — 2026-11-17, gate A24 (§2.4). The 32-entrant single-elimination board was readable at 50%, 100% and 150%, with horizontal and vertical scrolling at each level. |
| M8-2 | Click-to-swap and keyboard pick-up/drop (Enter/Space, arrows, Escape) work on bracket slots. | Interactive input model. | Done — 2026-11-17, gate A24 (§2.4). Click/click, Enter/arrows/Enter and Space swaps all worked, click-twice and Escape cancelled, and moving a bracketed team created no duplicate. |
| M8-3 | The match modal edits slots, scores, status, first-to, winner override and links; invalid input is rejected. | Full UI pass. | Done — 2026-11-17, gate A25 (§2.4). Every field saved; links now take over their target slot; a completed 0–0 match without a winner was rejected inline. |
| M8-4 | Adding and removing rounds and matches clears dependent links; removing a round is immediate. | Destructive interactive flow. | Done — 2026-11-17, gate A25 (§2.4). Add/rename round, **+ Match** and **×** worked; the removed round's inbound link was cleared; **Remove round** was disabled with one round left. |
| M8-5 | Round-robin groups, fixtures, standings, ordered tiebreakers and head-to-head tiebreaking behave correctly. | UI-level verification of tested rules. | Done — 2026-11-17, gate A26 (§2.4). Rename, add/remove team, fixture results, draws, tiebreaker reordering, head-to-head and the delete-group confirmation all behaved. A26-1 (head-to-head mini-table) and A26-2 (non-entrant name) were fixed during the pass. |
| M8-6 | Playoff generation, grand-final reset and champion override work from the UI. | Full UI pass. | Done — 2026-11-17, gate A27 (§2.4). Qualifier seeding, champion override/derivation and the reset toggle (reset on creates a second final, off does not) behaved. |
| M8-7 | Tournament data survives reload and malformed data is repaired rather than crashing. | Requires editing a real save file. | Done — 2026-11-17, gate A28 (§2.4). A group rename survived close/reopen; a completed grand final whose winner was outside both slots was reset to Pending on load with the repair logged and no crash. |
| M8-8 | The Teams deletion guard handles tournament entrants, slots, groups and champions. | Cross-view destructive flow. | Done — 2026-11-17, gate A28 (§2.4). The dialog listed entrants, winners/losers brackets, grand final, champion, groups and group fixtures, and **Remove references and delete** left no dangling id and a valid bracket. |

### M9 — Overlays and fonts

| ID | Check | Why manual | Status |
|---|---|---|---|
| M9-1 | Complete `docs/overlay-qa.md` in OBS. | Covered by R1. | Pending |
| M9-2 | Offline font check. | Covered by R2. | Pending |
| M9-3 | Original overlay drop-in compatibility. | Covered by R3. | Pending |
| M9-4 | Third-party timer overlay. | Covered by R4. | Pending |
| M9-5 | Archivo vs Saira visual review at 1920×1080 (`docs/fonts.md`); Barlow remains an untried third option. | Design judgment after R1 rejected Saira. | Pending |
| M9-6 | Tabular numerals, all-caps cap height/letter-spacing, roster layout, hero-ban and map-pool fit, names at the 32/24-character limits, transparent background. | Visual judgment in OBS. | Pending |
| M9-7 | A 6-caster `row-casters` layout clips the social row by about 14px with long handles. | Observed in the WebView2 pre-check; the same clipping occurs with a `sans-serif` fallback. Tracked in the overlay polish backlog. | Known |
| M9-8 | Font licence appears in the README/notices and the bundled `OFL.txt` files ship beside the fonts. | Content check; partly done. | Partial |

### M10 — Packaging and release

| ID | Check | Why manual | Status |
|---|---|---|---|
| M10-1 | `docs/build.md` §7 release checklist items 1–12 are complete. | Mix of local commands and human gates. | Pending |
| M10-2 | The packaged `--smoke-test` passes on the release binary. | Automated in CI; run locally too. | Done — local release binary and CI release run [43050860851](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/43050860851) (2026-11-11). |
| M10-3 | The installer creates the Start Menu folder, desktop shortcut, uninstall entry and correct icon. | Installer UI. | Pending |
| M10-4 | Uninstall removes the install and shortcuts; confirm whether `%APPDATA%` user data is intentionally preserved. | Installer UI and data-retention policy. | Pending |
| M10-5 | The portable ZIP contains the executable, `assets\`, README, LICENSE and notices, and runs with no install. | Covered by R6. | Partial — contents verified in the release workflow; running it without an install remains R6. |
| M10-6 | The release workflow publishes both artifacts, checksums and notes on a tag, and only artifacts on manual dispatch. | Covered by R9. | Partial — manual dispatch produced the workflow artifact, installer, portable ZIP and checksums; tag publishing remains R9. |
| M10-7 | `SHA256SUMS.txt` matches both artifacts. | Covered by R7. | Pending |
| M10-8 | The tag version equals the workspace version. | Workflow check; confirm on the first tag. | Pending |
| M10-9 | README and release notes match the shipped feature set and contain no backlog claims. | Editorial review. | Pending |

### Automated evidence (2026-11-11)

These checks passed locally and in CI; they do not replace the human gates above.

| Check | Evidence |
|---|---|
| `cargo fmt --all -- --check` | Local and CI run [43050843933](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/43050843933) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Local and the same CI run |
| `cargo test --workspace` | Local (all tests) and the same CI run |
| `cargo doc --workspace --no-deps` | Local and the same CI run |
| `THIRD_PARTY_NOTICES.md` regeneration | CI notices job, same run |
| `dx bundle --release --desktop --package-types nsis --features bundle` | Local and release run [43050860851](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/43050860851) |
| Packaged `--smoke-test` | Local release binary and the release run |
| Portable ZIP structure | Release run: `fruiterra-gc.exe`, `assets\`, README, LICENSE, notices |
| SHA-256 generation | Release run: two artifact hashes in `SHA256SUMS.txt` |
| Release candidate artifacts | Release run artifact `fruiterra-gc-1.0.0-windows-x64` (installer, portable ZIP, checksums) |

## 4. Manual intervention outside testing

| Item | Owner | Notes |
|---|---|---|
| OBS sign-off | Tester | The only gate the agent cannot substitute. |
| Clean-VM sign-off | Tester | Required before tagging `v1.0.0`. |
| SmartScreen acceptance | Maintainer | Unsigned v1; documented warning and checksums. |
| GitHub tag and release | Maintainer | Only after R1–R8 pass. |
| Font design review (Archivo vs Saira) | Maintainer/designer | Archivo is wired in as the R1 trial; a revert is a five-file `src:` swap plus the test prefix, no layout CSS change. |
| Overlay visual polish | Maintainer/designer | Backlogged; must preserve the current visual design and protocol. See `docs/backlog.md` §C. |
| Code signing certificate | Maintainer | Out of scope for v1; the workflow keeps a commented signing step. |

## 5. Sign-off log

| Date | Version | Environment | Items completed | Tester | Notes |
|---|---|---|---|---|---|
| 2026-11-15 | 1.0.0 | Windows 11 Pro 10.0.26200, OBS 31.0.4, WebView2 157.0.4341.57 | R1 (pass with notes), R2, R4 | Repository owner (ThePeachZA) | R1 re-run 2026-11-14 on the 1.0.0 fixture; rows 1–15 of `docs/overlay-qa.md` signed. R2 offline font check 2026-11-15: all seven overlays rendered Archivo with the network disconnected. R4 2026-11-15: third-party timer page received ticks and actions; count to target fixed in `e939e6b` and re-checked. |
| 2026-11-15 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A1, A2, A3 | Repository owner (ThePeachZA) | A1 save/restart, backup recovery and newer-schema refusal passed; M1-1, M1-2, M1-4 and M4-6 moved to Done. A2 logs passed: plain-text daily log, startup and events recorded, five daily files; M1-3 moved to Done. A3 cross-origin `?port=` passed over a Tailscale-served second origin; M2-2 moved to Done. |
| 2026-11-16 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A4 | Repository owner (ThePeachZA), run by the assistant | A4 inline data URL mode passed: both overlays rendered every image as `data:` URLs, `/state` used `data:image/…`, and switching back to served URLs re-projected live without a restart. M2-3 moved to Done. Inline `init` measured 337.9 MB with the 18.6 MB fixture; served returned to 52 KB. |
| 2026-11-16 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; second PC on the same LAN | A5 | Repository owner (ThePeachZA) | A5 LAN access passed. `0.0.0.0:3001`, Ethernet profile **Private**, second PC painted and updated live. Found that every overlay dialled `ws://localhost:<port>`, so D8-1 (origin-aware resolver) was pulled forward into v1; static tests updated. Firewall reduced to one inbound TCP rule on the Private profile with `RemoteAddress=LocalSubnet`; the UDP and old auto-generated TCP rules were removed, and the second PC still worked. LAN off returned the listener to `127.0.0.1`. M3-1 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A6 | Repository owner (ThePeachZA), run by the assistant | A6 port-in-use passed. With Node holding `0.0.0.0:3001`, **Start server** showed the bind error naming `0.0.0.0:3001` and `os error 10048`, stayed **Server failed**, and did not switch ports. After freeing the port, **Start server** reported **Listening on** and `/health` returned `ok` on 3001. M3-2 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A7 | Repository owner (ThePeachZA), run by the assistant | A7 per-file override passed. A marked `game-overlay.html` was served for that file only; the other six were unchanged. Empty, non-UTF-8 and locked overrides all fell back to the embedded page and logged a warning; a readable malformed file was served as-is; deleting the override restored the built-in. A7-1 fixed the runbook's "broken HTML" expectation by guarding empty and non-UTF-8 overrides and correcting the docs. M3-3 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A14 | Repository owner (ThePeachZA), run by the assistant | A14 second instance passed. A second launch exited without a window, the process list stayed at one `fruiterra-gc.exe`, the single listener stayed on the first pid, and the first instance kept serving. M4-4 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A10 | Repository owner (ThePeachZA), run by the assistant | A10 hide/show passed on the game overlay and `team-roster-home.html`: `visible: false` set `.overlay` to `display: none`, `visible: true` restored it, and neither page logged a console error. M3-7 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A22 | Repository owner (ThePeachZA), run by the assistant | A22 live updates passed. Subtitle, current map, map pick, hero ban and caster edits repainted the in-app overlay pages without a reload, and **Force update** doubled every message counter. **Copy URL** put `http://127.0.0.1:3001/overlays/game-overlay.html` on the clipboard. The pass found A22-1 (saved values missing from dynamic select options), fixed by adding `selected` to each generated option; first render re-checked across Match Controller, Teams, Talent and Tournament. M7-3 and M7-4 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A23 | Repository owner (ThePeachZA), run by the assistant | A23 scores and match rules passed. Derived score tracked completed maps (`2–1` → `2–2` after Dorado), a `1–1` draw was ignored, and manual `3–1` won while enabled and reverted cleanly. First to 4 reached the overlay, side Neutral/Defend-Attack changed visibility and icon pairing, loser picks off revealed the next pick on `map-select.html`, and **Swap sides** moved both bars and kept names and scores consistent. Fixture restored to derived `2–2`, side 1, loser picks on, swap off. M7-5 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A13 | Repository owner (ThePeachZA), run by the assistant | A13 error handling passed. Bind, invalid URL, non-image, oversized image, invalid bracket completion and port `0` each produced a readable message, all could be dismissed, and no message leaked a panic, stack trace or internal path. M4-3 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A18 | Repository owner (ThePeachZA), run by the assistant | A18 asset cleanup and overlays folder passed. An imported logo was deleted, its blob disappeared after the save, the unused count returned to `0 file(s), 0 B`, and a referenced Ilios blob plus the bundled Peach logo survived. **Open overlays folder** opened Explorer at the override directory. M5-6 and M5-7 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A16 | Repository owner (ThePeachZA), run by the assistant | A16 asset import paths passed with notes. PNG, JPEG, WebP and a safe SVG imported through the native picker, and a cancelled picker changed nothing. A valid URL imported while a malformed URL, a text file and a 21.9 MB image were rejected with readable messages, and an unsafe SVG was refused. An 18.8 MB PNG imported without freezing the UI. Duplicate imports shared one blob. A ban-icon link changed the overlay strip and hid the chosen art from the hero picker until **Use hero art** restored it; a game-mode link regrouped the map and **No game mode** returned it to **Other**. The runbook dedup wording was corrected (§2.6). M5-1 to M5-4 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary; in-app Chromium browser | A20 | Repository owner (ThePeachZA), run by the assistant | A20 Match Controller tabs passed with notes. Every tab's controls saved and repainted the overlays: title, subtitle, manual score, first-to and swap sides on the game overlay; added, reordered and removed map slots plus the current map on `map-select.html`; bans and portraits on `hero-bans.html`; the caster and panel row; the grouped map pool; the target timer; and the tournament and watermark logos. Both destructive resets cleared their data immediately, and the manual score survived **Reset all maps**. Invalid input is clamped, ignored or blocked rather than shown inline. A20-1 corrected the custom map-pool copy in the app and the wiki, and A20-2 records that the interviewee fields are not projected. M7-1 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, dx build --desktop binary | A24 | Repository owner (ThePeachZA), run by the assistant | A24 tournament board passed with notes. A generated 32-entrant single-elimination bracket showed five rounds and 62 slot buttons, was readable at 50%, 100% and 150% zoom, and scrolled horizontally and vertically. Click/click swapped two teams and persisted; a second click on the same slot cancelled; Tab reached the slots; Enter/ArrowDown/Enter and Space swaps both worked; Escape left the bracket unchanged. Moving an already-bracketed team kept 32 filled slots and 32 unique teams with no duplicates. Rounds render as columns without connector lines, so the runbook step now describes the column layout instead. M8-1 and M8-2 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A25 | Repository owner (ThePeachZA), run by the assistant | A25 tournament match dialog and rounds passed with notes. Slots, scores, status, first-to and a manual winner saved; links to Semifinal M1 slot 2 and Final M1 slot 1 saved and, after the A25-1 fix, filled the targets. Clearing the override returned the score-derived winner, a 0–0 Completed match without a winner was rejected inline, **Reset result** returned M1 to Pending and cleared its downstream slots, and add/rename/+ Match/× on a round worked while clearing the inbound link. **Remove round** was disabled in the one-round Grand final section. The A25-1 duplicate-incoming-link defect was fixed with two core regression tests. M8-3 and M8-4 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A26 | Repository owner (ThePeachZA), run by the assistant | A26 groups and tiebreakers passed with notes. Group rename, add/remove team, a completed 3–1 fixture and a 2–2 draw all produced the expected P/W/L/D/PF/PA/Diff/Pts. Two teams level on 1 point reordered when **Map differential** moved above **Head-to-head**, then followed the head-to-head result when it moved back to first. **Delete group** needed a second confirmation and **Cancel** restored the card. A26-1 fixed head-to-head so the mini-table is rebuilt inside each tie; A26-2 fixed a non-entrant group member showing its GUID. M8-5 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A27 | Repository owner (ThePeachZA), run by the assistant | A27 playoff, champion and grand-final reset passed. Two qualifiers per group seeded Semifinal M1 `A1 Team 5 vs B2 Team 2` and M2 `B1 王小明战队 vs A2 Team 1`. The champion picker showed Team 5 with the **Manual** badge and **Derived from the bracket** cleared it. On Autumn Invitational, a 2–3 grand final won by the losers-bracket team created a **Reset** round with reset on; after regenerating the bracket with reset off, the same result produced only one grand final. M8-6 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A28 | Repository owner (ThePeachZA), run by the assistant | A28 tournament reload, repair and team deletion guard passed with notes. A group rename survived close/reopen. A grand final whose saved winner was outside both slots was reset to Pending on load with `BracketCompletedWithoutWinner` logged and no crash. **Delete team** on Team 5 listed entrants, winners and losers brackets, the grand final, champion, groups and group fixtures across all three tournaments, and **Remove references and delete** left no dangling id, no duplicate slot and no Completed match without a valid winner. The backup was restored afterwards (32 teams, champion Team 5, grand final Completed `2 – 3`) and the next start logged no repair. M8-7 and M8-8 moved to Done. |
| 2026-11-17 | 1.0.0 | Windows 11 Pro 10.0.26200, `dx build --desktop` binary | A11 | Repository owner (ThePeachZA), run by the assistant | A11 keyboard-only pass passed with notes. Every destination, field, tab, dialog, picker and confirmation was reachable and operable with Tab/Shift+Tab/Enter/Space/Escape and the arrow keys; focus rings were visible throughout. Four defects were found and fixed: the vendored input focus override (A11-1), player-editor focus entry and return (A11-2), inline confirmation and match-dialog focus restore (A11-3), and bracket-slot accessible names (A11-4). Re-checked on a fresh `dx build --desktop` binary and the fixture was restored afterwards. M4-1 moved to Done. |
