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
| 15 | A third-party timer overlay receives `matchTimer` ticks and actions | ☑ |
| 16 | Team colour enters only from the outer edge of each frame; there is no centre colour bar or seam | ☑ |
| 17 | Ban-order text is neutral `--fgc-ink-on-art` on `--fgc-plate-dark`, never a team colour | ☑ |
| 18 | A transparent hero cut-out fills half the frame height and stands on the bottom, with no letterboxing | ☑ |
| 19 | Moment overlays wait 0.8s for the stinger; dense overlays use the shorter planned 0.35s delay | ☑ |
| 20 | The separate background source paints a solid opaque theme plate and sits below the scene sources in OBS | ☑ |
| 21 | `?motion=still`, `?tone=team` and `?fx=off` variants work from the Browser Source URL | ☑ |
| 22 | Reduced motion freezes the lattice but keeps the static composition | ☑ |
| 23 | Only `transform` and `opacity` animate; there is no per-frame JavaScript | ☑ |
| 24 | The canonical FX block exists only in the eight agreed overlays; timer and game overlay have none | ☑ |
| 25 | `every_overlay_carries_the_same_fx_block` passes and the sentinelled copies are byte-identical | ☑ |
| 26 | `?bg=opaque` paints a solid `--fgc-surface` plate and keeps the hexagon lattice visible over it; `?bg=transparent` hides the plate and keeps the lattice | ☑ |
| 27 | `?bg=off` / `?bg=on` normalise to opaque / transparent, and a scene overlay defaults to transparent unless seeded with `data-bg="opaque"` | ☑ |
| 28 | `background.html` stays opaque by default, and `?bg=transparent` removes its plate while keeping the glow and both lattices | ☑ |
| 29 | The app Background switch is an accessible `Switch` that rewrites Browser Source URLs and broadcasts a cached `background` section, so connected sources change without an OBS refresh | ☑ |
| 30 | `auto.html` forwards the background mode to its scene iframes | ☑ |
| 31 | Timer and game overlay still have no background layer or `data-bg` attribute | ☑ |
| 32 | The byte-equality test includes `.fgc-bg`, the remaining `data-bg` rule, the live and init-aware handler, and forbids the old `html[data-bg="opaque"] .fgc-fx` hide rule | ☑ |
| 33 | A `{"type":"background","data":{"opaque":true\|false}}` message changes `data-bg` and the plate in connected overlays without a refresh, while the lattice stays visible | ☑ |
| 34 | A `{"type":"init","data":{"background":{"opaque":true\|false}}}` frame restores the app-selected mode on a fresh or reloaded Browser Source | ☑ |
| 35 | All twelve overlays carry the early `html, body` overflow guard before any FX rule or iframe, so a refresh shows no transient scrollbars | ☑ |
| 36 | FX carriers report zero scrollbar gutters in a 1000x700 tab even though the `-360px` overscan reports `scrollWidth: 1360` and `scrollHeight: 738` | ☑ |
| 37 | Roster overlays use the wider team wash and bottom glow, a small core role badge fully inside the portrait, divider-style vertical rails outside the card row, a taller core plate with smaller pronouns, no name-plate code chip, and no long-name overflow | ☑ |
| 38 | Roster overlays remove the side rails, use proportional substitute hexagons, order both rows Damage, Tank, Support, and use a softer side wash with a full-width bottom glow | ☑ |
| 39 | Roster overlays replace the side wash with one subtle full-width bottom gradient that reaches both ends of the window, with no side-anchored or cut-off glow | ☑ |
| 40 | Roster logos have inset breathing room, the core/sub divider animates in, stored text colours are respected, and the bottom gradient balance compensates for background luminance; team properties can recommend Background and Text from the logo independently and can combine computed contrast with automatic logo colours | ☑ |
| 41 | Map select marks the current map clearly, presents team-coloured score plates with a visible winner, gives the winning logo and map a subtle themed glow, and keeps banned hero portraits square | ☑ |
| 42 | Map select and hero bans use the Clean Canvas: no score slabs, per-card headers, mode icons or `Now playing` pill; winners are centred in a soft circular halo; the opening plays once and later updates animate only changed data; scene overlays default to opaque | ☑ |
| 43 | Map pool uses the Clean Canvas without a heading, with plain mode icons, five columns, wrapped map names, a darker picked image and a readable pick glow derived from the choosing team's lightened colour; duplicate broadcasts are no-ops | ☑ |

