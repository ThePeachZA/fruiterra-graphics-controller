# Local control

Local control is a post-v1 feature for driving show actions from hardware
buttons and touch panels on the same computer as Fruiterra Graphics Controller.
It is one authenticated HTTP endpoint on the overlay server. It does not replace
the Match Controller, change the overlay protocol, or expose the app to a
network.

## How it works

Think of local control as a safe way for a button on the controller computer to
press the app's existing **Show**, **Hide** and scene controls. It does not
control OBS and it does not invent a new kind of overlay message. OBS keeps
showing the ordinary Browser Sources. The button talks to the app, the app
changes its overlay state, and the existing Browser Source connection repaints
the overlays.

There are three parts:

1. **The overlay server** is already running so OBS can load the Browser
   Sources. In post-v1 builds it also accepts `POST /control`. That extra route
   only accepts connections from the same computer.
2. **The token** is a secret shown in **Settings → Local control**. A Stream
   Deck or Touch Portal action includes it with every request, like a key.
3. **The app bridge** receives accepted actions from the server. The server does
   not edit the UI or the overlays itself; it hands the action to the running
   app, which calls the same functions used by the app's own controls.

The path for one button press is:

```text
Stream Deck / Touch Portal on this computer
  -> POST /control with token, action, timestamp and nonce
  -> overlay server checks the request
  -> desktop app applies the action through its existing controls
  -> OBS Browser Sources update over the normal overlay connection
```

Here is a concrete example. Suppose a Stream Deck key is set to **Hide**:

1. The key action creates a fresh Unix timestamp and a new random nonce.
2. It sends `{"action":"hide", ...}` to `http://127.0.0.1:<port>/control` with
   `Authorization: Bearer <token>`.
3. The server checks that the request came from this computer, that the token
   matches, that the timestamp is recent, and that the nonce has not been used
   already.
4. If those checks pass, the server forwards `hide` to the app and replies with
   HTTP 200. The reply means the request was accepted and handed to the app; the
   app then applies it on its UI task.
5. All connected overlays are hidden through the existing show message. Nothing
   changes in OBS itself.

The extra request values exist for specific reasons:

| Value | Why it is there |
|---|---|
| Token | Proves the client is allowed to send commands. Without it, any local program could press the app's controls. |
| Timestamp | Proves the command is being sent now. A request captured earlier stops working once it is more than 60 seconds old. |
| Nonce | Gives every press a unique ID. A repeated press or captured request cannot be applied twice within five minutes. |

Set-up is a one-time job. Start the overlay server from the app, copy the
endpoint and token into the hardware client, then assign one of the actions
below to each button. After that, the client generates a new timestamp and nonce
for every press. You do not need to touch the app again unless the port or token
changes.

The endpoint is available only after the overlay server is running. The app
shows the current address under **Settings → Local control**, using the loopback
address and the configured overlay port:

```text
POST http://127.0.0.1:<port>/control
```

The default port is `3001`. If the port changes, the settings card shows the new
address as soon as the server restarts.

## Token

Every request needs a bearer token. The app creates one on first use and stores
it in a plain file beside `app.json`:

```text
<app data root>\data\control-token
```

The app data root is resolved in this order:

1. the path passed with `--data-dir`;
2. the `FGC_DATA_DIR` environment variable;
3. in debug builds, `%APPDATA%\Fruiterra\Fruiterra Graphics Controller (dev)`;
4. in release builds, `%APPDATA%\Fruiterra\Fruiterra Graphics Controller`.

The token is a UUIDv4 string. The server keeps only a SHA-256 digest in memory
and compares submitted tokens in constant time. The token is not part of
`app.json`, project bundles or backups, so importing a project does not rotate
it and exporting one does not share it.

Treat the token like a password. The settings card masks it by default, offers a
reveal button and a copy button, and asks for confirmation before regenerating it.
Regeneration immediately invalidates every saved client configuration. If the
server is running when the token is regenerated, the app restarts the listener so
the new token takes effect. Stop and restart the server before using a token that
was regenerated while the server was stopped.

## Request format

Send JSON with `Content-Type: application/json` and an `Authorization` header in
the form `Bearer <token>`.

```http
POST /control HTTP/1.1
Host: 127.0.0.1:3001
Authorization: Bearer 0c7e0bd7-2c16-4d3a-9c70-4c44e2a56f31
Content-Type: application/json

{
  "action": "show",
  "timestamp": 1764147600,
  "nonce": "3f8a7c1d9b0e4a62"
}
```

| Field | Type | Rules |
|---|---|---|
| `action` | string | One of the actions below. |
| `timestamp` | integer | Unix time in seconds. The server rejects values more than 60 seconds away from its clock. |
| `nonce` | string | 8–128 characters: ASCII letters, digits, `-` or `_`. Each nonce may be used once within five minutes. |
| `scene` | string | Required only for `set_scene`; 1–128 characters limited to ASCII letters, digits, `-`, `_` or `.`. |

