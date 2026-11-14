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
| R1 | OBS overlay pass | M9 | Complete all 15 rows in `docs/overlay-qa.md` with seven Browser Sources at 1920×1080 and a loaded match. Record OBS, WebView2/CEF and app versions. | **Pass with notes** — 2026-11-14 re-run on the 1.0.0 fixture; no disconnects or flickers. Notes in §2.2. Formal OBS sign-off table still to be signed. |
| R2 | Offline font check | M9 | Disconnect the network, restart OBS and confirm every overlay still renders the active overlay font (Archivo) from `/assets/fonts/`. | Pending |
| R3 | Drop-in overlay compatibility | M2/M9 | Point an original pre-rebuild overlay HTML at the new server. Confirm `init` paints immediately and live messages still update it. | Pending |
| R4 | Third-party timer overlay | M7/M9 | Connect a timer overlay and confirm `matchTimer` ticks and start/pause/reset actions arrive. | Pending |
| R5 | Clean-VM installer | M10 | Install, launch, start the server, load one overlay in OBS, close and uninstall on a clean VM. Confirm no installer or shortcut leftovers. | Pending |
| R6 | Portable ZIP | M10 | Extract on a machine with no install, run `fruiterra-gc.exe`, confirm the UI assets and all seven overlays load. | Pending |
| R7 | SmartScreen and checksum | M10 | Run the unsigned installer from a downloaded copy. Confirm the warning is expected and `SHA256SUMS.txt` verifies both artifacts. | Pending |
| R8 | WebView2 bootstrap | M10 | On a VM without WebView2, confirm the installer installs the runtime. Confirm the portable build fails with a clear message instead. | Pending |
| R9 | Tagged release publish | M10 | After R1–R8, tag `v1.0.0`. Confirm the workflow publishes the installer, portable ZIP, checksums and release notes. | Pending |

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

## 3. Milestone register

### M0 — Workspace and guardrails

| ID | Check | Why manual | Status |
|---|---|---|---|
| M0-1 | The NSIS installer opens with the approved display name and icon. | Installer UI is not rendered in CI. | Pending |
| M0-2 | Unsigned binaries trigger the documented SmartScreen warning. | Requires a downloaded binary and a real desktop. | Pending |

### M1 — Core domain, store and migration

