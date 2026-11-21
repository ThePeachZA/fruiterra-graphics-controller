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

**Persistence decision:** session-only. `AppState::settings` holds persisted
broadcast, overlay and timer configuration; adding a shell-only preference there
would change the `app.json` model without a schema change. The compact flag stays
in a Dioxus signal in the shell and resets on restart. Text labels and each
button's `aria-label`/`title` stay in the DOM when the icons are visually
compact.

## Acceptance notes

- Compact mode works at phone/narrow widths and desktop widths.
- Hover and focus expose the destination name.
- A48-style CDP check confirms icon-only nav still identifies each target.

## Sequencing

Follows IMP-001 so the disclosure and affordance patterns are settled first.

## Verification (2026-11-21)

Closed · done. A `dx build --desktop` debug binary was driven in WebView2
158.0.4370.33 over CDP 9222 with an isolated APPDATA/WebView profile. At
1280×800 the shell grid changed from `248px 1032px` to `72px 1208px` by
activation click and again by keyboard Enter; at 375×812 it was `72px 303px`. The seven
`.fgc-nav__label` nodes stayed in the DOM in compact mode and were clipped to
`1px` / `inset(50%)`, while the accessibility tree exposed all seven button
names as non-ignored and every button's `aria-label`, `title` and `data-label`
matched. After keyboard navigation to Settings, `aria-current="page"` and a
`3px` inset accent indicator were present. The tooltip pseudo-element content
was `"Match Controller"` with opacity `0` when hidden and `1` on real pointer
hover at both widths and on real keyboard focus-visible at 1280. Document
`scrollWidth` equalled
`innerWidth` at 1280 and 375. The compact flag never entered `Settings` or
`app.json`. A49 records the same evidence; no screenshots were taken.
