# Backlog — post-parity and v1 correctness fixes

Living document. Ordered by priority within each section. IDs are stable for referencing in milestones.

## A. v1 correctness fixes (agreed as part of parity)

These are defects that currently make a shipped feature fail, risk data loss, or create an external runtime dependency. They are in scope for v1.

| ID | Fix | Detail |
|---|---|---|
| C1 | Local fonts | Replace all `https://design.penpot.app/...` `@font-face` URLs with locally served OFL fonts. Replace the Google Fonts `Inter` import in the UI with a bundled or system font. No runtime network dependency for fonts. Apotek is commercial (Kerns & Cairns) and cannot be redistributed; replace it with **Saira** (OFL 1.1, confirmed), behind neutral `FGC Display*` aliases. Review Archivo/Barlow once the overlays render. See `docs/assets.md` §4. |
| C2 | `currentMapIndex` off-by-one | Overlays tab “Force Update” sends `currentMapIndex + 1`; Match Controller sends the raw index. Standardise on the raw index (overlay adds 1 for display). |
| C3 | Stale asset caches | Team logo and player caches are never invalidated. Fixed structurally in `docs/assets.md`: assets are content-addressed, so a changed image gets a new hash and URL; the overlay cannot show a stale cached file. |
| C4 | Port source of truth | One configured port (default 3001), shared by Settings and Overlays; restart on change; clear error on bind failure; no silent fallback. |
| C5 | Broken menu URL | “Open Scoreboard” points at non-existent `scoreboard.html`; Map Pool is duplicated. If menus are retained, point at `game-overlay.html` and deduplicate. |
| C6 | `init` compatibility | Reproduce the keyed-map `init` shape exactly, including `scoreboard`, `teamRoster`, `casters` etc., so existing overlays render immediately on connect. |
| C7 | Atomic saves | Write `app.json` to a temp file and rename, so a crash/power loss cannot leave a truncated save. Keep one rotating backup. |
| C8 | Import validation | Validate imported match/app JSON; reject or repair unknown/invalid shapes instead of crashing. |
| C9 | Schema version | Add `schemaVersion` to `app.json` and a migration chain. Preserve unknown fields where safe. |
| C10 | Portable exports | Export an `app.json` plus only the referenced user blobs in a portable bundle, so backups restore on another machine. Bundled assets come from the binary. See `docs/assets.md` §1.7. |
| C11 | Safe path handling | Constrain file operations to the app-data/assets roots. Asset requests accept only a 64-character lowercase hex hash and an allow-listed extension; overlay overrides reject `..`, separators and non-`.html` names. See `docs/assets.md` §1.4 and §3. |
| C12 | Remove dead/incorrect state | `Match.scoreA/scoreB` writes, unused `matches[]`, `substitutions[]`, `SET_CHAMPION`, `rounds`, NDI fields: remove or wire up. No user-facing behaviour change. |
| C13 | Timer overlay decision | The timer broadcasts `matchTimer` but no overlay consumes it. Either ship a compatible timer overlay in v1 or explicitly mark the timer as control-only. |

### Font specifics (C1)

Apotek is a commercial typeface by Kerns & Cairns (Adobe Fonts / Type Network). The OTFs in the repo cannot be redistributed or self-hosted without a licence. Replace it with an OFL font.

Current remote mappings in the overlays (for reference):

| Family in CSS | PenPot asset ID | Local file available |
|---|---|---|
| `Apotek` | `930c6af4-df93-4ad1-bcc8-70f7dc350535` | **No regular OTF in repo** — closest is `ApotekMedium.otf` |
| `Apotek Light` | `ab6e05f8-3153-4842-823a-35e55273872a` | `ApotekLight.otf` |
| `Apotek Medium` | `1ba1930f-20e6-41b7-aa1c-b59ce1e844f7` | `ApotekMedium.otf` |
| `Apotek Semibold` | `1ba1930f-...` (same as Medium — never a real semibold) | `ApotekSemibold.otf` |
| `Apotek Black` | `937c3a15-8a62-408b-9be9-2f4404f76f30` | `Apotek_Black.otf` |

Recommended replacement: **Saira** (OFL 1.1), bundled as WOFF2 at weights 300/400/500/600/900, served from `/assets/fonts/`. Include `OFL.txt`. Use neutral `FGC Display*` aliases with a fallback stack (`'FGC Display', 'Apotek', sans-serif`). UI: bundle Inter (OFL) locally.

