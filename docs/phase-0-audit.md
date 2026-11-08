# Phase 0 — Audit of `ow-graphics-control`

Source: https://github.com/ThePeachZA/ow-graphics-control (MIT, 8 commits, no releases)
Snapshot audited: `master` zip downloaded 2026-11-06. Files read: all `src/**`, `electron/main.ts`, `electron/overlayServer.ts`, `electron/preload.ts`, `src/types/index.ts`, `src/store/AppContext.tsx`, all 7 files in `public/overlays/`, `public/assets/manifest.json`.

Not verified: running the app, packaging, real OBS rendering, Windows firewall behaviour. Runtime notes below are inferred from source.

---

## 1. Screens, forms and controls

### Shell
- Left sidebar with 7 nav items: Match Controller, Teams, Talent, Assets, Overlays, Tournament, Settings. Footer status "Ready". React error boundary around the app.
- Native Electron menu:
  - File: Import Data… (Ctrl+I), Export Data… (Ctrl+E), Exit.
  - View: reload, force reload, zoom, fullscreen, devtools.
  - Overlay: Start Server, Stop Server, Open Scoreboard (`scoreboard.html` — **does not exist**), Open Map Select, Open Hero Bans, Open Map Pool (listed twice), Open Team Roster Home/Away, Open Casters.
  - Help: Open Data Folder, Toggle Developer Tools.

### Match Controller (`src/views/MatchController.tsx`)
Header: Export Match (JSON), Import Match (JSON), side selector N / Attack-Defend / Defend-Attack, Force Update.

| Tab | Controls |
|---|---|
| Match | Title, Subtitle, Format (first to 1–4), Team A select, Team B select, Swap Sides toggle, auto score display, manual score override toggle + manual A/B scores, Tournament Logo selector, Sponsor/Watermark Logo selector |
| Map Selection | 8 map slots (slot 8 = Tiebreaker): map name select, game mode select, Current Map toggle (single), Complete toggle, Chosen By radio (A/B/None, hidden when loser picks enabled), map score A/B, Reset All Maps |
| Hero Bans | First-ban team (A/B), per-map hero-ban grids for Team A and Team B (8 maps) |
| Talent | Caster 1/2, Panel 1/2/3, Interviewer, Interviewee (talent select or custom name + subtext) |
| Map Pool | Select maps to include, grouped by game mode, count/limits |
| Timer | Mode countup / countdown / target-date; H/M/S; target date-time; alignment left/center/right; leading zeroes; standalone; Start, Pause, Reset |
| Config | Team colours on overlay toggle, show player portraits toggle, Tournament Logo, Sponsor Logo |

Empty state: "No Match Active" with Create New Match and Import Match.

Auto-broadcast: a 3 s interval broadcasts when a "pending update" flag is set. `Force Update` broadcasts immediately.

### Teams (`src/views/Teams.tsx`)
- Team list sidebar with Add; empty state; team item shows colour, logo, tag, name, player count.
- Selected team header with logo, name, tag, Delete Team.
- Tabs: Team Info / Players.
- Team Info: Team Name, Team Tag, Logo selector, Background colour, Text colour, Logo-BG colour, live preview.
- Players: list of player cards; add/edit player form with Name, Role (tank/damage/support), Position (core/sub/manager/coach), Portrait selector, Featured Hero select, Pronouns; Save/Delete/Cancel.

### Talent (`src/views/Talent.tsx`)
- List/grid of talent; add/edit form: Display Name (required), Real Name, Pronouns, Social platform (Twitter/X, Twitch, YouTube, Instagram, TikTok) + handle, Portrait selector; Save/Delete/Cancel.

### Assets (`src/views/Assets.tsx`, ~1.5k lines)
Seven category tabs: Logos, Portraits, Maps, Game Modes, Heroes, Roles, Sides.
- Logos/Portraits: gallery, import file, download from URL, rename, delete (move to temp), bulk select.
- Maps: name, game mode link, map image, icon/image fields, rename, delete.
- Game Modes: name, icon.
- Heroes: name, role link, hero portrait image, hero icon, tag modal, filter by role, bulk select.
- Roles: name, role icon.
- Sides: name, side icon.
- Deleted/replaced files go to `userData/assets/_temp`; Settings can clear temp.
- Bundled categories can be renamed/updated in the bundled manifest (writes to the bundled manifest in dev/packaged resources).

