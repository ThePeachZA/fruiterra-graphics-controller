# IMP-030: D8-8 — Remote-access settings clean-up

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C3, legacy ID **D8-8**.

Replaced the duplicate LAN switch and Access scope selector with two independent switches: Local network (`allowLan`) and Tailscale (`remoteAccess = Tailscale`). Windows Tailscale CLI calls use `CREATE_NO_WINDOW`. Done 2026-11-21; QA hash preserved.

**Status: closed · done.** See `docs/decisions.md` for the D8 decision trail.

No public Funnel exposure is claimed or enabled.
