# Backlog — post-parity and v1 correctness fixes

Living document. Ordered by priority within each section. IDs are stable for referencing in milestones.

## A. v1 correctness fixes (agreed as part of parity)

These are defects that currently make a shipped feature fail, risk data loss, or create an external runtime dependency. They are in scope for v1.

| ID | Fix | Detail |
|---|---|---|
| C1 | Local fonts | Replace all `https://design.penpot.app/...` `@font-face` URLs with locally served OFL fonts. Replace the Google Fonts `Inter` import in the UI with a bundled or system font. No runtime network dependency for fonts. Apotek is commercial (Kerns & Cairns) and cannot be redistributed; **Archivo** (OFL 1.1) is the active overlay font behind neutral `FGC Display*` aliases, with **Saira** retained as a reversible fallback. See `docs/assets.md` §4 and `docs/fonts.md`. |
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
| C14 | Map game modes | **Fixed 2026-11-14; corrected in A40 on 2026-11-19** — Map assets now use an inline **Game mode** select in Assets. Unlinked maps still group under **Other** on the map-pool overlay. The same value derives **Game mode** on map slots; typed modes are manual overrides. See section P. |
| C15 | Asset cleanup after a failed load | **Done 2026-11-21.** If `app.json` and `app.json.bak` are both unreadable, the app starts with the empty default state. Cleanup now refuses to build a keep set unless saved state can be read and the startup load succeeded, so neither **Clean up now** nor the post-save/exit collectors can delete user blobs. Settings disables the button and explains the pause; a full project-bundle import re-enables cleanup. Regression tests cover the paused, unreadable and union cases. See `docs/decisions.md`. |

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

Recommended replacement: **Archivo** (OFL 1.1) as the R1 trial, with **Saira** (OFL 1.1) retained for comparison. Both are bundled as WOFF2 at weights 300/400/500/600/900, served from `/assets/fonts/`, with `OFL-Archivo.txt` and `OFL.txt`. Use neutral `FGC Display*` aliases with a fallback stack (`'FGC Display', 'Apotek', sans-serif`). UI: bundle Inter (OFL) locally.

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

**Out of scope for v1** unless pulled forward: bracket overlay, forfeits as a result type, custom points, match history, external tournament-site import. Multi-stage playoffs were pulled forward into v1 as `BracketFormat::MultiStage`, with **Push to overlay** for group and bracket fixtures. The remaining items stay in the post-v1 list.

## B. v1 candidates (not committed — none are in v1)

These are small, isolated additions that would be cheap to pull forward *after* v1 is running. They are **not** part of the v1 milestone.

| ID | Item | Why it might be worth pulling forward later |
|---|---|---|
| B1 | ~~Show/hide overlay control~~ | **Done 2026-11-21.** Overlay view gained a **Hide overlays** / **Show overlays** button backed by a session-only signal. It broadcasts the existing `{type:"show", visible}` message, seeds start/restart with the current value, and never writes to `app.json`. A50 verified both viewports, pointer/keyboard activation, AX state, live overlay display and restart persistence; Doctrine IMP-019 closed · done. |
| B2 | ~~All overlay URLs with copy buttons~~ | **Done 2026-11-21.** D8-6/D8-9 added labelled Browser Source groups for each enabled access path; every group lists all seven overlay URLs with Open and Copy actions. A48 verified the groups and hidden state; Doctrine IMP-020 closed · done. |
| B3 | Overlay health/reconnect indicator | Surface connected client count and last broadcast time. |
| B4 | Basic hotkeys | Score +/-, next map, show/hide — high value during a live show. |
| B5 | Timer overlay | The timer is control-only in v1; a bundled broadcast timer can read the existing `matchTimer` messages. |
| B6 | ~~Bulk asset import~~ | **Shipped 2026-11-18 (touch-up 16).** Asset import accepts multiple files with per-file validation, hashing, dedupe and error reporting. Image pickers also support direct upload with auto-selection (touch-up 18). |

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

### C1. Overlay visual polish (deferred to a separate session)

The owner asked for overlay appearance work to be handled in a new session.
The seven shipped overlays are unchanged in v1: same filenames, protocol,
geometry and 1920×1080 layout. Any later design pass must keep those contracts
and be signed off in OBS before shipping.

### C2. Control and overlay requests raised during R1 (2026-11-12 — post-v1)

None of these are v1 fixes; they are recorded so the R1 pass does not turn into
scope creep.

| ID | Item | Scope sketch |
|---|---|---|
| D1 | Hero-ban redesign | Revisit the current-map-only hero-ban overlay and the Match Controller's ban editing together. Keep the wire message and 1920×1080 canvas; this is a design pass, not a protocol change. |
| D2 | Merged Map Selection / Hero Bans tab | The two Match Controller tabs edit closely related data. Merge them into one map-centric editor, keeping the existing per-map fields and validation. |
| D3 | Map-mode icons on map select | The map-select overlay currently shows only the map name. Add the bundled game-mode icon/name where the asset has a game-mode link (see C14). |
| D4 | Caster camera and audio via VDO Ninja | Embed or link caster camera feeds in the control UI and/or an overlay. Requires a capture/transport decision (VDO Ninja URL or WebRTC), a new UI surface and a privacy/consent note; not a protocol-only change. |
| D5 | Map-pool row animation | Animate each game-mode row asynchronously with a delayed left-to-right load instead of the current single pool reveal. CSS-only; keep the existing grouping and ordering. Requested 2026-11-14 during the R1 re-run. |
| D6 | Roster card autofill | Size the core player cards so they always fill the 1500px container (the same width as the team-name bar) instead of a fixed 284px. Must keep every card the same width and height at any roster size. Requested 2026-11-14 during the R1 re-run. |
| D7 | Hero-ban strip polish | The ban strips work and can use a linked custom icon, but the composition can be revisited alongside D1. No protocol change. Requested 2026-11-14 during the R1 re-run. |

### C3. Remote overlay access - LAN, Tailscale and Funnel (requested 2026-11-15 - post-v1)

**Goal.** One overlay server reachable from this PC by default, from the local
network through the existing **Allow LAN connections** flag, and from the user's
own Tailscale tailnet through an explicit switch. In post-v1 builds **Local
network** and **Tailscale** are independent and can both be on. Public exposure
through Tailscale **Funnel** remains a later option and is not implemented.
Tailscale is detected, never bundled: the feature stays off until the user turns
it on and the Tailscale client is installed.

**What A3 proved (2026-11-15).** An overlay page served from a second origin
connected to a non-default port and updated live. That run only worked because
the browser was on the host PC: every overlay still dialled
`ws://localhost:<port>`, so a second device would dial its own localhost. The
origin-aware resolver was therefore pulled forward into v1 (D8-1, 2026-11-16) so
LAN mode works on a real second device; the Tailscale plumbing remains the easy
part.

