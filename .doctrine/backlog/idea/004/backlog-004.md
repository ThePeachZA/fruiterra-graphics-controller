# IDE-004: Auto-overlay controller page (auto.html)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

The owner wants an `overlay/auto.html` page that can be used as a single OBS
Browser Source and controlled from inside the app, with a choice of which
overlays are included.

## Scope

- A new auto-overlay page that composes a user-selected subset of the seven
  bundled overlays.
- A UI control in the app to pick the included overlays and drive their
  visibility/state from one place.

## Constraints

- The seven existing overlay files are frozen for v1.0.0. This is a post-v1
  addition and must not silently rewrite them.
- Any state/schema/protocol addition needs its own explicit design and QA; do
  not assume the existing `/state` payload can carry the new selection.
- Follow the same reveal/hide and remote-access rules as Browser Source URLs.

## Open questions

- Does "auto" mean auto-stacking, auto-rotating, or auto-showing based on match
  state?
- Should the selection be per tournament, per match, or global?