### Overlays (`src/views/Overlay.tsx`)
- Server status + port input (localStorage key `overlay-server-port`, default 3001; changing while running restarts server).
- Start / Stop Server.
- Game Overlay URL with Copy button.
- Force Update Overlay (broadcasts a reduced `scoreboard` payload).
- Current match summary + instructions.

### Settings (`src/views/Settings.tsx`)
- Data Management: Export Data, Import Data (full app JSON).
- Storage: show userData/data/assets paths, open data folder, temp asset count, clear temp.
- Overlay Settings: "Default Browser Port" (writes `state.overlayServer.port`, default 3000) and NDI Output Name (no NDI implementation exists).
- About.

### Tournament (`src/views/Tournament.tsx`, ~1.8k lines, WIP)
- Tournament list, create modal: name, format (single elim / double elim / round robin), default first-to, grand-finals reset, number of groups.
- Tournament detail: settings panel, team/seed editing, upper/lower/grand-final brackets or groups + standings + matches, match modal with scores and complete/reset, champion display.
- No overlay consumes tournament state.

---

## 2. Data model (TypeScript → Rust)

| TS | Shape / notes | Rust target |
|---|---|---|
| `PlayerPosition` | `core \| sub \| manager \| coach` | `enum PlayerPosition` |
| `PlayerRole` | `tank \| damage \| support` | `enum PlayerRole` |
| `Team` | id, name, tag, record?, color, textColor, auxColor, logoAssetId?, players[] | `struct Team` |
| `Player` | id, name, role, position, portraitAssetId?, featuredHeroAssetId?, pronouns? | `struct Player` |
| `MapSlot` | name, type, gameModeAssetId?, imageAssetId?, completed, scoreA, scoreB, chosenBy `A\|B\|null` | `struct MapSlot` |
| `MapHeroBans` | firstBanTeam `A\|B\|null`, teamA: string[], teamB: string[] | `struct MapHeroBans` |
| `Match` | id, title, subtitle, teamA/teamB ids, maps[8], heroBans[8], mapPool[], firstTo, status `pre\|live\|post`, currentMap?, swapSides, useManualScore, manualScoreA/B, side `0\|1\|2`, talent?, loserPicks, showPlayerPortraits, substitutions[] | `struct Match` |
| `Talent` | id, displayName, realName?, pronouns?, socialMedia? {platform, handle}, portraitAssetId? | `struct Talent` |
| `MatchTalent` | caster1/2, panel1/2/3, interviewer, interviewee {talentId?, customName?, customSubtext?} | `struct MatchTalent` |
| `Substitution` | id, team A/B, playerIn, playerOut, timestamp | `struct Substitution` (currently unused) |
| `AppData` | teams, matches, tournaments, currentMatch, talents | export/import shape |
| `AssetEntry` | id, name, filename, path, category, addedAt, iconPath?, portraitPath?, gamemode?, gameModeAssetId?, imageAssetId?, roleAssetId? | `struct AssetEntry` |
| `AssetManifest` | logos, portraits, gameModes, roles, sides, maps, heroes | `struct AssetManifest` |
| `BracketFormat` | single-elim / double-elim / round-robin | `enum BracketFormat` |
| `MatchStatus` | pending / live / completed | `enum MatchStatus` |
| `BracketMatch` | id, round, matchNumber, team1Id?, team2Id?, scores, winnerId?, status, nextMatchId?, nextMatchSlot, loserNextMatchId?, loserNextMatchSlot, firstTo | `struct BracketMatch` |
| `BracketRound` | roundNumber, name, matches[] | `struct BracketRound` |
| `TournamentGroup` | id, name, teamIds[] | `struct TournamentGroup` |
| `RoundRobinStandings` | teamId, played, wins, losses, draws, pointsFor, pointsAgainst, points | `struct RoundRobinStandings` |
| `Tournament` | id, name, format, teams[{teamId, seed}], rounds[], winnersBracket[], losersBracket[], grandFinal[], groups[], roundRobinStandings map, championTeamId?, createdAt, status, defaultFirstTo, grandFinalsReset | `struct Tournament` |