## A2. v1 Tournament completion (agreed — generated board + full manual editing)

The Tournament View is completed properly in v1. Full design: `docs/tournament.md`. Acceptance criteria T1–T14 replace the earlier TR1–TR10:

| ID | Requirement |
|---|---|
| T1 | Generate a valid single-elimination bracket for any team count, with byes to top seeds and standard seeding. |
| T2 | Generate a valid double-elimination board: winners, losers, drop mapping and grand final. |
| T3 | Generate round-robin fixtures per group and compute standings. |
| T4 | Every generated slot, match, round and link is editable; teams drag between slots with swap semantics. |
| T5 | Auto-advance fills the next slot and loser slot on completion; never overwrites a manually placed slot. |
| T6 | Clearing/editing a result recursively clears the auto-filled downstream slots it produced. |
| T7 | Grand-final reset honours `grand_finals_reset`; both the reset and the champion can be overridden manually. |
| T8 | Champion is set automatically from the final unless manually overridden. |
| T9 | Round-robin tiebreakers are configurable, ordered, persisted and correctly applied, including head-to-head mini-tables. |
| T10 | Optional playoff generation from group qualifiers with cross-group seeding, then fully editable. |
| T11 | Match modal: scores, status, first-to, winner override, link editing, reset. |
| T12 | Full persistence/restore; standings recomputed; malformed data repaired rather than crashing. |
| T13 | Unit tests for generation, propagation, override, reset, tiebreakers and repair. |
| T14 | Keyboard-accessible slot movement and a usable board at 1080p with 32 teams. |

**Key semantics:** generation is a starting point, not a rule. The board is fully editable at any time, including after play starts. Auto-advance is on by default and is suppressed per slot once a slot is placed by hand. Groups can be edited mid-event; played results are retained.

**Out of scope for v1** unless pulled forward: bracket overlay, forfeits as a result type, multi-stage playoffs, custom points, match history, external tournament-site import. These remain in the post-v1 list.

## B. v1 candidates (not committed — none are in v1)

These are small, isolated additions that would be cheap to pull forward *after* v1 is running. They are **not** part of the v1 milestone.

| ID | Item | Why it might be worth pulling forward later |
|---|---|---|
| B1 | Show/hide overlay control | All overlays already implement `{type:"show", visible}`; a UI toggle is low effort and useful live. |
| B2 | All overlay URLs with copy buttons | Current UI exposes only the game overlay URL; the rest are hidden in the menu. |
| B3 | Overlay health/reconnect indicator | Surface connected client count and last broadcast time. |
| B4 | Basic hotkeys | Score +/-, next map, show/hide — high value during a live show. |
| B5 | Timer overlay | Makes the existing Timer tab actually usable on stream. |

## C. Post-v1 improvements (option 3 backlog)

### C0. Requested for later (confirmed 2026-11-06 — none in v1)

| ID | Feature | Scope sketch |
|---|---|---|
| P1 | **Undo / redo** | Command/history layer over the state store: every mutating action pushes an inverse (or a pre-state snapshot, bounded to N steps). Must not undo server start/stop or persistence itself. UI: Ctrl+Z / Ctrl+Shift+Z plus toolbar buttons. No protocol change. |
| P2 | **Presets / show templates** | Named, user-saved snapshots of match config (teams, rosters, map pool, casters, overlay settings) stored alongside `app.json`; load/duplicate/delete/rename; “save current as preset”. Needs a preset section in the persistence schema and a picker in Settings/Match Controller. |
| P3 | **Multiple OBS scenes / overlay sets** | Several named overlay layouts, each with its own set of active sections and URLs; switch live. Requires either per-scene query params on the overlay URLs (`?scene=`) or multiple server mount paths, plus UI to manage sets. Largest of the five. |
| P4 | **Tournament / bracket overlay** | Read-only 1920×1080 overlay rendering the completed bracket (single elim, double elim, groups/standings). New broadcast type plus a new overlay HTML file; must not change existing overlay protocol. |
| P5 | **Stream Deck / HTTP control API** | Authenticated HTTP endpoints for score ±, next map, show/hide, timer start/stop; documented for Bitfocus Companion / Stream Deck / scripts. Depends on the bind-address/CORS/auth work already listed under “Protocol and server”. |

