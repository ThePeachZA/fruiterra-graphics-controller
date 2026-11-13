# Overlay protocol — Phase 2 contract

Status: **approved**. Implements Phase 2 Q3 (shape parity, transport cleanups bounded by the subscription contract), with the Phase 5 content-addressed asset URLs applied.

This document is the compatibility contract. Existing overlay HTML must render against it unchanged. Any deviation is a bug.

---

## 1. Transport

| Item | Value |
|---|---|
| HTTP | `GET /overlays/<file>.html`, `GET /assets/<hash>.<ext>`, `GET /assets/fonts/<name>.woff2`, `GET /health`, `GET /state` |
| WebSocket | Same origin/port, root path (`ws://<host>:<port>/`) |
| Encoding | UTF-8 JSON, one object per frame |
| Server → client | `{ "type": <string>, "data": <object> }` |
| Client → server | `{ "type": "requestInit" }` (optional) |
| Reconnect | Client-side; all seven overlays retry every 2 s |
| Port | `?port=` query parameter on the overlay URL, default 3001 |

`/state` returns the current keyed state map (same object sent in `init`). `health` returns `{ "status": "ok", "port": <port>, "overlays": <connected count>, "version": "<app version>" }`.

---

## 2. Envelope and `init`

On connect the server sends:

```json
{ "type": "init", "data": { "<last type>": "<last payload>", … } }
```

- `data` is a **keyed map**, not a snapshot object. Keys are message types.
- Keys appear only after that section has been broadcast at least once. In practice, once a match is loaded the map contains `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`, `casters` and (if the timer has run) `matchTimer`.
- The five scoreboard-bearing keys hold the **same** scoreboard payload object. This duplication is deliberate: each overlay reads a different key.
- `casters` holds `{ "castersData": [...] }` — **not** the scoreboard payload.
- `matchTimer` holds the timer payload.
- `requestInit` triggers the same message again at any time.

**Compatibility rules**

| Overlay | Reads on `init` |
|---|---|
| `game-overlay.html` | `data.scoreboard` |
| `map-select.html` | `data.scoreboard` |
| `hero-bans.html` | `data.scoreboard` |
| `map-pool.html` | `data.scoreboard.mapPoolData` (falls back to `data.mapPoolData`) |
| `team-roster-home.html` | `data.teamRoster` |
| `team-roster-away.html` | `data.teamRoster` |
| `row-casters.html` | `data.casters.castersData` |

If any of those keys is missing on connect, the corresponding overlay renders blank until the next live broadcast. Our `init` is therefore built from current state and always includes every key the current match can populate.

---

## 3. Message catalogue

### 3.1 `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`

All five carry **the identical scoreboard payload**. The type name selects the subscriber, not the content.

```jsonc
{
  "type": "scoreboard",
  "data": {
    "teamA": {
      "name": "Team Name",
      "tag": "TAG",
      "record": "0-0",
      "color": "#ff0000",
      "logoBgColor": "#090909",
      "logoUrl": "http://host:3001/assets/3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa99887766554433221100.png"
    },
    "teamB": { /* same shape */ },
    "scoreA": 0,
    "scoreB": 0,
    "title": "Grand Final",
    "subtitle": "Upper Bracket",
    "currentMap": "Ilios",
    "currentMapIndex": 0,
    "firstTo": 3,
    "mapType": "Control",
    "status": "live",
    "side": 1,
    "swapSides": false,
    "loserPicks": false,
    "gameModeIcons": { "control": "<url>", "push": "<url>" },
    "heroIcons": { "Ana": "<url>" },
    "heroPortraits": { "Ana": "<url>" },
    "sideIcons": { "attack": "<url>", "defend": "<url>" },
    "roleIcons": { "tank": "<url>", "Tank": "<url>" },
    "teamAPlayers": [
      {
        "name": "Player",
        "role": "tank",
        "position": "core",
        "pronouns": "they/them",
        "portraitUrl": "<url>",
        "heroImageUrl": "<url>"
      }
    ],
    "teamBPlayers": [ /* same shape */ ],
    "maps": [
      {
        "name": "Ilios",
        "type": "Control",
        "completed": false,
        "scoreA": 0,
        "scoreB": 0,
        "mapImageUrl": "<url>",
        "chosenBy": "A",
        "chosenByName": "TAG"
      }
    ],
    "heroBans": [
      { "firstBanTeam": "A", "teamA": ["Ana"], "teamB": ["Kiriko"] }
    ],
    "mapPool": ["Ilios", "Nepal"],
    "mapPoolData": [
      {
        "modeName": "Control",
        "modeIcons": ["<url>"],
        "maps": [
          {
            "name": "Ilios",
            "imageUrl": "<url>",
            "chosenBy": "A",
            "chosenByLogo": "<url>"
          }
        ]
      }
    ],
    "castersData": [ /* same as 3.2 castersData */ ],
    "tournamentLogoUrl": "<url>",
    "watermarkLogoUrl": "<url>",
    "teamColorsEnabled": false,
    "showPlayerPortraits": true
  }
}
```