Additional persisted app state (not in `AppData`): `assets`, `tournamentLogoAssetId`, `watermarkLogoAssetId`, `teamColorsEnabled`, `overlayServer {running, port}`, `ndi {enabled, outputName}`.

---

## 3. WebSocket protocol (current)

Server: `ws` attached to the Express HTTP server. One WebSocket endpoint at `/` (no path).

| Direction | Message | Payload |
|---|---|---|
| Server → client, on connect | `{ type: "init", data: overlayState }` | `overlayState` is a **map keyed by last broadcast type**, e.g. `{ scoreboard: {...}, gameOverlay: {...}, casters: {...} }`. Not a full app snapshot. |
| Client → server | `{ type: "requestInit" }` | Server re-sends `init`. |
| Server → client | `{ type: "<section>", data: <section payload> }` | Sections: `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`, `casters`, `matchTimer`, `show`. |
| Server → client | `{ type: "show", visible: boolean }` | All overlay JS handles it; **the app never sends it**. |

- Updates are full per-section snapshots, not diffs.
- The app sends the **same `scoreboardData` object** under six type names: `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`; plus `casters` with `{ castersData }`.
- No heartbeat/ping, no sequence numbers, no versioning. Overlay clients reconnect every 2 s on close.
- Server state survives client disconnects but not server stop/start (new server object starts with empty `overlayState`).

### `scoreboardData` payload (authoritative broadcast shape)
```
teamA/teamB: { name, tag, record, color, logoBgColor, logoUrl } | null
scoreA, scoreB, title, subtitle, currentMap, currentMapIndex,
firstTo, mapType, status, side, swapSides, loserPicks,
gameModeIcons, heroIcons, heroPortraits, sideIcons, roleIcons,
teamAPlayers[], teamBPlayers[], maps[], heroBans[], mapPool[],
mapPoolData[], castersData[], tournamentLogoUrl, watermarkLogoUrl,
teamColorsEnabled, showPlayerPortraits
```
- `maps[]`: `{ name, type, completed, scoreA, scoreB, mapImageUrl, chosenBy, chosenByName }`
- `teamAPlayers[]`: `{ name, role, position, pronouns, portraitUrl, heroImageUrl }`
- `mapPoolData[]`: `{ modeName, modeIcons: string[], maps: [{ name, imageUrl, chosenBy, chosenByLogo }] }`
- `castersData[]`: `{ displayName, pronouns, socialMedia, portraitUrl }`
- Images are converted to base64 data URLs before broadcast.
- Scores are derived from completed maps unless `useManualScore`; draws are ignored.

---

## 4. HTTP server, static files, CORS, ports

Implemented inline in `electron/main.ts` (`createOverlayServer`); `electron/overlayServer.ts` is an unused legacy duplicate (port 3001, `/state`, `/overlays`, WS).

| Route | Content |
|---|---|
| `GET /overlays/*` | Static overlay HTML. Dev: `public/overlays`; packaged: `resources/overlays`. |
| `GET /assets/*` | Bundled assets (`public/assets` or `resources/assets`). |
| `GET /user-assets/*` | `userData/assets` (uploaded logos/portraits). |
| `GET /health` | `{ status: "ok", port }` |
| WS `/` | WebSocket upgrade |

- `cors()` with default `*`; `express.json()`; no auth, no rate limiting.
- `server.listen(port)` with no host → binds all interfaces (LAN exposed).
- Port: Overlay tab default 3001 (localStorage); Settings state default 3000; IPC `overlay:start` default 3001. Settings port and Overlay tab port are not synchronised.
- No port-conflict handling beyond returning an error from `listen`; no automatic fallback.
- Overlay URLs are built as `http://localhost:<port>/overlays/<file>.html?port=<port>`; the overlay JS reads `port` from the query string and connects to `ws://localhost:<port>`.

---

## 5. Electron main, IPC, persistence, assets