Each of these gets a design pass before implementation; none should be started until v1 has shipped and the protocol review checkpoint has passed.

### Protocol and server
- Versioned protocol (`v2`) with a real full-state snapshot on connect, then typed per-section updates.
- Optional diffs/patches for large payloads; message sequence numbers and acknowledgements.
- Heartbeat/ping and stale-client cleanup.
- Per-overlay subscriptions so a client only receives the sections it renders.
- Replace base64 data URLs with served asset URLs (or a hybrid), reducing frame size.
- Configurable bind address (localhost vs LAN), CORS allow-list, optional token auth, optional TLS.
- Headless/server-only mode.
- HTTP control API (score, show/hide, timer) for Stream Deck, Companion, Bitfocus, scripts.
- Server-side scene/preset switching and multiple simultaneous match outputs.

### Overlays and presentation
- Animate-in/out and show/hide API beyond `display` toggling; replay animations on demand.
- Live preview panel inside the app (iframe/WebView) for each overlay.
- Overlay theme/template system; scoreboard skins.
- Timer overlay with countup/countdown/target, alignment, leading zeroes, standalone mode.
- Tournament/bracket overlay.
- Lower-thirds, player intro, matchup, winner screens, ad/BRB loops.
- Per-show asset packs and overlay overrides folder (already planned in the base architecture).
- Font manager: user-selectable fonts, local by default.
- Localisation of overlay labels (e.g. “FIRST TO”, “MAP”).

### Match and show control
- Hotkeys and Stream Deck integration.
- Undo/redo for all edits.
- Presets/templates for match setups and map pools.
- Multiple matches/tournaments open at once; match history and stats.
- Best-of series automation (auto-advance maps, auto-complete match).
- Player substitutions UI (model already exists but unused).
- Import from the old Electron `app.json` (declined for v1; map legacy JSON into the domain if it is ever wanted).
- Scoreboard record/tag display options; per-team overlay overrides.

### Tournament (post-v1, after the v1 completion work)
- Bracket overlay output.
- Drag/drop reseeding and matchup editing.
- Forfeits, disqualifications and walkovers.
- Multi-stage playoffs and cross-group seeding.
- Match history and per-team tournament stats.

### Assets and data
- Asset deduplication, hashing, thumbnail cache, image optimisation (WebP/AVIF).
- Relative/portable asset storage; per-show asset packs.
- SQLite option if JSON becomes a bottleneck (probably not needed).
- Backup rotation, restore points, cloud/network sync.
- Schema migrations with downgrade protection.

### UI/UX
- Detachable panels and multi-window layouts.
- Light/dark theme; accessibility (keyboard navigation, focus, contrast).
- Localisation of the control panel.
- Better empty states, search/filter, bulk editing.
- In-app diagnostics: log viewer, server status, OBS test page.

### Platform and delivery
- macOS/Linux support.
- Auto-update; code signing.
- Installer + portable zip.
- CI matrix, automated overlay compatibility tests, headless WS integration tests.
- Crash reporting (opt-in).

## D. Review checkpoints

- **Font review (after overlay HTML is rendering):** compare Saira against Archivo and Barlow in situ at 1920×1080. Decision can change with a five-file swap.
- **Protocol review (after v1 ships):** decide whether to introduce a versioned v2 protocol; v1 stays byte-compatible.
- **Tournament overlay review (after v1 tournament completion):** decide whether to pull the bracket overlay forward from the post-v1 list.

## E. Risks / open questions

1. **Apotek licensing** for redistribution.
2. **No regular Apotek OTF** in the repo; Medium is the fallback.
3. **OBS compatibility testing** must be manual; no automated renderer in CI.
4. **Existing overlay HTML** has duplicated CSS, inconsistent init handling and unused `show`; we preserve behaviour in v1, clean up only with explicit approval.
5. **Windows-only** for v1 (confirmed). macOS/Linux remain a post-v1 packaging/QA exercise.
6. **Dioxus component library** choice still pending (Phase 3).
7. **Data migration** from Electron: path detection and schema differences.
8. **Timer has no overlay** in v1; it is control-only. Users who expect it on stream will notice — call this out in the README.