| ID | Check | Why manual | Status |
|---|---|---|---|
| M1-1 | Corrupt `app.json` falls back to `app.json.bak` in the real app-data folder without losing the last good save. | Store tests use temp folders; real permissions and paths are not exercised. | Pending |
| M1-2 | A change made immediately before closing the app is saved on shutdown. | Requires the real 500 ms saver loop and window close. | Pending |
| M1-3 | Daily logs appear under `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\` and stop at seven files. | File-system side effect over several days. | Pending |
| M1-4 | A newer `schemaVersion` is refused with a readable error when no usable backup exists, rather than silently overwriting data. | Requires editing the real save file and removing `app.json.bak`. | Pending |

### M2 — Protocol and projection

| ID | Check | Why manual | Status |
|---|---|---|---|
| M2-1 | An unmodified original overlay renders from the new server's `init` and live frames. | Covered by R3; needs the original HTML and OBS. | Pending |
| M2-2 | `?port=<port>` connects a third-party overlay served from another origin. | Needs a second HTTP origin and a browser. | Pending |
| M2-3 | `data_url` asset mode inlines images for a third-party overlay. | Needs the Settings toggle and a real overlay. | Pending |

### M3 — Server

| ID | Check | Why manual | Status |
|---|---|---|---|
| M3-1 | `Allow LAN connections` binds `0.0.0.0` and a second PC can load an overlay after allowing Windows Firewall access on private networks. | Firewall and second-machine behaviour cannot run in CI. | Pending |
| M3-2 | A port already in use shows a clear bind error and does not silently fall back. | Requires occupying the port with another process. | Pending |
| M3-3 | A per-file override copied into `%APPDATA%\...\overlays\` changes only that overlay after an OBS reload. | Needs the real app-data folder and OBS cache behaviour. | Pending |
| M3-4 | Seven simultaneous Browser Sources connect and stay connected without visible lag. | OBS/CEF resource behaviour. | Pending |
| M3-5 | Restarting the server while OBS is open reconnects and repaints every source. | Partially covered by the WebView2 harness; OBS remains. | Partial |
| M3-6 | Replacing a referenced image shows the new hash immediately, with no stale OBS cache. | Content-addressed URLs are tested; OBS caching is not. | Pending |
| M3-7 | `show` hides and shows each overlay when driven by a test page. | The app does not send `show`; needs the WebSocket test page. | Partial |

### M4 — App shell and settings

| ID | Check | Why manual | Status |
|---|---|---|---|
| M4-1 | Every destination and dialog is usable with the keyboard alone; focus order and focus rings are correct. | UIAutomation smoke does not prove the whole interaction model. | Partial |
| M4-2 | The UI is legible and unclipped at 100%, 125% and 150% Windows scaling. | Requires display scaling changes and visual judgment. | Pending |
| M4-3 | Error toasts and inline errors are readable and do not leak internal paths. | Requires triggering real failures. | Pending |
| M4-4 | Launching a second instance exits immediately instead of running a second window and server. | Needs two real process launches. | Pending |
| M4-5 | Inter renders locally in the UI with no Google Fonts request. | Network inspection in the packaged app. | Partial |
| M4-6 | State survives an app restart, including settings, teams, talent, match and tournament data. | Needs the real app-data directory. | Pending |

### M5 — Assets

| ID | Check | Why manual | Status |
|---|---|---|---|
| M5-1 | Native file picker imports PNG, JPG, WebP and safe SVG; cancelling leaves state unchanged. | Native dialogs cannot run in CI. | Pending |
| M5-2 | URL import succeeds for a valid image and rejects invalid, oversized or unsupported URLs with a clear message. | Requires real network and UI feedback. | Pending |
| M5-3 | A malicious SVG (script/event handler/external reference) is rejected in the UI. | Unit-tested; the user-facing path is not. | Partial |
| M5-4 | A large image near the size cap imports without freezing the UI. | Requires a real file and interactive timing. | Pending |
| M5-5 | Export creates `fruiterra-project-<unix>/` with `app.json`, `export.json` and referenced blobs; import on a second machine restores it. | Cross-machine filesystem test. | Pending |
| M5-6 | Cleanup removes only unreferenced user blobs and keeps bundled and referenced assets. | Unit-tested; verify through the UI. | Partial |
| M5-7 | “Open overlays folder” opens Explorer at the override directory. | Native shell integration. | Pending |

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
| M7-1 | All seven tabs work interactively and validation errors are shown inline. | Full UI pass. | Pending |
| M7-2 | Timer start/pause/reset, count-up/count-down/target, settings persistence and one-second ticks behave correctly. | Requires a running clock and tab switching. | Pending |
| M7-3 | Force Update and copy-URL work, and the URL matches the configured port. | Clipboard and live-server interaction. | Pending |
| M7-4 | Map, hero-ban, map-pool and talent edits update the overlays live. | WebView2 harness covered a subtitle edit; the remaining tabs need OBS. | Partial |
| M7-5 | Derived vs manual scores, draws ignored, first-to and side/swap/loser-pick rules behave as documented. | UI-level verification of tested rules. | Pending |
| M7-6 | The timer is clearly control-only and a third-party overlay receives ticks/actions. | Covered by R4. | Pending |

### M8 — Tournament

| ID | Check | Why manual | Status |
|---|---|---|---|
| M8-1 | A 32-team double-elimination board is usable at 1080p with 50–150% zoom and scrolling. | Usability and visual density need a human. | Pending |
| M8-2 | Click-to-swap and keyboard pick-up/drop (Enter/Space, arrows, Escape) work on bracket slots. | Interactive input model. | Pending |
| M8-3 | The match modal edits slots, scores, status, first-to, winner override and links; invalid input is rejected. | Full UI pass. | Pending |
| M8-4 | Adding and removing rounds and matches clears dependent links; removing a round is immediate. | Destructive interactive flow. | Pending |
| M8-5 | Round-robin groups, fixtures, standings, ordered tiebreakers and head-to-head tiebreaking behave correctly. | UI-level verification of tested rules. | Pending |
| M8-6 | Playoff generation, grand-final reset and champion override work from the UI. | Full UI pass. | Pending |
| M8-7 | Tournament data survives reload and malformed data is repaired rather than crashing. | Requires editing a real save file. | Partial |
| M8-8 | The Teams deletion guard handles tournament entrants, slots, groups and champions. | Cross-view destructive flow. | Pending |

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
| | | | | | |