- Window 1440×900, min 1280×720, dark background; WebView with context isolation, no node integration.
- Data paths: `app.getPath('userData')`; `data/app.json`; `assets/`; `assets/_temp/`.
- IPC handlers:
  - `app:getPaths`
  - `fs:readFile`, `fs:writeFile`, `fs:exists`, `fs:readDir`, `fs:mkdir`, `fs:copyFile`, `fs:readImageAsDataUrl`
  - `bundled:scanAssets`, `bundled:updateManifest`
  - `asset:importFile`, `asset:downloadUrl`, `asset:moveToTemp`, `asset:deleteFromTemp`, `asset:getTempAssets`, `asset:clearTemp`, `asset:getTempCount`
  - `dialog:openFile`, `dialog:saveFile`
  - `overlay:start`, `overlay:stop`, `overlay:status`, `overlay:broadcast`, `overlay:openUrl`
  - Menu events: `menu:import`, `menu:export`, `overlay:start`, `overlay:stop`.
- Persistence: React state → `app.json` with a 500 ms debounce. Saved fields: teams, matches, currentMatch, talents, assets, tournaments, tournamentLogoAssetId, watermarkLogoAssetId, teamColorsEnabled, overlayServer, ndi. No schema version.
- Load: merges saved assets with bundled assets (saved wins if non-empty), normalises `teamLogos` → `logos`, fills missing `side`, `mapPool`, `loserPicks`, `showPlayerPortraits`, `chosenBy`; fills 8 map slots/hero-ban slots.
- Asset scanning: reads `public/assets/manifest.json` and scans bundled folders (maps, game-modes, roles, sides, heroes/portraits, heroes/icons, logos); user uploads only for logos/portraits; asset paths are absolute.
- Export/import full data: Settings uses JSON save/open dialogs; Match Controller imports/exports a single match.
- No auto-backup; temp files are only deleted manually.

---

## 6. Overlay consumption

| Overlay | Accepted messages | Fields consumed | Notes / animations |
|---|---|---|---|
| `game-overlay.html` | `init` (uses `d.scoreboard`), `scoreboard`, `gameOverlay`, `show` | title, subtitle, teamA/B name/logoUrl/color/logoBgColor, scoreA/B, swapSides, teamColorsEnabled, side + sideIcons, currentMapIndex, firstTo, maps[] (type/completed/score), gameModeIcons, heroIcons, heroBans, tournamentLogoUrl, watermarkLogoUrl | Fixed 1920×1080. Entrance animations triggered once per page load (`fadeSlideDown`, `fadeSlideUp`, `slideFromCenter`, `swipeFromLeft/Right`, `fadeIn`); `show` only toggles `display`, does not replay animations. |
| `hero-bans.html` | `init` (`d.scoreboard`), `scoreboard`, `gameOverlay`, `heroBans`, `show` | teamA/B name/color/logoUrl, heroBans[currentMapIndex] (firstBanTeam/teamA/teamB), heroPortraits, maps, firstTo, currentMapIndex, teamColorsEnabled | Fixed layout; team colour theming. |
| `map-pool.html` | `init` (`d.scoreboard.mapPoolData` or `d.mapPoolData`), `scoreboard`, `mapPool`, `show` | mapPoolData[] grouped by mode, gameModeIcons | Sends `requestInit` on open. **App never broadcasts `mapPool`**; only works via `scoreboard`. |
| `map-select.html` | `init` (`d.scoreboard`), `scoreboard`, `gameOverlay`, `mapSelect`, `show` | maps[] (name/type/mapImageUrl/completed/scores/chosenByName), teamA/B, firstTo, currentMapIndex, loserPicks, gameModeIcons, tournamentLogoUrl | 8 cards; tiebreaker at index 7; per-card staggered entrance/expand animations. |
| `team-roster-home.html` | `init` (`d.teamRoster`), `scoreboard`, `gameOverlay`, `teamRoster`, `show` | teamA (home) name/logoUrl/color/logoBgColor, teamAPlayers[], roleIcons, teamColorsEnabled, showPlayerPortraits | Filters `position === "core"`, then subs; role icon mapping. |
| `team-roster-away.html` | same | same, using `teamB`/`teamBPlayers` | Identical file with `TEAM_SIDE='B'`. |
| `row-casters.html` | any payload with `castersData` (including `init`), `show` | castersData[] displayName/pronouns/socialMedia/portraitUrl | Sends `requestInit`; social icons. |
| `matchTimer` | no overlay file exists | `{hours, minutes, seconds, settings}` or `{action: start\|pause\|reset}` | Broadcast only; no consumer. |

