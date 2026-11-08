# Overlay QA checklist

Manual release gate for the seven bundled overlays. Run at **1920×1080** in OBS with a loaded match, transparent background, and the overlay server running.

Design reference: `docs/assets.md` §5.

## Per-overlay checks

| # | Check | Pass |
|---|---|---|
| 1 | Overlay renders on first load from `init` (no blank frame until the next change) | ☐ |
| 2 | No network requests to `design.penpot.app` or Google Fonts | ☐ |
| 3 | Text renders in Saira at the correct weight | ☐ |
| 4 | Long team names, tags and player names do not overflow | ☐ |
| 5 | Accented names render correctly; a CJK name falls back cleanly | ☐ |
| 6 | Logos, portraits and map images load from `/assets/<hash>` | ☐ |
| 7 | Replacing an image updates the overlay immediately (new hash, no stale cache) | ☐ |
| 8 | Server restart → overlay reconnects within ~2 s and repaints | ☐ |
| 9 | `show` hides and shows the overlay (test page) | ☐ |
| 10 | `map-pool` groups and ordering match the original | ☐ |
| 11 | `game-overlay` map number, first-to and side indicators are correct | ☐ |
| 12 | `hero-bans` uses the correct current map and first-ban order | ☐ |
| 13 | `team-roster-home` / `team-roster-away` show the correct side | ☐ |
| 14 | `row-casters` order and social icons are correct | ☐ |
| 15 | A third-party timer overlay receives `matchTimer` ticks and actions | ☐ |

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
| Version | |
| Date | |
| Tester | |
| OBS version | |
| WebView2 / CEF version | |
| Notes | |

No automated pixel rendering runs in CI; this checklist is the compatibility gate. Any failure blocks the release until fixed or explicitly accepted and recorded in the notes.
