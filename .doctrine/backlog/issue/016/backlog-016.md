# ISS-016: C15 — Asset cleanup after a failed load

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §A, legacy ID **C15**.

If `app.json` and `app.json.bak` are both unreadable, start with empty default state. Cleanup refuses to build a keep set unless saved state loaded successfully, so neither manual nor post-save/exit collectors delete user blobs.

**Status: closed · fixed.** Migrated from `docs/backlog.md` §A, C15. Done 2026-11-21 with regression tests; see `docs/decisions.md`.

This record preserves v1 provenance; it is not new post-v1 work.
