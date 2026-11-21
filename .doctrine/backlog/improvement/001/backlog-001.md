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

## Verification (2026-11-21)

Closed · fixed. A `dx build --desktop` debug binary was driven in WebView2
158.0.4370.33 over CDP 9222 with an isolated APPDATA/WebView profile. Computed
styles at 1280×800 showed a persistent accent-coloured chevron on both
`CollapsiblePanel` and `Disclosure`, a 3px accent summary edge while collapsed,
and hover/focus-visible summary treatment using existing tokens. A native
summary click opened the panel and keyboard Enter closed it; the panel chevron
moved from `rotate(-90deg)` to `rotate(0deg)` and back. The panel stayed open
while compact navigation was toggled, proving open state survives an unrelated
shell change. A49 records the same evidence. No screenshots were taken, and the
authoritative QA files were untouched because the desktop run used an isolated
profile.
