# CHR-005: A17 second-machine project import

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §Q (A17 project bundle export) and the A17 acceptance in `docs/decisions.md`.

Export was accepted for release, but the second-machine import run remains unrun. The acceptance note says “Accept A17; authorize tag” while explicitly recording the import gap.

**Status: triaged.** Do not claim second-machine restore was verified.

The code-level second-directory path is covered by
`export_import_round_trips_on_a_second_directory` and by the transactional export
and import hash-mismatch tests; all 10 `asset_store::tests` passed on
2026-11-21. That is not a substitute for the UI import on a clean machine.

`docs/manual-qa-runbook.md` §A17 now has an explicit outstanding-follow-up block:
steps 4–6 must be run on a second Windows account or VM with the v1 portable
build, an exported bundle, **Import project…**, and checks of teams, talent,
assets, the match and both tournaments. Record the bundle listing and
before/after evidence on both machines before changing A17 from blocked-accepted
to Pass or closing this item.
