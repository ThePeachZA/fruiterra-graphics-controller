# IMP-023: B5 — Timer overlay

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §B, legacy ID **B5**.

Bundle a broadcast timer that reads the existing `matchTimer` messages. The
timer was control-only in v1; C13 records that v1 decision.

**Done 2026-11-21.** The owner narrowed this item to a plainly rendered timer
with no extra bells or whistles. `crates/server/overlays/timer.html` is a
standalone Browser Source at `/overlays/timer.html`. It renders the last
`matchTimer` tick, applies the tick's `settings.leadingZeroes` and
`settings.alignment`, renders the `reset` payload, and obeys top-level `show`.
It has no local clock, no countdown logic, no new wire field, no persisted
state and no auto-overlay scene entry.

The page uses the existing embedded-overlay route and origin-aware WebSocket
resolver with `requestInit` and a 2 s reconnect. A local `data:` favicon keeps
the Browser Source from making a missing-resource request. Static guards are in
`crates/server/tests/timer_overlay.rs`, separate from the frozen seven-overlay
suite. README, protocol, build, wiki and QA docs now describe the standalone
source without changing the v1 C13 decision.

Gate A55 in `docs/manual-qa.md` and `docs/manual-qa-runbook.md` records real
desktop WebView2/CDP evidence at 1920×1080: live ticks advanced, pause held,
stop reset, unpadded and left/right states updated, `show` toggled `display`,
the canvas stayed transparent, and there were no external requests, failed
responses or console errors. Authoritative QA hashes were restored and
`tailscale serve status` remained `No serve config`. No screenshots were taken
or published.
