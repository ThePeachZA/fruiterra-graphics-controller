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
| B3 | ~~Overlay health/reconnect indicator~~ | **Done 2026-11-21.** The Overlay view shows **Connected clients** and **Last broadcast** using the existing status poll and a session-only broadcast timestamp, with a `role="status"` reconnect note when the server is running with zero clients. A51 verified 0/1 client states, broadcast-age ticking, restart and disconnect transitions, both viewports and the unchanged QA state; Doctrine IMP-021 closed · done. |
| B4 | ~~Basic hotkeys~~ | **Merged into IDE-004 on 2026-11-21.** The owner wants hotkeys only inside the auto-overlay controller: `1`–`7` select a bundled scene and arrow keys cycle the included subset. No global app or OS shortcuts. Doctrine IMP-022 closed · promoted. |
| B5 | ~~Timer overlay~~ | **Done 2026-11-21.** A standalone `timer.html` renders the existing `matchTimer` ticks with the sent alignment and leading-zeroes settings; it has no local clock and is not in the auto-overlay scene list. A55 verified live ticks, pause, stop, settings, `show`, transparency and no external requests. Doctrine IMP-023 closed · done. |
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

**Design pass started 2026-11-22.** The owner’s direction is OWWC27-inspired
Night Broadcast: dark chrome, team colours as primary identity, Fruiterra
orange as the system signal, geometry and protocol unchanged. Philosophy,
tokens and layout concepts are in
[`docs/overlay-refresh-design.md`](overlay-refresh-design.md); the owner’s
review answers are recorded in §8.

**Pilot done 2026-11-22.** `game-overlay.html` is the first overlay restyled
to the Night Broadcast tokens. The team bars are now dark surfaces with a 1px
inset line and inverse score chips, the retired white/black/purple palette is
gone, the match bug sits on the raised surface, and the format text carries
the orange accent. Filenames, wire protocol, geometry and the 1920×1080 canvas
are unchanged, and no official marks were introduced. A57 in
[`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence in both team-colour states.

**Rollout done 2026-11-22.** The remaining six match overlays and the
standalone timer now carry the same tokens:
`hero-bans.html`, `map-pool.html`, `map-select.html`, `row-casters.html`,
`team-roster-home.html`, `team-roster-away.html` and `timer.html`. The frozen
contracts hold — same filenames, same wire types and payload fields, same DOM
ids and classes, no new message types, no schema or persistence change, no
geometry move, 1920×1080 transparent canvas, no remote assets and no new
fonts. `auto.html` stays a pure iframe host, so it is out of the restyle; it
was verified separately and its missing favicon link was fixed (A59).

Per-overlay targets are in
[`docs/overlay-refresh-specs.md`](overlay-refresh-specs.md) (§4.9 records the
one deviation: the row-casters social icons take `--fgc-text-muted` through a
stylesheet rule rather than a literal `fill` attribute, with the same resolved
colour `rgb(141, 153, 173)`). A58 and A59 in
[`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) record the real WebView2
evidence: a per-overlay geometry/font/network comparison against the
pre-rollout baseline, an exact Night Broadcast colour check in the neutral
(team-colours-off) state, and a scene-switcher check of `auto.html`.

No overlay remains on the old palette.

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
- Plain standalone `timer.html` renderer done under IMP-023 (B5); it reads the app's countup/countdown/target ticks and applies sent alignment and leading zeroes. Richer timer presentation modes remain deferred.
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

Migrated records: **74**. Breakdown: issue/closed · fixed × 15, issue/closed · done × 1, improvement/closed · done × 25, improvement/triaged × 4, idea/triaged × 22, chore/closed · done × 3, chore/triaged × 2, issue/triaged × 1, chore/closed · accepted × 1. CHR-004 was closed · done after the font review reconciliation; B1, B2 and B3 were closed · done as post-v1 work.

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
| B1 | IMP-019 | improvement | closed · done | Show/hide overlay control |
| B2 | IMP-020 | improvement | closed · done | All overlay URLs with copy buttons |
| B3 | IMP-021 | improvement | closed · done | Overlay health/reconnect indicator |
| B4 | IMP-022 | improvement | closed · promoted | Basic hotkeys (merged into IDE-004) |
| B5 | IMP-023 | improvement | closed · done | Timer overlay |
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
| M9-7 | ISS-018 | issue | closed · fixed | six-caster row clips social row |
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
| Owner request | IDE-004 | idea | closed · done | Auto-overlay scene switcher (auto.html) |
| Follow-up note | IDE-024 | idea | triaged | Protocol v2 review checkpoint |
| Follow-up note | IDE-025 | idea | triaged | Tournament overlay review checkpoint |
| Follow-up note | IMP-032 | improvement | closed · done | Placeholder TAG image for empty slots |
| Follow-up note | IDE-026 | idea | triaged | In-app variable and script inspector |
| Follow-up note | IDE-027 | idea | triaged | Legacy Electron app.json importer |
| Owner request | IDE-028 | idea | open | Animated HTML stinger transitions (team and head-to-head) |
| Owner request | IDE-029 | idea | open | Flythrough overlay with looping video and lower-third series data |

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

## Z. Post-v1 overlay health — IMP-021 (2026-11-21)

B3 is now terminal as Doctrine **IMP-021**, closed · done. The Overlay view
reports **Connected clients** from the existing server-status poll and **Last
broadcast** from a session-only timestamp in `ServerController`. The timestamp
is recorded only when a frame is actually sent, including the initial
start/restart frames, and is cleared on start, restart and stop. A one-second
view clock ages the display; a running server with zero clients also shows a
`role="status"` reconnect note. Nothing is added to `AppState`, `app.json`,
the protocol, the server runtime, routes or the bundled overlays.

A51 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real desktop
WebView DOM, accessibility-tree and `/health` evidence at 1280×800 and 375×812:
0/1 client counts, `Just now` → `1s ago` ageing, restart reseeding, disconnect
and reconnect notes, and `Never` after stopping. Both viewports had
`scrollWidth == innerWidth`, with the new rows and note inside the viewport and
exposed in the accessibility tree. At 375×812 the pre-existing floating timer
initially covered Stop/Restart; after a normal wheel scroll they were
hit-testable and Stop worked. Harness and authoritative QA hashes were
unchanged, `tailscale serve status` was `No serve config`, and no screenshots
were taken or published. The authoritative hashes remain
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`.

## AA. Post-v1 auto-overlay scene switcher — IDE-004 / IMP-022 (2026-11-21)

IDE-004 is closed · done. IMP-022 (B4, basic hotkeys) was closed · promoted
into it, so there is one merged feature and no separate hotkey work elsewhere.
One OBS Browser Source now loads the new `auto.html` page and shows exactly one
iframe from the owner's included subset of the seven bundled overlays. The
default session starts with only `game-overlay.html` included and active.

`autoOverlay` is an additive wire section,
`{ "type": "autoOverlay", "data": { "included": [...], "active": "..." | null } }`.
The app broadcasts it on start, restart and every selection change, so it is
also present in `init` and `/state` while the server runs. Nothing is added to
`AppState` or `app.json`: selection and active state stay in Dioxus signals for
the session. The seven frozen v1 overlays are unchanged; `auto.html` is a new
post-v1 page that allow-lists only those seven filenames and sanitises both
`included` and `active`.

The Overlay view uses native checkboxes, native radios, a `role="status"` scene
line and the existing `Panel` / `CollapsiblePanel` / `Disclosure` patterns. Keys
`1`–`7` activate the overlay at that fixed bundled position when it is
included; `ArrowLeft` and `ArrowRight` cycle the included subset. Handlers live
only on the controller container, ignore form-control events, call
`stop_propagation()` only for handled keys, and never install a global OS hook.
Removing the active scene falls back to the first remaining included scene; an
empty subset clears the active scene and hides every frame. The auto URL follows
the same hidden/reveal and access-group rules as the other Browser Source
URLs.

A52 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real desktop
WebView evidence at 1280×800 and 375×812: seeded state, connect and reconnect
client counts, scoped hotkeys and excluded-scene no-ops, active-scene fallback
and empty subset, restart reseeding, status and disclosure accessibility, hidden
and revealed URL, eight-source count, and no feature overflow at either width.
Two defects found in that run were fixed and re-verified before sign-off. No
schema, persisted state, v1 overlay, Funnel setting or public exposure changed.
No screenshots were taken or published; the authoritative QA hashes remain
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` remained `No serve config`.

## AB. Post-v1 caster row fix — ISS-018 / M9-7 (2026-11-21)

ISS-018 is closed · fixed. The owner approved the scoped fix after the v1
closeout: keep the caster wire protocol, `app.json` schema and 1920×1080 overlay
geometry unchanged, and solve the clipping inside `row-casters.html`.

The real WebView2 baseline reproduced M9-7 with six casters:
`@anextremelylongcastersocialhandle` crossed the card right edge by `26.75px`
and the left edge by `96.92px`, and `@longcasterhandlefive` crossed the left
edge by `23.67px`. The fix constrains `.talent-subtext` to the card content box
with `max-width: 100%` and `min-width: 0`, keeps `.social-icon` at
`flex: 0 0 20px`, and lets `.social-handle` and `.pronouns` ellipsize with
`overflow: hidden`, `text-overflow: ellipsis` and `white-space: nowrap`. The
rendered handle span also carries the full handle in `title`.

