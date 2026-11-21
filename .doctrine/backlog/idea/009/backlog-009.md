# IDE-009: P4 — Tournament / bracket overlay

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §C0, legacy ID **P4**.

Read-only 1920×1080 overlay rendering the completed bracket (single elim, double elim, groups/standings). New broadcast type plus overlay HTML file; must not change the existing overlay protocol.

**Status: triaged.** Each of these gets a design pass before implementation; none was started before v1 shipped and the protocol review checkpoint passed.

P5 (Stream Deck / HTTP control API) is already represented by Doctrine **IMP-003** and was not duplicated here.
