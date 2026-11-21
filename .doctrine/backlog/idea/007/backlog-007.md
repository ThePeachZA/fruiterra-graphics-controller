# IDE-007: P2 — Presets / show templates

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C0, legacy ID **P2**.

Named user-saved snapshots of match config stored alongside `app.json`: load, duplicate, delete, rename and “save current as preset”. Needs a preset section in persistence and a picker in Settings/Match Controller.

**Status: triaged.** Each of these gets a design pass before implementation; none was started before v1 shipped and the protocol review checkpoint passed.

P5 (Stream Deck / HTTP control API) is already represented by Doctrine **IMP-003** and was not duplicated here.