A53 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the post-fix DOM and
computed-style evidence at 1920×1080. All six social rows stayed inside their
250px content boxes; the long handles ellipsized at 142/240px and 159/394px,
their pronouns at 47/79px and 29/72px, and each full handle remained in `title`.
The 1- and 4-caster states kept their 900px and 415px cards, the document stayed
1920×1080, and there were zero page console errors. The static guard
`row_casters_social_row_stays_inside_the_card` was added to
`crates/server/tests/overlays.rs`. `cargo fmt --all`, format check, check,
clippy, test, `git diff --check` and `dx build --desktop` passed; no schema,
protocol, persisted state or other v1 overlay changed. No screenshots were
taken or published, the authoritative QA hashes remain
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`.

## AC. Post-v1 empty-logo placeholder — IMP-032 (2026-11-21)

IMP-032 is closed · done. It completes the owner's deferred A9 note: when a
team has no logo, the game overlay's 75×55 logo slot now shows the team TAG on
the existing logo background instead of an empty coloured box. The tag comes
straight from the existing scoreboard projection; blank tags fall back to a
short name abbreviation (`Northwind Sentinels` → `NS`, `王小明战队` → `王小`).
No protocol, schema, persisted state or asset changed.

The placeholder is a text span inside the shipped logo element. It uses
`FGC Display SemiBold` at 24px, is confined to the 63×43 logo content box with
`max-width:100%`, `overflow:hidden`, `text-overflow:ellipsis` and
`white-space:nowrap`, and keeps the complete tag in `title`. It is hidden as
soon as `logoUrl` is present, cleared for `null` teams, and left alone for a
partial frame with no team fields. Team-colour mode sets a contrast colour from
the logo background: white on `#20242b`, dark on `#f5f5f5`.

A54 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence at 1920×1080. The live server `init` from the untouched fixture painted
`T3`/`T1`; replayed frames verified partial-frame preservation, logo-present
hiding, `LONGCASTERTAG` ellipsis at 63/227px with the full title, name
fallbacks, `null`-team clearing and contrast colours. Document size and logo
geometry stayed at 1920×1080 and 75×55, with zero page console errors. The
static guard `game_overlay_shows_a_tag_placeholder_for_empty_logo_slots` was
added to `crates/server/tests/overlays.rs`. `cargo fmt --all`, format check,
check, clippy, test, `git diff --check` and `dx build --desktop` passed; no
screenshots were taken or published, the authoritative QA hashes remain
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
`A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`, and
`tailscale serve status` was `No serve config`.

## AD. Post-v1 timer overlay — IMP-023 / B5 (2026-11-21)

IMP-023 is closed · done. The owner narrowed B5 to a plainly rendered timer
with no extra features, so the implemented `crates/server/overlays/timer.html`
is only a renderer for the existing `matchTimer` wire contract. It shows the
last tick, applies the tick's `settings.leadingZeroes` and `settings.alignment`,
renders the `reset` payload, and obeys top-level `show`. It intentionally has no
local clock, no countdown logic, no new message type, no persisted state and no
auto-overlay scene entry; add it to OBS as its own Browser Source.

The page uses the existing embedded-overlay route, origin-aware WebSocket
resolver, `requestInit` handshake and 2 s reconnect. It has a local `data:`
favicon so a Browser Source makes no missing-resource request. Static guards
live in `crates/server/tests/timer_overlay.rs`, separate from the frozen
seven-overlay suite, so the new page cannot weaken the v1 font, network or
subscription assertions. README, protocol, build, wiki and QA docs now describe
the standalone source while leaving the v1 C13 decision intact.

A55 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence. Live app ticks advanced to `00:00:02`, pause held that value, stop
reset to `00:00:00`, unpadded text rendered `0:0:4`, left/right placement put the
text at x=96/x=1422.55, and `show` changed `display` between `none` and `flex`.
The 1920×1080 document stayed transparent with the bundled font loaded;
external requests, failed responses and console errors were all empty. The QA
app resolved its data directory through Windows Known Folders despite an
isolated `APPDATA`, so the QA-mutated state was archived and the authoritative
hashes restored. No screenshots were taken or published, and
`tailscale serve status` remained `No serve config`.

## AE. Stinger transitions idea — IDE-028 (2026-11-21)

IDE-028 is open and tagged `area:overlay`, `needs-design`, `post-v1`,
`user-requested` and `prior-art:docs-c`. It needs IDE-004 (auto-overlay host)
and follows IDE-018 (§C presentation roadmap). No code was implemented: this
item records the owner's request for animated HTML stingers that incorporate
tournament and team logos, available separately and also built into the auto
overlay. A team transition should feature that team's logo; the
`game-overlay.html` transition should show the two logos head-to-head.

The item's prose records research across OWL, OWWC and OWCS, LoL Worlds/MSI/LCK/
LPL, and VALORANT VCT. The common convention is a modular, late-bound system:
one shared motion language and swappable team/event data, rather than unrelated
bespoke clips. OWL used fast 2D swipe/wipe motion; Riot toolkits handled many
teams with late-bound logos and colours; OWCS used regional wordmarks and
palettes; VCT used timing tokens and responsive text; LCK kept overlays
restrained. The proposed direction is separate transparent HTML pages plus a
reusable auto-overlay module, with fallbacks for missing logos and colours and
no CDN or official-mark copying.

## AF. Flythrough overlay idea — IDE-029 (2026-11-21)

IDE-029 is open and tagged `area:overlay`, `needs-design`, `post-v1`,
`user-requested` and `prior-art:docs-c`. It follows IDE-018. No code was
implemented: this item records the owner's request for a flythrough overlay
with a looping video file and a lower-third bug showing who is playing, current
score, current round/map and ban information. The owner will share a personal
example before design starts, and that reference must be recorded in the item
first.

Research found no formal industry use of the term “flythrough”; the closest
documented patterns are broadcast/package openers, perspective-grid or
push-through transitions, team/player intros and persistent match bugs. Riot's
world-feed practice favours clean, localisation-friendly graphics zones; OWL
favoured fast 2D motion; VCT/BUCK used perspective grids for forward momentum;
MSI 24 showed the value of one adaptive motif; and community Overwatch package
docs list the data a match bug carries. The proposed direction is a transparent
1920×1080 page with a local muted looping video, a compact lower third, the
existing overlay projection as its data source, a still fallback, explicit
empty states and no remote assets. Official marks remain out of scope without
permission.

## AG. Timer hour-display setting (2026-11-21)

The owner asked to customise whether hours show, with hidden hours folded into
the minutes (`1 hour 12 minutes` → `72 minutes`). The timer stays plain: this
adds one **Show hours** switch beside Leading zeroes and Standalone timer, and
one persisted `settings.timer.showHours` field. It adds no countdown logic,
visual treatment, extra overlay or new message type to `timer.html`.

`showHours` defaults to `true`, so existing state files and third-party overlays
keep the previous behaviour. A `matchTimer` tick still carries the true `hours`
and `minutes`; `timer.html` does the roll-up only for display and keeps the last
tick settings so a `reset` frame without `settings` still renders correctly.
Changing the switch rebroadcasts one tick so a paused overlay updates at once.

A56 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: `01:12:00` toggled to `72:00` and back, `/state` and `app.json` stored
`showHours: false`, a live tick rendered `71:59`, pause held it, Stop reset to
`72:00`, and `show`, transparency, font and network/console checks stayed clean.
Authoritative QA hashes and `No serve config` were rechecked and no screenshots
were taken or published.

## AH. Owner overlay review list — talent overlays and the Night Broadcast pass (2026-11-22)

The owner reviewed the Night Broadcast rollout and asked for a set of changes.
The agreed order is **overlays first, theme last**: the theme system touches
every overlay's tokens, so building it first would mean re-doing each overlay
on top of it afterwards.

| Slice | Item | State |
|---|---|---|
| A | Split the talent row: casters only on `row-casters.html`, plus new `row-hosts.html` (host + analysts) and `row-interviews.html` (interviewer + guest) | **Done — A60** |
| B | Team roster: core player cards fill the container width instead of a fixed 284px (substitutes keep their fixed width) | **Done — A61** |
| C | Map pool: stagger each game-mode column so the reveal animates column by column | **Done — A62** |
| D | Map select: show the ban thumbnails over the bottom of the map image, minimal | **Done - A63** |
| E | Game overlay: drop the outline rings, put the team logo on the outer end, move the ban icon next to the score, add a team-abbreviation toggle, review gradients and background textures | **Done - A64** |
| F | Hero bans: bigger redesign - two large hero-art panels with `1ST BAN` / `2ND BAN` tags and the mode and map name in a dark bottom strip | **Done - A65** |
| G | Theme system: a built-in Light theme beside Night Broadcast, JSON theme import, live switching with no rebuild | **Done - A66** |
| H | Font weights: align every overlay to one type scale and fix the weights that were being ignored | **Done - A67** |
| I | Hero bans: a full-bleed OWWC27 layout with the team logo in the corner, the role hexagon, the hero name over the art and the ban order in the team colour | **Done - A68** |
| J | Hero bans: make the hexagons a deliberate shape language instead of a texture that just exists | **Done - A69** |

The theme model was chosen as a **built-in Light theme plus JSON import**: the
app ships the dark Night Broadcast theme and a Light theme, and the user can
import a JSON file of token overrides. A user theme cannot arrive as a `.css`
file, because the overlay server only serves `/overlays/{file}`,
`/assets/fonts/{name}` and image assets, so the active theme must travel as a
payload field like `teamColorsEnabled` does.

### Slice A — talent overlay split (A60, 2026-11-22)

`castersData` already carried the on-air `role` of each assignment, so the
split needs no change to that array. The only protocol change is additive: the
`casters` payload gained an optional `interviewee` object, because the
interviewee was never projected and the new interview overlay needs them.

- `row-casters.html` renders only the `caster` entries.
- `row-hosts.html` renders the `host` and `analyst` entries, each with a role
  tag.
- `row-interviews.html` renders the `interviewer` and the guest card, with the
  guest in the accent colour and a `Guest` tag.
- All three now clear the row when the filtered list is empty, which fixes the
  stale-card behaviour of the old caster row, and they only re-render when the
  rendered set actually changes.

The app lists nine overlays now, so the auto-overlay scene switcher and the
`1`–`9` hotkeys cover the two new rows as well.

