# IMP-026: D8-3 — Tailscale detection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C3, legacy ID **D8-3**.

Finds `tailscale.exe` on PATH then Program Files, runs only `status --json` and `version` on a blocking task, maps backend state to plain UI states and enables the Tailscale switch when connected. Done 2026-11-21.

**Status: closed · done.** See `docs/decisions.md` for the D8 decision trail.

No public Funnel exposure is claimed or enabled.