All overlays load fonts from `https://design.penpot.app/...` (remote). The repo also contains Apotek OTF files under `public/assets/fonts`, unused by overlays.

---

## 7. Tournament status (WIP)

Implemented:
- Single elimination, double elimination and round-robin/group generation; snake seeding; byes; group standings; match score entry; champion display; tournament CRUD; settings; team/seed editing.

Known incomplete/buggy:
- Double-elim losers-bracket wiring is simplistic; `nextMatchId`/`loserNextMatchId` slots are not reliably propagated through all rounds.
- `grandFinalsReset` is stored and toggleable but no reset logic exists.
- `championTeamId` is never set automatically from the grand final; `SET_CHAMPION` exists but is unused.
- Winner/loser propagation only advances one level; later rounds can remain stale.
- "Reset Match" clears only the immediate next match, not downstream results.
- `rounds` field is unused (bracket lives in `winnersBracket`/`losersBracket`/`grandFinal`).
- Drag/drop seeding and matchup editing are fragile.
- No tournament state is broadcast to any overlay.
- No persistence tests; tournament state is only as good as the saved JSON.

---

## 8. Parity checklist

Status legend: **P** = must match exactly, **F** = functional parity (behaviour may be fixed), **X** = known-broken; decide whether to preserve or fix.

| ID | Area | Current behaviour | Acceptance criterion | Status |
|---|---|---|---|---|
| S1 | Shell | 7-item sidebar, error boundary | Same 7 destinations and persistent navigation | P |
| S2 | Menus | File import/export, overlay open, data folder, devtools | Equivalent actions in-app (menu optional in Dioxus) | F |
| S3 | Dark theme | Dark UI, CSS modules | Visually equivalent dark theme | F |
| M1 | New match | Creates 8 empty map + 8 hero-ban slots | Same defaults, firstTo 3, loserPicks true, portraits true | P |
| M2 | Match info | title/subtitle/format/team A/B/swap | Same fields and effects | P |
| M3 | Score | Auto from completed maps or manual override | Same calculation, draws ignored | P |
| M4 | Map slots | 8 slots, name/mode/current/complete/chosenBy/scores | Same; tiebreaker = slot 8 | P |
| M5 | Loser picks | Auto-computes chosenBy from previous loser | Same algorithm | F |
| M6 | Hero bans | Per-map, first-ban team, two hero lists | Same data and overlay output | P |
| M7 | Talent assignment | 6 talent slots + custom interviewee | Same | P |
| M8 | Map pool | Select maps grouped by mode | Same grouping/sorting (Flashpoint/Push last) | P |
| M9 | Timer | countup/countdown/target, alignment, leading zeroes, standalone | Same UI and broadcasts | P (overlay missing) |
| M10 | Branding | Tournament logo + sponsor/watermark logo | Same data URLs | P |
| M11 | Match import/export | Single-match JSON | Same file shape | P |
| M12 | Force update | Broadcast reduced scoreboard | Same effect (fix `currentMapIndex` off-by-one) | F |
| M13 | Auto broadcast | 3 s pending-update interval | Same sections and timing | P |
| T1 | Team CRUD | list/add/delete, name/tag/logo/3 colours | Same | P |
| T2 | Players | add/edit/delete, role/position/portrait/hero/pronouns | Same | P |
| TA1 | Talent CRUD | display/real name/pronouns/social/portrait | Same | P |
| A1 | Asset categories | 7 categories, gallery/detail | Same categories | P |
| A2 | Import assets | file import, URL download, rename, temp delete | Same, with safe path handling | F |
| A3 | Bundled assets | manifest + folder scan, saved-wins merge | Same resolution order | P |
| O1 | Server control | start/stop, configurable port, status | Same | P |
| O2 | Overlay URLs | game overlay URL + menu for others | All 7 URLs listed with copy buttons | F |
| O3 | Health | `/health` | Same JSON | P |
| O4 | Static routes | `/overlays`, `/assets`, `/user-assets` | Same paths | P |
| O5 | CORS | `*` | Configurable; default parity | F |
| O6 | WS init | keyed map of last broadcasts | Byte-compatible `init` for existing overlays | P |
| O7 | WS updates | full per-section snapshots | Same types and shapes | P |
| O8 | Show/hide | overlays support `show`; app never sends | UI control added | F |
| O9 | Reconnect | client retries every 2 s | Same resilience + optional heartbeat | F |
| P1 | Persistence | `app.json`, 500 ms debounce | Same location/format + schema version | P/F |
| P2 | Export/import | full app JSON, single match JSON | Same shapes | P |
| P3 | Paths | `userData/data`, `userData/assets`, `_temp` | Equivalent app-data layout | F |
| G1 | Settings | data management, storage, port, NDI name, about | Same controls (NDI may be deferred) | F |
| B1 | Tournament | single/double/RR, groups, standings, bracket | Decide: complete, port WIP, or defer | X |
| V1 | Game overlay | scoreboard/match bug/mode order/bans/branding | Pixel-compatible with existing HTML | P |
| V2 | Hero bans overlay | team header + bans per map | Same | P |
| V3 | Map pool overlay | grouped pool + chosen-by logos | Same | P |
| V4 | Map select overlay | 8 cards + tiebreaker + animations | Same | P |
| V5 | Roster overlays | home/away, core+subs, roles/portraits | Same | P |
| V6 | Casters overlay | row layout, socials, portraits | Same | P |
| V7 | Timer overlay | **missing** | Decide whether to build one | X |