A60 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the casters row showed only the two casters, the hosts row showed the
analyst and the host with their role tags, the interview row showed the
interviewer and the guest, the wire carried the interviewee, and every overlay
stayed transparent, 1920×1080, font-loaded and free of external requests,
failed responses and console errors.

### Slice B — flexible roster cards (A61, 2026-11-22)

The core player row was hard-wired to 284px cards, which only fit the row at
exactly five players: a sixth card ran off the canvas and a shorter roster
left a gap. The cards now share the row with `flex: 1 1 0` and `min-width: 0`,
and the row no longer centres itself, so the cards always fill the 1500px
roster width. Substitutes keep their fixed 284px width because they are a
separate, right-aligned strip.

The card width is now decided by the roster size: five cards are still 284px,
three become 486.66px and six become 233.33px. Card height, padding, the role
badge, the portrait box and the text blocks are unchanged, and the name still
auto-shrinks before it ellipsizes.

A61 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: five, three and six core cards all filled the row to within a
sub-pixel, the substitutes held 284px, and both roster overlays stayed
transparent, 1920×1080, font-loaded and free of external requests, failed
responses and console errors.

### Slice C — map pool column stagger and the picked-map glow (A62, 2026-11-22)

The map pool revealed every mode column at once, which made the pool read as one
flat block. Each column now reveals on its own with a `columnReveal` keyframe
(fade plus a 36px rise) and a 0.13s stagger, starting 1.1s after the header.
The columns also had to survive the live frame: the pool was rebuilt on every
`scoreboard` frame, which wiped a running reveal, so the renderer now keeps a
signature of the pool contents and skips a rebuild when nothing changed.

The owner then asked for the OWWC27 look on a picked map. The old picked scrim
was a flat wash drawn inside the greyscaled image, so the colour was drained
before it was seen. Picked maps now carry a bottom-weighted accent gradient
over the art and a matching gradient plus a 2px accent rule on the name band,
with the team logo painting above the glow. The greyscale stays, so the pick
still reads as picked.

A62 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the four columns became visible 190ms apart with delays of 1100,
1230, 1360 and 1490ms, the picked map kept `grayscale(1)` under a
`rgba(255, 122, 26, …)` gradient, the accent rule resolved to
`rgb(255, 122, 26)`, the logo stacked above the glow, and the overlay stayed
transparent, 1920×1080, font-loaded and free of external requests, failed
responses and console errors.

### Slice D — map select ban thumbnails (A63, 2026-11-22)

The map select overlay had no trace of the hero bans, so the pick and the ban
were two separate pieces of information on screen. Following the OWCS27 and
OWWC27 references, the banned heroes now sit inside the map image: a compact
row of portrait thumbnails anchored to the bottom-left of the art, each with a
numbered diamond badge in the corner.

The order follows the first-ban team, so the hero banned first is number 1 and
the other team's ban is number 2. The wire was not changed: the overlay already
receives `heroBans` (one entry per map) and `heroPortraits` from the scoreboard
payload. The strip is a sibling of the map image inside a new
`.map-card__visual` wrapper, so a completed map's greyscale filter never drains
the colour from the ban portraits.

A63 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the first map showed Ana as ban 1 and Kiriko as ban 2, the second
showed Winston before Tracer because team B banned first, the third showed
Sojourn and Lucio, and the map without bans stayed empty. The strips sat inside
the image area, the portraits were not greyscaled, the badge resolved to a 45°
rotation with a counter-rotated label, and the overlay stayed transparent,
1920×1080, font-loaded and free of external requests, failed responses and
console errors.

### Slice E - game overlay rework (A64, 2026-11-22)

The owner reviewed the game overlay and asked for four changes. The three
hairline `inset 0 0 0 1px var(--fgc-line)` rings on the team bars, the top bars
and the centre bug are gone; the panels now carry a soft drop shadow instead.
The file never had gradients or background textures, so that part of the
review resolved to keeping the panels flat and opaque rather than adding any.

The team logo moved to the outer end of each bar and the ban block moved in
next to the score, following the OWWC27 and OWCS27 references: the home bar
reads Logo | Name | Ban | Score | Side and the away bar reads Side | Score |
Ban | Name | Logo. The team name now hugs the logo, so its alignment flipped
with the order.

A new **Team abbreviations** switch in Match Controller -> Config -> Overlay
appearance shows the short team tag instead of the full name. It is a persisted
display setting, `settings.display.teamAbbreviations`, defaulting to off so
existing broadcasts keep the full name. The wire carries it as
`teamAbbreviations` in the scoreboard payload, alongside the existing
`teamColorsEnabled` flag, and the overlay falls back to the full name when the
field is absent.

A64 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the home bar measured Logo | Name | Ban | Score | Side at x = 43,
118, 466 and 513, the away bar measured Score | Ban | Name | Logo at x = 1348,
1403, 1450 and 1798, no ring remained and the panels computed to
`rgba(0, 0, 0, 0.38) 0px 6px 22px 0px`. The full names rendered first, then
the real UI toggle changed them to the NWS and LONG tags and `/state` reported
`teamAbbreviations: true`. The overlay stayed transparent, 1920×1080,
font-loaded and free of external requests, failed responses and console
errors.

### Slice F - hero bans redesign (A65, 2026-11-22)

The hero bans overlay was the last one to keep its old layout. Following the
OWCS27 and OWWC27 references, the two ban portraits became two large hero-art
panels side by side. Each panel carries a white ordinal tag at the bottom-left
of the art, reading `1ST BAN` for the team that banned first and `2ND BAN` for
the other, and the hero's name sits in a dark band under the art. A team with
more than one ban keeps the rest as small portraits in the bottom-right corner
of the art.

The mode and the map name moved out of the old map-info block and into a dark
bottom strip, which reads the game mode with its icon, `MAP N` and the map
name. The wire was not changed: the overlay already receives `currentMap`,
`currentMapIndex`, `mapType`, `gameModeIcons`, `heroBans` and `heroPortraits`
in the scoreboard payload.

The review also found that the content overflowed the canvas by 30px: the
column was 1110px tall in a 1080px canvas. The padding dropped from 150px to
120px and the panel row grew to 630px, so the column is now exactly 1080px.

A65 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the two panels measured 747.5px each with a 25px gap, the home art
showed Ana and the away art Kiriko, the tags read `1ST BAN` and `2ND BAN` on a
`rgb(238, 242, 248)` plate, the second home ban sat in the extra row, the strip
read `CONTROL`, `MAP 1` and `ILIOS` with the mode icon, and the container's
bottom edge landed exactly on 1080px with no document scroll. The overlay
stayed transparent, font-loaded and free of external requests, failed
responses and console errors.

### Slice G - theme system (A66, 2026-11-22)

The owner asked for a theme system: a built-in Light theme beside Night
Broadcast, a JSON theme import, and live switching with no rebuild.

A theme travels as a new `theme` message rather than an imported stylesheet,
because the overlay server only serves `/overlays/{file}`, `/assets/fonts/{name}`
and image assets. The message carries the preset name and the **complete** set
of CSS custom properties to override. Each overlay tracks the properties it
applied and removes any that the next theme does not list, so an empty token
map restores the stylesheet defaults. The Night Broadcast values stay in the
overlays' own `:root` blocks, which keeps one source of truth, and only the
Light theme is defined in Rust.

Two settings drive it: `settings.display.theme` (Night Broadcast or Light) and
`settings.display.themeTokens`, the imported overrides. The import accepts
either a flat `{ "--fgc-surface": "#fff" }` object or a
`{ "name": "...", "tokens": { ... } }` file, and rejects anything that is not
a `--token` name or a plain colour value. The picker lives in Match Controller
-> Config -> Overlay theme. The `theme` frame is part of the ordinary
projection, so switching repaints every connected overlay at once and the
choice is saved with the rest of the settings.

A66 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the hero-bans panel went from `rgb(11, 13, 18)` to
`rgb(244, 246, 251)` and the ban tag from `rgb(238, 242, 248)` to
`rgb(17, 21, 29)` the moment the picker changed, eighteen custom properties
were applied inline, `/state` carried the preset and its tokens, `app.json`
stored `light`, switching back removed every inline property and restored the
stylesheet defaults exactly, and the timer overlay followed the same switch.
The overlays stayed transparent, 1920×1080, font-loaded and free of external
requests, failed responses and console errors.

### The font-weight consistency pass (A67, 2026-11-22)

The last item of the review list was a font-weight pass under **General**. The
owner chose the consistency scope: define one type scale, align every overlay
to it, and fix the genuine mismatches such as the two different score weights.
Sizes stay as they are.

The audit found a real bug underneath the cosmetic mismatches. Several rules
declared a `font-weight` their font family did not have, so the weight was
silently ignored and the text rendered at Regular. The game overlay's bar name
and bar score, the title, the subtitle, the map number and the format label
all inherited `body { font-family: 'FGC Display' }`, which only ships the 400
face, so every one of them rendered at Regular no matter what the rule asked
for. The timer overlay made the same mistake differently: its `@font-face`
declared the regular alias `'FGC Display'` at weight 600, so it hijacked the
alias every other overlay used for the 400 face.

The fix is a rule rather than a list of patches: **the alias carries the
weight, and a rule never declares a `font-weight` of its own.**

| Tier | Weight | Alias |
|---|---|---|
| Display | 900 | `FGC Display Black` |
| Strong | 600 | `FGC Display SemiBold` |
| Body | 500 | `FGC Display Medium` |
| Secondary | 400 | `FGC Display` |
| Quiet | 300 | `FGC Display Light` |