Use a fresh timestamp and a new nonce for every button press. Clients must
generate these values themselves; they cannot reuse a captured request. A nonce
is remembered for five minutes, so a duplicate press inside that window is
rejected rather than applied twice.

| Action | Extra field | Effect |
|---|---|---|
| `show` | — | Shows every connected overlay with the existing show message. |
| `hide` | — | Hides every connected overlay with the existing show message. |
| `force_update` | — | Rebuilds and resends every current overlay section. |
| `set_scene` | `scene` | Activates one auto-overlay scene. |
| `next_scene` | — | Moves to the next included auto-overlay scene. |
| `previous_scene` | — | Moves to the previous included auto-overlay scene. |

The scene value is the bundled overlay file name, for example
`game-overlay.html`, `map-select.html`, `hero-bans.html`, `map-pool.html`,
`team-roster-home.html`, `team-roster-away.html`, `row-casters.html`,
`row-hosts.html` or `row-interviews.html`. The server checks the shape of the
value but does not know which scenes are configured for the current match. The
app applies the scene only when that file is included in the auto-overlay set;
unknown or excluded values are ignored. Use **Next scene** and **Previous
scene** to move through the included set.

## Responses

A valid request returns HTTP 200 and a short JSON body:

```json
{ "ok": true, "action": "show" }
```

`set_scene` also echoes the requested scene:

```json
{ "ok": true, "action": "set_scene", "scene": "game-overlay.html" }
```

Every control response sends `Cache-Control: no-store`. The control route has no
CORS layer, so a browser page on another origin cannot read its response even if
it can reach the endpoint.

These are the guard responses:

| Status | Error code | Meaning |
|---|---|---|
| 503 | `control_unavailable` | The app has not installed a control bridge. Start the server from the app rather than launching a bare listener. |
| 401 | `unauthorised` | The bearer token is missing or does not match. |
| 403 | `loopback_only` | The request did not arrive from `127.0.0.1` or `::1`. |
| 400 | `stale_timestamp` | The timestamp is outside the 60-second window or the system clock cannot be read. |
| 400 | `invalid_nonce` | The nonce has an invalid shape. |
| 409 | `replayed_nonce` | That nonce was already used inside its five-minute lifetime. |
| 429 | `rate_limited` | More than 30 requests arrived from the same peer inside one minute. |
| 422 | — | The JSON body or action value is malformed. |

Errors use the same JSON shape:

```json
{ "error": "replayed_nonce" }
```

Do not retry a rejected request with the same nonce. Generate a fresh request;
otherwise the replay guard will reject it again.

## Remote-access boundary

Local network and Tailscale settings change which addresses can load OBS Browser
Sources only. They do not expose `/control`. The control route accepts loopback
connections in every build, including when both remote switches are on. Tailscale
Serve handles `/fgc` and `/` for overlays and their WebSocket; it does not route
`/control` to a remote device.

Funnel and public exposure are not part of this feature. Do not forward the
overlay port through a router, tunnel or reverse proxy to reach the control API.
Supporting LAN or Tailscale control requires a separate security design with
request signing before it can be considered.

## Troubleshooting

| Problem | What to check |
|---|---|
| The endpoint returns `control_unavailable` | Start the overlay server from the app. The listener is intentionally useless as a standalone control server. |
| Every request returns `unauthorised` | Copy the current token from **Settings → Local control**. A regenerate invalidates older copies. Keep `Bearer` in the header and do not add quotes. |
| Requests from another machine return `loopback_only` | That is expected. Run the controller on the same computer as the app. Remote overlay access does not enable remote control. |
| Repeated presses return `replayed_nonce` | The client is resending an old body. Generate a new nonce and current timestamp for each press. |
| Requests return `rate_limited` | Wait for the one-minute window to expire. The limit is 30 requests per peer per minute. |
| A scene press does nothing | Confirm the scene is included in the auto-overlay set. The app ignores scenes that are not currently configured. |
| A Stream Deck or Touch Portal button stops working after a token regenerate | Update the token in that client. No old token can be restored from the app. |

## Client examples

The [control examples](../../examples/control/README.md) directory contains a
plain HTTP walkthrough, an API Monkey Action Lua script for Stream Deck, and a
Touch Portal helper script. These are request templates and setup guides, not
importable `.streamDeckProfile` or `.tpz` files.

## Related pages

- [Settings](Settings)
- [Remote overlays](Remote-Overlays)
- [Match Controller](Match-Controller)
- [Troubleshooting](Troubleshooting)
