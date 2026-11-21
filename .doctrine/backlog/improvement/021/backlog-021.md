# IMP-021: B3 — Overlay health/reconnect indicator

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §B, legacy ID **B3**.

Surface connected client count and last broadcast time.

The Overlay view now shows **Connected clients** and **Last broadcast** in the
existing `KeyValues` panel. The count comes from the existing `ServerStatus`
poll. `ServerController` keeps a session-only last-broadcast timestamp, records
it only when a frame is actually sent (including the initial start/restart
plan), and clears it on start, restart and stop. A one-second view clock ages the
value as `Just now`, seconds, minutes or hours, or shows `Never` before any
frame. While the server is running with zero clients, a `role="status"` note
asks the operator to open or refresh an OBS Browser Source. No schema, protocol,
server runtime, route, persisted setting or v1.0.0 overlay changed.

A51 verified the change in the real desktop WebView at 1280×800 and 375×812:
stopped showed `0 clients` / `Never` and no note; start with no clients showed
`Running · 0 clients`, `Just now`, `/health {"overlays":0,...}` and a
non-ignored `role="status"` note; an overlay target moved `/health` to
`overlays:1`, the UI to `1 client` and cleared the note; hiding overlays reseeded
`Just now` and the age advanced to `1s ago`; restart briefly returned to
`0 clients` with the note before the overlay reconnected; closing it returned to
`0 clients`, the note and an aging broadcast; stopping cleared the value to
`Never`, removed the note and disabled the toggle. `scrollWidth` equalled
`innerWidth` at both widths, the new rows and note stayed inside the viewport,
and the accessibility tree exposed them. At 375×812 the floating timer
initially covered Stop/Restart; after a normal wheel scroll they were
hit-testable and Stop worked. Harness and authoritative QA hashes were
unchanged, `tailscale serve status` was `No serve config`, and no screenshots
were taken. Doctrine IMP-021 was closed on 2026-11-21.

**Status: closed · done.**