Display is the biggest headline on a screen, Strong is a score, a count or a
map name, Body is a person's or team's name and every label, Secondary is
small supporting text and Quiet is handles, pronouns and separators. The
twenty-three edits that follow it align the overlays: the game overlay's name,
subtitle, map number and format label became Medium and its score and title
SemiBold, the hero bans first-to count and map name became SemiBold, the map
pool mode name became Medium, the map select name, score and score divider
became SemiBold, the roster pronouns became Light to match the talent rows,
and the timer moved to the SemiBold alias it was already loading.

Four static guards in `crates/server/tests/overlays.rs` keep it honest: no
`font-weight` may appear outside an `@font-face`, every declared face must be
a tier and load the matching Archivo file, the nine match overlays must ship
all five faces, and each role must resolve to the same weight everywhere.

A67 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: twenty-one nodes were probed with `CSS.getPlatformFontsForNode`,
which reports the platform font a node actually painted with rather than what
the stylesheet asked for. Every probe painted the intended face, including the
game overlay bar name and bar score that had been rendering at Regular, and
every computed `font-weight` came back `400` normal, which is the rule working
as intended. The overlays stayed transparent, 1920×1080 and free of external
requests, failed responses and console errors.

### Slice I - the OWWC27 hero bans redesign (A68, 2026-11-22)

The owner was not happy with the hero bans screen: too many elements were on
it. Following the OWWC27 reference, the screen is now two full-bleed halves
with the hero art filling the frame, the team logo in the outer top corner, a
role hexagon, the hero name hovering over the lower half of the art and the
ban order underneath it in the colour of the team that made the ban.

The old header (team names, logos, scores and FIRST TO) and the bottom map
strip are gone. The map and mode stay on the map select overlay, where they
already were.

Two new pieces of hero data make the reference possible:

- **`Asset.transparent`** - a link to a transparent cut-out of the hero. The
  overlay prefers it and falls back to the regular hero art when it is
  missing, so a hero with no cut-out still renders.
- **`Asset.role`** - the hero's role, shown as the hexagon. There was no
  hero-to-role data anywhere before, so this is a new field on the hero asset
  itself, edited from the hero's **Properties** dialog.

Both travel on the scoreboard payload as `heroTransparent` and `heroRoles`,
keyed by the hero name, next to the existing `heroPortraits` and `heroIcons`.

The first version of the background was a restrained hexagon texture: a masked
accent-coloured fill at 14% opacity over the two halves and the centre
banner. The owner rejected it in slice J, below.

A68 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the two halves measured 905px each with a 110px centre banner and
reached the full 1080px height, the home logo sat at x = 43 and the away logo
at x = 1777 so each is in its outer corner, the role hexagon resolved to the
support and damage icons, the names rendered at 92px in the Display face, and
the ban labels rendered in the two team colours. The transparent slot was
preferred for the home hero and the regular art was used for the away hero.
The overlay stayed transparent, 1920x1080 and free of external requests, failed
responses and console errors.

### Slice J - the honeycomb wall (A69, 2026-11-22)

The owner's verdict on the first hexagons was that they "kind of just exist
meaninglessly there with no creativity behind it". They were right. The tile
was 56x98, which is roughly sixteen cells across a half, so it read as a
screen door rather than as structure. It was uniform, so it had no direction.
It was painted in the generic accent, the same hue as the ban label, so it
carried no information. It sat at `z-index: 0` behind the full-bleed art,
which meant it was invisible on every hero that had regular art and only
showed through a transparent cut-out. And it had also been pushed into the
centre banner, the one place the OWWC27 reference has none.

The hexagons are now the overlay's shape language instead of a texture. The
reference uses the same shape three times - a hexagonal team logo badge, a
hexagonal role badge and a honeycomb wall behind the hero - so the hexagon now
appears at three scales in one composition.

- **The wall** sits behind the art. It is a 56x100 tessellating SVG used as a
  mask at `180px 322px`, which is about five cells across a 905px half. The
  fill is `var(--fgc-surface-raised)` and the joints are
  `var(--fgc-line-strong)`, so the wall is neutral and the accent stays
  reserved for the ban label. A gradient fades it up into the surface from
  30% down, which clears the caption and gives the field a direction.
- **The grid** is a faint copy of the same tile in front of the art and the
  scrim, so the wall reads on regular hero art as well as through a cut-out.
  It is weighted to the outer edge of the half, where the team logo sits and
  the hero is not, and it is denser towards the caption. It never reaches the
  centre of the half, so the hero is never veiled.
- **The badges** share the wall's pointy-top hexagon. The team logo grew from
  100x100 to 100x115 and uses a `drop-shadow`, because a `clip-path` clips a
  box shadow away. Its artwork moved into the band the hexagon inscribes so
  the badge never cuts into the logo.
- **The banner** no longer carries a texture. It is the seam where the two
  teams meet, so it is a gradient from the home team's colour to the away
  team's colour.

No payload changed. The wall, the grid and the badges are static; only the
team colours, the logo artwork and the ban order follow the payload.

A69 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the wall, art, grid and caption computed to `z-index` 0, 1, 3 and 4;
the wall measured `rgb(17, 21, 29)` with `rgb(58, 67, 86)` joints, which is
not the accent `rgb(255, 122, 26)`; the mask size was `180px 322px`, about
five cells across a 905px half; the team logo measured 100x115 with the
pointy-top hexagon and its artwork in the inscribed band; the role badge
measured 60x68 with the same hexagon; and the banner computed to a gradient
between the two team colours. The overlay stayed transparent, 1920x1080 and
free of external requests, failed responses and console errors.

## AI. Content packs and the project folder (2026-11-22)

Two owner requests on 2026-11-22: publish a fully functional Overwatch project
as extra content on GitHub, and stop the terminal build from resetting the
project that was being used for testing.

### The reset

The application always resolved its data root through `ProjectDirs`, with no
environment variable and no command-line flag, so the debug build and the
shipped build shared one project folder. Every QA run swapped `app.json` in
and out of that folder and put the fixture back at the end, which is why the
terminal build kept opening the testing state.

`crates/core/src/paths.rs` now resolves the root in this order: `--data-dir`,
the `FGC_DATA_DIR` variable, a debug-only `Fruiterra Graphics Controller (dev)`
folder beside the release folder, and finally the platform project folder. The
active folder is shown in **Settings → Project folder**. A second bug was
fixed at the same time: `load_state` used to fall back to an empty project
when `app.json` could not be parsed, and the saver then wrote that empty
project over the operator's file. An unreadable file is now moved aside as
`app.json.broken-<n>` and reported.

### Content packs

A content pack is artwork you can share without sharing a project.
`crates/core/src/content_pack.rs` defines the format: a `pack.json` manifest
plus the `blobs/` folder, hash-verified on the way in. Installing a pack
**adds** images to the library and leaves teams, players, talent, tournaments,
settings and the running match untouched. A link that no longer resolves, for
example one that pointed at an icon the application used to bundle, is repaired
from the pack; a link the operator set themselves is never overwritten.

The Assets tab gained **Export pack…**, **Import pack…** and **Import pack
from URL…**; the URL route downloads a zip release, unpacks it and installs it
in one step. The command line gained `--import-pack <folder>`.

### A generic binary

The application used to bundle the Overwatch game-mode, role and side icons.
They now live in the pack, so the binary ships only its placeholder watermark
and the overlays resolve those icons from whatever the project holds. Because
the project stored links to the old bundled icons, the pack repairs them.