| ID | Item | Scope sketch |
|---|---|---|
| D8-1 | Origin-aware overlay connection | **Fixed 2026-11-16, pulled forward into v1** so LAN mode works on a second device. All seven overlays resolve `ws(s)://location.host` when the app served the page, keep `?host=`/`?port=` overrides for third-party pages, and use `wss` when the page is HTTPS. Static tests pin the resolver and reject a `ws://localhost` literal. No message, filename or DOM change. |
| D8-2 | Remote-access settings | **Done 2026-11-21 (UI first; simplified in D8-8).** Settings → Remote access persists the access mode in `overlay.remoteAccess`; the existing `allowLan` flag remains the bind source of truth. Off and LAN are live and share the existing bind flag, with a status line and per-overlay copy and Open in browser actions. Tailscale unlocked when detection landed in D8-3 and started serving in D8-4; Funnel stays unavailable until public exposure is designed. D8-8 replaced the early selector with two independent switches. See `docs/decisions.md`. |
| D8-3 | Tailscale detection | **Done 2026-11-21.** Finds `tailscale.exe` on PATH, then `%ProgramFiles%\Tailscale` and `%ProgramFiles(x86)\Tailscale`. Runs only `tailscale status --json` and `tailscale version` on a blocking task; maps `BackendState` with `Self.DNSName`/`Self.Online` to not installed, stopped, needs login or connected. Settings shows the short version, this device's MagicDNS name, plain remedies and a Refresh button. The Tailscale switch is enabled when connected. See `docs/decisions.md`. |
| D8-4 | Serve lifecycle | **Done 2026-11-21.** With the Tailscale switch on, starting the server installs two handlers, `/fgc` for pages and assets and `/` for the WebSocket endpoint that five frozen overlays dial at the root: `tailscale serve --bg --yes --https=443 --set-path <path> http://127.0.0.1:<port>`. Stopping the server, switching Tailscale off, or exiting removes only those two paths with targeted `--set-path <path> off`, leaving other handlers intact. A missing handler is treated as success and `tailscale serve reset` is never used. Settings shows the active/failed state. See `docs/decisions.md`. |
| D8-5 | Funnel (explicit opt-in) | The same path prefix through `tailscale funnel` on 443. Requires HTTPS certificates and Funnel permission in the ACL. Confirmation dialog naming the public exposure, plus an unguessable path token. Resets to off on every launch. |
| D8-6 | URL generation | **Done 2026-11-21.** Tailscale: `https://<dnsname>/fgc/overlays/<file>.html` with no `?port=`, shown once detection supplies the MagicDNS name and on the Overlay, Settings and Match Controller → Config pages. LAN: `http://<lan-ip>:<port>/overlays/<file>.html`; both switches off: `http://127.0.0.1:<port>/overlays/<file>.html`. When both switches are on, both URL sets are listed side by side. Served asset URLs continue to use the single primary base. Funnel is not offered in the two-switch UI; a persisted value only shows a pending notice and local URLs until D8-5 is designed, and no token appears in Tailscale URLs. A companion `/` Serve handler is installed so the five frozen overlays can still reach their root WebSocket. See `docs/decisions.md`. |
| D8-7 | Docs and QA | **Done 2026-11-21.** Wiki source page "Remote overlays" added and clearly labelled post-v1; LAN, Settings, OBS, Troubleshooting and FAQ pages cross-link it without claiming v1.0.0 has Tailscale. Runbook §7 adds A42–A47 for a second LAN PC, a tailnet device, a phone on cellular, host-side URL/Serve checks, teardown and the two-switch/console clean-up. A42–A48 are now Pass: A42 from the owner's Windows 11/OBS 33.0.1 LAN result, A44 from the owner's second-tailnet-device OBS report with noted evidence gaps, A45 from the owner's Samsung S25 FE cellular result, and A43/A46/A47/A48 from assistant evidence. Teardown evidence shows only `/fgc` and `/` are removed and a clean node reports `No serve config`. The A45 map-count observation is post-v1 work tracked as **ISS-001** (§W), closed fixed after the later overlay verification. |
| D8-8 | Remote-access settings clean-up | **Done 2026-11-21.** Replaced the duplicate LAN switch and **Access scope** selector with one **Remote access** panel containing two independent switches: **Local network** (existing `allowLan`) and **Tailscale** (existing `remoteAccess = Tailscale`). Both may be on; the status reads **Local network + Tailscale**. Windows Tailscale CLI calls now use `CREATE_NO_WINDOW`, so no CMD/CONHOST window pops during status/version/serve checks. No schema change; the QA `app.json` hash is preserved. Runbook A47 records CDP, process-watch and Serve evidence. |
| D8-9 | Dual Browser Source URL sets and UI pass | **Done 2026-11-21.** `OverlayUrls` now derives every enabled URL set from state, so Local network and Tailscale appear as separate labelled groups when both switches are on. The served-asset base is unchanged. Remote access gained a two-card overview; Browser sources use per-path cards with hidden URLs and disabled copy/open actions until revealed. Native `details` disclosures were added to secondary sections across Settings, Overlay, Match Controller, Teams, Talent, Tournament and Assets. No schema, protocol or overlay change. |

**Design notes.**

- *Resolver (D8-1).* The whole feature rests on this change:

  ```js
  const q = new URLSearchParams(location.search);
  const override = q.has('port') || q.has('host');
  const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
  const target = override
    ? `${q.get('host') || 'localhost'}:${q.get('port') || '3001'}`
    : location.host;
  const ws = new WebSocket(`${scheme}://${target}`);
  ```

  `?port=` keeps its current meaning (localhost:port), so A3 and third-party
  pages do not change. App-served pages become origin-relative, which is what
  makes LAN and proxied access work. `crates/server/tests/overlays.rs` should
  assert the resolver and that no overlay still contains the literal
  `ws://localhost`.
- *Why shell out.* Tailscale's embedding library (tsnet) is Go, so a Rust app
  would need a sidecar or a rewrite. Driving the installed client honours the
  user's login, tailnet and ACLs, keeps the installer small and keeps the
  feature optional. A missing client is a normal state, not an error.
- *Security model.* Overlays are read-only: the server ignores inbound client
  messages, so a remote viewer can read team names and scores but cannot change
  state. In Tailscale mode the tailnet ACL is the access boundary. Funnel is
  public, so it needs the warning, the token and a documented "turn it off when
  the show ends" habit. Keep CORS as-is, do not log the token, and consider
  serving only `/overlays/*` and `/assets/*` on the Funnel path (block `/state`).
- *Failure modes to design for.* Client missing or too old; tailscaled stopped;
  logged out; MagicDNS off; HTTPS certificates not enabled for the tailnet;
  Funnel not permitted by the ACL; another app already serving 443 or `/fgc`;
  the DNS name changing after a machine or tailnet rename; a stale serve config
  left by a crash (reconcile at startup and offer **Stop remote access**).
- *Phasing.* 1) D8-1 plus static tests; the feature is usable at that point with
  a manually run `tailscale serve` (document it). 2) D8-2 to D8-4, UI and
  lifecycle. 3) D8-5, token and Funnel. 4) D8-7, docs and QA.