The current design language, the per-overlay playbook and the motion rules are
in [`docs/overlay-design-language.md`](overlay-design-language.md). The
post-v1 evidence for these checks is in `docs/manual-qa.md` §7 and the repeatable
steps are in `docs/manual-qa-runbook.md` sections A72, A73, A74, A75, A76, A77,
A78, A79, A80, A81, A82, A83, A84 and A85. A75 and A76 remain historical records; A77 supersedes
their hexagons-hidden claim and its refresh caveat. A79 supersedes the earlier
roster experiment that put the reduced hexagon abbreviation, then the team tag or
country code, in the name plate; that idea remains deferred to the backlog. A80
supersedes the A79 side rails and the side-anchored glow; the owner rejected both
after review and asked for the full-width bottom glow instead. A81 supersedes
the remaining A80 side wash: both rosters now paint one subtle bottom gradient
across the entire 1920px frame, so no home or away edge treatment remains. A84
supersedes the A83 chrome direction: map select and hero bans now use the Clean
Canvas, centred winner treatment and targeted one-shot reveal, with opaque
backgrounds as the default. A85 applies the same philosophy to map pool, removes
the heading and mode hexagons, darkens picked art, and derives a readable glow
from the choosing team's colour.

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
  Saira remains bundled as a reversible fallback.
- **Network:** no requests left `127.0.0.1:3001`; only the overlay WebSocket and
  local `/assets/...` URLs were used.
- **Live update:** editing the match subtitle in Match Controller changed
  `/state` and the game overlay without a reload.
- **`show`:** `updateOverlay({ type: 'show', visible: false/true })` toggled
  `.overlay` between `none` and `block` in all seven overlays.
- **Reconnect:** after the server was restarted, `/health` reported
  `overlays: 1` for the focused tab and `overlays: 7` after reloading all tabs;
  the focused overlay repainted from the replayed cache.
- **M9-7, fixed after v1:** the original 6-caster `row-casters` row clipped long
  handles; real WebView2 measured `+26.75px` right / `-96.92px` left for
  `@anextremelylongcastersocialhandle` and `-23.67px` left for
  `@longcasterhandlefive`. The same baseline clipping occurred with a
  `sans-serif` fallback, so it was pre-existing layout behaviour rather than a
  font regression; no CSS was changed in the font port. ISS-018 later confined
  `.talent-subtext` to the card content box, fixed the icon at 20px, and
  ellipsized long handles/pronouns while retaining the full handle in `title`.
  Gate A53 measured all six social rows inside their card content boxes with
  unchanged 1920×1080 geometry and card widths.
- **IMP-032, fixed after v1:** empty team logo slots in `game-overlay.html` now
  render the team TAG, or a short name abbreviation when the tag is blank, on
  the existing logo background. The placeholder hides as soon as a real logo
  is supplied, clears for unassigned teams, ellipsizes long tags while keeping
  the full value in `title`, and uses a contrast colour against the logo
  background. Gate A54 verified the live server `init` plus seven replayed
  states in real WebView2 at 1920×1080 with no geometry change and zero page
  console errors.

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
| Notes | Rows 1–14 signed off after the R1 pass with notes on 2026-11-14 (§2.2 of `docs/manual-qa.md`); row 15 signed off after the R4 third-party timer pass on 2026-11-15. |

No automated pixel rendering runs in CI; this checklist is the compatibility gate. Any failure blocks the release until fixed or explicitly accepted and recorded in the notes.