The pack is published at
[`ThePeachZA/fruiterra-overwatch`](https://github.com/ThePeachZA/fruiterra-overwatch)
with release `v2`: 250 assets, 200,035,760 bytes, 29 repaired map links and
none left dangling. It is installed in the operator's project, which holds the
four real teams, the tournament and the 13-map pool.

The `v1` tag still points at the first manifest - 157 assets with the operator's
own team logos and player portraits - while the `v1` download was replaced in
place with the 250-asset artwork set, so the tag and the asset disagreed about
what the release was. `v2` tags the commit that holds the manifest the asset
actually ships, and the release notes match the download. `v1` stays where it is
for anyone who already links to it.

The `v2` download was verified end to end against the real importer. The zip
unpacks to `pack.json` and `blobs/` at the root of the archive, which is the
layout the importer accepts, and
`fruiterra-gc.exe --data-dir <scratch> --import-pack <unpacked>` reported
`Installed 250 asset(s) and repaired 0 link(s) from the content pack (250
file(s), 200035760 bytes)`. The check ran in a scratch project folder, so no
real project was written to.

A70 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: before the install the map pool rendered one "Other" column with no
mode icons, the install toast read `Installed 10 asset(s) from the content
pack (10 file(s), 166.3 KB) Repaired 29 link(s).`, and afterwards the pool
rendered five columns - Control, Escort, Hybrid, Flashpoint and Push - with
all five icons loaded and 13 maps. The wire carried all five game-mode icons,
the three role icons and the two side icons. The QA ran against a copy of the
project in a separate `--data-dir`, so the operator's own projects were never
written to.

## AJ. Assets library rework and the hero artwork sync (2026-11-23)

Four owner requests on 2026-11-23: paginate the assets view because it took
forever to load, stop the application freezing while the image previews
loaded, clean up the option fields at the top of the assets page and define
project bundles against content packs, and finally add a sort order.

### Why the page froze

Every visible card asked the application for the whole blob of its image and
it was base64-encoded into a `data:` URL. Nothing was downscaled, and the
cache of those strings was never emptied, so a library of 150 assets at around
3.7 MB each held roughly 740 MB of base64 text for the lifetime of the
process. The encoding also ran on the application's own task, so the window
stopped responding while the previews appeared.

The overlay server already serves every asset at `/assets/<hash>.<ext>` with a
one-year immutable cache header, and that route does not depend on the asset
mode. The thumbnails now use it: the application hands the WebView a URL and
the image is fetched and decoded by the browser instead of by the application.
The inlined path is still there as a fallback for when the server is stopped,
and its cache is now capped.

### The page

- The library is the **first** panel on the page. The import controls moved
  onto one row under the toolbar, so the page opens with the assets rather
  than with a wall of panels.
- The library is **paged**, 24 assets at a time, with a pager that shows the
  range and the page count. A filter, a search or a new order returns to the
  first page.
- A **Sort assets** list offers Name (A-Z), Name (Z-A), Category, Newest first
  and Oldest first.
- A **skeleton** placeholder holds the space of a thumbnail that is still
  loading, and it respects `prefers-reduced-motion`.
- **Project bundles** moved out of the assets page and became **Settings ->
  Project backup**, where a whole-project move belongs. The assets page keeps
  **Content packs** and now says plainly that a pack adds artwork while a
  backup moves the project.

### The hero artwork

The owner pointed at `D:\Game Assets\Overwatch\Heroes` and a Google Sheet of
roles and map modes. A new `--sync-heroes <folder>` command reads the sheet
and the art folder and sets, for every hero, the ban icon from
`Icons\Square`, the badge from `Icons\Hexagon`, the transparent cut-out from
`Transparent\` and the role from the sheet, and it links every map to its game
mode. The project now has 54 heroes with a role, 54 with a cut-out, 54 with the
square ban icon and 49 with a badge - Dmon, Doctrine, Echo, Shion and Sierra
have no hexagonal source file - and all 29 maps resolve to a game mode.

The badge is a new optional attachment on a hero, `Asset.badge`, carried on
the wire as `heroBadges`, so the hero bans overlay can show it.

### The pack

The published pack was rebuilt from the synced project and now carries 250
assets: 211 hero files, 29 maps and the 10 game-mode, role and side icons.
Team logos and player portraits are deliberately left out; they belong to a
team, not to shared artwork. The repository README was corrected to match.

A71 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real WebView2
evidence: the library pages and sorts and filters, all 24 thumbnails served
over HTTP with no data URL among them, the project backup in Settings and the
badge, transparent art and role sections in the hero dialog.

## AK. Hero bans cut-out placement, outer edge wash and reveal delay (2026-11-23)

Owner feedback on 2026-11-23 after the honeycomb wall shipped: the transparent
hero cut-outs sat letterboxed in the middle of each half, the hexagons were too
opaque, the centre seam read as a seam, the team colour was on the ban label
instead of on the half, the `1ST BAN` / `2ND BAN` wording was shouted, and the
reveal started too early for a stinger or a scene transition.

### The cut-outs

The cut-out files are a 1920x1080 transparent canvas with the hero standing on
the bottom, so `object-fit: contain` fitted the whole canvas into the half and
left the hero at roughly 47% of the height, floating in the middle. The overlay
now measures the hero instead of trusting the canvas. `subjectBox(img)` scans
the alpha channel for the hero's opaque bounding box and `placeCutout(img, box)`
computes `const scale = box.height / subject.height;`, then centres that box on
the half with `transform-origin: bottom center`, so the hero fills the half
height and stands on the bottom. The `<img>` keeps its natural size and is
absolutely positioned, and `.hero-half__art--cutout-fallback` covers a canvas
that cannot be read.

### The hexagons and the seam

The wall joints went from opacity 0.5 to 0.3, the lit grid copy in front of the
art from 18% and 7% to 11% and 4%, and the floor wash from 6% to 3.5%. The
`.hero-bans__centre` element and its CSS were deleted, so the two halves tile
the full 1920 width with no banner between them.

### The edge wash and the labels

`.hero-half__edge` is a new layer at `z-index: 4`, over the art and over the
honeycomb. The home half washes in from the left edge with `--team-a-color` and
the away half from the right with `--team-b-color`, both set on `#hero-bans`
with `style.setProperty`. The ban-order label no longer takes the team colour;
it is neutral `var(--fgc-ink-on-art)`. `1ST BAN` / `2ND BAN` became
`First ban` / `Second ban` in the markup, and the CSS still uppercases them on
screen.

### The reveal delay

`--hero-reveal-delay: 0.8s` on `:root` is read by `triggerAnimations()`, so the
halves start at 0.8s and 0.92s and the captions at 1.15s and 1.27s. A stinger or
a scene transition can finish before the bans appear.

### Left open

The hero bounding-box centre includes the weapon and the outstretched arm, so
the box centre is not the visual centre. The measured offset was 19px, which is
1% of a 960px half, and that was accepted; a per-hero visual anchor would be
needed to do better, and nothing in the app asks the operator for one.

A72 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the real browser
evidence: 26 assertions ran, 24 passed and the 2 centring checks were accepted at
19px, the cut-outs were measured and scaled to the half height, the hexagons
were fainter, the seam was gone, the team colour washed in from the outer edge,
the labels were neutral and read First ban / Second ban, the reveal waited 0.8s,
and `cargo test -p fgc-server --test overlays` passed 29 tests.

## AL. Animated background overlay (2026-11-23)

The owner asked: "maybe you can help me make an animated background using html
and css? But it would probably be a separate overlay as well as built into the
existing ones." The separate overlay is the part that shipped. A follow-up
requirement landed after the page was built: "For the background overlay, please
note that it should also have a completely solid and opaque colour based on the
theme colour tokens."

### What shipped

A new bundled page, `crates/server/overlays/background.html`, titled **Animated
Background**. It is a persistent Browser Source that sits beneath the scene
overlays: the nine scenes replace one another, while the background stays on
screen for the whole broadcast. It is a tenth entry in `OVERLAY_FILES` and the
first one listed, so it appears in every Browser Source URL set, but it is not
part of the new `SCENE_OVERLAY_FILES`, so the auto scene switcher, its include
checks and its number hotkeys still hold exactly the nine scenes.

The page paints a solid, opaque plate from the theme tokens and is asset-free.
It loads no fonts and no images, paints no text, and requests only the document,
the WebSocket and its inline SVG mask.

### The plate

The plate is `background: var(--fgc-surface)` on `html` and `body`, with
`background-color: var(--fgc-surface)` on `.background`. Under Night Broadcast
`--fgc-surface` is `#0b0d12` (`rgb(11, 13, 18)`); the Light theme's
`LIGHT_THEME_TOKENS` sets `#f4f6fb` (`rgb(244, 246, 251)`). Because the colour
is a token rather than a literal, a `theme` message repaints the whole frame,
so the backdrop always belongs to the selected theme.

Hiding the source has to take the plate with it, otherwise the Browser Source
leaves a solid block on screen while the artwork is gone. A `show` message of
`visible: false` therefore adds `background-hidden` to `<html>`, and
`html.background-hidden` plus `html.background-hidden body` set
`background: transparent`, leaving the frame empty until the source is shown
again. The dead `--hero-reveal-delay` token copied over from hero bans was
removed in the same pass.

### The composition

A root `.background` element holds three layers. `.background__glow` is an
oversized ambient wash made of two radial gradients, one from each side of the
frame. Over it sit `.background__lattice--ambient` and
`.background__lattice--lit`, two copies of the shared hexagon SVG mask at
`mask-size: 180px 322px`, the same cell the hero bans wall uses. The ambient
copy is the quiet structure at opacity 0.13; the lit copy is brighter, weighted
to the outer edges, and leaves the centre of the frame comparatively plain for a
game feed.

### The motion

Three CSS animations, all `ease-in-out` and infinite: `glowBreathe` runs 84s and
takes the glow from opacity 0.46 to 0.72 with a small scale, while
`latticeDriftForward` (88s) and `latticeDriftReverse` (72s) drift the two
lattices in opposite directions. Only `transform` and `opacity` are animated, so
there is no repaint of the 1920x1080 surface and no per-frame JavaScript at all;
the file has no `requestAnimationFrame`. Both drift paths are closed, so they
reverse without a visible jump, and the lattice is overscanned by
`inset: -180px -220px` so a masked edge never enters the frame.

### The variants

OBS preserves the query string on a Browser Source URL, so the variants are URL
parameters rather than extra pages. `?motion=still` adds a `motion-still` class
that turns every animation off and leaves a composed first frame; any other
value drifts, so `?motion=drift` is just the default. `?tone=team` adds
`tone-team`, which swaps the two neutral greys for the live team colours and
falls back to the same greys when a team has no colour. The OS
`prefers-reduced-motion: reduce` preference turns the animations off through a
media query as well, so an operator who has asked for less motion gets the still
composition whichever URL they use.

### The wiring

`crates/app/src/server.rs` split the old nine-entry list in two: `OVERLAY_FILES`
is now `[&str; 10]` and starts with `background.html`, and the new
`SCENE_OVERLAY_FILES` is `[&str; 9]` and holds the scene overlays.
`crates/app/src/state.rs` and `crates/app/src/views/overlay.rs` use
`SCENE_OVERLAY_FILES` for the auto scene switcher, its include checks and its
number hotkeys, while the Overlay view still lists every `OVERLAY_FILES` entry
as a Browser Source URL. The note under the URL sets now reads "Each set
contains the auto scene switcher plus all ten overlays." The new static test
`background_overlay_is_broadcast_safe_and_supports_url_variants` guards the
layers, the three animations, the transform/opacity-only rule, the reduced-motion
and URL variants, the inlined Night Broadcast tokens and the absence of a
per-frame loop.

### Left open

The owner also wanted the background "built into the existing ones". This slice
ships the separate source only; the scene overlays still paint their own
surfaces, and because the new plate is opaque the background source has to sit
below them in the OBS scene collection. Folding the background into `auto.html`
was left for a later pass.

### QA

A73 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the browser
evidence: the solid plate in both themes (`rgb(11, 13, 18)` under Night
Broadcast and `rgb(244, 246, 251)` under Light), the `show` hide rule, the three
layers, the 84s, 88s and 72s animations, the two lattice matrices on load, the
shared 180px 322px hexagon mask, the `?motion=still` and `?tone=team` variants,
an empty console, and `cargo test -p fgc-server --test overlays` passing 30
tests. A screenshot of the running page showed the opaque plate with the lattice
over it.

## AM. Overlay design language rollout (2026-11-23)

This section tracks the work that grew out of the hero bans rework, the
animated background and the design council review of 2026-11-23. The owner asked
for all of the overlay updates, the design philosophy and the ideas to be
written down in detail; the canonical reference is
[`docs/overlay-design-language.md`](overlay-design-language.md). This section
records what is done and what is still open. Decisions are locked in
[`docs/decisions.md`](decisions.md) under "Overlay design language decisions".

### The ordered rollout

The council set a deliberate order so each step has a stable base. One overlay
or one mechanism is changed at a time, and each change carries a spec section,
a QA row, a runbook section and a source-text test where needed.

1. **Tokens in every overlay - done.** Every overlay inlines its own
   `--fgc-*` token block; there is no shared stylesheet URL. `auto.html` is the
   preview host and does not need the block itself.
2. **The canonical FX block plus the standalone background - partly done.**
   `background.html` shipped in commit `9600a65` with the solid opaque plate.
   The transparent canonical FX block shipped in eight scene overlays on
   2026-11-23.
3. **The block-equality test - done.**
   `every_overlay_carries_the_same_fx_block` extracts the sentinelled text and
   compares it byte for byte across the eight carriers. It also guards the
   overscan, mask tile, toggle, reduced-motion rule and the absence of
   per-frame JavaScript.
4. **Pilot the FX in `timer.html` - cancelled by the owner.** The owner said on
   2026-11-23 that there should be no background overlay attached to the timer
   or the game overlay. The first native carrier is `hero-bans.html`, which
   already had the hexagon wall. No FX block, sentinel or `.fgc-fx` element is
   present in `timer.html`.
5. **`game-overlay.html` and `hero-bans.html` - planned.** Hero bans shipped the
   FX mechanism and its visual rework in A72. The game-overlay visual refresh is
   still planned and will not add the FX layer, because of the owner's
   exclusion.
6. **The rest of the overlays - the transparent FX part is done.** The block is
   now in `map-pool.html`, `map-select.html`, `row-casters.html`,
   `team-roster-home.html`, `team-roster-away.html`, `row-hosts.html` and
   `row-interviews.html`, alongside hero bans. The individual visual refreshes
   below are still open.
7. **Reuse the cut-out locator - open.** `subjectBox()` and `placeCutout()` in
   `hero-bans.html` should be copied into any other overlay that receives a
   transparent 1920×1080 subject. If more than three overlays need it, guard the
   copies with another byte-equality test.
8. **The per-overlay design refreshes - open.** The council's plan is below.
9. **The app-level background toggle - done as a URL rewrite plus a cached
   section; FX injection open.** The Overlays view holds a session-only
   `background_opaque` signal behind an accessible `Switch` in a `ToggleRow`,
   appends `?bg=opaque` or `?bg=transparent` to the ten files in
   `BACKGROUND_MODE_FILES`, and broadcasts a `background` section to connected
   sources. Opaque paints the theme plate and keeps the hexagons over it;
   transparent hides the plate and keeps the lattice. `auto.html` forwards the
   mode to its scene iframes. A77 added the keyed init-frame parser so a Browser
   Source refresh restores the app-selected mode. No OBS refresh is needed. The
   implementation does not inject `data-fx`, so the separate app-level FX
   switch remains open.
10. **The early document overflow guard - done.** A78 added an `html, body`
    overflow guard to all twelve overlays so overscanned FX layers cannot show
    transient scrollbars on refresh.
11. **Documentation - continuous.** The design-language reference, the specs,
    the checklist, the QA register, the runbook, the decisions log and this
    backlog are part of each change, not a final tidy-up.

### Per-overlay visual plan

- **Game overlay.** Replace the centre bar with a hexagon score plate and
  hexagonal logo chips, leave the centre of the frame for the game capture, and
  wash team colour in from both outer edges. Keep the existing bars, names,
  scores, bans and side indicators otherwise unchanged. No FX layer.
- **Hero bans.** Shipped in A72: two full-bleed halves, subtler hexagons, the
  alpha-scanned cut-out placed on the bottom of each half, the outer-edge wash,
  neutral ban-order labels and the 0.8s reveal. The FX layer is now in place;
  further polish should be small and measured.
- **Map pool.** Keep the rectangular table so rows remain scannable. Add
  hexagon status chips, an outer-edge wash and staggered reveal rows. Never put
  a hex grid on a map thumbnail.
- **Map select.** Add a hexagon name plate and hexagon vote chips. A hexagon
  clip on the thumbnail is acceptable only if it does not cut the corners off
  the image. Put the team washes at the bottom outer corners, not in a centre
  bar. Keep the ban thumbnails and their diamond numbers working.
- **Row casters, row hosts and row interviews.** Add hexagon portrait frames and
  role chips, a per-card outer-edge wash, and reuse the cut-out locator where a
  transparent portrait is supplied. Keep the current role filtering exactly as
  it is: casters, hosts and interviewers with their guest card.
- **Team roster home and away - A79 identity, refined by A80.** The home wash
  enters from the left edge and the away wash from the right, both 760px wide.
  A80 softened the wash alpha to 0.12 and replaced the side-anchored radial glow
  with one 1920px-wide linear glow across the whole bottom of the frame at alpha
  0.14. The core role badge remains a 56×56 hexagon fully inside the portrait
  with a 40×40 icon. The substitute badge is now 46×54 with a 34×34 icon inside
  the 72×76 portrait, fixing the squashed look of the earlier 58×58 badge.
  Divider-style vertical rails were tried in A79 and removed in A80; do not
  reintroduce `.core-stage` or `.core-rail`. Both core and substitute rows use
  the existing role values in Damage, Tank, Support order with a stable tiebreak
  inside each role. The core plate is 126px with smaller pronouns. A hexagon
  separator still sits between core and substitutes, and the rows stay
  rectangular and readable. Keep the flexible card widths from A61. The original
  avatar-cell abbreviation is not shipped: the name plate is plain after the
  owner scrapped both the initials and the country/tag experiment. See decisions
  D14 and D15 and QA gates A79 and A80.
- **Timer.** Add a hexagon plate and keep tabular figures. Do not add
  per-frame JavaScript, an FX layer or a background plate. The owner's
  exclusion is absolute.
- **Auto.** Add hexagon preview cards and load every preview with `?fx=off`, so
  the preview and the live source do not render the lattice twice. The previews
  are the only place where the FX state is deliberately forced off.

### Risks and mitigations

| Risk | Mitigation in place or planned |
|---|---|
| OBS GPU cost over a long broadcast | Only `transform` and `opacity` animate; cycles are 72s or longer; `will-change` is on one layer; `?fx=off` and `?motion=still` give an immediate escape. Planned: a one-minute OBS watch in each overlay sign-off. |
| A visible drift seam if the hexagon SVG does not tile on 180×322 | The FX drift travels exactly `(180px, 322px)`, one mask repeat, and the layer overscans by `-360px`. The equality test guards both numbers. Planned: a visual seam check on the per-overlay refreshes. |
| OBS may not honour `prefers-reduced-motion` | The CSS media query freezes motion but keeps the lattice; explicit `?motion=still` and `?fx=off` variants do not depend on the OS preference. |
| The alpha-scan locator is fragile against soft edges and drop shadows | Threshold 8 is recorded; a canvas that cannot be read uses the `object-fit: cover` fallback. Planned: reuse the same two functions and test each new transparent asset before sign-off. |
| Neutral ink on very light artwork | `--fgc-ink-on-art` always sits on `--fgc-plate-dark`, never directly on the art. Planned: check both themes on every new text-over-art treatment. |
| Hexagon clipping cuts the corners off a logo | Badges use one inscribed band at 100×115 and 60×68. Planned: inspect any new clipped asset at both sizes before rollout. |
| Table readability | Hexagons are forbidden on table rows and map thumbnails; the roster cards keep their existing widths. Planned: keep the rectangular table in map pool and rosters. |
| A core role badge crosses back into the name plate | Guarded by A79 and retained in A80: the badge is 56×56 at `bottom: 16px`, the plate is 126px, and the source-text plus browser probes require a negative overlap. The old 76px badge at `bottom: -36px` is forbidden. |
| A substitute hexagon looks squashed | Fixed by A80: the badge is 46×54 with a 34×34 icon and an 8px inset inside the 72×76 portrait. Source-text assertions require all four dimensions; the browser probe measured a 0.852 width/height ratio and 0px centre offset. |
| Divider-style rails return around the card row | Removed by A80 after the A79 experiment. No `.core-stage` or `.core-rail` remains; source-text assertions forbid all four rail selectors, and the browser probe found zero rail nodes. |
| Roster role order drifts away from the game's role list | Fixed by A80: `ROLE_RANK` orders damage, tank, support; `sortPlayersByRole()` applies a stable index tiebreak to both filtered rows, and source-text plus browser probes require DAMAGE, TANK, SUPPORT in both rows. |
| The bottom glow is side-anchored again | Fixed by A80: `::after` is 1920px wide, centred with `translateX(-50%)`, and paints `linear-gradient(0deg, var(--fgc-glow), transparent 100%)`. Source-text assertions forbid `radial-gradient`, and browser probes measured the same full-width glow on both sides. |
| Country flag or team tag code returns as a half-finished identity chip | Deferred by the owner in A79. No `player-code` or `teamCode` remains; source-text assertions forbid them, and a real implementation needs a new payload field with its own decision and QA gate. |
| Source-text tests break on unrelated markup edits | The equality test narrows its extraction to the sentinel block and asserts specific rules. When an unrelated edit breaks it, fix the copy or narrow the assertion; do not delete the equality check. |
| Copy-paste drift between duplicated blocks | Sentinel comments and byte comparison across exactly eight files, with the file list asserted so removing a carrier is deliberate. |
| Preview and live sources render the effect twice | Planned: `auto.html` previews load `?fx=off`. Today the auto previews do not carry the block. |
| OBS source-refresh semantics for the app-level toggle | The `bg` switch broadcasts a cached section that connected sources handle, and A77 added the keyed init-frame parser so a refreshed source keeps the selected mode. Planned: use the same cached-section pattern for any future `data-fx` switch; A75 records the old URL-only behaviour, A76 the live-message fix and A77 the refresh fix. |
| An opaque plate inside a scene source could look like a second background | Accepted as an operator choice and refined by A77: opaque is opt-in for the eight carriers, `background.html` stays the default backdrop, and opaque keeps the hexagons visible on the plate so the product identity survives both theme and mode changes. |
| FX overscan shows transient scrollbars while a Browser Source loads | Fixed by A78: all twelve overlays declare an early `html, body` overflow guard before any FX or iframe markup, and a source-text test plus zero-gutter browser probe guard it. |
| OBS version floor | Floor is 31.0.4 / CEF 116; `color-mix()` is safe there. Any newer requirement needs a new decision. |
| Schedule realism | One overlay or one mechanism per change, each with four artefacts. Ideas stay here until they have a QA row and a runbook section. |

### Ideas not yet scheduled

A stinger hand-off tuned to the 0.8s reveal, a flythrough introduction overlay,
a single app switch for every overlay's FX backed by `data-fx` injection (the
shipped `bg` toggle uses the same cached-section pattern but only changes
`data-bg`), a shared cut-out locator helper, and a stable OBS capture for visual
regression testing. Also deferred: a country flag or country code in the roster
name plate. The owner asked to experiment with it during A79, then said to
scrap it and revisit it later. No country or nationality field exists in
`Player`, `PlayerDto` or `TeamDto`, so a real implementation requires a new
payload field and a separate decision. These are ideas only. They
do not change the shipped contract until they are promoted with a decision and
a QA gate.