- *Open questions.* Can a normal GUI process run `tailscale serve` on Windows
  without elevation (operator user)? Does `--set-path` plus `off` remove only
  our path? Does the proxy pass WebSocket upgrades and `/assets/*` unchanged?
  How slow is first-request certificate issuance? Which CEF/WebView2 versions
  in OBS load `wss` from another machine cleanly?
- *Out of scope.* Bundling Tailscale or headscale, ACL management, exit nodes,
  TLS for plain LAN, per-overlay access control, several remote origins at once,
  and any auth beyond the tailnet ACL and the Funnel token.

Related: B2 (copy buttons for every overlay URL), B3 (connection status) and the
"configurable bind address ... optional TLS" item under **Protocol and server**.

### C4. Custom overlay author documentation (requested 2026-11-21 - post-v1)

Owner request: document how someone can write their own overlay using the
variables, script examples and data the app already exposes natively. There is
no in-app variable/script inspector in v1; adding one is a separate feature
request, not part of this item.

| ID | Item | Scope sketch |
|---|---|---|
| D9 | Custom overlay author documentation | **Done 2026-11-21.** Published as the wiki page **Custom overlays** and linked from Home, `_Sidebar.md`, **Overlays in OBS** and **FAQ**. Docs only: add a wiki page for authors writing their own overlay HTML, linked from **Overlays in OBS**, **FAQ**, the Home guide table and `_Sidebar.md`. Reuse what already exists rather than inventing a new contract: `docs/protocol.md`, the seven bundled overlay scripts, the R4 timer test page in `docs/manual-qa-runbook.md`, the copyable Browser Source URLs and Settings values, and the `/state` and `/health` endpoints. Cover the origin-aware resolver (`ws(s)://location.host`, `?host=`/`?port=` overrides, 2 s reconnect, `requestInit`), the keyed `init` map and every emitted type (`scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`, `casters`, `matchTimer`, top-level `show`), a copyable minimum listener, shared scoreboard field names and quirks, served vs inline asset URLs, OBS 1920×1080 setup, LAN/no-auth caveats and the per-file override folder. Reconcile the stale `ws://localhost` wording in `docs/protocol.md` §7 and `docs/assets.md` §2.2 with D8-1. No code, protocol, server, bundled-overlay or in-app inspector change. |

### Protocol and server
- Versioned protocol (`v2`) with a real full-state snapshot on connect, then typed per-section updates.
- Optional diffs/patches for large payloads; message sequence numbers and acknowledgements.
- Heartbeat/ping and stale-client cleanup.
- Per-overlay subscriptions so a client only receives the sections it renders.
- Replace base64 data URLs with served asset URLs (or a hybrid), reducing frame size.
- Configurable bind address (localhost vs LAN), CORS allow-list, optional token auth, optional TLS.
- Remote overlay access over the user's Tailscale tailnet, optional Funnel, and origin-aware overlay URLs (see §C3).
- Headless/server-only mode.
- HTTP control API (score, show/hide, timer) for Stream Deck, Companion, Bitfocus, scripts.
- Server-side scene/preset switching and multiple simultaneous match outputs.

### Overlays and presentation
- Further overlay polish (see §C1): fit, spacing and long-name handling without changing the design or protocol.
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
- More than one group stage and one playoff; deeper multi-stage formats after the shipped v1 implementation.
- Match history and per-team tournament stats.

### Assets and data
- Further image optimisation (WebP/AVIF) beyond the shipped hash-keyed lazy thumbnail cache.
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

- **Font review (after the R1 re-run): Done.** R1 selected Archivo; M9-5 is Done and A41 Pass confirmed the seven overlays at 1920×1080. Saira stays bundled as a reversible fallback; Barlow remains untried. A future swap is five WOFF2 files and `src:` URLs plus the `FONT_PREFIX` test constant; no layout CSS change. Tracked in Doctrine `CHR-004`.
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

## F. Pre-release UI touch-up sweep (shipped 2026-11-18)

Nineteen pre-release touch-ups shipped in the app: hidden overlay URLs with a show option, optional auto-start server, Settings timer section removed, sidebar reorder and Lucide icons, automatic team-colour sampling from logos, logo background off by default with matching projection, player-card preview, merged **Maps & Bans** with a hero-bans toggle, match/map-pool export and import, **Loser picks** moved to Config, saved matches with scheduled start times and a match library, talent type dropdown, floating timer, conditional Tournament **Board/Groups** tabs by format, thumbnail loading/cache performance work, multi-file asset import, gallery card fixes, and quick upload in image pickers. Details are recorded in `docs/decisions.md`.

## G. Final pre-release changes (2026-11-18)

- **Timer widget.** The floating timer is now the only timer control. Settings
  and the Match Controller **Timer** tab are removed; the widget provides icon
  Start/Pause/Stop actions and an expandable settings panel.
- **Talent roles.** `TalentType` is Caster, Host, Analyst, Interviewer or Other,
  stored on each match assignment. The per-person `type` field is removed by
  schema v2; legacy `panelist` and `observer` values map to Analyst and Other.
- **Tournaments.** `BracketFormat::MultiStage` generates groups followed by an
  elimination bracket and shows both **Board** and **Groups**. Group fixtures
  and bracket matches expose **Push to overlay**.
- **Assets.** Changing a map's game-mode link refreshes derived map-slot mode
  snapshots when they still match the old derived value; manually entered modes
  are preserved. Thumbnails load lazily, decode asynchronously and keep a
  hash-keyed data-URL cache.
- **Overlays.** The seven bundled overlays ship unchanged apart from two scoped
  functional fixes: `hero-bans.html` (A8 note) and `game-overlay.html` (A9 stale
  logo clearing). Visual polish is deferred to a separate post-v1 session.

## H. Final QA feedback fixed before v1

- **Talent assignment layout.** A38 passed functionally, but the tester flagged
  visible slot labels as unnecessary and asked for one person per row. The
  Talent tab now removes those labels and renders six rows with the person
  picker and **Role** select horizontally aligned. The slot name remains as an
  accessibility group label. A38 was re-run and passed before the v1 tag.

## I. A39 final QA findings (2026-11-18)

A39 failed with nine tester findings on the Tournament view:

1. The **Groups** tab should appear before **Board**.
2. A multi-stage playoff should stay empty until the group stage is completed.
3. Removing a team from a group needs a confirmation dialog.
4. Fixture results should autosave when added; the results **Save** button should go.
5. Fixture rows should offer a pencil/details button for score, status and
   winner verification instead of inline score and status controls.
6. Bracket placement when groups complete is currently wrong.
7. **Push** and **Edit** controls overflow in the board view.
8. “Push match to match controller” must be distinguished from the future
   “push tournament” graphic overlay.
9. The board should look like a traditional bracket with directional lines
   between rounds, not a kanban-style board.

The v1 disposition of these findings is recorded in `docs/manual-qa.md`.

The 2026-11-18 re-check was also rejected, with five additional v1 blockers:

