# IMP-020: B2 — All overlay URLs with copy buttons

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §B, legacy ID **B2**.

Expose every overlay URL with copy/open actions instead of only the game overlay URL. D8-6/D8-9 later added labelled URL groups with copy/open, so verify before closing this legacy item.

**Status: closed · done.** No code change was needed. D8-6/D8-9 added labelled
Browser Source groups for every enabled access path, and `OverlayUrls` iterates
`OVERLAY_FILES` so each group contains all seven URLs with per-row Open and Copy
actions. A48 verified the grouped URLs and hidden-state behaviour; Doctrine
IMP-020 was closed on 2026-11-21.