### QA

A74 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the browser evidence
for the canonical FX block: one `.fgc-fx` at `display: block`, `inset: -360px`,
`mask-size: 180px 322px`, `opacity: 0.16`, a 72s drift, `?fx=off` removing it,
a transparent body and an empty console. The timer and game overlay exclusions
are part of the gate. Reduced motion is covered by the source-text assertion
because the test browser reported the preference false. A75 records the
historical background mode gate: `hero-bans.html?bg=opaque` showed the solid
`rgb(11, 13, 18)` plate with `.fgc-fx` hidden and a transparent body;
`?bg=transparent` restored the 72s lattice; `background.html?bg=transparent`
removed its plate while keeping all three animations. The app toggle was
checked as a URL rewrite for the ten modes-aware sources only; A76 proved the
cached `background` section and the app's accessible Switch change connected
sources without a refresh. A77 supersedes both on the lattice rule and adds
the refresh proof: a fresh and reloaded hero-bans tab kept `bgMode: opaque`,
the `rgb(11, 13, 18)` plate and a visible drifting lattice, while background
kept its opaque plate through reload. A78 records the early overflow guard and
zero scrollbar gutters on fresh and reloaded tabs. A79 records the paired roster
refinement: a 760px wash with a mirrored 320px bottom glow, a 56×56 core role
badge fully inside the portrait instead of overlapping the plate, 20×520
divider-style rails outside the 1500px card row, a 126px core plate with
18px/22px pronouns, a 76px substitute strip with 14px/16px pronouns, no code
chip, and long names guarded by fit plus ellipsis and `title`. Fresh home and
away probe tabs proved the colour-on and colour-off states, badge and rail
geometry, separator order, zero gutters, hidden overflow and empty consoles;
fmt, clippy, workspace tests and `dx build --desktop` passed. Screenshots were
captured but could not be reviewed by an observer because of an OpenCode Go
usage limit, so visual sign-off at 1920x1080 and browser-emulated reduced
motion remain outstanding owner checks.

