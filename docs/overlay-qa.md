# Overlay QA checklist

Manual release gate for the seven bundled overlays. Run at **1920×1080** in OBS with a loaded match, transparent background, and the overlay server running.

Design reference: `docs/assets.md` §5. The consolidated list of manual gates
across all milestones is in `docs/manual-qa.md`.

## Per-overlay checks

| # | Check | Pass |
|---|---|---|
| 1 | Overlay renders on first load from `init` (no blank frame until the next change) | ☑ |
| 2 | No network requests to `design.penpot.app` or Google Fonts | ☑ |
| 3 | Text renders in the active overlay font (Archivo) at the correct weight | ☑ |
| 4 | Long team names, tags and player names do not overflow; text shrinks, then ellipsizes, and card heights stay fixed | ☑ |
| 5 | Accented names render correctly; a CJK name falls back cleanly | ☑ |
| 6 | Logos, portraits and map images load from `/assets/<hash>` | ☑ |
| 7 | Replacing an image updates the overlay immediately (new hash, no stale cache) | ☑ |
| 8 | Server restart → overlay reconnects within ~2 s and repaints | ☑ |
| 9 | `show` hides and shows the overlay (test page) | ☑ |
| 10 | `map-pool` groups and ordering match the original | ☑ |
| 11 | `game-overlay` map number, first-to and side indicators are correct | ☑ |
| 12 | `hero-bans` uses the correct current map and first-ban order | ☑ |
| 13 | `team-roster-home` / `team-roster-away` show the correct side | ☑ |
| 14 | `row-casters` order and social icons are correct | ☑ |
| 15 | A third-party timer overlay receives `matchTimer` ticks and actions | ☐ |

## Automated pre-check

Recorded on **2026-11-11** before the manual OBS pass. This is a regression
smoke test, not a substitute for the OBS sign-off below.

- **Environment:** Windows, WebView2 (Chromium) driven by the OpenCode browser
  tools against the bundled overlays served from `http://127.0.0.1:3001`.
  Seeded state: 2 teams with rosters, 4 talent, 8 map assets, 8 hero assets, a
  map pool, 5 maps with scores and picks, hero bans and a live match.
- **Static tests:** `cargo test -p fgc-server --test overlays` passes. The tests
  assert that every overlay is embedded, subscribes to its expected message
  types, contains no external URL (only the SVG `xmlns`), declares exactly the
  five `FGC Display*` faces, and that all five active overlay WOFF2 files
  (Archivo) plus the Archivo and Saira OFL texts are embedded and valid.
- **Canvas:** all seven overlays reported `.overlay` at 1920×1080 and
  `body.scrollWidth/Height` at 1920×1080 with `meta viewport width=1920`.
- **Initial render:** every overlay painted its `init` content on first load
  (game overlay score and subtitle, map select picks, hero bans order, grouped
  map pool, both rosters, caster order and social icons).
- **Fonts:** each overlay requested the Archivo faces it uses from
  `/assets/fonts/archivo-*.woff2`; no font or image request 404ed. Accented,
  Cyrillic and CJK names rendered in the roster DOM (SØREN, ДМИТРИЙ ВОЛКОВ,
  王小明). Archivo loads with `document.fonts.check` true for the used faces;
  Saira remains bundled for the A/B comparison.
- **Network:** no requests left `127.0.0.1:3001`; only the overlay WebSocket and
  local `/assets/...` URLs were used.
- **Live update:** editing the match subtitle in Match Controller changed
  `/state` and the game overlay without a reload.
- **`show`:** `updateOverlay({ type: 'show', visible: false/true })` toggled
  `.overlay` between `none` and `block` in all seven overlays.
- **Reconnect:** after the server was restarted, `/health` reported
  `overlays: 1` for the focused tab and `overlays: 7` after reloading all tabs;
  the focused overlay repainted from the replayed cache.
- **Known observation:** a 6-caster `row-casters` layout clips the social row by
  about 14px with long handles. The same clipping occurs with a `sans-serif`
  fallback, so it is pre-existing layout behaviour rather than a font
  regression; no CSS was changed in the font port.

## Files covered

- `game-overlay.html`
- `map-select.html`
- `hero-bans.html`
- `map-pool.html`
- `team-roster-home.html`
- `team-roster-away.html`
- `row-casters.html`

## Release sign-off

| Field | Value |
|---|---|
| Version | 1.0.0 |
| Date | 2026-11-15 |
| Tester | Repository owner (ThePeachZA) |
| OBS version | 31.0.4 |
| WebView2 / CEF version | 157.0.4341.57 |
| Notes | Rows 1–14 signed off after the R1 pass with notes on 2026-11-14 (§2.2 of `docs/manual-qa.md`) and the R2 offline font pass on 2026-11-15. Row 15 (timer) is a third-party overlay and is covered by gate R4. |

No automated pixel rendering runs in CI; this checklist is the compatibility gate. Any failure blocks the release until fixed or explicitly accepted and recorded in the notes.
