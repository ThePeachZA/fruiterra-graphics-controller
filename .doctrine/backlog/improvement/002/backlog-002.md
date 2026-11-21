# IMP-002: Collapsible nav sidebar

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

The seven-destination nav bar takes more width than needed. The owner wants a
compact mode that reduces it to the existing SVG icons only.

## Scope

- A user-controlled compact navigation mode showing only the Lucide icons.
- Preserve the active destination indicator, accessible names, tooltips/titles,
  keyboard navigation, and screen-reader labels.
- Remember the choice in app state/persistence only if that fits the existing
  settings model; otherwise document why it stays session-only.

## Constraints

- Reuse the current icon set and theme tokens.
- No new routes and no change to the seven destinations.
- Labels must never disappear from the accessibility tree.

## Acceptance notes

- Compact mode works at phone/narrow widths and desktop widths.
- Hover and focus expose the destination name.
- A48-style CDP check confirms icon-only nav still identifies each target.

## Sequencing

Follows IMP-001 so the disclosure and affordance patterns are settled first.