---

## 9. Key findings and risks

1. **`init` is not a state snapshot** — it is a map of last-broadcast section payloads. Any Rust implementation must reproduce this shape exactly or all overlays show stale/blank data on first connect.
2. **Same payload, six type names** — the app broadcasts `scoreboardData` as `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`. Changing to per-overlay minimal payloads would break the "drop-in" guarantee unless overlays are updated.
3. **`map-pool.html` never receives `mapPool`** — it relies on `scoreboard.mapPoolData`. A "clean" refactor that only sends `mapPool` would break it.
4. **`show` is implemented in every overlay but never sent.** There is no hide/show or animate-in/out control in the app.
5. **Timer has no overlay.** `matchTimer` is broadcast into the void.
6. **`currentMapIndex` off-by-one** in the Overlays tab Force Update (`+1`) versus Match Controller (raw index). Game overlay adds 1 for display, so Force Update shows the wrong map number.
7. **Port confusion**: state default 3000, Overlay tab 3001, start default 3001; Settings port does not drive the Overlays tab. Port conflict surfaces as a start error only.
8. **Stale asset caches**: team logo and player caches are never invalidated when the selected team/roster/logo changes; overlays can keep old logos/portraits until restart.
9. **Data-URL broadcasts are heavy**: every update base64-encodes logos, portraits, map images and hero art, then sends six copies. Large shows will produce multi-MB WS frames every 3 s.
10. **Remote fonts**: all overlays fetch Apotek from `design.penpot.app`; offline OBS or a dead URL breaks typography. Bundled OTFs exist but are not referenced.
11. **Server exposure**: binds all interfaces, CORS `*`, no auth. Fine on a trusted LAN, risky on public Wi-Fi.
12. **Absolute asset paths** in `app.json` make exports non-portable across machines.
13. **Dead/unused model fields**: `Match.scoreA/scoreB` (reducer writes them, type omits them), `matches[]`, `substitutions[]`, `ndi`, `Tournament.rounds`, `SET_CHAMPION`.
14. **Tournament is not overlay-connected** and has propagation bugs (see §7).
15. **Menu bug**: "Open Scoreboard" points at a non-existent `scoreboard.html`; Map Pool is duplicated.
16. **No schema version** in `app.json`; migrations are ad-hoc normalisation.

---

## 10. Questions for Phase 1

1. 1:1 parity first, or improvements in v1?
2. Tournament: complete, port WIP, or defer?
3. New features wanted?
4. Windows-only?