**Nullability, exactly as consumed:**

- `teamA` / `teamB` may be `null`; overlays guard for it.
- `logoUrl`, `mapImageUrl`, `chosenByLogo`, `portraitUrl`, `heroImageUrl`, icon map values may be `null`.
- `currentMapIndex` is `null` when no map is selected. `game-overlay` adds 1 for display (`MAP ${index + 1}`); the server sends the raw 0-based index (C2).
- `currentMap` / `mapType` are `""` when no map is selected.
- `record` defaults to `"0-0"`.
- `heroBans` is always an array; entries may be missing for maps that have none.
- `mapPoolData` is `[]` when the pool is empty.

**Key-name quirks preserved:**

| Field | Quirk |
|---|---|
| `roleIcons` | Keyed by both lower-case and original-case role name. |
| `gameModeIcons` | Keyed by lower-case mode name. |
| `heroIcons` / `heroPortraits` | Keyed by hero **name**, case-sensitive. |
| `maps[].type` | Game mode display string, not the map name. |
| `mapPoolData[].modeIcons` | Always an array (0 or 1 entries), never a string. |
| `mapPoolData` ordering | Alphabetical, except `Flashpoint` and `Push` last. |

### 3.2 `casters`

```json
{
  "type": "casters",
  "data": {
    "castersData": [
      {
        "displayName": "Caster",
        "pronouns": "he/him",
        "socialMedia": { "platform": "twitter", "handle": "@caster" },
        "portraitUrl": "<url>"
      }
    ]
  }
}
```

- Order: `caster1`, `caster2`, `panel1`, `panel2`, `panel3`, `interviewer`.
- `row-casters.html` also accepts `castersData` at the top level and inside the scoreboard payload, so the same array is embedded in `scoreboard` as well.

### 3.3 `matchTimer`

Two shapes, exactly as the original sends them:

```json
{ "type": "matchTimer", "data": { "hours": 0, "minutes": 12, "seconds": 34,
  "settings": { "alignment": "center", "leadingZeroes": true, "standalone": false } } }
```

```json
{ "type": "matchTimer", "data": { "action": "start" } }
{ "type": "matchTimer", "data": { "action": "pause" } }
{ "type": "matchTimer", "data": { "action": "reset", "hours": 0, "minutes": 0, "seconds": 0 } }
```

- No bundled overlay consumes `matchTimer` in v1 (C13). It is still sent so third-party timer overlays keep working.
- `action` messages do **not** include `hours`/`minutes`/`seconds` except `reset`.

### 3.4 `show`

```json
{ "type": "show", "visible": true }
```

- `visible` sits at the **top level** of the frame, beside `type`. There is no `data` object; every bundled overlay reads `data.visible` where `data` is the whole frame.
- Implemented by all seven overlays; **not sent by the app in v1** (no UI toggle). The B1 backlog item turns it on.
- Must remain supported server-side; no schema change.

---

## 4. Broadcast policy

1. A section is broadcast only when its content actually changes (serialised comparison, not object identity).
2. All five scoreboard type names are still sent together when the scoreboard changes — they are distinct subscriptions, not duplicates to collapse.
3. Timer tick: `matchTimer` is sent once per second while running (same as the original), and once on settings change while paused.
4. The original's 3 s polling debounce disappears. Dioxus state changes trigger projection + broadcast directly; there is no `pendingOverlayUpdate` flag.
5. New clients always get `init` first, then live messages.
6. Force Update resends every projected section even when unchanged. Timer frames are separate and are not replayed by Force Update, matching the original controller.

---

## 5. Asset URLs (transport cleanup)

| Original | v1 |
|---|---|
| base64 data URL in `logoUrl`, `portraitUrl`, `mapImageUrl`, icon maps, `tournamentLogoUrl`, `watermarkLogoUrl` | Absolute URL: `http://<host>:<port>/assets/<sha256>.<ext>` |
| `readImageAsDataUrl()` per asset per broadcast | Resolver returns a URL string; no file I/O during projection in `served` mode |
| Same base64 payload duplicated six times per broadcast | Small URL strings; duplication is cheap |
| Category paths (`/assets/logos/...`, `/user-assets/...`) | Content hash only; category is not part of the URL |

