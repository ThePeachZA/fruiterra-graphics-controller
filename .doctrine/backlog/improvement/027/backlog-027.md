# IMP-027: D8-4 — Serve lifecycle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C3, legacy ID **D8-4**.

Start installs `/fgc` for pages/assets and `/` for WebSocket; stop, Tailscale off or app exit removes only those paths with targeted `--set-path <path> off`. `tailscale serve reset` is never used. Done 2026-11-21.

**Status: closed · done.** See `docs/decisions.md` for the D8 decision trail.

No public Funnel exposure is claimed or enabled.
