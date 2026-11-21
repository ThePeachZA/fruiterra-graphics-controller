# IDE-006: P1 — Undo / redo

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C0, legacy ID **P1**.

Command/history layer over the state store: every mutating action pushes an inverse or bounded pre-state snapshot. Ctrl+Z / Ctrl+Shift+Z plus toolbar buttons. Must not undo server start/stop or persistence itself. No protocol change.

**Status: triaged.** Each of these gets a design pass before implementation; none was started before v1 shipped and the protocol review checkpoint passed.

P5 (Stream Deck / HTTP control API) is already represented by Doctrine **IMP-003** and was not duplicated here.
