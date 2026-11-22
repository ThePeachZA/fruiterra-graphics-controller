# Custom overlays

You can build your own HTML overlay against the same WebSocket feed that drives
the seven bundled overlays. This page is a practical guide to what version 1
already exposes. It does not add a new API, UI panel or scripting feature; it
documents the existing protocol. The formal contract is in
[`docs/protocol.md`](https://github.com/ThePeachZA/fruiterra-graphics-controller/blob/main/docs/protocol.md).

> [!NOTE]
> There is no in-app variable inspector or script generator in version 1. The
> data sources available to an author are the WebSocket feed, `/state`,
> `/health`, the bundled overlay HTML, and the copyable URLs and Settings
> values. An in-app inspector would be separate post-v1 work.

## Choose where to host your overlay

- **Replace a bundled overlay.** Copy it into the **Overlays folder** from
  Settings, keeping the exact filename, then edit that copy. It is served from
  the app's own origin, so no query parameters are needed. See
  [Overlays in OBS](Overlays-in-OBS).
- **Host your own page.** Serve your HTML from another web server or a local
  file and point it at the app server with `?port=` (and `?host=` when the page
  is not on the same machine). The app does not list custom pages on the
  **Overlay** page, so use your own URL directly in OBS.

A custom page must be a normal OBS Browser Source at **1920×1080** with a
transparent background, the same as the bundled overlays.

## Connect to the feed

The bundled overlays resolve their WebSocket target from the page they were
served from:

```js
const q = new URLSearchParams(location.search);
const host = q.get('host');
const port = q.get('port');
const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
const target = host || port
  ? `${host || 'localhost'}:${port || '3001'}`
  : (location.host || 'localhost:3001');
const ws = new WebSocket(`${scheme}://${target}/`);
```

- A page served by the app uses `location.host`, so the same HTML works on this
  PC, over LAN, or through a proxy.
- A third-party page uses `?port=3001` for the default local server. Add
  `?host=192.168.1.20` when the app is on another machine. An HTTPS page uses
  `wss` automatically.
- The server sends `init` as soon as the socket opens. Sending
  `{ "type": "requestInit" }` on open is optional but harmless; it asks for the
  same `init` again at any time.
- The server ignores every other client message. Overlays are read-only.
- If the connection drops, reconnect after about two seconds. The bundled
  overlays do this, and repaint from the next `init`.

## A minimal listener

Save this as a starting point and fill in the three render functions:

```html
<!doctype html>
<meta charset="utf-8">
<title>My FGC overlay</title>
<div id="root">connecting…</div>
<script>
const root = document.getElementById('root');

function connect() {
  const q = new URLSearchParams(location.search);
  const host = q.get('host');
  const port = q.get('port');
  const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
  const target = host || port
    ? `${host || 'localhost'}:${port || '3001'}`
    : (location.host || 'localhost:3001');
  const ws = new WebSocket(`${scheme}://${target}/`);

  ws.onopen = () => ws.send(JSON.stringify({ type: 'requestInit' }));

  ws.onmessage = ({ data }) => {
    const frame = JSON.parse(data);
    const value = frame.data || frame;

    if (frame.type === 'init') {
      root.textContent = 'ready';
      renderScoreboard(value.scoreboard);
      renderCasters(value.casters && value.casters.castersData);
      renderTimer(value.matchTimer);
      return;
    }

    switch (frame.type) {
      case 'scoreboard':
      case 'gameOverlay':
      case 'mapSelect':
      case 'heroBans':
      case 'teamRoster':
        renderScoreboard(value);
        break;
      case 'casters':
        renderCasters(value.castersData);
        break;
      case 'matchTimer':
        renderTimer(value);
        break;
      case 'show':
        root.style.display = frame.visible ? 'block' : 'none';
        break;
    }
  };

  ws.onclose = () => setTimeout(connect, 2000);
}

function renderScoreboard(s) { /* update your DOM */ }
function renderCasters(list) { /* update your DOM */ }
function renderTimer(t) { /* update your DOM */ }

connect();
</script>
```

The `frame.data || frame` line is deliberate. Most frames use the envelope
`{ "type": ..., "data": ... }`, but `show` puts `visible` at the top level with
no `data` object.

## Messages you can listen for

| Frame | Payload | Notes |
|---|---|---|
| `init` | A keyed map of the last payloads | Keys appear once that section has been broadcast at least once. Paint from this on connect. |
| `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster` | The same scoreboard payload | The type name selects the subscriber, not the content. Read the key your overlay needs. |
| `casters` | `{ "castersData": [...], "interviewee": { ... } }` | The same array is also embedded in the scoreboard payload. Each entry carries the on-air `role` (`caster`, `host`, `analyst`, `interviewer`, `other`). |
| `matchTimer` | A tick (`hours`, `minutes`, `seconds`, `settings.showHours`) or an action (`start`, `pause`, `reset`) | The standalone `timer.html` renders ticks and folds hours into minutes when `showHours` is `false`; third-party timer overlays use the same frames. |
| `show` | No `data`; top-level `visible` | Supported by the seven bundled match overlays and `timer.html`; added after v1. |

Unchanged sections are not re-sent. If you join late or miss an update, use
`requestInit` to get the current map again.

## Variables in the scoreboard payload

The scoreboard payload is the main source of broadcast data. The full field
list is in `docs/protocol.md` §3.1; the practical grouping is:

| Group | Fields |
|---|---|
| Teams | `teamA`, `teamB` → `name`, `tag`, `record`, `color`, `logoBgColor`, `logoUrl` |
| Match | `scoreA`, `scoreB`, `title`, `subtitle`, `currentMap`, `currentMapIndex`, `firstTo`, `mapType`, `status`, `side`, `swapSides`, `loserPicks` |
| Maps and bans | `maps[]` (`name`, `type`, `completed`, `scoreA`, `scoreB`, `mapImageUrl`, `chosenBy`, `chosenByName`), `mapPool`, `mapPoolData[]`, `heroBans[]` |
| People | `teamAPlayers[]`, `teamBPlayers[]` (`name`, `role`, `position`, `pronouns`, `portraitUrl`, `heroImageUrl`), `castersData[]` (`role`, `displayName`, `pronouns`, `socialMedia`, `portraitUrl`) |
| Icons | `gameModeIcons`, `heroIcons`, `heroPortraits`, `sideIcons`, `roleIcons` |
| Display | `tournamentLogoUrl`, `watermarkLogoUrl`, `teamColorsEnabled`, `showPlayerPortraits` |

Rules worth knowing before you bind these to your DOM:

- `teamA` / `teamB` may be `null`. Guard for it.
- `logoUrl`, `mapImageUrl`, `chosenByLogo`, `portraitUrl`, `heroImageUrl` and
  icon-map values may be `null`.
- `currentMapIndex` is a raw **0-based** index, and `null` when no map is
  selected. `currentMap` and `mapType` are empty strings in that case.
- `maps[].type` is the game-mode display string, not the map name.
- `roleIcons` is keyed by both lower-case and original-case role names.
  `gameModeIcons` is keyed by lower-case mode name. `heroIcons` and
  `heroPortraits` are keyed by hero name, case-sensitive.
- `mapPoolData[].modeIcons` is always an array, never a string. Its order is
  alphabetical except `Flashpoint` and `Push` last.
- `heroBans` is always an array. When Hero bans are off it is `[]` and
  `heroIcons` is `{}`. `mapPoolData` is `[]` when the pool is empty.

## Inspect the live feed

- Open `http://127.0.0.1:3001/state` for the current keyed map. It is the same
  object as `init.data`.
- Open `http://127.0.0.1:3001/health` for the status, port, connected-client
  count and app version.
- Use your browser's developer tools, **Network → WS**, to watch frames live.
- The R4 timer test page in
  [`docs/manual-qa-runbook.md`](https://github.com/ThePeachZA/fruiterra-graphics-controller/blob/main/docs/manual-qa-runbook.md)
  is a small copyable example of a third-party listener.
- **Force Update** in the app resends the projected sections; timer frames are
  separate and are not replayed by it.

## Images and fonts

By default image fields contain absolute URLs such as
`http://127.0.0.1:3001/assets/<sha256>.<ext>`. Use them directly in `img.src`
or CSS `background-image`; a changed image gets a new hash and URL, so OBS will
not cache a stale version.

If your overlay needs inline images, set **Settings → Asset URLs → Inline data
URLs**. Use fonts installed on the OBS machine or the app's locally served
fonts under `/assets/fonts/`. Do not add runtime web-font downloads.

## OBS checklist

- Width **1920**, height **1080**, transparent background.
- Leave **Shutdown source when not visible** off so the socket stays connected.
- Turn on **Refresh browser when scene becomes active** only if you need CSS
  entrance animations to replay.
- In LAN mode, remember version 1 has no authentication. Use only a trusted
  network and switch it off when the show ends.

## Related pages

- [Overlays in OBS](Overlays-in-OBS)
- [Settings](Settings)
- [Troubleshooting](Troubleshooting)
- [FAQ](FAQ)
- [Full protocol contract](https://github.com/ThePeachZA/fruiterra-graphics-controller/blob/main/docs/protocol.md)
