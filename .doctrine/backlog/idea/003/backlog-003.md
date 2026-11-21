# IDE-003: Better app logo

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

The current app mark is functional but weak. The owner wants a stronger logo
for the desktop app and project surfaces.

## Scope

- A new app mark/wordmark direction, including a small icon form that remains
  legible in the nav/taskbar and a full form for README/wiki headers.
- Update shipped app icons and documentation images only where the existing
  asset pipeline already supports it.

## Constraints

- Do not change any OBS overlay artwork or Browser Source output.
- Keep licensing and attribution clear for any new source artwork.
- No schema, protocol, or behaviour changes.

## Open questions

- Should the logo keep the current Fruiterra palette and "GRAPHICS CONTROLLER"
  lockup, or is a broader rebrand in scope?
- Which formats must ship (ICO/PNG/SVG) for the current build pipeline?