1. Edit statuses to only be upcoming and complete.
2. Rename "Board" to "Bracket".
3. Design still doesn't show a traditional bracket, elements are top-aligned and
   the linking lines between then don't make sense (two matches should go
   directly into one match with no crossing).
4. Remove arrows to move a match up or down, replace with a draggable element
   and an ability to lock/unlock the dragging (locked by default). Clarified by
   the owner: the draggable element is a **participant**, not a match card; drag
   a participant onto another slot in the same round to swap them.
5. The elements feel very condensed: we want to have 'comfy' padding (this
   applies to all elements in every UI element across the whole app). The owner
   later judged the applied spacing slightly too comfy and asked to revisit it
   during the whole-app GUI review.
6. While re-running the runbook, the owner asked for **Push to match** to become
   an icon and to add a confirmation before it overrides Match Controller. The
   confirmation reads **Replace the current Match Controller match with this
   fixture?** with **Cancel** and **Push to match** actions.
7. After items 2 and 3 passed, the owner superseded the remaining runbook flow
   with a setup-model change: once a tournament has been initially started or
   generated, teams and fixtures can no longer be added.
8. The generation step must open a preview in a pop-up window before committing,
   showing seeding, group distribution and bracket placement details for
   confirmation.
9. Dragging must be removed from the live Groups and Bracket spaces and allowed
   only inside that generation preview.
10. The owner reported that the current drag implementation still does not work
    and must be made functional in the preview before A39 can continue.

## J. A39 post-step-7 owner findings (2026-11-19)

After item 5 step 7, the owner reported that the hash change may have come from
generating instead of cancelling, and added five v1 findings:

1. Regenerating and clearing a generated stage are different actions. Provide
   separate controls and require confirmation before a reset; no add control may
   silently discard a bracket.
2. Seeding must be editable inside the bracket generation preview, with placements
   updating from the edited order before **Generate**.
3. Match/group-fixture status must be a two-state toggle switch rather than a
   select; only **Upcoming** and **Complete** are valid.
4. The double-elimination lower bracket connectors must join the correct matches
   without crossing, and upper-bracket losers must show dotted drop stubs into
   the lower bracket.
5. Preview dragging needs a visible, animated drag indicator so the drag is
   obviously active.

Reset controls and confirmation dialogs were added to Setup, Groups and Bracket.
Bracket previews now support seed-number editing and seed-handle dragging. Match
details dialogs use `Switch` controls, bracket geometry follows stored winner
links with dotted upper-bracket drop stubs, and the preview renders a fixed drag
ghost with a subtle pulse. QA resumes with A39 item 5 step 7 only.

## K. A39 item 8 owner finding (2026-11-19)

After item 8 passed, the owner noticed that the bracket **Edit match** dialog
still contains a **Delete match** action. This contradicts the v1 scope that
locks structural additions and removals after generation. The destructive block
is now gated by `tournament_is_generated`, so it cannot appear for any live
bracket match while its Edit/Push card remains available.

## L. A39 item 9 owner finding (2026-11-19)

The owner failed the whole-app spacing review: spacing was still too generous,
navigation was difficult in Tournament and Match Controller, and more
collapsible sections were requested, specifically the Tournaments sidebar. The
fix uses scoped CSS reductions instead of changing the global spacing tokens, so
overlay component dimensions and the earlier accepted spacing decisions are
untouched. The Tournaments sidebar can now collapse to a narrow panel by mouse
or keyboard, with `aria-expanded`, `aria-controls` and focus retained on the
toggle. Match Controller saved matches are behind a collapsed
`Saved matches (n)` disclosure, and hero-ban fields are grouped under a
collapsible disclosure when hero bans are enabled. A reusable
`.fgc-disclosure` pattern was added for future sections.

The owner passed the re-check with one follow-up: bracket match cards should be
wider and the space between separate matches narrower. The bracket constants
now use a 248 px card, 56 px column gap and 108 px row height, down from 216/80/144.

## M. A39 item 10 owner finding (2026-11-19)

The owner passed item 10 with this exact answer:

> Pass; Small Request, move the buttons to regenerate round robin / bracket /
> reset to a settings menu in line with the Group Stage / Bracket Heading +
> Select number of groups or brackets within the preview screen. Note: The
> Playoff preview and generaete should not be in the group tab, but rather the
> bracket tab in multistage tournaments

The multi-stage playoff preview and generate action live on **Bracket**, not
**Groups**. The owner then superseded the dropdown part of the request:

> Hmm, on second thought, instead of a dropdown menu, have all the buttons in a
> horizontal row where the settings button currently is. Make sure that we
> iconise and colour code them too so that it's easily understood and
> intepretable

`StageSettingsMenu`/`.fgc-stage-menu` were replaced by `StageActionTone`,
`StageAction`, `.fgc-stage-actions` and an optional `Panel` `actions` slot. Each
heading now has one horizontal row: `▷` Preview in accent orange and `↻`
Regenerate in amber, plus `⌫` Reset in red once generated. Each keeps a visible
label and matching `aria-label`/`title`. No dropdown ID or stage-settings menu
remains. A later item 11 follow-up removed the green `+ Group` action and the
preview's group-count selector, so no Add tone remains. Items 1–10 are accepted;
item 11 passed with the follow-ups below.

## N. A39 item 11 owner finding (2026-11-19)

The owner failed item 11 with this exact answer:

> Fail: 1. when toggling the status, sometimes a message about "at least two
> teams must qualify for a playoff" appears but doesn't do anything; 2. There is
> still a generate card with text in on the Setup page - remove it entirely;
> 3. Group match details does not contain the advanced winner selection option
> nor a customisation on the first to field like the bracket does; 4. Everything
> else is a pass

Fixes:

1. The app's group-result store method now calls the silent
   `core_set_group_result`. The result is still saved and a multi-stage playoff
   still auto-seeds on the transition to a complete group stage, but seeding
   errors are no longer surfaced while toggling a fixture status. Bracket still
   reports why a playoff cannot be generated.
2. The Setup **Generate** panel was deleted outright, including its entrant,
   regenerate/reset note and preview buttons. SetupTab no longer takes an
   `on_preview` handler; generation starts only from the Groups/Bracket heading
   action rows.
3. `GroupMatch` gained persisted `winner` and `winner_manual` fields with serde
   defaults. New core helpers derive, override and clear the effective winner;
   standings and head-to-head use it. The group fixture editor now has a
   **First to** select (1–4) and **Advanced settings → Winner** with **Derived
   from scores** and one option per slot, matching the bracket editor. A manual
   winner marks the fixture completed, survives later score edits, drives the
   standings and auto-seeds the playoff when it completes the group stage. The
   read-only label was renamed from **Winner (derived)** to **Effective winner**.

New tests cover group winner/first-to persistence and playoff seeding from a
manual winner.

The owner then passed item 11 with this exact follow-up:

> Pass: remove "choose the group count in the preview"; remove "+ Group" outside
> of the group generation

Both follow-ups are implemented: the group preview no longer has a group-count
selector or the "Choose the group count in the preview." note, and the live
**Groups** heading no longer has `+ Group` or any manual group creation path.
Groups come only from the generation preview via the heading action. Focus after
deleting a group or removing a team returns to that generation action. With the
follow-ups applied, A39 items 1–11 are accepted.

