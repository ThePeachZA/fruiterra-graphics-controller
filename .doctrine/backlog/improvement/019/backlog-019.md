# IMP-019: B1 — Show/hide overlay control

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §B, legacy ID **B1**.

All overlays implement `{type:"show", visible}`; add a UI toggle for live use. The Overlay view now has a **Hide overlays** / **Show overlays** button backed by an ephemeral `AppStore` signal. It broadcasts the existing `Outbound::Show { visible }` message and seeds every start/restart plan with the current value, so a hidden overlay stays hidden across a server restart but is not written to `app.json`.

A50 verified the control in the real desktop WebView at 1280×800 and 375×812: pointer and keyboard activation, AX label/pressed state, `Live overlays` row, `.overlay` display `none`/`block`, hidden state across server restart, disabled state when stopped, and unchanged QA hashes. Doctrine IMP-019 was closed on 2026-11-21.

**Status: closed · done.** Delivered as a session-only post-v1 control with no protocol, schema, route or overlay-file change.
