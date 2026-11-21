# ISS-001: Added maps hidden on map-select overlay

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner finding during A45, 2026-11-21.

## Problem

On a post-v1 build, using **+ Add map** in Match Controller → Maps & Bans adds
slots labelled `Tiebreaker` instead of `Map #<numerical sequence>`, and the new
maps do not appear on `map-select.html` until the format/first-to value is
changed. The owner expects the overlay to show every map present in the
**Maps & Bans** view.

## Evidence from owner

- Exact Tailscale URL used: `https://sekhmet.note-hydra.ts.net/fgc/overlays/map-select.html`
- Phone: Samsung S25 FE, Tailscale client 1.104.6, cellular data.
- The phone overlay did load and update over the tailnet; A45 is Pass for
  connectivity. This issue is the separate map-count/name observation.
- Owner wording: "New 'Tiebreaker' maps were added, instead of 'Map #<numerical
  sequence>'" and "No new maps were shown on the overlay until the format was
  changed (the overlay should show based on how many maps are on the 'Maps &
  Bans' view)".

## Root cause

- `crates/app/src/views/match_controller.rs` labels every slot at or beyond
  `default_map_slots(first_to)` as `Tiebreaker`; user-added slots inherit that
  label even when they are ordinary extra maps.
- `crates/server/overlays/map-select.html` computes its card count as
  `(firstTo || 3) * 2 - 1`, adding one card only for the legacy current-map
  tiebreaker case. Changing the format changes `firstTo`, which is why the new
  maps appear only after that edit.
- `crates/protocol/src/projection.rs` already projects every entry in
  `match_state.maps`, so this is not a schema or payload truncation problem.

## Scope

- Label extra user-added slots by their sequence (`Map 6`, `Map 7`, …), or use a
  separate explicit tiebreaker marker only where the slot is truly a
  tiebreaker.
- Make `map-select.html` render the number of slots in the projected `maps`
  array, not a count derived from `firstTo`. Preserve the legacy tiebreaker
  behaviour for existing saved projects.
- Add an overlay/static test so `map-select.html` cannot regress to a
  first-to-derived card count.

## Constraints

- Post-v1 only. The seven bundled overlay files are frozen for v1.0.0; any edit
  needs an explicit scoped-freeze decision like A8/A9, or lands after v1.
- No schema, protocol payload or save-file format change is expected.
- A45 remains Pass; do not retroactively fail the remote-access result for this
  separate map-count defect.

## Acceptance notes

- Adding a map in Maps & Bans shows a numbered label and a new card on
  `map-select.html` without changing first-to/format.
- The legacy tiebreaker case still renders when selected as current map.
- QA evidence includes before/after CDP or OBS captures plus `/state` excerpts.