## O. Overlay appearance deferred (2026-11-19)

The owner asked for the overlay visual pass to be reverted and handled in a new
session. The seven bundled HTML files are restored byte-for-byte from the
pre-change backups, and no appearance-only artifacts or documentation remain in the
pending v1 changes. v1 ships the original overlays.

## P. A40 asset properties, map modes and attached hero icons (2026-11-19)

The owner first failed A40 with this exact answer:

> Fail at step 2: the maps don't respect the game mode in the selector. Note that it does update correctly in map pool. Further note, I'm still not happy with the current system of assigning map types, and by extension, hero ban icons, to a map / hero. In the old react application, we use to give a tag to a map to change it's mode, based on the existing mode icons that were imported. Basically like a properties or taxonomy approach.

Scope choice: **Tags + bug fix (Recommended)**.

The old behaviour stored a free-text mode on each map slot and only refreshed
it when the text still matched the previous derived value and the slot name
matched the map. Blank, renamed or hand-typed slots therefore stayed stale even
though Map Pool followed the asset. `Asset.game_mode` now drives both Map Pool
and map slots; `MapSlot.game_mode_manual` records a hand-typed mode, and
changing the map name re-derives the mode and clears that flag. Visible labels
say **Game mode**, not “tag”.

The owner then rechecked the inline tag selects and answer:

> The game mode tag works fine (just maybe remove tag from the label. However, the hero icon doesn't necessarily work well, as it is showing two names for both images, as they are both uploaded as "hero" category. Maybe for heroes, we have the ability to open up an image properties, and upload a hero icon directly onto that page? Since the icon shouldn't get reused for any other hero. This also adds scalability in case we want to add other elements, like videos or animations down the line.

The hero half of that answer replaced the inline **Ban icon tag** select with a
hero **Properties…** dialog and a direct upload flow:

- `Asset.icon` still points at the ban icon, so `/state` and the overlay are
  unchanged.
- The uploaded image is a Hero asset with `Asset.attachment =
  { owner: <hero id>, kind: "heroBanIcon" }`. It is hidden from the library,
  hero pickers and ban lists, so only one name appears for the hero.
- Replacing or removing a ban icon deletes the old attached asset; deleting a
  hero deletes every attachment it owns. Legacy library icons are unlinked but
  never deleted.
- `AssetAttachmentKind` is extensible for future media such as animations or
  videos without inventing another library category.
- `#[serde(default)]` keeps older state files valid; no schema version change.

Assistant CDP checks passed for both the first tag implementation and the final
properties rework. The final pass covered the map slot → Map Pool → game-mode
derivation loop, hero ban-icon upload/replace/remove/delete, hero and map
rename, URL-import dedupe, and thumbnail loading after scrolling and navigation.
`cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (53 tests) and `dx build --desktop` passed. QA state
was restored from `app-before-a40.json` at hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` after each
verification pass, with all 23 user blobs present. The owner recheck found one
preview-refresh defect and one layout issue; both were fixed and the owner then
passed A40 on 2026-11-20. Details are in `docs/manual-qa.md` and
`docs/manual-qa-runbook.md` §A40.

### A40 QA fixes: signal borrow panics (2026-11-19)

Two panics were found while exercising the final properties rework in the built
desktop app. Both came from holding a `Signal::peek()` borrow across `mutate()`,
which tries to take a write borrow of the same signal:

- `AppStore::delete_asset()` held `&self.state.peek().assets` while checking the
  attachment flag, producing `AlreadyBorrowed` on the delete path.
- `AppStore::set_asset_link()` held the same borrow while validating an editable
  asset and its attachments, producing `AlreadyBorrowed` when a map **Game mode**
  or hero attachment was edited.

Both paths now copy the small values they need and drop the read borrow before
calling `mutate()`. Test-only `AppStore::test_store()` and
`ServerController::test_controller()` constructors were added so the regression
tests can run without background tasks; they are behind `#[cfg(test)]`. New
tests `deleting_a_hero_removes_its_attached_icon` and
`setting_a_map_game_mode_through_the_store_refreshes_slots` cover the two
borrow paths. No user-facing behaviour changed.

### A40 owner recheck fixes: live preview and card actions (2026-11-20)

The owner rechecked A40 and answered:

> Fail at step 5: Replacing ban icon did override the image, but the preview image did not update until closing and reopening the properties menu. Extra note: the properties button does not fit well on the card, and the text is being truncated, review the placement of the button. Everything else passed

Two fixes shipped:

- `AssetThumb` used `use_future` with a clone of the first `asset` prop. When the
  hero replaced its attached icon, the parent passed a new icon asset but the
  running future still held the old one, so the data URL never refreshed until
  the dialog was remounted. `AssetThumb` now mirrors the prop into a signal and
  resolves it with `use_resource`, which reruns whenever that signal changes.
  CDP confirmed the dialog's `.fgc-asset-properties__icon-image` changed from
  image A to image B while the dialog stayed open.
- Hero card actions now wrap onto two rows. **Properties…** takes the full
  first row with the Outline variant; **Rename** and **Delete** share the
  second. At 1360×860 the button measured 203px wide with `scrollWidth` 201 and
  `clientWidth` 201, so the label is no longer truncated.

After these fixes `cargo fmt --all`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (53 tests) and `dx build --desktop` passed again. CDP
also rechecked thumbnails, card metrics and `/state` `gameOverlay.heroIcons`.
QA state was restored from `app-before-a40.json` at hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` with all 23
user blobs present; temporary QA icons and blobs were removed. The owner then
passed the recheck in `docs/manual-qa-runbook.md` §A40 on 2026-11-20, so A40 is
accepted.

## Q. A17 project bundle export (2026-11-20)

The owner failed A17 with this exact answer:

> Fail: Error message "Something needs attention  Could not export the project: asset blob hash mismatch: expected 348188027503e8f22cbfe1bd20604fb09d48e7be21c349c9d1017ce3fb6d4383, got f31394496e59eb16785024dd57985adc8a0253c4755eb30eab91a774fb9c7a66"; no app.json or export.json available; images are copied though in /blobs/blobs/<>; How do I install the app when we haven't made a release yet or committed changes?

The corrupt QA blob was restored from the correct `Curro.png` source; all 23
user blobs now hash-match. `AssetStore::export_bundle()` now reads and verifies
every referenced blob before creating the bundle, so a corrupt source cannot
leave `blobs/` without `app.json` and `export.json`. The regression test
`export_rejects_a_hash_mismatch_before_writing_anything` covers that failure.

Both export and import now call `blob_path(directory, …)`. `blob_path()` already
prepends `blobs`, so passing a root that already contained `blobs` produced the
owner-observed `blobs/blobs/...` layout. The round-trip test asserts the flat
layout. CDP triggered **Assets → Export project…** and the resulting bundle held
only `app.json`, `export.json` and `blobs/`, with 23 verified user blobs, no
nested `blobs/blobs` and no bundled art. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` and `dx build --desktop` passed. QA `app.json` stayed at
hash `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`. On
recheck the owner answered **Blocked**: no second Windows account/VM or
release/installer was available for the second-machine import, so A17 is not a
Pass. On 2026-11-21 the owner explicitly accepted that result for release with
**“Accept A17; authorize tag”**. A17 is blocked-accepted; second-machine import
remains unrun. The install workaround is in `docs/manual-qa-runbook.md` §A17.
Follow-up is tracked as Doctrine `CHR-005`; the runbook now spells out the exact
steps 4–6 needed to close the gap, and the code-level second-directory round-trip
was re-verified on 2026-11-21 without changing the blocked-accepted result.

## R. A8 hero-bans owner note (2026-11-20)

The owner passed A8 with this exact answer:

> Pass: note that hero ban images should only apply on game-overlay.html, not on hero-bans.html; hero-bans.html should also ensure to crop images to the correct dimensions to ensure there is no overflow

This note superseded the overlay freeze for `crates/server/overlays/hero-bans.html`
only. At that point the other six shipped overlay HTML files remained
byte-for-byte unchanged; A9 later approved a separate scoped change to
`game-overlay.html`.

`Projector::icon_asset_ids()` now gathers attached and legacy linked icon IDs,
and `icon_identities()` adds their `(hash, ext)` content identities. Both
`heroIcons` and `heroPortraits` skip those IDs and identities, so a duplicate
Hero entry carrying an icon's bytes cannot replace the hero's large art or put a
ban icon on `hero-bans.html`. `hero-bans.html` continues to read `heroPortraits`;
`game-overlay.html` alone reads `heroIcons` for the ban strips. New tests:
`duplicate_icon_files_do_not_become_hero_portraits`,
`hero_ban_icons_are_confined_to_the_game_overlay` and
`hero_bans_crops_portraits_inside_their_slots`.

The hero-bans CSS now gives columns and portrait wrappers `min-width: 0` and
`min-height: 0`, clips each wrapper with `overflow: hidden`, and renders images
with `display: block`, `object-fit: cover` and `object-position: center`. CDP at
1920×1080 measured a `200×2000` and a `2000×200` portrait filling their slots at
`200,240`–`1720,840` with no document overflow; the game overlay still showed the
distinct ban icons. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
and `dx build --desktop` passed. QA `app.json` was restored to hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

## S. A9 owner recheck (2026-11-20)

A9 failed on recheck. With **Team 3** and **Team 1** loaded and neither logo
assigned, `/state` correctly reported `null` for both sides, but the live
`game-overlay.html` source still painted two old logos until OBS was refreshed.
Pressing **Clear** had the same symptom: the app and `/state` cleared the logo,
but the OBS source kept the old image until restart.

The cause is in the shipped overlay script: `homeLogo.style.backgroundImage`
and `awayLogo.style.backgroundImage` are assigned only when the corresponding
`logoUrl` is truthy. There is no `else` branch to remove the stale background.
A fresh OBS source works because it has no previous background image.

The owner approved a scoped override of the overlay freeze for
`crates/server/overlays/game-overlay.html`. The script now assigns
`backgroundImage` unconditionally: the URL when present, otherwise `none`.
An explicit `null` team also clears its slot; an absent team object in a partial
frame is left alone. CDP verified stale logos clearing, `null` teams clearing,
and new logos still applying. The static test
`game_overlay_clears_stale_team_logos` locks in the branches. Only
`game-overlay.html` and `hero-bans.html` differ among the seven overlays.

Deferred overlay-refresh note from the owner: when no logo is set, show a
placeholder image containing the team's **TAG** or another abbreviation of the
team name so the slot is never empty. This is not being implemented in the A9
fix. The owner rechecked after the fix and answered **“Pass.”** A9 is accepted
and M3-6 is Done.

## T. A29–A36 owner QA (2026-11-20)

- **A29 — Pass.** Overlay URLs and the base URL stay hidden until **Show URLs**,
  all eight appear while revealed, and **Hide URLs** restores the hidden state.
  **Start server on launch** persists and auto-starts the server after restart.
  Settings has no duplicated timer controls and points to the floating widget.
  Touch-ups 1–3 are accepted.
- **A30 — Pass.** Sidebar order and icons match; the floating timer is present
  and usable on every destination, ticks and Start/Pause/Stop stay in sync with
  connected consumers, and placement is legible without obscuring controls at
  100%, 125%, and 150%. Touch-ups 4, 13, and 19 are accepted.
- **A31 — Pass.** Logo colour sampling and contrast text are sensible;
  **Separate logo background** defaults off and changes projected `logoBgColor`
  correctly; the roster card preview updates with every player edit. Touch-ups
  5, 6, and 7 are accepted.
- **A32 — Pass with notes.** All five steps passed; Touch-ups 8, 10, and 11 are
  accepted. Owner raised five follow-ups: hide hero-ban icons when bans are off,
  add a countdown-to-scheduled-match timer mode, default map count by first-to
  (1/3/5/7), always show all map name/mode options, and align map-field sizing
  with other selectors. All five are implemented and assistant-verified before
  `v1.0.0` (§2.19).
- **A33 — Pass with note.** Save/load/delete, export/import and validation all
  passed; Touch-ups 9 and 11 are accepted. Owner noted that **+ Add map** did
  not add a map. A33-1 is Done: added slots carry `userAdded: true` and survive
  validation, save and reload.
- **A34 — Pass.** Per-assignment roles persist and drive casters labels; legacy
  `panelist`/`observer` values migrate to schema 2 aliases; quick upload is
  available in the relevant pickers and auto-selects the imported asset.
  Touch-ups 12, 18 and the final talent-role change are accepted.
- **A35 — Pass.** Multi-file import reports per-file errors without aborting
  valid files; gallery cards and thumbnails render responsively; rename, delete
  and link flows are unaffected. Touch-ups 15, 16 and 17 are accepted.
- **A36 — Pass.** Round robin shows Groups only; single and double elimination
  show Board only; multi-stage shows both with the Board locked until groups are
  complete. Format changes and reload preserve `format: "multi-stage"` and both
  stages. Touch-up 14 and the final multi-stage change are accepted.
- **A41 — Pass.** All seven bundled overlays passed human OBS sign-off at
  1920×1080: data-driven states, team colours, icon/mode links, long names,
  reconnect/`show`, offline Archivo and one-minute watch checks. M9-5 and M9-6
  are complete; appearance work is deferred.

## U. Release gate R9 (2026-11-21)

- **R9 — Pass.** After the A17 blocked acceptance, all changes were committed as
  `c244d29` and pushed, and annotated tag `v1.0.0` was pushed against workspace
  version `1.0.0`. Release run
  [44826182532](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/44826182532)
  passed every step. The
  [v1.0.0 release](https://github.com/ThePeachZA/fruiterra-graphics-controller/releases/tag/v1.0.0)
  published the installer, portable ZIP and `SHA256SUMS.txt`; all three were
  downloaded and the published checksums matched both artifacts. The release
  notes matched the shipped template, contained no backlog claims and repeated
  the known limitations. M10-1, M10-6 and M10-8 are Done, and `docs/build.md`
  §7 items 1–12 are complete. R3, R5 and A17 retain their recorded acceptances.

## V. Wiki screenshots (requested 2026-11-21 - revisit later)

Backlog ID: **D10** (docs/QA, parked). A small, curated set of screenshots for
wiki pages where a picture removes real ambiguity. D9 is now implemented, but
owner review found the capture process/output unsatisfactory, so the
2026-11-21 screenshots were deleted and nothing was published or committed.
No decorative imagery.

Before the first image is added:
- Store files in `docs/wiki/images/` as compressed PNG. The `*.png` binary rule
  already exists in `.gitattributes`.
- Reference them with relative Markdown, for example
  `![OBS Browser Source setup](images/obs-browser-source.png)`, with alt text
  and a short caption.
- **Done 2026-11-21.** `scripts/publish-wiki.ps1` now syncs
  `docs/wiki/images/` into the wiki repo. Do not commit images directly to the
  wiki repo.

Order and target pages, highest value first:
1. `docs/wiki/Overlays-in-OBS.md` — OBS Browser Source properties showing a
   copied URL and 1920×1080. `images/obs-browser-source.png`.
2. `docs/wiki/Overlays-in-OBS.md` — the app's Overlay page with URLs revealed
   and **Copy URL** visible. `images/app-overlay-urls.png`.
3. `docs/wiki/Match-Controller.md` — the compact timer widget.
   `images/match-timer-widget.png`.
4. `docs/wiki/Match-Controller.md` — the maps/hero-bans tabs in an edited match.
   `images/match-maps-bans.png`.
5. `docs/wiki/Tournament.md` — the bracket board or the group-to-playoff
   generation preview, whichever is clearer.
   `images/tournament-bracket-board.png`.
6. `docs/wiki/Assets.md` — import and project-bundle flow.
   `images/assets-import-bundle.png`.
7. `docs/wiki/Settings.md` — overlay network and override-folder fields.
   `images/settings-overlay-network.png`.

Cap at seven images for the existing v1 guide. Revisit after D9 lands; if the
custom-overlay page needs a visual, give it one annotated example in place of a
lower-priority image above rather than simply adding an eighth. No app,
protocol, server or bundled-overlay changes — screenshots show the shipped v1
UI as-is.

## W. A45 owner finding — added maps on map-select (2026-11-21)

A42 and A45 passed from the owner's second-device results, closing D8-7. During
A45 the owner used a Samsung S25 FE on cellular with Tailscale 1.104.6 and this
exact URL:

```text
https://sekhmet.note-hydra.ts.net/fgc/overlays/map-select.html
```

The remote connection, live update and cellular toggle passed. The owner also
reported:

> Used "Add Map" button. New "Tiebreaker" maps were added, instead of "Map
> #<numerical sequence>". No new maps were shown on the overlay until the format
> was changed (the overlay should show based on how many maps are on the
> "Maps & Bans" view.

Tracked as Doctrine **ISS-001**, now closed · fixed and tagged `user-reported`,
`post-v1`, `qa:a45`, `area:overlay`, `area:ui`, `map-count`, `fixed`, `verified`.
The original code labelled every slot beyond `default_map_slots(first_to)` as
`Tiebreaker` (`crates/app/src/views/match_controller.rs`) and `map-select.html`
derived its card count from `firstTo` rather than the projected `maps` array
(`getMapsToShow`). `crates/protocol/src/projection.rs` already sent every map,
so no schema or payload change was needed.

The post-v1 fix labels every slot `Map {index + 1}`, makes `map-select.html` use
`maps.length` via `projectedMapCardCount(maps)`, and keeps the legacy extra card
only for `maps.length === TIEBREAKER_INDEX && currentMapIndex === TIEBREAKER_INDEX`.
`firstTo` state and `getMapsToShow` were removed from the
overlay; no protocol, server runtime or v1.0.0 overlay change was made.
`crates/server/tests/overlays.rs` gained
`map_select_card_count_follows_projected_maps`, which rejects any return to a
first-to-derived card count.

Assistant verification: `cargo fmt --all -- --check`,
`cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` and `dx build --desktop` passed. With `firstTo` held at
1, CDP showed `1 map slot(s)` / `["Map 1"]` / `/state mapCount 1` before adding,
then `4 map slot(s)` / `Map 1…4` / `mapCount 4` after three **+ Add map** clicks,
then `8 map slot(s)` / `Map 1…8` / `mapCount 8` after four more; no `Tiebreaker`
label appeared. The live overlay rendered four and then eight `.map-card`
elements. At eight cards the 1920×1080 layout occupied `x = 200…1720`,
`y = 271…871` with 20px gaps, document size `1920×1080` and no overflow. QA
`app.json` and `app.json.bak` were restored to hashes
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`;
`tailscale serve status` was `No serve config`, and no screenshots were taken
or published.

A45 stays Pass — this was a separate post-v1 map-count/label defect and is now
fixed. The seven bundled overlay files remain frozen for v1.0.0; this edit is
post-v1 only.

<!-- doctrine-migration-map:start -->

## X. Doctrine backlog migration map (2026-11-21)

Every legacy ID in this document and its companion design/QA records now has a
Doctrine backlog record. Legacy IDs remain provenance labels only: they are
not canonical IDs and are tagged on each record as `legacy:<id>` or
`prior-art:<source>`. P5 was already migrated to Doctrine **IMP-003** and was
not duplicated. The unnumbered §C bullets were migrated as seven grouped
umbrella records to be split when promoted.

Migrated records: **74**. Breakdown: issue/closed · fixed × 15, issue/closed · done × 1, improvement/closed · done × 22, improvement/triaged × 7, idea/triaged × 22, chore/closed · done × 3, chore/triaged × 2, issue/triaged × 1, chore/closed · accepted × 1. CHR-004 was closed · done after the font review reconciliation.

| Legacy | Doctrine | Kind | Status | Title |
|---|---|---|---|---|
| C1 | ISS-002 | issue | closed · fixed | Local overlay fonts |
| C2 | ISS-003 | issue | closed · fixed | `currentMapIndex` off-by-one |
| C3 | ISS-004 | issue | closed · fixed | Stale asset caches |
| C4 | ISS-005 | issue | closed · fixed | Port source of truth |
| C5 | ISS-006 | issue | closed · fixed | Broken menu URL |
| C6 | ISS-007 | issue | closed · fixed | `init` compatibility |
| C7 | ISS-008 | issue | closed · fixed | Atomic saves |
| C8 | ISS-009 | issue | closed · fixed | Import validation |
| C9 | ISS-010 | issue | closed · fixed | Schema version |
| C10 | ISS-011 | issue | closed · fixed | Portable exports |
| C11 | ISS-012 | issue | closed · fixed | Safe path handling |
| C12 | ISS-013 | issue | closed · fixed | Remove dead/incorrect state |
| C13 | ISS-014 | issue | closed · done | Timer overlay decision |
| C14 | ISS-015 | issue | closed · fixed | Map game modes |
| C15 | ISS-016 | issue | closed · fixed | Asset cleanup after a failed load |
| T1 | IMP-004 | improvement | closed · done | Generate a valid single-elimination bracket |
| T2 | IMP-005 | improvement | closed · done | Generate a valid double-elimination board |
| T3 | IMP-006 | improvement | closed · done | Generate round-robin fixtures and standings |
| T4 | IMP-007 | improvement | closed · done | Preview and adjust generated placements |
| T5 | IMP-008 | improvement | closed · done | Auto-advance without overwriting manual placements |
| T6 | IMP-009 | improvement | closed · done | Recursive downstream clear on result edit |
| T7 | IMP-010 | improvement | closed · done | Grand-final reset and champion override |
| T8 | IMP-011 | improvement | closed · done | Automatic champion from final |
| T9 | IMP-012 | improvement | closed · done | Configurable round-robin tiebreakers |
| T10 | IMP-013 | improvement | closed · done | Optional playoff generation with preview |
| T11 | IMP-014 | improvement | closed · done | Match modal editing |
| T12 | IMP-015 | improvement | closed · done | Tournament persistence and repair |
| T13 | IMP-016 | improvement | closed · done | Tournament unit tests |
| T14 | IMP-017 | improvement | closed · done | Keyboard access and 1080p 32-team board |
| T15 | IMP-018 | improvement | closed · done | Multi-stage groups, playoff restore and Push to overlay |
| B1 | IMP-019 | improvement | triaged | Show/hide overlay control |
| B2 | IMP-020 | improvement | triaged | All overlay URLs with copy buttons |
| B3 | IMP-021 | improvement | triaged | Overlay health/reconnect indicator |
| B4 | IMP-022 | improvement | triaged | Basic hotkeys |
| B5 | IMP-023 | improvement | triaged | Timer overlay |
| B6 | IMP-024 | improvement | closed · done | Bulk asset import |
| P1 | IDE-006 | idea | triaged | Undo / redo |
| P2 | IDE-007 | idea | triaged | Presets / show templates |
| P3 | IDE-008 | idea | triaged | Multiple OBS scenes / overlay sets |
| P4 | IDE-009 | idea | triaged | Tournament / bracket overlay |
| D1 | IDE-010 | idea | triaged | Hero-ban redesign |
| D2 | IDE-011 | idea | triaged | Merged Map Selection / Hero Bans tab |
| D3 | IDE-012 | idea | triaged | Map-mode icons on map select |
| D4 | IDE-013 | idea | triaged | Caster camera and audio via VDO Ninja |
| D5 | IDE-014 | idea | triaged | Map-pool row animation |
| D6 | IDE-015 | idea | triaged | Roster card autofill |
| D7 | IDE-016 | idea | triaged | Hero-ban strip polish |
| D8-1 | ISS-017 | issue | closed · fixed | Origin-aware overlay connection |
| D8-2 | IMP-025 | improvement | closed · done | Remote-access settings |
| D8-3 | IMP-026 | improvement | closed · done | Tailscale detection |
| D8-4 | IMP-027 | improvement | closed · done | Serve lifecycle |
| D8-5 | IMP-028 | improvement | triaged | Funnel (explicit opt-in) |
| D8-6 | IMP-029 | improvement | closed · done | URL generation |
| D8-7 | CHR-002 | chore | closed · done | Docs and QA |
| D8-8 | IMP-030 | improvement | closed · done | Remote-access settings clean-up |
| D8-9 | IMP-031 | improvement | closed · done | Dual Browser Source URL sets and UI pass |
| D9 | CHR-003 | chore | closed · done | Custom overlay author documentation |
| D10 | CHR-001 | chore | triaged | Wiki screenshots for the v1 guide |
| M9-7 | ISS-018 | issue | triaged | six-caster row clips social row |
| R3 | CHR-006 | chore | closed · accepted | original pre-rebuild overlay HTML |
| A17 | CHR-005 | chore | triaged | A17 second-machine project import |
| §C group | IDE-017 | idea | triaged | Protocol and server roadmap (§C) |
| §C group | IDE-018 | idea | triaged | Overlays and presentation roadmap (§C) |
| §C group | IDE-019 | idea | triaged | Match and show control roadmap (§C) |
| §C group | IDE-020 | idea | triaged | Tournament roadmap (§C) |
| §C group | IDE-021 | idea | triaged | Assets and data roadmap (§C) |
| §C group | IDE-022 | idea | triaged | UI and UX roadmap (§C) |
| §C group | IDE-023 | idea | triaged | Platform and delivery roadmap (§C) |
| Follow-up note | CHR-004 | chore | closed · done | Font A/B review checkpoint |
| Follow-up note | IDE-024 | idea | triaged | Protocol v2 review checkpoint |
| Follow-up note | IDE-025 | idea | triaged | Tournament overlay review checkpoint |
| Follow-up note | IMP-032 | improvement | triaged | Placeholder TAG image for empty slots |
| Follow-up note | IDE-026 | idea | triaged | In-app variable and script inspector |
| Follow-up note | IDE-027 | idea | triaged | Legacy Electron app.json importer |

No v1.0.0 content, schema, protocol or QA state was changed by the migration.
Screenshots remain unpublished; D10 is parked, not approved.

<!-- doctrine-migration-map:end -->

## Y. Post-v1 UI polish — IMP-001 and IMP-002 (2026-11-21)

The owner selected the UI-polish slice after the v1 closeout. Both items are
now terminal: **IMP-001**, more visible collapsible elements, is closed · fixed;
**IMP-002**, collapsible nav sidebar, is closed · done.

IMP-001 keeps native `details` semantics and strengthens the existing
`CollapsiblePanel` / `Disclosure` components: a persistent accent chevron,
accent summary edge while collapsed, and hover/focus-visible treatment from
existing theme tokens. There is no new disclosure system and no custom keyboard
handling. A native summary click opened the panel, Enter closed it, and the open
state survived an unrelated compact-nav toggle.

IMP-002 adds a session-only compact navigation toggle built from the existing
Lucide `PanelLeftClose` / `PanelLeftOpen` icons. The shell grid moves from
`248px 1032px` to `72px 1208px` at 1280×800 and is `72px 303px` at 375×812.
The seven labels stay in the DOM as clipped text with matching `aria-label`,
`title` and `data-label`; the accessibility tree exposes all seven names, and
hover/focus reveals a tooltip built from `data-label`. Active destination
indication and `aria-current` remain intact. The flag stays in a Dioxus shell
signal because `AppState::settings` is persisted broadcast/overlay/timer
configuration; adding a view preference there would change `app.json` without a
schema change.

A49 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real desktop
WebView evidence at 1280×800 and 375×812. No schema, protocol, server runtime,
route, destination or v1.0.0 overlay changed. No screenshots were taken or
published, and the authoritative QA hashes remain
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`.