- Default: **served URLs**.
- Fallback: `asset_urls = data_url` regenerates the old behaviour for third-party overlays that need inline images. This is a setting, not a code branch scattered through the projection.
- **Verification requirement:** all seven bundled overlays must be tested against served URLs before the fallback can be considered optional. `background-image: url(...)` and `img.src` both accept absolute URLs, but this is confirmed by test, not assumption.
- URLs must be absolute (`http://host:port/...`) so an overlay opened from a different origin still resolves assets.
- A changed image produces a new hash and therefore a new URL, so OBS cannot show a stale asset after a replace (C3).

---

## 6. Compatibility tests (golden files)

Fixtures live in `protocol/tests/fixtures/`:

| Fixture | Asserts |
|---|---|
| `scoreboard_full.json` | Field-for-field equality with the original payload for the same input |
| `scoreboard_minimal.json` | Null teams, no map, empty bans/pool |
| `init_after_broadcast.json` | Keyed map shape and duplicate keys |
| `init_empty.json` | Empty map before any match |
| `casters.json` | `{ castersData }` wrapper and ordering |
| `timer_tick.json` | Timer tick shape |
| `timer_action_start.json`, `timer_action_pause.json`, `timer_action_reset.json` | All three timer control shapes |
| `show.json` | Top-level `visible` boolean, no `data` envelope |

Fixture provenance: the fixtures were derived field-by-field from the original
Electron projection (`src/views/MatchController.tsx`, `electron/main.ts` and the
seven overlay HTML files) and are regenerated only when the wire contract
intentionally changes. They are **not** a snapshot of the original app's runtime
output; every field was checked against the source listed above. `asset_urls.rs`,
`init.rs` and `quirks.rs` cover the URL, cache and quirk rules that a single
fixture cannot express.

Test rules:

1. Compare **parsed JSON**, not raw text (key order is irrelevant to overlays).
2. Every field present in the fixture must be present in the output; extra fields are a failure unless the fixture is explicitly marked `allow_extra`.
3. Any intentional difference is documented in the test name and in this file.
4. A separate render smoke test loads each overlay HTML in a headless browser, feeds it the fixture, and asserts the expected DOM text (team names, scores, map number). This is the closest thing to automated OBS compatibility we can get (risk E3).

### 6.1 Implementation notes (M2)

These behaviours are locked by tests and are not wire changes:

- **Hero art convention.** A `Hero` asset's own file is its large portrait art. `Asset.icon` links the small icon asset, and an optional `Asset.portrait` link overrides the large art. `heroPortraits` and a player's `heroImageUrl` read the override if present, else the asset itself. This preserves the original's `hero.path || hero.portraitPath` preference and fixes the original's `heroImageUrl` always being `null` (hero entries had no `path`).
- **Hero icon fallback.** `heroIcons` prefers the linked `icon`, then falls back to the hero's own art (or its `portrait` link) so imported heroes with no icon link still render. An asset that is already linked as another hero's icon is skipped, which keeps one hero's icon from leaking onto another. Locked by `crates/protocol/tests/quirks.rs`.
- **Watermark fallback.** When no watermark is configured, or the configured asset id is not in the catalogue, the bundled `Peach` logo is used. A configured asset whose bytes are unavailable in `data_url` mode resolves to `null` rather than silently switching to `Peach`, matching the original.
- **Empty auxiliary colour.** An empty `auxColor` serialises as `#090909`, mirroring the original's `auxColor || '#090909'`.
- **Map-pool ordering.** Mode groups are ordered with `Flashpoint` and `Push` last and the rest alphabetically. Rust compares mode names bytewise; all bundled mode names are ASCII, so this matches the original's `localeCompare` for the supported data set.
- **`data_url` mode.** Projection never reads the filesystem. The caller supplies a byte loader to `DataUrlResolver`; a missing file yields a `null` URL, exactly like the original's `readImageAsDataUrl` failure path.
- **`show` is not cached.** `SectionCache` only stores broadcast sections. `show` is a direct frame and never appears in `init`, matching the original `overlayState` behaviour.

---

## 7. What is explicitly *not* changing

- Type names, envelope shape, `init` keyed map.
- Field names, casing, nesting, nullability, and the quirks in §3.1.
- `?port=` query parameter and `ws://localhost:<port>` connection pattern.
- Overlay HTML file names and paths under `/overlays/`.
- `show` semantics (`display: block/none`).

## 8. What *is* changing (and why)

| Change | Reason | Risk |
|---|---|---|
| Served asset URLs instead of base64 | Frame size, CPU, no repeated disk reads | Third-party overlays expecting data URLs → fallback toggle |
| No 3 s polling; event-driven broadcast | Latency and CPU | None expected; same message shapes |
| `currentMapIndex` always raw 0-based | C2 off-by-one | A third-party overlay that compensated for the old bug would now be off by one |
| Unchanged sections not re-sent | Bandwidth | A third-party overlay that relied on periodic re-sends must use `requestInit` |
| `init` built from live state | New clients render immediately | Key set must be complete; covered by fixtures |
