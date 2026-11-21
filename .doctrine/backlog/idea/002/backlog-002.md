# IDE-002: Remote browser control of Match Controller

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

The controller UI is desktop-only. The owner wants to open a browser page on
another device and drive the Match Controller, using the same remote-access
rules as the overlay URLs.

## Scope

- A browser-reachable control surface for the live match actions the desktop
  controller exposes (score, map/ban state, timer, show control).
- The page must honour the existing Local network / Tailscale switches and
  status model from `docs/backlog.md` C3 D8-8/D8-9.
- No Funnel or public exposure until D8-5 is designed and approved.

## Constraints

- This is a write-capable surface, unlike the read-only overlays. It needs an
  explicit authentication/authorisation design before implementation; tailnet
  membership alone is not enough if the page is reachable beyond the tailnet.
- Do not change the v1.0.0 payload, schema, protocol, or frozen overlay files.
- A control API is likely shared with IMP-003; design it once, not twice.

## Acceptance notes

- Browser control is unavailable when both remote switches are off.
- LAN and Tailscale entry points follow the same URL shapes and reveal/hide
  rules as Browser Source URLs.
- A second-device QA case is added to `docs/manual-qa-runbook.md` §7.

## Open questions

- Which actions are safe to expose first?
- What credential or token model is acceptable for LAN and tailnet use?
