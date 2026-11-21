# IMP-003: Stream Deck and Touch Portal control integration

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21. Two submitted entries were merged here:
"Integration with Stream Deck" and "Stream Deck and Touch Portal integration".

## Prior art

`docs/backlog.md` P5 already sketches a Stream Deck / HTTP control API for
score changes, next map, show/hide, timer start/stop, and Bitfocus Companion.
This item expands that intent to Touch Portal profiles as well.

## Scope

- One authenticated local control API for the small set of show actions that
  make sense on hardware buttons and touch panels.
- Example profiles/presets for Stream Deck and Touch Portal.
- Documentation under `docs/wiki/` that respects the same remote-access rules
  as the overlays.

## Constraints

- No Funnel/public exposure until D8-5 is designed and approved.
- Because these actions write state, define authentication and replay
  protection before exposing the API on LAN or Tailscale.
- Do not change v1.0.0, the schema, protocol payload, or overlays.

## Acceptance notes

- A Stream Deck and a Touch Portal client can drive the same documented actions.
- Unauthorised clients are rejected.
- Teardown and remote-access QA cases are recorded in the runbook.

## Sequencing

Establish the control API here before IDE-002 adds a browser control surface.
