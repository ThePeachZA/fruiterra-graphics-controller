# ISS-017: D8-1 — Origin-aware overlay connection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C3, legacy ID **D8-1**.

All seven overlays resolve `ws(s)://location.host` when served by the app, keep `?host=`/`?port=` overrides for third-party pages and use `wss` on HTTPS. Static tests reject a `ws://localhost` literal. Fixed 2026-11-16 and pulled forward into v1 so LAN mode works on a second device.

**Status: closed · fixed.** See `docs/decisions.md` for the D8 decision trail.

No public Funnel exposure is claimed or enabled.
