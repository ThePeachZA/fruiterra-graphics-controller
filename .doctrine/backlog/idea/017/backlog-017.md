# IDE-017: Protocol and server roadmap (§C)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from the unnumbered `docs/backlog.md` §C catalogue. Bullets are grouped by section; no item was invented beyond the source text.

- Versioned protocol (`v2`) with a real full-state snapshot on connect, then typed per-section updates.
- Optional diffs/patches for large payloads; message sequence numbers and acknowledgements.
- Heartbeat/ping and stale-client cleanup.
- Per-overlay subscriptions so a client only receives the sections it renders.
- Replace base64 data URLs with served asset URLs (or a hybrid), reducing frame size.
- Configurable bind address, CORS allow-list, optional token auth and optional TLS.
- Remote overlay access over the user’s Tailscale tailnet, optional Funnel and origin-aware URLs (see §C3/D8).
- Headless/server-only mode.
- HTTP control API (score, show/hide, timer) for Stream Deck, Companion, Bitfocus and scripts.
- Server-side scene/preset switching and multiple simultaneous match outputs.

**Status: triaged.** This is an umbrella record for the section; split into individual Doctrine items when a bullet is promoted to design or implementation.

Items already promoted separately (for example IMP-003, the D8 series, P1–P4 and B4/B5) are listed here only as cross-references.
