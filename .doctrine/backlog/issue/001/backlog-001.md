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

## Fix (post-v1, 2026-11-21)

- `crates/app/src/views/match_controller.rs` now labels every slot
  `Map {index + 1}`. An explicit saved name such as `Tiebreaker` stays in the
  map-name field; the slot label no longer changes with `firstTo`.
- `crates/server/overlays/map-select.html` uses the projected `maps.length` as
  the card-count source of truth. The legacy extra card remains only when
  `maps.length === TIEBREAKER_INDEX && currentMapIndex === TIEBREAKER_INDEX`.
  The unused `firstTo` state and `getMapsToShow` are gone; no schema, protocol
  payload, server runtime or v1.0.0 overlay change was made.
- `crates/server/tests/overlays.rs` adds
  `map_select_card_count_follows_projected_maps`, which pins the new function
  and the legacy condition and rejects any return to `getMapsToShow`/`firstTo`.

## Verification (assistant, 2026-11-21)

- Checks passed: `cargo fmt --all -- --check`,
  `cargo check --workspace --all-targets`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` and `dx build --desktop`.
- CDP against the built app while `firstTo` stayed at 1: before adding maps,
  Maps & Bans reported `1 map slot(s)` with labels `["Map 1"]` and `/state`
  `mapSelect.maps.length = 1`. Three **+ Add map** clicks produced
  `4 map slot(s)` with `["Map 1", "Map 2", "Map 3", "Map 4"]`, no
  `Tiebreaker`, and `/state` `mapCount = 4`. Four more clicks produced
  `8 map slot(s)` with `Map 1` through `Map 8`, still no `Tiebreaker`, and
  `mapCount = 8`.
- The live overlay at `http://127.0.0.1:3001/overlays/map-select.html` rendered
  four `.map-card` elements after the first three additions and eight after the
  next four, without changing format. In the eight-card check the 1920×1080
  layout had cards at `x = 200…1720`, `y = 271…871`, 20px gaps, document size
  `1920×1080` and no document overflow.
- QA state was restored from the authoritative backup to `app.json`
  `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` and
  `app.json.bak`
  `A2B4908D62A28986B1F093B4C53DA5961A6B4461AE0A3D74A33D9B447630FE8C`.
  `tailscale serve status` reported `No serve config`. No screenshots were
  taken or published.

**Status: closed · fixed.** A45 remains Pass; this was separate post-v1 work.
