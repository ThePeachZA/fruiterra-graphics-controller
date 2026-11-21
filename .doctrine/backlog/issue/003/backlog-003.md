# ISS-003: C2 — `currentMapIndex` off-by-one

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §A, legacy ID **C2**.

Standardise on the raw zero-based index: overlays add 1 for display. The old Overlays → Force Update path sent `currentMapIndex + 1` while Match Controller sent the raw index.

**Status: closed · fixed.** Migrated from `docs/backlog.md` §A, C2. Protocol tests pin the raw zero-based wire value in `crates/protocol/tests/quirks.rs`.

This record preserves v1 provenance; it is not new post-v1 work.
