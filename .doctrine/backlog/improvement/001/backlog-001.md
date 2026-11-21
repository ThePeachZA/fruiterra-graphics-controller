# IMP-001: More visible collapsible elements

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

D8-9 added native `details` disclosures to secondary sections, but the collapsed
state is still visually quiet. The owner wants the affordance to be obvious
without expanding everything by default.

## Scope

- Stronger summary treatment: persistent chevron, clearer hover/focus state,
  and optional short hint text where a section name alone is ambiguous.
- Keep primary panels open; only secondary sections start collapsed.
- Reuse the existing theme tokens and `CollapsiblePanel` / `Disclosure`
  components rather than adding another disclosure system.

## Constraints

- No new UI state library or custom keyboard handling.
- `summary` must keep valid heading/phrasing children for screen readers.
- Do not change schema, protocol, server behaviour, or overlays.

## Acceptance notes

- A collapsed section is identifiable in a screenshot without reading the code.
- Keyboard focus and activation still work through native `details`.
- Open state survives unrelated state changes, as checked in A48.
