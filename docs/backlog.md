# Backlog — post-parity and v1 correctness fixes

Living document. Ordered by priority within each section. IDs are stable for referencing in milestones.

## A. v1 correctness fixes (agreed as part of parity)

These are defects that currently make a shipped feature fail, risk data loss, or create an external runtime dependency. They are in scope for v1.

| ID | Fix | Detail |
|---|---|---|
| C1 | Local fonts | Replace all `https://design.penpot.app/...` `@font-face` URLs with locally served OFL fonts. Replace the Google Fonts `Inter` import in the UI with a bundled or system font. No runtime network dependency for fonts. Apotek is commercial (Kerns & Cairns) and cannot be redistributed; **Archivo** (OFL 1.1) is now the active overlay font behind neutral `FGC Display*` aliases, with **Saira** retained for the A/B review. See `docs/assets.md` §4 and `docs/fonts.md`. |
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
| C14 | Map game-mode links | **Fixed 2026-11-14** — Map assets now have a **Game mode…** link editor in Assets. Unlinked maps still group under **Other** on the map-pool overlay. Found during manual QA on 2026-11-12. |
| C15 | Asset cleanup after a failed load | **Open — found 2026-11-15.** If `app.json` and `app.json.bak` are both unreadable, the app starts with the empty default state. The startup GC is skipped (`loaded == false`), but **Clean up now** and the post-save GC still build the keep set from the empty state plus `store.load().ok()`, which is `None` on the same error, so every user blob is deleted. Guard both paths on a successful load (or an explicit import) and show a notice instead of deleting. See `docs/decisions.md`. |

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
| B6 | Bulk asset import | The importer opens one file per dialog. Add a multi-select import for the chosen category, with the same validation, hashing and dedupe as the single-file path. Requested 2026-11-12 during QA fixture setup. |

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

### C1. Overlay visual polish (requested 2026-11-11 — post-v1)

A focused pass over the seven existing overlays so they look good and play nice
in OBS. This is **not** a redesign: the current layouts, hierarchy and visual
identity stay. The overlays must remain drop-in compatible with the existing
protocol and the 1920×1080 canvas.

| ID | Item | Scope |
|---|---|---|
| O1 | Social row fit | Fix the 6-caster `row-casters` social-row overflow (~14px with long handles) without changing the card layout. |
| O2 | Long-name handling | The R1 fix already shrinks then ellipsizes team, player, talent, map and hero names on one line, with 32/24-character input limits. Remaining polish: tags, pronouns and handles, and any multiline treatment if the design ever wants it. |
| O3 | Spacing and alignment | Tighten inconsistent padding, baselines and vertical rhythm while keeping the existing composition. |
| O4 | Safe areas | Keep content clear of the 1920×1080 edges and OBS scaling; no scrollbars or accidental overflow. |
| O5 | Visual QA sweep | Re-run the full overlay checklist in OBS after the pass and record the result. |

Guardrails:

- Preserve the current visual design: same sections, hierarchy, colours and
  typography roles. This is polish, not a new skin.
- No protocol changes: same filenames, message types, `init` handling, DOM
  contracts used by CSS, and `show` behaviour.
- Keep the five `FGC Display*` faces and local-only assets.
- Prefer CSS-only changes; keep JavaScript changes minimal and covered by the
  static overlay tests.
- Every layout change must be checked in OBS at 1920×1080 and reflected in
  `docs/overlay-qa.md`.
- Do not pull in backlog features such as a timer overlay, bracket overlay,
  scenes or new broadcast types.
- Update `docs/assets.md` §5 and the QA checklist if a behaviour or expected
  layout changes.

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

**Goal.** One overlay server with four access scopes: this PC (default), the
local network (the existing **Allow LAN connections**), a remote device over the
user's own Tailscale tailnet, and optionally the public internet through Tailscale
**Funnel**. Tailscale is detected, never bundled: the feature stays off until the
user turns it on and the Tailscale client is installed.

**What A3 proved (2026-11-15).** An overlay page served from a second origin
connected to a non-default port and updated live. That run only worked because
the browser was on the host PC: every overlay still dials
`ws://localhost:<port>`, so a second device would dial its own localhost. Remote
access therefore needs an origin-aware connection first (D8-1); the Tailscale
plumbing is the easy part.

| ID | Item | Scope sketch |
|---|---|---|
| D8-1 | Origin-aware overlay connection | Replace `ws://localhost:${port}` in all seven overlays with a resolver: same-origin `ws(s)://location.host` when the app served the page, `?host=`/`?port=` overrides for third-party pages, `wss` when the page is HTTPS. No message, filename or DOM change. |
| D8-2 | Remote-access settings | **Settings -> Remote access**: Off / LAN / Tailscale / Funnel, a status line (client version, tailnet name, serve/funnel state), and copy/open links per overlay for the active scope. |
| D8-3 | Tailscale detection | Find `tailscale.exe` (PATH, then `%ProgramFiles%\Tailscale`); read `tailscale status --json` (`BackendState`, `Self.DNSName`, `Self.Online`) and `tailscale version`. Show plain-text remedies for not installed, stopped or logged out. |
| D8-4 | Serve lifecycle | On server start in Tailscale mode run `tailscale serve --bg --https=443 --set-path /fgc http://127.0.0.1:<port>`; remove only this app's config when the server stops, the mode changes or the app exits. Never run `tailscale serve reset`. |
| D8-5 | Funnel (explicit opt-in) | The same path prefix through `tailscale funnel` on 443. Requires HTTPS certificates and Funnel permission in the ACL. Confirmation dialog naming the public exposure, plus an unguessable path token. Resets to off on every launch. |
| D8-6 | URL generation | Tailscale/Funnel: `https://<dnsname>/fgc/overlays/<file>.html` (no `?port=`); LAN: `http://<lan-ip>:<port>/overlays/<file>.html`; local: unchanged. The token appears only in Funnel URLs. |
| D8-7 | Docs and QA | Wiki page "Remote overlays" (setup, ACLs, firewall, teardown, troubleshooting); runbook tests for a second LAN PC, a tailnet device and a phone on cellular; evidence that teardown leaves `tailscale serve status` empty. |

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
- Overlay visual polish pass (see §C1): fit, spacing and long-name handling without changing the design or protocol.
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

- **Font review (after the R1 re-run):** compare Archivo (current) against Saira in situ at 1920×1080, using the same fixture. Barlow remains untried. A decision either way is a five-file WOFF2 and `src:` swap plus the test prefix; no layout CSS change.
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
