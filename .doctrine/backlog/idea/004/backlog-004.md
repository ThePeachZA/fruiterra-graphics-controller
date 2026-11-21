# IDE-004: Auto-overlay scene switcher (auto.html)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21. IMP-022 (B4, basic hotkeys) is merged here: the only
hotkeys are the scene-switching keys inside the auto-overlay controller. There
is no global app shortcut, global OS hook, or hotkey support elsewhere.

## Problem

The owner wants one OBS Browser Source that can switch between a chosen subset
of the seven bundled overlays, controlled from inside the app. B4's hotkeys are
useful only in that controller, so the two requests are one feature.

## Approved design

- Add a post-v1 `auto.html` page served by the existing `/overlays/{file}`
  route. It opens one WebSocket connection, reads an explicit `autoOverlay`
  section, and shows exactly one iframe from the included subset.
- Keep the included overlay list and active overlay session-only in `AppStore`.
  They are not added to `AppState` or `app.json`.
- Add `autoOverlay` to the wire protocol as `{ included: string[], active:
  string | null }`. The app broadcasts it on start, restart and every
  selection/active change. Existing sections and the seven overlays are
  unchanged.
- In the Overlay view, add a full-width controller with native checkboxes for
  inclusion, native radios for the active scene, a status region, and the
  revealable auto Browser Source URL in each access group.
- Keys `1`–`7` activate the overlay at that fixed bundled-overlay position when
  it is included. `ArrowLeft` / `ArrowRight` cycle through the included subset.
  Handlers live on the controller container, ignore events from native form
  controls, and call `stop_propagation()` / `prevent_default()` only for handled
  keys.

## Constraints

- The seven existing overlay files are frozen for v1.0.0. This is a post-v1
  addition and must not silently rewrite them.
- Selection and active state stay session-only; no persistence schema change.
- The new page allow-lists the seven bundled filenames and accepts an active
  scene only when it is included. No arbitrary `src` or external URL.
- Follow the same reveal/hide and remote-access rules as Browser Source URLs.
- Verify in the real desktop WebView at 1280×800 and 375×812, including the
  hotkeys, hidden URL state, reconnect after server restart and no horizontal
  overflow.