A80 supersedes the A79 rails and side-anchored glow. The owner asked to remove
the left and right decorations, shrink the substitute role icon so its hexagon
is no longer squashed, keep both rows in Damage, Tank, Support order, and make
the gradient softer and subtler with the glow across the whole bottom of the
window rather than following home or away. Both roster files now use wash alpha
0.12 and glow alpha 0.14, keep the 760px side-origin wash at 90deg or 270deg,
and paint a 1920px centred linear-gradient glow from the bottom with no radial
gradient. The core badge stays 56×56 with a 40×40 icon at 16px inset; the
substitute badge is 46×54 with a 34×34 icon, 8px inset and zero centre offset.
`ROLE_RANK` and `sortPlayersByRole()` sort core and substitute lists stably by
damage, tank and support, with unknown roles grouped with damage and original
order preserved inside each group. Source-text assertions forbid `.core-stage`,
`.core-rail` and `radial-gradient(` and require the new helper. Fresh 1920×1080
probe tabs with scrambled player input proved the rendered order on both sides,
the new wash and full-width glow, centred badges with no rails, long-name
ellipsis with full `title`, separator order, zero gutters, hidden overflow and
empty consoles. `git diff --check`, fmt, fmt check, clippy, workspace tests and
`dx build --desktop` passed; the overlay suite still contains 36 tests. Home and
away screenshots were captured, but the observer subagent again failed with an
OpenCode Go usage limit, so OBS visual sign-off remains an owner check.

A81 replaces the last A80 side wash with one full-width bottom gradient. The
owner said the gradient "still doesn't look like it is working properly", that
it was "still showing on the side, and getting cut off before reaching the end",
and restated the target: "The gradient comes from the bottom of the overlay
stretching to each end. It should be a subtle bottom glow essentially". The root
cause was a 1920px `::after` layer centred inside a 760px side-anchored parent,
so OBS clipped the glow before the frame edge. Both roster files now position
`.roster-wash` at `left: 0`, `right: 0`, `bottom: 0` with `height: 420px` and
paint `linear-gradient(0deg, var(--fgc-wash), transparent 100%)`. The `::after`
layer uses the same left and right edges, sits at `bottom: -100px`, measures
300px high and blurs 24px. After the owner said "Make it a little less harsh",
the final alphas are 0.08 for the wash and 0.09 for the glow, with the same
full-width bottom-only geometry. Source-text assertions forbid the 760px wash,
the 90/270deg side gradients, side-specific wash selectors and radial gradients,
and require the new full-width rules and alphas. Fresh 1920×1080 probe tabs with
home tab `tab_e28b29ad-ba2a-4ec3-8304-268ce2858155` and away tab
`tab_f9c46bbf-fba2-45e2-903d-76d69da8aa00` proved the 1920×420 wash at y 660,
the matching team-colour alphas, zero gutters, hidden overflow and empty
consoles. `git diff --check`, fmt, fmt check, clippy, workspace tests and
`dx build --desktop` passed; the overlay suite still contains 36 tests. Home and
away screenshots were captured in `opencode-browser-3KLrG5` and
`opencode-browser-SeRdqL`, but visual review remained unavailable because of the
OpenCode Go usage limit, so OBS sign-off remains with the owner.

A82 completes the roster polish and replaces the first text-colour-mode design.
The owner asked for margin around uploaded logos, an animated divider between
core and substitutes, respect for selected name and background colours, and a
dark-blue and bright-pink gradient that read at the same strength. The paired
rosters now give `.team-logo` a 12px content-box inset while the panel keeps its
full background, animate the separator with `dividerReveal`, two `dividerLine`
animations and `dividerHex` with a reduced-motion fallback, apply
`teamData.textColor` before the Night Broadcast contrast fallback, and scale wash
and glow alphas by sRGB luminance. The owner then revised team properties so
automatic colour from logo applies to Background and Text individually with
recommended values, computed contrast affects Text only, and both ideas can run
together. `Team` now carries independent defaulted `colorFromLogo`,
`textColorFromLogo` and `computedContrast` flags; the old single `textColorMode`
is ignored. `sample_logo()` returns a palette of up to eight sampled colours,
and `harmonize_text_color()` keeps the sampled hue while raising contrast when a
logo-derived Text is too close to a manual Background. Raw CDP on Inglorious
Blasters proved every switch and combination, and the original state was
restored with its hash unchanged. All gates passed with 93 app tests and 36
overlay tests. The owner opened the app in OBS at 1920×1080 and reported: "A82
is a pass." This closes the visual sign-off.

A83 refreshes `map-select.html` after the owner chose it over the game overlay.
They asked to keep the current theme philosophy, rework the map scores, make the
winning team logo more visible with a subtle map glow, keep square hero images
for now, and make the current map clear. The active card now carries a 2px inset
accent border and one `Current map` hexagon badge. Completed boards use two
team-coloured score plates with a glowing winner plate and a colon divider. The
winning logo has a hexagon halo, and completed map images have a faint radial
winner-colour wash. `safeHexColor()` keeps malformed colours from breaking the
card, and `createMapCard()` derives both winners from completed scores. Ban
portraits remain 58×58 square images; hexagonal hero art stays deferred. Browser
probes measured the badge, borders, plates, glows and square portraits, then
proved the winner treatment reverses between `#0f3b8f` and `#ff5c7a`. All gates
passed with 38 overlay tests. Screenshot capture failed because the desktop
window was not visible, so OBS visual sign-off remains with the owner.

A84 applies the Clean Canvas direction to map select and hero bans. Map select
loses the score slabs, per-card header, mode icon and `Now playing` pill; the
winner sits at the art centre with a soft circular team-colour halo instead of a
large hex plate. Completed art darkens under a subtle radial winner wash. The
opening reveal settles once, after which duplicate frames cause no animation,
changed map content crosses through a ghost text fade under a dark curtain, and
only a changed ban runs its per-side reveal. Unchanged hero art and role icons
stay mounted. Match Controller hero pickers became native selects, and scene
overlays now default to an opaque background. The owner reviewed the running
overlays and accepted A84 on 2026-11-24.

