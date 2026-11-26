# Local control examples

These examples drive the same authenticated endpoint used by the app's **Local
control** settings card. The endpoint is for hardware buttons on the computer
running Fruiterra Graphics Controller. It is not a remote-control API.

Read [Local control](../../docs/wiki/Local-Control.md) for the complete contract,
including token storage, replay rules and remote-access boundaries.

## Before you start

1. Open **Settings → Local control** and note the endpoint. The default is
   `http://127.0.0.1:3001/control`.
2. Reveal and copy the bearer token, or read `<app data root>\data\control-token`
   from disk. The token is a 36-character UUIDv4 string.
3. Start the overlay server from the Overlay view. `/control` returns
   `control_unavailable` until the app installs the bridge.
4. Keep the server, Stream Deck or Touch Portal installation, and this helper on
   the same Windows account and computer.

The app data root is the `--data-dir` path when set, otherwise `FGC_DATA_DIR`,
otherwise `%APPDATA%\Fruiterra\Fruiterra Graphics Controller (dev)` in a debug
build, otherwise `%APPDATA%\Fruiterra\Fruiterra Graphics Controller`.

## Actions

| `action` | Extra field | Effect |
|---|---|---|
| `show` | — | Show all overlays. |
| `hide` | — | Hide all overlays. |
| `force_update` | — | Resend every current overlay section. |
| `set_scene` | `scene` | Activate a configured auto-overlay scene, for example `game-overlay.html`. |
| `next_scene` | — | Move to the next included scene. |
| `previous_scene` | — | Move to the previous included scene. |

Every request needs `action`, a current Unix `timestamp` in seconds, a fresh
`nonce`, and the `Authorization: Bearer <token>` header. `set_scene` also needs a
separate `scene` field. Nonces are unique for five minutes; timestamps more than
60 seconds from the server clock are rejected. A peer may send at most 30
requests per minute.

## Plain HTTP example

The following uses placeholders for the values a real client generates. Replace
`<port>` with the port shown in Settings and set `FGC_TOKEN` in your environment
without putting the literal token in a script or shell history.

```bash
curl -sS -X POST "http://127.0.0.1:<port>/control" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $FGC_TOKEN" \
  -d '{"action":"hide","timestamp":1764147600,"nonce":"example-nonce-001"}'
```

A successful response is:

```json
{ "ok": true, "action": "hide" }
```

`set_scene` uses a separate field:

```json
{
  "action": "set_scene",
  "timestamp": 1764147600,
  "nonce": "example-nonce-002",
  "scene": "game-overlay.html"
}
```

PowerShell clients can read the token without printing it:

```powershell
$token = (Get-Content "$env:APPDATA\Fruiterra\Fruiterra Graphics Controller\data\control-token" -Raw).Trim()
$body = @{
  action    = 'show'
  timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
  nonce     = '{0}-{1}' -f [DateTimeOffset]::UtcNow.ToUnixTimeSeconds(), [guid]::NewGuid().ToString('N')
} | ConvertTo-Json -Compress
Invoke-RestMethod -Method Post -Uri 'http://127.0.0.1:3001/control' `
  -Headers @{ Authorization = "Bearer $token" } `
  -ContentType 'application/json' -Body $body
```

If the app runs with `FGC_DATA_DIR`, read the token from
`$env:FGC_DATA_DIR\data\control-token` instead. A debug build may use the
`(dev)` data root described above.

## Client guides

- [Stream Deck with API Monkey](stream-deck-api-monkey.md)
- [Touch Portal](touch-portal-plugin.md)
- [Touch Portal helper script](scripts/control-touch-portal.ps1)

These are setup guides and scripts. No importable `.streamDeckProfile` or `.tpz`
file is supplied, because the exact profile format depends on the installed
plugin and Touch Portal version. Build the buttons using the steps in each guide.

## Security notes

- Never paste the token into a URL, a shared profile file, chat or a screenshot
  intended for publication.
- Regenerating the token in Settings invalidates all saved client copies.
- LAN and Tailscale switches affect overlays only. They do not enable remote
  access to `/control`.
- Control responses have `Cache-Control: no-store` and no permissive CORS. Do not
  add a browser-based proxy that weakens those boundaries.
- If a button fails with `replayed_nonce`, generate a fresh nonce; do not resend
  the same JSON body.
