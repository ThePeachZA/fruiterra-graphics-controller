# Manual QA and intervention register

This register collects every check that cannot be completed by CI or by an
automated agent. It covers the work shipped in M0–M9 and the release gates in
M10. The two authoritative checklists remain [`docs/overlay-qa.md`](overlay-qa.md)
and [`docs/build.md`](build.md) §7; this file records what is still outstanding
and why it needs a human.

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

## 2. Release-blocking gates

| ID | Gate | Milestone | What to do | Status |
|---|---|---|---|---|
| R1 | OBS overlay pass | M9 | Complete all 15 rows in `docs/overlay-qa.md` with seven Browser Sources at 1920×1080 and a loaded match. Record OBS, WebView2/CEF and app versions. | Pending |
| R2 | Offline font check | M9 | Disconnect the network, restart OBS and confirm every overlay still renders Saira from `/assets/fonts/`. | Pending |
| R3 | Drop-in overlay compatibility | M2/M9 | Point an original pre-rebuild overlay HTML at the new server. Confirm `init` paints immediately and live messages still update it. | Pending |
| R4 | Third-party timer overlay | M7/M9 | Connect a timer overlay and confirm `matchTimer` ticks and start/pause/reset actions arrive. | Pending |
| R5 | Clean-VM installer | M10 | Install, launch, start the server, load one overlay in OBS, close and uninstall on a clean VM. Confirm no installer or shortcut leftovers. | Pending |
| R6 | Portable ZIP | M10 | Extract on a machine with no install, run `fruiterra-gc.exe`, confirm the UI assets and all seven overlays load. | Pending |
| R7 | SmartScreen and checksum | M10 | Run the unsigned installer from a downloaded copy. Confirm the warning is expected and `SHA256SUMS.txt` verifies both artifacts. | Pending |
| R8 | WebView2 bootstrap | M10 | On a VM without WebView2, confirm the installer installs the runtime. Confirm the portable build fails with a clear message instead. | Pending |
| R9 | Tagged release publish | M10 | After R1–R8, tag `v1.0.0`. Confirm the workflow publishes the installer, portable ZIP, checksums and release notes. | Pending |

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
| M1-4 | A newer `schemaVersion` is refused with a readable error rather than silently overwriting data. | Requires editing the real save file. | Pending |

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
| M4-4 | Launching a second instance focuses or exits the first instance instead of running two servers. | Needs two real process launches. | Pending |
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
| M8-4 | Adding/removing rounds and matches clears dependent links after confirmation. | Destructive interactive flow. | Pending |
| M8-5 | Round-robin groups, fixtures, standings, ordered tiebreakers and head-to-head mini-tables behave correctly. | UI-level verification of tested rules. | Pending |
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
| M9-5 | Saira vs Archivo/Barlow visual review at 1920×1080 (`docs/fonts.md`). | Design judgment; only needed if Saira is questioned. | Pending |
| M9-6 | Tabular numerals, all-caps cap height/letter-spacing, roster layout, hero-ban and map-pool fit, 20+ character names, transparent background. | Visual judgment in OBS. | Pending |
| M9-7 | A 6-caster `row-casters` layout clips the social row by about 14px with long handles. | Observed in the WebView2 pre-check; the same clipping occurs with a `sans-serif` fallback. Tracked in the overlay polish backlog. | Known |
| M9-8 | Font licence appears in the README/notices and the bundled `OFL.txt` files ship beside the fonts. | Content check; partly done. | Partial |

### M10 — Packaging and release

| ID | Check | Why manual | Status |
|---|---|---|---|
| M10-1 | `docs/build.md` §7 release checklist items 1–12 are complete. | Mix of local commands and human gates. | Pending |
| M10-2 | The packaged `--smoke-test` passes on the release binary. | Automated in CI once M10 lands; run locally too. | Pending |
| M10-3 | The installer creates the Start Menu folder, desktop shortcut, uninstall entry and correct icon. | Installer UI. | Pending |
| M10-4 | Uninstall removes the install and shortcuts; confirm whether `%APPDATA%` user data is intentionally preserved. | Installer UI and data-retention policy. | Pending |
| M10-5 | The portable ZIP contains the executable, `assets\`, README, LICENSE and notices, and runs with no install. | Covered by R6. | Pending |
| M10-6 | The release workflow publishes both artifacts, checksums and notes on a tag, and only artifacts on manual dispatch. | Covered by R9. | Pending |
| M10-7 | `SHA256SUMS.txt` matches both artifacts. | Covered by R7. | Pending |
| M10-8 | The tag version equals the workspace version. | Workflow check; confirm on the first tag. | Pending |
| M10-9 | README and release notes match the shipped feature set and contain no backlog claims. | Editorial review. | Pending |

## 4. Manual intervention outside testing

| Item | Owner | Notes |
|---|---|---|
| OBS sign-off | Tester | The only gate the agent cannot substitute. |
| Clean-VM sign-off | Tester | Required before tagging `v1.0.0`. |
| SmartScreen acceptance | Maintainer | Unsigned v1; documented warning and checksums. |
| GitHub tag and release | Maintainer | Only after R1–R8 pass. |
| Font design review (Saira vs Archivo/Barlow) | Maintainer/designer | Optional; a five-file swap, no CSS change. |
| Overlay visual polish | Maintainer/designer | Backlogged; must preserve the current visual design and protocol. See `docs/backlog.md` §C. |
| Code signing certificate | Maintainer | Out of scope for v1; the workflow keeps a commented signing step. |

## 5. Sign-off log

| Date | Version | Environment | Items completed | Tester | Notes |
|---|---|---|---|---|---|
| | | | | | |