A85 refreshes `map-pool.html` with the same philosophy. The heading and
`fitText`/`fitMapNames` pass are gone; a five-column canvas uses a bottom
gradient, seam overscan and two-line name wrapping. Mode icons are plain marks.
Columns, cards and mode marks enter on staggered beats. A picked image darkens
to `grayscale(1) brightness(0.56) saturate(0.45)` and gains a radial winner
wash, bottom wash, crest reveal and name rule in the choosing team's colour.
The keyed init frame is unwrapped before the pool read, fixing the fallback to
orange. `lightenedRgbTriplet()` lifts dark teams 45% toward white while keeping
the hue. Stable slot reconciliation keeps duplicate broadcasts as no-ops and
runs changed content and picks in one batch. The owner reviewed the final live
probe, asked for the lighter dark-team colour, then approved A85 on 2026-11-24.

Fixed risks from A84 and A85:

| Risk | Status | Guard |
|---|---|---|
| Map select returns to dashboard chrome | Fixed by A84 | Source assertions forbid the current badge, score boards, mode icons and `Now playing`; the Clean Canvas test also forbids the old inset accent border. |
| A live update replays the opening animation | Fixed by A84 | Opening state settles once; duplicate payloads produce zero animations and only changed cards animate. |
| Unchanged hero art reloads when another ban changes | Fixed by A84 | Image `src` is compared before replacement; role icon updates are conditional. |
| Scene overlays open transparent by default | Fixed by A84 | `AppStore` seeds `background_opaque` as `true`; transparent remains explicit. |
| Map pool keeps a redundant heading or clipping long names | Fixed by A85 | No header or fit pass remains; names shrink, then clamp to two lines. |
| Game-mode icons regain hexagon frames | Fixed by A85 | No `clip-path` remains in `map-pool.html`, and the source test forbids `clip-path: polygon(`. |
| Pick state falls back to orange from nested init data | Fixed by A85 | Init unwraps `d.scoreboard || d`; live probe proved `127 120 149` for `#17093e`. |
| Dark team colours disappear against picked art | Fixed by A85 | `PICK_LIGHTEN_MIX = 0.45` lifts the derived glow toward white while preserving hue. |

A86 refreshes the three talent-row overlays and widens the interview subject to
the active match. The Match Controller Interview panel can now select a talent,
a player restricted to the two teams on the Match tab, or a custom name with its
own subtext and quick-uploaded portrait. The projection composes these sources
into the same `interviewee` wire object, so no consumer needs a schema change. A
player uses their portrait first and their featured hero's art as the fallback,
with team, role, position and pronouns in the subtext. On screen, the role pill,
hexagon frame, wash and photo gradients are gone; the guest keeps its accent
name and stable `guest` slot.

The owner also asked for load-in animations. All three row overlays now hold the
first reveal for 500ms, replay that reveal when the controller hides and shows a
source, and animate only changed slots after the opening settles. A late guest
data frame is no longer skipped: it receives the same startup delay as the
interviewers. The canonical FX block and the opaque Night Broadcast plate stay
untouched. The owner reviewed the running controller and overlays and approved
A86 on 2026-11-25.

Fixed risks from A86:

| Risk | Status | Guard |
|---|---|---|
| Interview subject cannot come from the active roster | Fixed by A86 | The source select lists only players on the two current match teams, and projection precedence is talent → player → custom. |
| Custom guests need a slow asset-library detour | Fixed by A86 | The Guest image picker exposes one `Quick upload…` action and stores the selected asset on the custom reference. |
| Role chips and hexagon frames obscure the row photos | Fixed by A86 | Source assertions forbid `.talent-role`, `.talent-image-frame`, the wash and both photo gradients. |
| Replaying the row reveal also replays unrelated slots | Fixed by A86 | Duplicate frames are no-ops; after the opening settles only changed stable slots animate. |
| The guest arrives after the opening frame and never animates | Fixed by A86 | Late changed cards use the startup delay, and the live-load trace shows the guest animating with a 500ms delay. |

A87 refreshes `game-overlay.html` inside the five fixed 1920×1080 boxes. The info
bar stays at `x43 y13 525x25`, the mode bar at `x1348 y13 525x25`, the centre
status bar at `x821 y12 278x50`, and the team bars at `x43 y52 525x55` and
`x1348 y52 525x55`. Side colour follows screen position, not team identity, so
blue stays on the left and red on the right after a side swap. Team colours are
only ever gradients or washes, strongest at the inner score end of each bar;
there are no flat team-colour fills. The logo tile is transparent, the artwork
overscans the bar and is clipped on all four sides, and the crop fades inward
toward the name. Team names use the heavier SemiBold face with a 27px fit that
drops to 18px, and long titles, subtitles, names and active modes ellipsise
inside their fixed boxes.

Hero bans leave the team bars and become compact 47×50 chips flanking the centre
status bar at `x762 y12` and `x1111 y12`, with 12px gaps to the status bar. Each
chip holds a 35×42 art-only tile with a 5px radius, inset keyline, bottom scrim,
visually hidden text and a 2px blue-left or red-right accent. The chips enter
with `banChipIn` over 0.36s; initial delays are 2.5s and 2.62s, and a later ban
change enters immediately with a 120ms second-chip stagger. A dataset key keeps
duplicate frames from replaying the reveal. The owner asked for this entrance
to start earlier after reviewing the first timing, and the final live probe
showed opacity 0 before the entrance and a visibly interpolated frame after an
early seek.

Records move from the team library to the current match. `Team.record` is gone;
`Match` now carries `tournament_id`, `team_a_record` and `team_b_record`, all
optional with serde defaults. `fgc_core::tournament::team_record()` prefers a
played group win-loss record and falls back to the entrant seed. Projection
resolves a manual match override first, then the selected tournament, then
`0-0`; the `record` field on the wire DTO keeps its original shape. Match
Controller Setup gained a Tournament select and Team A/B record inputs with
derived placeholders, while the Teams view no longer offers a Record field.
Empty logo slots on hero bans, map select and map pool now show a tag or name
initial through the theme-independent `--fgc-tag-ink: #eef2f8` token, with dark
scrims where the surrounding art would wash them out. The owner reviewed the
staged overlays and approved A87 on 2026-11-25.

Fixed risks from A87:

| Risk | Status | Guard |
|---|---|---|
| The refresh changes broadcast geometry or swaps side colours with team identity | Fixed by A87 | Source assertions fix all five boxes and require position-based blue-left/red-right gradients, including `.swapped`. |
| Team colour returns as a flat fill | Fixed by A87 | Only gradients and washes are allowed in the team bars; source guards reject flat fills. |
| A large logo or long name breaks the fixed chrome | Fixed by A87 | The logo is transparent, overscanned and clipped on all four sides; names and headers fit or ellipsise inside their boxes. |
| A record belongs to a team rather than the match it is broadcast in | Fixed by A87 | Records live on `Match`, the Teams field is removed, and projection preserves the existing DTO `record` shape. |
| Empty logo initials disappear under the Light theme | Fixed by A87 | Placeholders use the fixed `--fgc-tag-ink` token and dark backplates on the pick overlays. |
| The ban entrance starts too late or replays on duplicate frames | Fixed by A87 | Initial delays are 2.5s/2.62s, live changes run at 0/120ms, and the reveal is keyed by dataset state. |

A88 answers the owner's request for a way to replace an image in Assets. Every
imported asset card now has a **Replace…** action, and the preview dialog has the
same action for the asset it shows. Choosing it opens a confirmation that names
the asset and explains that its identity and display name stay the same, that
every team, player and overlay reference follows the new image, and that the old
image data is reclaimed after the next save. The system file picker then supplies
a replacement in the same formats as an import. The store inserts the new bytes
by content hash, copies only the hash, extension and original filename onto the
existing asset, and leaves the id, name, category, added date, hero attachments
and variant flags untouched. Bundled art and hero attachment images are rejected,
and the button disables while the replacement is being read and stored.

Fixed risks from A88:

| Risk | Status | Guard |
|---|---|---|
| Replacing an image breaks every reference to it | Fixed by A88 | The asset id is preserved and only hash, extension and original filename change; a state test asserts identity, name, category, added date and icon survive. |
| Bundled or attached art is replaced through the new button | Fixed by A88 | The UI hides the action for bundled assets and the store rejects bundled and attached ids. |
| The new action is unreachable or loses keyboard focus | Fixed by A88 | A real WebView2 probe found the button on every user card, opened the confirmation from both card and preview, and saw a visible focus outline while tabbing. |

A89 answers the owner's request for clearer Maps and Bans controls. The view
now receives the team library and resolves each side from the selected team id.
A team's name is used first, then its tag, and the old `Team A` or `Team B`
label remains only when no team is selected. The intro note, **Chosen by**
options, both score labels, **First ban** options, both ban labels and their
accessibility names all use the resolved names. The stored side values stay `A`
and `B`, so fixtures, projection and overlay state are unchanged. The owner
reviewed the running controller at desktop and phone widths, including the
expanded HERO BANS section, and approved A89 on 2026-11-25.

Fixed risks from A89:

| Risk | Status | Guard |
|---|---|---|
| Generic side names hide which team is being edited | Fixed by A89 | The view resolves the selected team id and renders its name, then its tag, before falling back to `Team A` or `Team B`. |
| Clearer labels change the stored pick or ban side | Fixed by A89 | Display strings changed only; select values remain `A` and `B`, and a state test asserts name, tag and empty-selection resolution. |
| Long team names clip or break the layout | Fixed by A89 | A real WebView2 probe at 1280x800 and 375x812 showed the full names in selects and labels with no clipping, overlap or horizontal overflow. |