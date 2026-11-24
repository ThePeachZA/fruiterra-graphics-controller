# Per-overlay visual specifications — Fruiterra Night Broadcast refresh

**Status:** **implemented and QA'd 2026-11-22.** All seven overlays in scope
(`hero-bans.html`, `map-pool.html`, `map-select.html`, `row-casters.html`,
`team-roster-home.html`, `team-roster-away.html`, `timer.html`) are restyled to
the Night Broadcast tokens. A58 in
[`docs/manual-qa.md`](manual-qa.md) records the real WebView2 evidence; one
recorded deviation is noted in §4.9.

**Source of truth:** `docs/overlay-refresh-design.md` (§1–§8, philosophy, token system, layout concepts, owner decisions).  
**Reference pilot:** `crates/server/overlays/game-overlay.html` — its `:root` token block (lines 52–93) is the approved pattern to imitate in spirit; it is DONE and is reference only.  
**Scope (this spec):** `hero-bans.html`, `map-pool.html`, `map-select.html`, `row-casters.html`, `team-roster-home.html`, `team-roster-away.html`, `timer.html`.  
**Out of scope:** `auto.html` (pure iframe host); `game-overlay.html` (already refreshed — reference only).

**Frozen contracts (hard constraints):** same filenames; identical wire message types and payload field names; identical DOM ids/classes the JS relies on; no new message types; no schema/persistence changes; same pixel geometry (no box, padding, font-size, gap, radius or position moves); 1920×1080 transparent canvas; `html`/`body` background stays `transparent`; no remote/CDN assets, no external URLs, no new fonts, no new files; no official league or team marks. The static test requires the literal string `@font-face` exactly 5 times in each of the 7 match overlays (never in a comment or string literal); `timer.html` keeps its single Archivo 600 face.

Retired values to purge everywhere (CSS, inline SVG `fill`, JS string literals): `#fff`, `#ffffff`, `#090909`, `#151515`, `#eeeeee`, `#eee`, `#ac46f7`, `#444444`, `#2d2d2d`, `#1a1a1a`, `#666`, `rgba(172, 70, 247, ...)`, `rgba(9, 9, 9, 0.6)` and `rgba(0, 0, 0, 0.3)` scrims. Replace every one with a Night Broadcast token or a token-derived value. `rgba(0, 0, 0, 0.65)` on `timer.html` is explicitly NOT retired (kept per the timer special note).

No new visual features: no new panels, badges, or animation beyond colour/state change.

---

## 0. Shared token block (inline in every overlay file)

Each overlay inlines its own `:root` block, exactly as it inlines its own `@font-face` rules today. No shared external stylesheet. Copy verbatim into each target file's `<head>` `<style>` block, placed after the `@font-face` rules and before the existing layout CSS (the pilot places it at lines 49–93 of `game-overlay.html`).

```css
/* === NIGHT BROADCAST TOKENS ===
   Shared design tokens for the Fruiterra overlay refresh. Each overlay
   inlines this block, exactly as it inlines its own font faces. */
:root {
  --fgc-canvas: transparent;

  --fgc-surface: #0b0d12;
  --fgc-surface-raised: #11151d;
  --fgc-surface-inverse: #eef2f8;
  --fgc-surface-muted: #151922;

  --fgc-line: #2a3140;
  --fgc-line-strong: #3a4356;
  --fgc-line-on-inverse: #0b0d12;

  --fgc-text: #eef2f8;
  --fgc-text-muted: #8d99ad;
  --fgc-text-faint: #6b7688;
  --fgc-text-inverse: #0b0d12;

  --fgc-accent: #ff7a1a;
  --fgc-accent-strong: #ff9346;
  --fgc-accent-soft: rgb(255 122 26 / 16%);
  --fgc-on-accent: #0b0d12;

  --fgc-success: #3ecf8e;
  --fgc-warning: #f5b84b;
  --fgc-danger: #ef5b5b;

  --fgc-space-1: 4px;
  --fgc-space-2: 8px;
  --fgc-space-3: 12px;
  --fgc-space-4: 16px;
  --fgc-radius-sm: 4px;
  --fgc-radius-md: 6px;
  --fgc-radius-lg: 10px;
  --fgc-radius-pill: 999px;
  --fgc-margin: 43px;

  --fgc-duration-micro: 120ms;
  --fgc-duration-state: 240ms;
  --fgc-duration-reveal: 480ms;
  --fgc-duration-scene: 900ms;
  --fgc-ease-out: cubic-bezier(0.22, 1, 0.36, 1);
}
```

Token usage rules (from §3.1):
- `--fgc-surface` is the default chrome. `--fgc-surface-inverse` is reserved for score chips and other high-contrast readouts.
- `--fgc-accent` (orange) is the system signal: one primary state per view (active map, live score, selected caster, first-ban indicator). `--fgc-accent-soft` for fills/scrims; `--fgc-accent` for text/rules.
- Team colours (`--fgc-team-a` / `--fgc-team-b`, set dynamically via inline style) are the primary identity where relevant, with the matching `--fgc-team-a-text` / `--fgc-team-b-text` colour. Neutral dark chrome fallback when colours are off or the surface is not team-owned.
- `--fgc-success` / `--fgc-warning` / `--fgc-danger` are semantic states only, never decoration.

---

## 1. hero-bans.html

### 1.1 Purpose & wire
Shows the two team score chips, team names/logos, first-to format, hero ban portraits, current ban hero per side, map type/name, and first/second ban order indicators in the footer. Consumes wire message types `scoreboard`, `gameOverlay`, and `heroBans`; reads `teamA`, `teamB`, `scoreA`, `scoreB`, `teamColorsEnabled`, `teamA.color`, `teamB.color`, `firstTo`, `heroBans` (with `currentMapIndex`, `firstBanTeam`, `teamA[]`, `teamB[]`), `heroPortraits`, and `maps[]`.

### 1.2 Surface plan

> This table records the first Night Broadcast pass of hero bans, before the
> full-bleed rework. The current layout is §1.9 to §1.13; the table is kept for
> the record and no longer describes the shipped overlay.

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.team-name` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (neutral dark bg) |
| `.team-score` | background `#fff`, color `#090909` | background `var(--fgc-surface-inverse)` → `rgb(238, 242, 248)`; color `var(--fgc-text-inverse)` → `rgb(11, 13, 18)` | inverse score chip |
| `.first-to-section` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface |
| `.first-to-text` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier |
| `.ban-order` (both `.ban-order-1-container` and `.ban-order-2-container`) | background `#ac46f7` | `var(--fgc-accent)` → `rgb(255, 122, 26)` | accent (first/second ban indicator) |
| `.ban-order-text` | color `#fff` | `var(--fgc-on-accent)` → `rgb(11, 13, 18)` | text on accent |
| `.ban-hero-name` | background `#fff`, color `#090909` | background `var(--fgc-surface-inverse)` → `rgb(238, 242, 248)`; color `var(--fgc-text-inverse)` → `rgb(11, 13, 18)` | inverse chip (ban hero name) |
| `.map-info` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface (map strip) |
| `.map-info-type` | color `#ac46f7` | `var(--fgc-accent)` → `rgb(255, 122, 26)` | accent text (map number) |
| `.map-info-name` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (map name) |

Team-section backgrounds (`#home-section`, `#away-section`, class `.team-section`): currently `#090909` fallback / `d.teamA.color` / `d.teamB.color` set inline by JS. New: fallback `var(--fgc-surface)` → `rgb(11, 13, 18)`; when team colours are on, `d.teamA.color` / `d.teamB.color` with matching contrast text (see §1.4).

Placeholder/empty state: `.ban-hero-name-text` shows `-` when no ban; colour `--fgc-text-inverse` on `--fgc-surface-inverse`. No error styling.

### 1.3 State matrix
- Team colours ON: `.team-section` fill = `d.teamA.color` / `d.teamB.color`; `.team-name` = computed contrast colour (see §1.4); score chip stays `--fgc-surface-inverse` with `--fgc-text-inverse` text (score chips never team-coloured).
- Team colours OFF: `.team-section` = `--fgc-surface`; `.team-name` = `--fgc-text`; score chip = inverse plate.
- Empty/placeholder: ban hero text = `-`; `.ban-order` shows `FIRST`/`SECOND` (default `FIRST`/`SECOND` when `firstBan` is null); map name = `TBD`/`MAP N`.
- Active ban: the `.ban-order` container on the first-ban side reads `FIRST`, the other `SECOND`; both use `--fgc-accent` fill with `--fgc-on-accent` text.
- Map populated vs TBD: `.map-info-type` = `MAP N` (accent); `.map-info-name` = map name or `TBD` (`--fgc-text`).

### 1.4 JS helpers
Add a contrast helper identical in behaviour to the pilot's `getContrastColor`:

```
getContrastColor(hexColor) {
  // returns '#0b0d12' when luminance > 0.5, else '#eef2f8'
  return luminance > 0.5 ? '#0b0d12' : '#eef2f8';
}
```

Apply it to `.team-name` when team colours are enabled:
- `team-coloured` → `el.homeName.style.color = getContrastColor(d.teamA.color)`; `el.awayName.style.color = getContrastColor(d.teamB.color)`.
- Not team-coloured → `el.homeName.style.color = ''` (reverts to CSS `var(--fgc-text)`); `el.awayName.style.color = ''`.

Placeholder colour fallback: `teamLogoPlaceholderColor` is not used in hero-bans (no logo placeholder text); the hero-bans placeholder is the `-` dash in `.ban-hero-name-text`, styled `--fgc-text-inverse` on `--fgc-surface-inverse`.

Team-section fallback colour: replace hardcoded `'#090909'` with `var(--fgc-surface)` (resolves to `rgb(11, 13, 18)`).

### 1.5 Font/face note
`@font-face` count stays exactly 5 (Light 300, Regular 400, Medium 500, SemiBold 600, Black 900). Aliases:
- `.team-name`, `.first-to-text`, `.ban-order-text`, `.ban-hero-name-text`, `.map-info-name`: `'FGC Display Medium'` (500), sizes 24–36px.
- `.team-score`: `'FGC Display SemiBold'` (600), 48px, `tabular-nums`.
- `.map-info-type`: `'FGC Display Medium'` (500), 24px.

### 1.6 Favicon
`<link rel="icon" href="data:,">` is currently ABSENT from `<head>`. Add it immediately after `<meta name="viewport">` (before `<title>`), matching `game-overlay.html` line 6 and `timer.html` line 6.

### 1.7 CDP-verifiable assertions
- `getComputedStyle(document.querySelector('.team-score')).backgroundColor === 'rgb(238, 242, 248)'` and `.color === 'rgb(11, 13, 18)'`.
- `getComputedStyle(document.querySelector('.ban-order')).backgroundColor === 'rgb(255, 122, 26)'` and `.color` of `.ban-order-text` === `rgb(11, 13, 18)`.
- `getComputedStyle(document.querySelector('.map-info')).backgroundColor === 'rgb(11, 13, 18)'`; `.map-info-type` color === `rgb(255, 122, 26)`; `.map-info-name` color === `rgb(238, 242, 248)`.
- With team colours OFF: `.team-section` background === `rgb(11, 13, 18)`; `.team-name` color === `rgb(238, 242, 248)`.
- With team colours ON and a dark team colour: `.team-name` color === `rgb(238, 242, 248)`; with a light team colour: `.team-name` color === `rgb(11, 13, 18)`.
- `@font-face` literal count === 5; none inside a comment or string literal.

### 1.8 Retired-value purge map (hero-bans.html)
- `#fff` → `--fgc-text` / `--fgc-surface-inverse` / `--fgc-on-accent` context (`.team-name`, `.first-to-text`, `.ban-order-text`, `.ban-hero-name` bg, `.map-info-name`, `.team-score` bg).
- `#090909` → `--fgc-surface` / `--fgc-text-inverse` context (`.team-score` color, `.first-to-section`, `.ban-hero-name-text`, `.map-info`, JS fallback `'#090909'` ×2).
- `#ac46f7` → `--fgc-accent` (`.ban-order` background, `.map-info-type` color).

### 1.9 Owner-requested redesign - the OWWC27 full-bleed layout (2026-11-22)

The owner reviewed the two-panel layout and said there were "too many elements
on the screen". The overlay was rebuilt around the OWWC27 hero bans reference:
two full-bleed halves that fill the screen, with the hero art as the picture
and only four things on top of it - the team logo in the outer top corner, the
role hexagon, the hero name over the lower half of the art and the ban order
underneath in the team colour.

The header (team names, logos, scores and the first-to count) and the map
strip are **gone**. The first ban of each side is the only one shown; any
further bans stay on the map select cards.

**New surface plan.** The old `.hero-panel`, `.hero-panel__art`,
`.hero-panel__tag`, `.hero-panel__extra`, `.hero-panel__band`,
`.hero-panel__name`, `.map-strip` and the whole `.header` are gone.

| Element selector | Token value | Treatment |
|---|---|---|
| `.hero-bans` | - | the full-bleed row, `display: flex` |
| `.hero-half` | background `var(--fgc-surface)` | one half per team, `flex: 1 1 0` |
| `.hero-half__wall` | background `var(--fgc-surface-raised)`, masked with the honeycomb tile | the honeycomb wall behind the art (see §1.11) |
| `.hero-half__grid` | a white radial and linear gradient, masked with the honeycomb tile | the faint honeycomb in front of the art, weighted to the outer edge (see §1.11) |
| `.hero-half__art` | - | the hero art, `object-fit: cover` |
| `.hero-half__art--cutout img` | - | `object-fit: contain; object-position: center bottom` for a transparent cut-out |
| `.hero-half__scrim` | `linear-gradient(to top, rgb(0 0 0 / 78%) 0%, rgb(0 0 0 / 42%) 24%, rgb(0 0 0 / 0%) 56%)` | legibility under the caption |
| `.hero-half__logo` | background `var(--fgc-plate-dark)` | the 100x115 hexagonal team logo badge, outer top corner, `clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)` |
| `.hero-half__logo-art` | `background-size: contain` | the artwork inside the band the hexagon inscribes, so the badge never cuts into it |
| `.hero-half__role` | background `var(--fgc-plate-dark)` | the 60x68 role badge, the same pointy-top hexagon as the logo |
| `.hero-half__name` | color `var(--fgc-ink-on-art)` | the hero name, Display 900 |
| `.hero-half__order` | color `var(--fgc-accent)`, overridden inline with the team colour | the `1ST BAN` / `2ND BAN` label |
| `.hero-bans__centre` | a `linear-gradient` between the two team colours | the 110px banner between the halves (see §1.11) |

`--fgc-plate-dark` and `--fgc-ink-on-art` are local tokens in the same `:root`
block, because the white role icons, the white team logos and the hero name
over art need a dark plate and near-white ink whichever theme is active.

**New data.** Two per-hero facts were added, both optional:
- `Asset.transparent` links a hero to a transparent cut-out of the art. The
  overlay prefers it and falls back to the regular hero art.
- `Asset.role` records the hero's role (tank, damage or support), which the
  overlay shows as the role hexagon.

They reach the overlay in the scoreboard payload as `heroTransparent` and
`heroRoles`, both keyed by the hero name. The role hexagon only appears when
the hero has a role and the app has a role icon for it.

**State matrix.** The ban order follows the first-ban team: when
`firstBanTeam` is `A` the home half reads `1ST BAN` and the away half
`2ND BAN`; when it is `B` the two swap. The label is written in the colour of
the team that made the ban and falls back to `var(--fgc-accent)`. A side with
no ban shows `-` as its name and an empty art box.

**JS helpers.** `renderHalf(art, role, roleIcon, name, heroName)` picks the
transparent art before the regular art and resolves the role hexagon.
`updateHeroBans(teamABans, teamBBans)` renders the first ban of each side.
`updateBanOrder(firstBan)` writes the two labels, and `applyTeamColour(order,
colour)` paints them.

### 1.10 CDP-verifiable assertions (redesign)

- Two `.hero-half` elements fill the width (905px each at 1920x1080) with the
  110px `.hero-bans__centre` banner between them, and both reach the full
  1080px height.
- `#home-art img` and `#away-art img` resolve to the first ban of each side
  and load; a cut-out computes `object-fit: contain` and
  `object-position: 50% 100%`, the fallback computes `object-fit: cover`.
- `#home-name` and `#away-name` hold the banned hero's name in
  `FGC Display Black` at 92px, colour `rgb(238, 242, 248)`.
- `#home-order` and `#away-order` read `1ST BAN` and `2ND BAN` and compute to
  the two team colours.
- `#home-logo` sits at `left: 43px` and `#away-logo` at `right: 43px`, both
  100x115 on a `rgb(11, 13, 18)` plate with the pointy-top hexagon
  `clip-path`.
- `#home-role` and `#away-role` are 60x68 with the same hexagon `clip-path`,
  and their icons load.
- The scrim computes to the gradient above.
- No `#header`, no `#map-strip` and no `.hero-panel` remain.

### 1.11 Owner-requested redesign - the honeycomb wall (2026-11-22)

The owner reviewed the first full-bleed version and said the hexagons "kind
of just exist meaninglessly there with no creativity behind it". They were
right: the tile was small enough to read as a screen door, it was uniform, it
was painted in the generic accent (the same hue as the ban label, so it
carried no information), it sat behind the full-bleed art where it could not
be seen at all, and it had also been pushed into the centre banner, which is
the one place the OWWC27 reference has none.

The hexagons are now the overlay's **shape language** instead of a texture.
The OWWC27 reference uses the same shape three times - the team logo is a
hexagonal badge, the role badge is a hexagon, and the backdrop is a honeycomb
wall - so the hexagon now appears at three scales in one composition.

**The wall.** `.hero-half__wall` sits behind the art at `z-index: 0` and is
built from a 56x100 tessellating SVG used as a mask at `mask-size: 180px
322px`. That is roughly five cells across a 905px half, which reads as
architecture rather than noise. The fill is `var(--fgc-surface-raised)` and
the joints are `var(--fgc-line-strong)`, so the wall is neutral and the
accent stays reserved for the ban label. A `::after` gradient fades the wall
up into the surface from 30% down, which keeps the caption area clean and
gives the field a direction.

**The grid.** `.hero-half__grid` sits in front of the art and the scrim at
`z-index: 3` and uses the same tile, so the wall reads on regular hero art as
well as through a transparent cut-out. Its background is a radial gradient
weighted to the **outer** edge of the half (where the team logo sits and the
hero is not) plus a linear gradient that is denser towards the caption. The
away half mirrors the radial. It never reaches the centre of the half, so the
hero is never veiled.

**The badges.** The team logo and the role badge now share the wall's
pointy-top hexagon, `clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%,
0% 75%, 0% 25%)`. The logo grew from 100x100 to 100x115 and uses a
`drop-shadow` instead of a `box-shadow`, because a box shadow is clipped by a
`clip-path`. The artwork moved into `.hero-half__logo-art`, the band the
hexagon inscribes, so the badge never cuts into the logo.

**The banner.** `.hero-bans__centre` no longer carries a texture. It is the
seam where the two teams meet, so it is a `linear-gradient` from the home
team's colour to the away team's colour, each blended 42% into the surface.
The two colours arrive as `--team-a-color` and `--team-b-color`, which the
overlay sets from the scoreboard payload.

**State matrix.** Nothing about the payload changed. The wall, the grid and
the badges are static; only the team colours, the team logo artwork and the
ban order follow the payload, exactly as before.

### 1.12 CDP-verifiable assertions (honeycomb wall)

- `.hero-half__wall` computes to `z-index: 0`, `.hero-half__art` to `1`,
  `.hero-half__grid` to `3` and `.hero-half__caption` to `4`.
- The wall computes to `rgb(17, 21, 29)` with `rgb(58, 67, 86)` joints, and
  its mask size is `180px 322px` (about five cells across a 905px half).
- The wall is not the accent colour: `rgb(17, 21, 29)` is not
  `rgb(255, 122, 26)`.
- The grid computes to a radial and a linear gradient, and the away half
  mirrors the radial to `at 100% 46%`.
- `#home-logo` and `#away-logo` are 100x115 with the pointy-top hexagon
  `clip-path`, and `#home-logo-art` is the inscribed 100x57.5 band.
- `#home-role` and `#away-role` are 60x68 with the same `clip-path`.
- The pointy-top hexagon appears exactly twice in the file: the logo badge and
  the role badge.
- `.hero-bans__centre` computes to a `linear-gradient` between the two team
  colours, and no `.hero-half__texture` element remains.

### 1.13 Owner-requested redesign - the cut-out placement, the outer edge wash and the reveal delay (2026-11-23)

The owner reviewed the honeycomb build and raised five things: the transparent
hero cut-outs were letterboxed in the middle of the half with the hero small and
floating, the hexagons were too opaque, the centre seam read as a seam, the team
colour sat on the ban label instead of on the half, and the `1ST BAN` /
`2ND BAN` wording was shouted. The reveal also started immediately, so a stinger
or a scene transition was cut short by the bans appearing on top of it. The
overlay was reworked around each point.

**The cut-out placement.** The cut-out files are a full 1920x1080 transparent
canvas with the hero standing on the bottom, so `object-fit: contain` scaled the
whole canvas into the half and left the hero at roughly 47% of the height. The
cut-out is now placed by script. `subjectBox(img)` draws the image into an
offscreen canvas and scans the alpha channel (alpha above 8) for the hero's
opaque bounding box; `placeCutout(img, box)` computes
`const scale = box.height / subject.height;` and centres that box on the half
with `transform-origin: bottom center`, so the hero fills the half height and
stands on the bottom. The `<img>` inside `.hero-half__art--cutout` keeps its
natural size (`width: auto; height: auto;`), is absolutely positioned, and the
`transform` alone moves and scales it.
`.hero-half__art--cutout-fallback` covers a canvas that cannot be read and falls
back to `object-fit: cover` with `object-position: center bottom`.

**The hexagons.** The wall joints dropped from `opacity: 0.5` to `0.3`. The lit
grid copy in front of the art dropped from `rgb(255 255 255 / 18%)` and
`/ 7%` to `/ 11%` and `/ 4%`, and the floor wash from `6%` to `3.5%`. The wall
fill and the scrim are unchanged.

**No seam.** `.hero-bans__centre` and all of its CSS are gone. The two
`.hero-half` elements tile the full 1920 width, 0-960 and 960-1920, with no
banner between them.

**The edge wash.** `.hero-half__edge` is a new absolutely positioned layer at
`z-index: 4`, over the art and over the honeycomb. The home half paints
`linear-gradient(to right, ...)` from `--team-a-color` and the away half
`linear-gradient(to left, ...)` from `--team-b-color`, each
`color-mix(in srgb, <colour> 46%, transparent)` at the outer edge,
`16%` at 18% and transparent by 38%. The two properties are set on the
`#hero-bans` container with `style.setProperty('--team-a-color', ...)` and
`style.setProperty('--team-b-color', ...)`. The ban-order label no longer takes
the team colour.

`color-mix()` needs Chrome 111 or newer, which is OBS 31 and newer. The
documented floor for this project is OBS 31.0.4, so the wash is safe, but an
older OBS would drop the whole declaration and leave the edge bare.

**The label.** `1ST BAN` / `2ND BAN` became `First ban` / `Second ban` in the
markup and in `updateBanOrder()`, and the CSS `text-transform: uppercase` still
renders them `FIRST BAN` / `SECOND BAN`. The label is neutral -
`color: var(--fgc-ink-on-art)` (`#eef2f8`) - instead of the accent.

**The reveal delay.** `--hero-reveal-delay: 0.8s` on `:root` is read by
`triggerAnimations()`. The home half runs
`heroReveal 0.9s cubic-bezier(0.16, 1, 0.3, 1) var(--hero-reveal-delay) both`,
the away half the same animation at
`calc(var(--hero-reveal-delay) + 0.12s)`, and the two captions
`captionReveal 0.7s cubic-bezier(0.22, 1, 0.36, 1)` at
`calc(var(--hero-reveal-delay) + 0.35s)` and `+ 0.47s`.

**Wire/contract.** Nothing about the payload changed. `--team-a-color` and
`--team-b-color` still come from `teamA.color` and `teamB.color` in the
scoreboard payload, and the cut-out still arrives as `heroTransparent` keyed by
hero name with `heroPortraits` as the fallback.

**Retired values.** `object-fit: contain; object-position: 50% 100%` on the
cut-out image; `.hero-bans__centre` and its gradient; the wall joint
`opacity: 0.5`; the grid tints `18%` and `7%`; the floor wash `6%`; the
`1ST BAN` / `2ND BAN` literals; and `color: var(--fgc-accent)` on
`.hero-half__order`.

### 1.14 CDP-verifiable assertions (cut-out placement, edge wash and reveal delay)

- The document is 1920x1080 and `getComputedStyle(document.body).backgroundColor`
  is `rgba(0, 0, 0, 0)`.
- No `.hero-bans__centre` element exists, and the two `.hero-half` rectangles
  tile `0-960` and `960-1920`.
- Both `.hero-half__art` boxes carry `hero-half__art--cutout`, and both images
  report `naturalWidth` 1920 and `naturalHeight` 1080.
- An alpha-channel scan of a cut-out image drawn into a canvas returns a subject
  box, and the placed image's scaled box fills the half height with its bottom
  on the half's bottom edge.
- The `.hero-half__grid` background resolves to a radial and a linear gradient
  whose first stop is `rgba(255, 255, 255, 0.11)`, and the away half mirrors the
  radial to `at 100% 46%`.
- `getComputedStyle(wall, '::before').opacity` is `0.3`.
- `.hero-half--home .hero-half__edge` background starts
  `linear-gradient(to right` and `.hero-half--away .hero-half__edge` starts
  `linear-gradient(to left`.
- `getComputedStyle(document.querySelector('#hero-bans'))` reports
  `--team-a-color` and `--team-b-color` equal to the two live team colours.
- `#home-order` and `#away-order` `textContent` read `First ban` and
  `Second ban`, compute `text-transform: uppercase`, and colour
  `rgb(238, 242, 248)`.
- `getComputedStyle(document.documentElement)` reports `--hero-reveal-delay` as
  `0.8s`, and the two halves report animation delays of 0.8s and 0.92s.
- `getAnimations()` on a half reports `heroReveal` with an 800ms delay and a
  900ms duration, and after it finishes the opacity is 1 with
  `transform: matrix(1, 0, 0, 1, 0, 0)`.
- The role badge icon loads at 500x500.

---

## 2. map-pool.html

### 2.1 Purpose & wire
Renders the dark map-pool art grid: a header, then one column per game mode, each column containing map cards (image, name bar, chosen-by team logo overlay). Consumes `scoreboard` / `mapPool` via `init` → `mapPool` redirection; reads `mapPoolData[]` (modeName, maps[], modeIcons), `gameModeIcons`. Message types `scoreboard` and `mapPool`.

### 2.2 Surface plan

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.map-pool-header` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (96px Black title) |
| `.mode-header` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface (mode header bar) |
| `.mode-name` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier |
| `.map-image` (placeholder, no imageUrl) | background-color `#2d2d2d` | `var(--fgc-surface-muted)` → `rgb(21, 25, 34)` | placeholder surface |
| `.map-image.chosen::after` | background `rgba(0, 0, 0, 0.3)` | **Replaced 2026-11-22.** The flat scrim became `.map-pick-glow`, a bottom-up accent gradient on its own layer | gradient (picked-map glow) |
| `.map-name` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface (map name bar) |
| `.map-name-text` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier |
| `.empty-pool` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (empty state) |

Placeholder/empty state: `.empty-pool` shows "No maps in pool" in `--fgc-text` on transparent canvas. Placeholder map image (no `imageUrl`) is `--fgc-surface-muted`; a chosen map keeps the greyscale artwork and gains the `.map-pick-glow` gradient.

**Picked-map glow (owner request, 2026-11-22).** The old scrim was a child of the
greyscaled `.map-image`, so the filter drained the accent out of it and it
rendered as a flat grey wash. The glow is now its own layer, `.map-pick-glow`,
sitting above the artwork and below the team logo:

- `linear-gradient(to top, --fgc-pick-glow 0%, --fgc-pick-glow-mid 30%, --fgc-pick-glow-faint 52%, transparent 72%)`.
- The layer covers the artwork only (`bottom: 42px`, the name-bar height).
- `.map-card.picked .map-name` adds `box-shadow: inset 0 2px 0 var(--fgc-accent)` as the rule under the artwork, plus a faint accent wash on the bar.
- The three glow tokens live next to the accent tokens, so the theme system can retune them in one place.

### 2.3 State matrix
- Default: all map cards on `--fgc-surface-muted` placeholder surface when no image; name bar `--fgc-surface`; mode header `--fgc-surface`.
- Chosen/picked: `.map-image.chosen` keeps `filter: grayscale(100%)`; the `.map-pick-glow` layer carries the accent gradient and the `.map-card.picked .map-name` bar carries the accent rule; the team logo overlay sits above the glow (user asset, neutral chrome).
- Empty pool: `.empty-pool` text `--fgc-text`, no error styling.
- Team colours: do not spill into map-pool chrome (neutral only). Chosen-by logo is a user asset with no colour change to the card.

### 2.4 JS helpers
In `createMapCard`, replace the placeholder fallback:
- `background-color: #2d2d2d` → `background-color: var(--fgc-surface-muted)` (resolves `rgb(21, 25, 34)`).
No luminance/contrast function exists in this file; none is required (no dynamic team text here).

### 2.5 Font/face note
`@font-face` count stays exactly 5. Aliases:
- `.map-pool-header`: `'FGC Display Black'` (900), 96px.
- `.mode-name`: `'FGC Display SemiBold'` (600), 20px.
- `.map-name-text`: `'FGC Display SemiBold'` (600), 18px, `tabular-nums` not required (map names are text).

### 2.6 Favicon
`<link rel="icon" href="data:,">` is currently ABSENT. Add it after `<meta name="viewport">`, before `<title>`.

### 2.7 CDP-verifiable assertions
- `getComputedStyle(document.querySelector('.map-pool-header')).color === 'rgb(238, 242, 248)'`.
- `getComputedStyle(document.querySelector('.mode-header')).backgroundColor === 'rgb(11, 13, 18)'`; `.mode-name` color === `rgb(238, 242, 248)`.
- Placeholder `.map-image` background === `rgb(21, 25, 34)`.
- Chosen `.map-pick-glow` background-image contains `linear-gradient` and `255, 122, 26`; the picked `.map-name` box-shadow contains `255, 122, 26`.
- `.map-name` background === `rgb(11, 13, 18)`; `.map-name-text` color === `rgb(238, 242, 248)`.
- `@font-face` literal count === 5.
- Each `.map-column` carries a `columnReveal` animation with its own delay, 0.13s after the previous column.

### 2.8 Retired-value purge map (map-pool.html)
- `#fff` → `--fgc-text` (`.map-pool-header`, `.mode-name`, `.map-name-text`, `.empty-pool`).
- `#090909` → `--fgc-surface` (`.mode-header`, `.map-name`).
- `#2d2d2d` → `--fgc-surface-muted` (`.map-image` placeholder; JS `background-color: #2d2d2d` fallback).
- `rgba(0, 0, 0, 0.3)` → the `.map-pick-glow` accent gradient (replaced 2026-11-22).

---

## 3. map-select.html

### 3.1 Purpose & wire
Renders vertical map-select cards: one active (expanded) slot plus inactive slots, each with image, type/name, chosen-by tag, and score board for completed maps. Consumes `scoreboard`, `gameOverlay`, `mapSelect`; reads `currentMapIndex`, `loserPicks`, `gameModeIcons`, `maps[]` (name, type, scoreA, scoreB, mapImageUrl, completed, chosenByName), `heroBans[]` (firstBanTeam, teamA, teamB) and `heroPortraits` (hero name to portrait URL), `teamA`, `teamB`, `tournamentLogoUrl`. Message types `scoreboard`, `gameOverlay`, `mapSelect`.

### 3.2 Surface plan

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.map-select-title` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (128px Black title) |
| `.map-card` | background `#fff` | `var(--fgc-surface-raised)` → `rgb(17, 21, 29)` | card surface (active + inactive) |
| `.map-card__image--placeholder` | background `linear-gradient(180deg, #1a1a1a 0%, #2d2d2d 100%)` | `var(--fgc-surface-muted)` → `rgb(21, 25, 34)` (solid) | placeholder surface |
| `.map-card__image--completed::after` | background `rgba(0, 0, 0, 0.3)` | `var(--fgc-accent-soft)` → `rgba(255, 122, 26, 0.16)` | token-derived completed scrim |
| `.map-card__text` | background `#fff` | `var(--fgc-surface-raised)` → `rgb(17, 21, 29)` | text panel surface |
| `.map-card__type` | color `#ac46f7` | `var(--fgc-accent)` → `rgb(255, 122, 26)` | accent text (mode) |
| `.map-card__name` | color `#090909` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (map name, now on dark surface) |
| `.map-card__chosen-by` | color `#666` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |
| `.map-card__board` | background `rgba(9, 9, 9, 0.6)` | `rgb(11, 13, 18 / 0.60)` (token-derived from `--fgc-surface`) | semi-transparent score board |
| `.map-card__score`, `.map-card__score-divider` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (tabular nums) |
| `.watermark` | background `#fff` | `var(--fgc-surface-inverse)` → `rgb(238, 242, 248)` | inverse surface (bottom band) |
| `.watermark__text` | color `#090909` | `var(--fgc-text-inverse)` → `rgb(11, 13, 18)` | text on inverse surface |

Placeholder/empty state: cards with no map show `--fgc-surface-muted` image and `TBC` text; no error styling.

### 3.3 State matrix
- Default/inactive: card surface `--fgc-surface-raised`; placeholder image `--fgc-surface-muted`; type `--fgc-accent` only when a real mode exists, else `TBC` in `--fgc-text`; name `--fgc-text`.
- Active/current: same surface treatment; card expands via existing `flex: 1.8` geometry (unchanged); no colour change beyond the active slot's normal treatment (accent already used for type).
- Completed: image gets `--fgc-accent-soft` scrim; score board `rgb(11, 13, 18 / 0.60)`; team logo overlay (user asset).
- Empty/TBC: `--fgc-surface-muted` placeholder; `TBC` in `--fgc-text`; chosen-by hidden.
- Hero bans: a card with bans carries a `.map-card__bans` strip inside the map image; a card without bans carries none.
- Team colours: neutral only in map-select; chosen-by logo is a user asset.

**Ban thumbnails (owner request, 2026-11-22).** The OWCS27 and OWWC27 map
selects put the banned heroes inside the map image rather than in a separate
panel, so the pick and the bans read as one card:

- `.map-card__visual` wraps the map image and the strip, so the strip is anchored to the bottom of the artwork.
- The strip is a sibling of `.map-card__image`, never a child, so a completed map's `filter: grayscale(100%)` cannot drain the colour from the ban portraits.
- `.map-card__ban` is a 58×58 portrait crop; `.map-card__ban-number` is a 24×24 accent diamond (`rotate(45deg)`) with a counter-rotated label, so the ban order reads as a number.
- The order follows `firstBanTeam`: the team that banned first gets ban 1, the other team gets ban 2.
- No wire change: `heroBans` and `heroPortraits` are already part of the scoreboard payload.

### 3.4 JS helpers
No luminance/contrast function exists in this file; none required. The placeholder image branch:
- `background-color: #2d2d2d` (in the gradient) is removed entirely; `.map-card__image--placeholder` becomes solid `var(--fgc-surface-muted)`.

### 3.5 Font/face note
`@font-face` count stays exactly 5. Aliases:
- `.map-select-title`: `'FGC Display Black'` (900), 128px.
- `.map-card__type`: `'FGC Display Medium'` (500), 18px.
- `.map-card__name`: `'FGC Display Medium'` (500), 24px.
- `.map-card__chosen-by`: `'FGC Display'` (400), 14px.
- `.map-card__score`, `.map-card__score-divider`: `'FGC Display Medium'` (500), 48px, `tabular-nums` (score already sets it? the class does not set tabular-nums; add `font-variant-numeric: tabular-nums` to `.map-card__score` — this is a text-treatment clarification, not a geometry change).
- `.watermark__text`: `'FGC Display Medium'` (500), 28px.

### 3.6 Favicon
`<link rel="icon" href="data:,">` is currently ABSENT. Add it after `<meta name="viewport">`, before `<title>`.

### 3.7 CDP-verifiable assertions
- `getComputedStyle(document.querySelector('.map-select-title')).color === 'rgb(238, 242, 248)'`.
- `.map-card` background === `rgb(17, 21, 29)`.
- `.map-card__image--placeholder` background === `rgb(21, 25, 34)`.
- `.map-card__image--completed::after` background === `rgba(255, 122, 26, 0.16)`.
- `.map-card__text` background === `rgb(17, 21, 29)`.
- `.map-card__type` color === `rgb(255, 122, 26)`; `.map-card__name` color === `rgb(238, 242, 248)`.
- `.map-card__chosen-by` color === `rgb(141, 153, 173)`.
- `.map-card__board` background === `rgba(11, 13, 18, 0.6)`.
- `.map-card__score` color === `rgb(238, 242, 248)`.
- `.watermark` background === `rgb(238, 242, 248)`; `.watermark__text` color === `rgb(11, 13, 18)`.
- `@font-face` literal count === 5.
- The `.map-card__bans` strip sits inside `.map-card__visual` and never inside `.map-card__image`.
- The ban portraits compute `filter: none`, so a completed map keeps its colour under the strip.
- The badge transform is a 45° rotation (`matrix` with `b > 0` and `c < 0`) and the label transform is the counter-rotated −45°.

### 3.8 Retired-value purge map (map-select.html)
- `#fff` → `--fgc-text` / `--fgc-surface-raised` / `--fgc-surface-inverse` context (`.map-select-title`, `.map-card`, `.map-card__text`, `.map-card__score`, `.map-card__score-divider`, `.watermark`).
- `#090909` → `--fgc-text` (`.map-card__name`) / `--fgc-text-inverse` (`.watermark__text`).
- `#ac46f7` → `--fgc-accent` (`.map-card__type`).
- `#1a1a1a`, `#2d2d2d` → `--fgc-surface-muted` (`.map-card__image--placeholder` gradient replaced).
- `rgba(0, 0, 0, 0.3)` → `--fgc-accent-soft` (`.map-card__image--completed::after`).
- `#666` → `--fgc-text-muted` (`.map-card__chosen-by`).
- `rgba(9, 9, 9, 0.6)` → `rgb(11, 13, 18 / 0.60)` (`.map-card__board`).

---

## 4. row-casters.html

### 4.1 Purpose & wire
Renders the caster desk row: camera panels (portrait images) plus a shared lower bar with talent name, social handle, separator, and pronouns. Supports single-caster (narrow) and multi-caster (wide) layouts. Consumes `scoreboard` / `gameOverlay` / caster data; reads `castersData[]` (displayName, portraitUrl, socialMedia {platform, handle}, pronouns). Message type `scoreboard`/`gameOverlay` (no dedicated caster message type; piggybacks on existing).

### 4.2 Surface plan

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.talent-image` (placeholder, no portraitUrl) | background-color `#2d2d2d` | `var(--fgc-surface-muted)` → `rgb(21, 25, 34)` | placeholder surface |
| `.talent-text` | background `#fff` | `var(--fgc-surface-raised)` → `rgb(17, 21, 29)` | card text panel |
| `.talent-name` | color `#090909` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (48px Medium) |
| `.social-handle` | color `#ac46f7` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |
| `.separator` | color `#ac46f7` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |
| `.pronouns` | color `#ac46f7` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |
| Inline SVG social icons (Twitter, Twitch, YouTube, Instagram, TikTok) — 5× `fill="#ac46f7"` | `fill="#ac46f7"` | `fill="#8d99ad"` (resolved `--fgc-text-muted`) | icon colour |

Icon-colour justification: social icons are neutral metadata, not system signals. `--fgc-accent` (orange) is reserved for live/active/selected states; using it on every social icon would dilute the signal. `--fgc-text-muted` keeps icons legible but subordinate to the talent name. Resolved hex `#8d99ad` is used because an SVG presentation `fill` attribute cannot consume a CSS custom variable.

Placeholder/empty state: no casters → container renders nothing (no error styling). Missing portrait → `--fgc-surface-muted` placeholder. Missing social/pronouns → those sub-elements are omitted by JS (no empty placeholder styling).

### 4.3 State matrix
- Default/multi-caster: `.talent-text` = `--fgc-surface-raised`; `.talent-name` = `--fgc-text`; handle/separator/pronouns = `--fgc-text-muted`; icons = `#8d99ad`.
- Single-caster: same surface treatment; container narrows to `max-width: 900px` (geometry unchanged).
- Team colours: not applicable (casters are neutral; no team colour spill).
- Missing portrait: `--fgc-surface-muted` placeholder surface, never a broken image.

### 4.4 JS helpers
In `createCasterCard`, replace the placeholder fallback:
- `background-color: #2d2d2d` → `background-color: var(--fgc-surface-muted)` (resolves `rgb(21, 25, 34)`).
No luminance/contrast function exists; none required (caster name is always `--fgc-text` on `--fgc-surface-raised`).

The five `get*Icon()` functions return SVG with `fill="#ac46f7"`; replace each with `fill="#8d99ad"`.

### 4.5 Font/face note
`@font-face` count stays exactly 5. Aliases:
- `.talent-name`: `'FGC Display Medium'` (500), 48px.
- `.social-handle`, `.separator`, `.pronouns`: `'FGC Display Light'` (300), 24px.

### 4.6 Favicon
`<link rel="icon" href="data:,">` is currently ABSENT. Add it after `<meta name="viewport">`, before `<title>`.

### 4.7 CDP-verifiable assertions
- `.talent-image` placeholder background === `rgb(21, 25, 34)`.
- `.talent-text` background === `rgb(17, 21, 29)`.
- `.talent-name` color === `rgb(238, 242, 248)`.
- `.social-handle`, `.separator`, `.pronouns` color === `rgb(141, 153, 173)`.
- Each inline social-icon SVG `path` attribute `fill` === `#8d99ad` (verify via `getAttribute('fill')` on the path element, or computed `fill` === `rgb(141, 153, 173)`).
- `@font-face` literal count === 5.

### 4.8 Retired-value purge map (row-casters.html)
- `#2d2d2d` → `--fgc-surface-muted` (`.talent-image` placeholder; JS fallback).
- `#fff` → `--fgc-surface-raised` (`.talent-text`).
- `#090909` → `--fgc-text` (`.talent-name`).
- `#ac46f7` → `#8d99ad` (`.social-handle`, `.separator`, `.pronouns` color; 5× inline SVG `fill="#ac46f7"`).

### 4.9 Recorded deviation — social icon fill mechanism
The target colour is met, but the mechanism differs from §4.2/§4.7. The five
social icon paths carry **no `fill` attribute**; the muted tier is applied by a
stylesheet rule instead:

```css
/* Social icons are neutral metadata, so they take the muted text tier.
   A stylesheet rule keeps them on the shared token; an SVG presentation
   attribute could not read a custom property. */
.social-icon path { fill: var(--fgc-text-muted); }
```

Reason: a literal `fill="#8d99ad"` on each inline path would hard-code the
value instead of tracking `--fgc-text-muted`, and the spec's own justification
required the shared token. The resolved result is identical.

Verified in a real WebView2 build with the CDP `CSS.getComputedStyleForNode`
domain (`getComputedStyle(path).fill` is not exposed in this Chromium build):
the resolved `fill` was `rgb(141, 153, 173)` for all five icon paths, and the
rule `.social-icon path { fill: var(--fgc-text-muted); }` was present in the
stylesheet. `--fgc-text-muted` resolved to `#8d99ad` on every icon.

---

## 5. team-roster-home.html & team-roster-away.html (shared spec)

### 5.1 Purpose & wire
Renders a team roster: header (logo + name), five core player cards in a row, and substitute cards right-aligned. Two files are byte-identical except `<title>` ("Team Roster - Home" vs "Team Roster - Away") and `const TEAM_SIDE` (`'A'` vs `'B'`). Consumes `scoreboard`, `gameOverlay`, `teamRoster`; reads `teamA`/`teamB`, `teamAPlayers`/`teamBPlayers`, `teamColorsEnabled`, `teamData.color`, `teamData.logoUrl`, `teamData.logoBgColor`, `roleIcons`, `showPlayerPortraits`, player `portraitUrl`/`heroImageUrl`/`role`/`name`/`pronouns`/`position`.

### 5.2 Surface plan (identical for home and away)

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.team-header` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface (team header bar) |
| `.team-logo` | background-color `#eee` | `var(--fgc-surface-inverse)` → `rgb(238, 242, 248)` | logo plate |
| `.team-name` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (72px Black) |
| `.player-card-core` | background `#fff` | `var(--fgc-surface-raised)` → `rgb(17, 21, 29)` | card surface (core, 284×600) |
| `.player-card-core .role-badge` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface (role icon well) |
| `.player-card-core .player-name` | color `#090909` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (36px Medium) |
| `.player-card-core .player-pronouns` | color `#444444` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |
| `.player-card-sub` | background `#fff` | `var(--fgc-surface-raised)` → `rgb(17, 21, 29)` | card surface (substitute) |
| `.player-card-sub .role-badge` | background `#090909` | `var(--fgc-surface)` → `rgb(11, 13, 18)` | surface |
| `.player-card-sub .player-name` | color `#090909` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (24px Medium) |
| `.player-card-sub .player-pronouns` | color `#444444` | `var(--fgc-text-muted)` → `rgb(141, 153, 173)` | muted text |

Placeholder/empty state: no team → `teamName` = "NO TEAM" (`--fgc-text`), cards cleared. Missing logo → logo plate stays `--fgc-surface-inverse` empty; name still `--fgc-text`. Missing portrait/hero image → core card `.hero-image` = `--fgc-surface-muted` (already the existing fallback; keep, just re-tokenize to `var(--fgc-surface-muted)`).

### 5.3 State matrix
- Team colours OFF: `.team-header` = `--fgc-surface`; `.team-name` = `--fgc-text`; logo plate = `--fgc-surface-inverse`; player cards = `--fgc-surface-raised`; names = `--fgc-text`; pronouns = `--fgc-text-muted`.
- Team colours ON: `.team-header` = `teamData.color` (user asset); `.team-name` = computed contrast text (`--fgc-team-a-text`/`--fgc-team-b-text` equivalent via luminance helper); logo plate stays `--fgc-surface-inverse` (logo plates are never team-coloured); player cards stay neutral `--fgc-surface-raised` (roster cards are not team-owned surfaces; team colour appears only on the header per §4.5).
- Core vs substitute: identical surface treatment; core cards are larger (600px) and use role badges; substitutes are shorter rows. Colour tokens identical for both.
- Empty roster: no cards; "NO TEAM" in `--fgc-text`.

### 5.4 JS helpers
Contrast helper (duplicate the pilot's logic in each file since files are standalone):

```
const hex = teamData.color.replace('#', '');
const r = parseInt(hex.substr(0, 2), 16);
const g = parseInt(hex.substr(2, 2), 16);
const b = parseInt(hex.substr(4, 2), 16);
const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
const textColor = luminance > 0.5 ? '#0b0d12' : '#eef2f8';
```

New return values:
- Luminance > 0.5 → `'#0b0d12'` (was `'#090909'`).
- Luminance ≤ 0.5 → `'#eef2f8'` (was `'#ffffff'`).

Applied to `el.teamName.style.color` when team colours are ON; when OFF, `el.teamName.style.color = ''` (reverts to CSS `var(--fgc-text)`).

Placeholder colour fallbacks:
- `el.teamLogo.style.backgroundColor = teamData.logoBgColor || 'var(--fgc-surface-inverse)'` (was `|| '#eee'`) → resolves `rgb(238, 242, 248)`.
- Team-header background fallback: `el.teamHeader.style.backgroundColor = 'var(--fgc-surface)'` (was `'#090909'`) → resolves `rgb(11, 13, 18)`.
- Core-card hero-image fallback: `background-color: var(--fgc-surface-muted)` (was `'#2d2d2d'`) → resolves `rgb(21, 25, 34)`.

### 5.5 Font/face note
`@font-face` count stays exactly 5 in each file. Aliases:
- `.team-name`: `'FGC Display Black'` (900), 72px.
- `.player-card-core .player-name`: `'FGC Display Medium'` (500), 36px.
- `.player-card-core .player-pronouns`: `'FGC Display Medium'` (500), 24px.
- `.player-card-sub .player-name`: `'FGC Display Medium'` (500), 24px.
- `.player-card-sub .player-pronouns`: `'FGC Display Medium'` (500), 18px.

### 5.6 Favicon
`<link rel="icon" href="data:,">` is currently ABSENT from both files. Add it after `<meta name="viewport">`, before `<title>`, in both `team-roster-home.html` and `team-roster-away.html`.

### 5.7 CDP-verifiable assertions (apply to both home and away files)
- `.team-header` background === `rgb(11, 13, 18)` (colours OFF) or `teamData.color` (colours ON).
- `.team-logo` background === `rgb(238, 242, 248)`.
- `.team-name` color === `rgb(238, 242, 248)` (colours OFF); computed contrast (colours ON).
- `.player-card-core` / `.player-card-sub` background === `rgb(17, 21, 29)`.
- `.player-card-core .role-badge` / `.player-card-sub .role-badge` background === `rgb(11, 13, 18)`.
- `.player-name` color === `rgb(238, 242, 248)` (core and sub).
- `.player-pronouns` color === `rgb(141, 153, 173)` (core and sub).
- Core-card `.hero-image` fallback background === `rgb(21, 25, 34)`.
- `@font-face` literal count === 5 in each file.

### 5.8 Retired-value purge map (both files, identical)
- `#090909` → `--fgc-surface` / `--fgc-text` context (`.team-header`, `.role-badge`, `.player-name`; JS `'#090909'` fallback + contrast dark branch).
- `#eee` → `--fgc-surface-inverse` (`.team-logo` background-color; JS `'#eee'` fallback).
- `#fff` → `--fgc-surface-raised` / `--fgc-text` context (`.team-name`, `.player-card-core`, `.player-card-sub`; JS contrast light branch `'#ffffff'`).
- `#444444` → `--fgc-text-muted` (`.player-pronouns`).

### 5.9 Owner-requested geometry change — flexible core cards (2026-11-22)
The frozen-geometry rule above is deliberately broken here, at the owner's
request: "player cards should always fill flexibly to the container width,
instead of maintaining a fixed width (exception for substitutes)".

`.player-card-core` no longer holds `width: 284px; flex-shrink: 0`. It now uses
`flex: 1 1 0; min-width: 0`, and `.core-players` no longer centres itself, so
the cards always fill the 1500px roster width. The card width follows the
roster size: five cards are 284px, three are 486.66px and six are 233.33px.
Previously a sixth card ran 304px off the canvas.

Substitutes keep their fixed 284px width because they are a separate,
right-aligned strip. Card height (600px), padding, the role badge, the portrait
box and the text blocks are unchanged.

---

## 6. timer.html

### 6.1 Purpose & wire
Standalone clock for non-game contexts (starting soon, intermission, lower-third bugs, schedule). One element: `.fgc-timer__value`, 160px, centered or left/right aligned via `data-align`. Consumes `init` (matchTimer) and `matchTimer` messages; reads `hours`, `minutes`, `seconds`, `settings.leadingZeroes`, `settings.showHours`, `settings.alignment`. Message types `init`, `matchTimer`, `show`. This overlay is NOT part of the game overlay (§3 owner decision 3).

### 6.2 Surface plan

| Element selector | Current colour(s) | New token value | Treatment |
|---|---|---|---|
| `.fgc-timer__value` | color `#fff` | `var(--fgc-text)` → `rgb(238, 242, 248)` | text tier (160px Archivo 600) |
| `.fgc-timer__value` text-shadow | `0 2px 12px rgba(0, 0, 0, 0.65)` | KEEP unchanged | legibility shadow over transparent canvas |

No background surface (transparent canvas). No other colour-bearing rules. Alignment via `data-align` left/right/center is unchanged geometry.

### 6.3 State matrix
- Running/paused: same colour treatment; no colour change for state (state is conveyed by the digits themselves, not colour).
- Hidden: `.fgc-timer--hidden` → `display: none !important` (unchanged).
- Team colours: not applicable.

### 6.4 JS helpers
No colour helpers exist in this file; none required. No retired values in JS.

### 6.5 Font/face note
`@font-face` count stays exactly 1 (Archivo 600 only — this is the exception to the 5-face rule; `timer.html` is not in the 5-face static test list). Alias: `.fgc-timer__value` uses `'FGC Display'` (600), 160px, `font-variant-numeric: tabular-nums`. Keep `Arial` fallback in the font stack as-is (it is not a remote asset and is not a "new font").

### 6.6 Favicon
`<link rel="icon" href="data:,">` is ALREADY present (line 6). Confirm it stays; no change needed.

### 6.7 CDP-verifiable assertions
- `getComputedStyle(document.querySelector('.fgc-timer__value')).color === 'rgb(238, 242, 248)'`.
- `.fgc-timer__value` `font-size === '160px'`, `font-weight === '600'`, `font-family` contains `FGC Display`, `font-variant-numeric === 'tabular-nums'`.
- `.fgc-timer__value` `text-shadow === '0px 2px 12px rgba(0, 0, 0, 0.65)'` (unchanged).
- `html`/`body` `background-color === 'rgba(0, 0, 0, 0)'` (transparent).
- `@font-face` literal count === 1.

### 6.8 Retired-value purge map (timer.html)
- `#fff` → `--fgc-text` (`.fgc-timer__value` color).
- `rgba(0, 0, 0, 0.65)` is NOT retired (kept per the timer special note).

---

## 7. Cross-file rules & geometry proposals

### Geometry proposals (owner approval required)
None. This spec changes surface, colour, text tier and state treatment only; no box, padding, font-size, gap, radius, or position is moved in any file. If the owner later wants layout changes (e.g., adding `font-variant-numeric: tabular-nums` to `.map-card__score` in map-select, noted in §3.5 as a text-treatment clarification), they must be proposed here and approved before implementation.

### Favicon summary
- Added to: `hero-bans.html`, `map-pool.html`, `map-select.html`, `row-casters.html`, `team-roster-home.html`, `team-roster-away.html`.
- Already present (unchanged): `timer.html`.

### @font-face count summary
- 5 faces (unchanged): `hero-bans.html`, `map-pool.html`, `map-select.html`, `row-casters.html`, `team-roster-home.html`, `team-roster-away.html`.
- 1 face (unchanged): `timer.html`.

### Retired values — final check
After the refresh, none of the following may appear in any in-scope file (CSS, inline SVG `fill`, or JS string literal): `#fff`, `#ffffff`, `#090909`, `#151515`, `#eeeeee`, `#eee`, `#ac46f7`, `#444444`, `#2d2d2d`, `#1a1a1a`, `#666`, `rgba(172, 70, 247, ...)`, `rgba(9, 9, 9, 0.6)`, `rgba(0, 0, 0, 0.3)`. Exception: `rgba(0, 0, 0, 0.65)` remains only in `timer.html`.

## 8. game-overlay.html (owner review, 2026-11-22)

The owner reviewed the game overlay and asked for four changes. This section
records them; it is the first spec section for `game-overlay.html`, because the
file was already in the Night Broadcast token style before the review.

### 8.1 Outline rings removed

The three hairline rings are gone:

| Selector | Before | After |
|---|---|---|
| `.team-bar` | `box-shadow: inset 0 0 0 1px var(--fgc-line);` | `box-shadow: 0 6px 22px rgb(0 0 0 / 38%);` |
| `.top-bar` | `box-shadow: inset 0 0 0 1px var(--fgc-line);` | `box-shadow: 0 4px 16px rgb(0 0 0 / 32%);` |
| `.match-bug` | `box-shadow: inset 0 0 0 1px var(--fgc-line);` | `box-shadow: 0 4px 16px rgb(0 0 0 / 32%);` |

### 8.2 Bar order and alignment

The DOM order is unchanged; the visual order comes from the CSS `order`
property, so a third-party override of the DOM still works.

| Bar | Visual order |
|---|---|
| Home | Logo 1, Name 2, Ban 3, Score 4, Side 5 |
| Away | Side 1, Score 2, Ban 3, Name 4, Logo 5 |
| Home, swapped | the away order |
| Away, swapped | the home order |

The team logo therefore sits on the outer end of each bar, the ban block sits
next to the score, and the name hugs the logo. The name alignment flipped with
the order: home is `justify-content: flex-start; text-align: left`, away is
`justify-content: flex-end; text-align: right`, and the `.swapped` variants
invert those.

### 8.3 Team abbreviations

A new persisted display setting, `settings.display.teamAbbreviations`,
defaults to `false`. It reaches the overlay as `teamAbbreviations` in the
scoreboard payload, next to `teamColorsEnabled`. The overlay keeps the full
name when the field is absent or `false`, and shows the team's short `tag`
when it is `true`. The switch lives in Match Controller → Config → Overlay
appearance.

### 8.4 Gradients and background textures

The file never had gradients or background textures, and the house style forbids
"no full opaque panels, no large gradients, no centre-frame decoration" and
"no heavy gradients, no oversized glows". The review therefore resolved to
keeping the panels flat and opaque, with the removed rings replaced by a soft
drop shadow. No gradient or texture was added.

### 8.5 CDP-verifiable assertions

- No `inset 0 0 0 1px var(--fgc-line)` remains in the file.
- The bars compute to `rgba(0, 0, 0, 0.38) 0px 6px 22px 0px` and the top bar
  and bug to `rgba(0, 0, 0, 0.32) 0px 4px 16px 0px`.
- Sorting each bar's visible children by x yields the order in §8.2.
- The logo is the outermost block on both bars.
- The ban block is directly adjacent to the score on both bars.
- The name's computed `text-align` is `left` on the home bar and `right` on the
  away bar.
- With `teamAbbreviations` on, the rendered name equals the team's `tag`.

## 9. Theme system (owner request, 2026-11-22)

### 9.1 Model

A theme is a set of CSS custom properties, not a stylesheet. The overlay
server only serves `/overlays/{file}`, `/assets/fonts/{name}` and image
assets, so an imported `.css` could never reach a page. Instead the app sends
a `theme` message:

```json
{ "type": "theme", "data": { "preset": "light", "tokens": { "--fgc-surface": "#f4f6fb" } } }
```

`tokens` is the **complete** set of properties to override. An overlay keeps
the list of properties it applied and removes any that the next theme does not
list, so a theme with no tokens restores the stylesheet defaults.

### 9.2 Where the values live

| Set | Lives in | Notes |
|---|---|---|
| Night Broadcast | Each overlay's own `:root` block | The single source of truth for the dark theme; an overlay with no `theme` message still renders correctly |
| Light | `fgc_protocol::LIGHT_THEME_TOKENS` | 18 colour overrides; spacing, radii and durations are never themed |
| Imported | `settings.display.themeTokens` | Layered over the selected preset |

### 9.3 Applied tokens

`--fgc-surface`, `--fgc-surface-raised`, `--fgc-surface-inverse`,
`--fgc-surface-muted`, `--fgc-line`, `--fgc-line-strong`,
`--fgc-line-on-inverse`, `--fgc-text`, `--fgc-text-muted`, `--fgc-text-faint`,
`--fgc-text-inverse`, `--fgc-accent`, `--fgc-accent-strong`,
`--fgc-accent-soft`, `--fgc-on-accent`, `--fgc-success`, `--fgc-warning` and
`--fgc-danger`.

A theme may only override names that the overlays declare. The static test
`the_light_theme_only_names_tokens_the_overlays_declare` guards that, so a new
token cannot drift away from the stylesheets.

### 9.4 Overlay contract

Every bundled overlay carries the same helper, before its message handler:

```js
let appliedThemeTokens = [];

function applyTheme(payload) {
  const tokens = payload && typeof payload === 'object' && payload.tokens && typeof payload.tokens === 'object' ? payload.tokens : {};
  const root = document.documentElement;
  appliedThemeTokens.forEach((name) => {
    if (!Object.prototype.hasOwnProperty.call(tokens, name)) root.style.removeProperty(name);
  });
  appliedThemeTokens = Object.keys(tokens);
  appliedThemeTokens.forEach((name) => {
    root.style.setProperty(name, String(tokens[name]));
  });
}
```

The message handler applies `data.theme` for a `theme` frame and
`data.theme` for an `init` frame, then returns.

### 9.5 CDP-verifiable assertions

- The document element carries the theme's custom properties as inline styles
  while a theme is active, and carries none after switching back.
- A themed element's computed colour matches the theme's value, for example
  `rgb(244, 246, 251)` for `--fgc-surface` in the Light theme.
- `--fgc-surface-inverse` inverts between the themes, which proves the panels
  were repainted rather than merely restyled.
- The `theme` key is present in `init` and in `/state`.
- The saved `app.json` carries `settings.display.theme`.

## 10. Type scale (owner review, 2026-11-22)

The last item of the owner's overlay review was a font-weight pass. The scope
chosen was a consistency pass: one type scale, every overlay aligned to it,
and the genuine mismatches fixed. Sizes were not changed.

### 10.1 The rule

**The alias family carries the weight, and a rule never declares a
`font-weight` of its own.**

| Tier | Weight | Alias |
|---|---|---|
| Display | 900 | `FGC Display Black` |
| Strong | 600 | `FGC Display SemiBold` |
| Body | 500 | `FGC Display Medium` |
| Secondary | 400 | `FGC Display` |
| Quiet | 300 | `FGC Display Light` |

### 10.2 Roles

| Role | Tier | Where |
|---|---|---|
| The biggest headline on a screen | Display | map pool header, map select title, roster team name, hero bans hero name |
| A score, a count, a map name | Strong | game overlay bar score and title, map select name, score and score divider, map pool map name, timer value |
| A person's or team's name, any label | Body | game overlay bar name and logo placeholder, subtitle, format label and map number, hero bans ban order, map select type label, roster and talent names, social roles, watermark text |
| Small supporting text | Secondary | map select chosen-by label |
| Handles, pronouns, separators | Quiet | social handle, separator, pronouns, roster pronouns |

### 10.3 Why the rule exists

A rule that names the `FGC Display` family and asks for weight 500 gets
nothing: the family only ships the 400 face, so the weight is dropped and the
text renders at Regular. That was true of the game overlay's bar name, bar
score, title, subtitle, map number and format label, all of which inherited
`body { font-family: 'FGC Display' }`. The timer overlay had the same problem
inverted: its `@font-face` declared the regular alias at weight 600 and so
hijacked the alias every other overlay used for the 400 face. It now declares
`'FGC Display SemiBold'`.

### 10.4 The guards

Four static tests in `crates/server/tests/overlays.rs` keep the scale honest:

- `every_overlay_uses_the_shared_type_scale` - no `font-weight` outside an
  `@font-face`, every declared face is a tier and loads the matching Archivo
  file, and every family a rule names is a tier the file declares.
  `auto.html` is special-cased: it declares no face of its own.
- `the_match_overlays_all_ship_the_five_display_faces` - the nine match
  overlays declare all five faces. `auto.html` and `timer.html` are excluded.
- `the_timer_overlay_uses_the_semi_bold_alias` - the timer loads the SemiBold
  face under the shared alias.
- `the_type_scale_gives_each_role_one_weight` - the drift guard: scores are
  SemiBold in map select and the game overlay; map names are SemiBold in map
  pool and map select; the mode label is Medium in map pool and map select;
  pronouns are Light in the two roster files and the three talent rows; names
  are Medium in the game overlay and the roster; the hero bans hero name is
  Display in the hero bans overlay.

### 10.5 CDP-verifiable assertions

- `CSS.getPlatformFontsForNode` reports the platform font a node painted with.
  WebView2 reports the font file's family and weight, so all five bundled
  faces arrive as `Archivo SemiBold` with a weight suffix: `Light` is 300,
  `Medium` is 500, `SemiBold` is 600 and `Black` is 900.
- Every text node paints the face its tier calls for.
- Every computed `font-weight` is `400` normal.

## 11. background.html (owner request, 2026-11-23)

The owner asked for an animated background in HTML and CSS: "maybe you can help
me make an animated background using html and css? But it would probably be a
separate overlay as well as built into the existing ones." It ships first as a
standalone **persistent Browser Source** that sits beneath the scene overlays.
The nine scene overlays replace one another; the background stays on screen for
the whole broadcast, so it is a source of its own and is deliberately absent
from the auto scene switcher and its number hotkeys.

The page paints no text and loads no fonts, and it is asset-free: the only
network requests are the document, the WebSocket and the inline SVG mask data
URL. It is **not** a transparent wash. `html`, `body` and `.background` all
paint a solid, opaque plate from `var(--fgc-surface)`, so the overlay is a real
backdrop and a `theme` message repaints the whole frame.

### 11.1 Purpose & wire

A 1920x1080 opaque plate built from a soft ambient glow and two drifting copies
of the product's hexagon lattice. It is the backdrop beneath everything else
rather than a scene in its own right, and it paints the active theme's surface
colour, so it reads as part of the broadcast rather than a wash laid over it.

It carries the same WebSocket client as the other overlays and consumes
`init`, `theme`, `scoreboard`, `gameOverlay`, `heroBans` and `show`:

- `theme` (and an `init` carrying `theme`) runs `applyTheme()`, which applies
  the token set and removes any property the new set omits. Because the plate
  is painted from `--fgc-surface`, a theme message repaints the whole backdrop,
  not just the washes.
- `scoreboard`, `gameOverlay`, `heroBans` and an `init` carrying `scoreboard`
  feed `updateTeamColour()`, which writes `teamA.color` and `teamB.color` to
  `--team-a-color` / `--team-b-color` on `#background`, and removes them when a
  colour is absent so a later match cannot inherit an earlier wash.
- `show` toggles `.background` between `block` and `none` and toggles
  `background-hidden` on `<html>`. The rule turns the plate transparent along
  with the artwork, so a hidden source leaves no solid colour behind.
- The socket target is `location.host`, with `?host=` and `?port=` overrides,
  matching the rest of the overlays.

### 11.2 Surface plan

| Element selector | Token value | Treatment |
|---|---|---|
| `html`, `body`, `.background` | `background: var(--fgc-surface)` on the root and body, `background-color: var(--fgc-surface)` on `.background` | the solid plate: Night Broadcast `#0b0d12` → `rgb(11, 13, 18)`, Light `#f4f6fb` → `rgb(244, 246, 251)` |
| `.background` | 1920x1080, `pointer-events: none`, `aria-hidden="true"` | the frame over the plate |
| `.background__glow` | `inset: -12%`, two radial gradients at `4% 54%` and `96% 38%`, `--background-a` at 14% and `--background-b` at 12% | the ambient wash |
| `.background__lattice--ambient` | `background-color: var(--fgc-line-strong)`, `opacity: 0.13`, `z-index: 1` | the quiet honeycomb |
| `.background__lattice--lit` | two radial gradients weighted to `-6% 52%` and `106% 38%`, `opacity: 0.68`, `z-index: 2` | the lit honeycomb |
| `.background__lattice` | the shared hexagon SVG `mask-image`, `mask-size: 180px 322px`, `inset: -180px -220px` | the drifting layer, overscanned so no edge is ever exposed |
| `html.background-hidden`, `html.background-hidden body` | `background: transparent` | the hide rule, so a hidden source takes the plate with it |

The layer aliases are `--background-a: var(--fgc-text-muted)` (`#8d99ad`) and
`--background-b: var(--fgc-line-strong)` (`#3a4356`). Under `.tone-team` they
become `var(--team-a-color, var(--fgc-text-muted))` and
`var(--team-b-color, var(--fgc-line-strong))`, so a missing team state resolves
to the same quiet greys. The aliases live on `.background`, which keeps team
colours from colliding with the root theme overrides.

The lit lattice is weighted to the outer edges, which leaves the centre of the
frame comparatively plain for a game feed, a desktop or another graphic.

### 11.3 Motion design

Three CSS animations, all `ease-in-out` and `infinite`, and all on the
compositor:

| Animation | Layer | Duration | What changes |
|---|---|---|---|
| `glowBreathe` | `.background__glow` | 84s | `opacity` 0.46 to 0.72 and `scale(1)` to `scale(1.045)` |
| `latticeDriftForward` | `.background__lattice--ambient` | 88s | `translate3d(-48px, -28px, 0)` to `translate3d(48px, 28px, 0)` and back |
| `latticeDriftReverse` | `.background__lattice--lit` | 72s | `translate3d(64px, -36px, 0)` to `translate3d(-64px, 36px, 0)` and back |

Only `transform` and `opacity` appear in the keyframes; nothing paint-heavy is
animated. The file runs no `requestAnimationFrame` and no per-frame JavaScript.
Both drift paths are closed, so they reverse without a loop jump, and the
stylesheet notes that even the fastest movement stays below 0.08 px/frame at
60 fps. The two lattices start at the 0% keyframe values, which is why a fresh
load measures `matrix(1, 0, 0, 1, -48, -28)` for the ambient layer and
`matrix(1, 0, 0, 1, 64, -36)` for the lit layer.

### 11.4 URL variants and reduced motion

OBS preserves the query string on a Browser Source URL, so the variants ship as
parameters instead of extra bundled pages:

- `?motion=still` adds `motion-still` to `<html>`. `.motion-still` sets
  `animation: none` on the glow and both lattices, leaving a composed first
  frame. Any other `motion` value, and no value at all, leaves the drift
  running; `?motion=drift` is not a separate mode.
- `?tone=team` adds `tone-team` to `<html>`, switching the two aliases to the
  live team colours. Without it the background stays neutral, whatever the
  payload says.
- `@media (prefers-reduced-motion: reduce)` sets `animation: none` on both
  layers, independent of the query string, so the OS accessibility preference
  wins.
- `--fgc-surface` owns the plate. The inlined Night Broadcast block sets
  `#0b0d12`, and the Light theme's `LIGHT_THEME_TOKENS` sets `#f4f6fb`, so the
  backdrop follows the active theme. `?tone=team` recolours only the two washes,
  never the plate.
- `?host=` and `?port=` retarget the WebSocket for third-party pages.

The query string is read once at load, so changing a variant needs a reload.

### 11.5 CDP-verifiable assertions

- `document.title` is `Animated Background` and
  `getComputedStyle(document.body).backgroundColor` is `rgb(11, 13, 18)`.
- `.background` measures 1920x1080 and carries `aria-hidden="true"`.
- The computed `background-color` of `html`, `body` and `.background` is
  `rgb(11, 13, 18)` under Night Broadcast, and all three are opaque.
- A `theme` message carrying `--fgc-surface: #f4f6fb` repaints `html`, `body`
  and `.background` to `rgb(244, 246, 251)`; removing the override restores
  `rgb(11, 13, 18)`.
- Exactly one `.background__glow`, one `.background__lattice--ambient` and one
  `.background__lattice--lit` exist.
- `getComputedStyle` reports `opacity: 0.13` on the ambient lattice and `0.68`
  on the lit lattice, and the glow starts at `0.46`.
- On load the ambient lattice computes `transform: matrix(1, 0, 0, 1, -48, -28)`
  and the lit lattice `matrix(1, 0, 0, 1, 64, -36)`.
- `getAnimations()` reports `glowBreathe` at 84000ms, `latticeDriftForward` at
  88000ms and `latticeDriftReverse` at 72000ms. `getTiming().iterations` reads
  `null`, which is the JSON form of `Infinity` and therefore an infinite loop,
  and `getTiming().easing` reads `linear`, because the `ease-in-out` easing is
  declared in the animation shorthand and lands on the keyframes rather than on
  the effect timing.
- Both lattices compute `mask-size: 180px 322px`.
- With `?motion=still`, `<html>` carries `motion-still` and both layers compute
  `animation-name: none`.
- With `?tone=team`, `<html>` carries `tone-team`, the layer aliases resolve to
  the two team colours and the plate stays `rgb(11, 13, 18)`; without it they
  resolve to `#8d99ad` and `#3a4356`.
- With a `show` message of `visible: false`, `<html>` carries
  `background-hidden`, `.background` computes `display: none` and the body
  computes `rgba(0, 0, 0, 0)`; a `visible: true` message restores all three.
- With emulated reduced motion, both layers compute `animation-name: none`.
- The page loads no font, image or external file: the requests are the
  document, the WebSocket and the inline SVG mask data URL, and the console is
  empty.

## 12. Canonical FX block and the built-in lattice (owner decision, 2026-11-23)

The owner asked for the animated background as a separate overlay "as well as
built into the existing ones", then narrowed the built-in part on 2026-11-23:
"Note: there should be no background overlay attached to the timer or the game
overlay". The built-in treatment is therefore a **transparent lattice layer**,
not the opaque plate from §11. The plate belongs only to the standalone
`background.html`, which sits below the scenes in OBS. This section covers the
transparent layer that can sit over a scene without changing its geometry,
colours or text.

The layer ships in eight scene overlays: `hero-bans.html`, `map-pool.html`,
`map-select.html`, `row-casters.html`, `row-hosts.html`, `row-interviews.html`,
`team-roster-home.html` and `team-roster-away.html`. It does not ship in
`timer.html` or `game-overlay.html`, by the owner's decision. It also does not
ship in `auto.html`, whose previews will load `?fx=off`, or in
`background.html`, which already has its own richer three-layer motion.

### 12.1 Purpose & wire

Each carrier gets one `div.fgc-fx[aria-hidden="true"]` as the first child of
`<body>`, before its existing root. The layer adds no message handling, no
network request and no state of its own. A short inline script reads the `fx`
parameter once at load and writes `data-fx` to `<html>`. The CSS then hides or
shows the layer through that attribute.

Server injection of the document-level `data-fx` attribute remains **planned,
not shipped**. The reserved hook is still there: when the server eventually
injects `data-fx="off"`, the layer will default off unless an explicit
`?fx=on` in the Browser Source URL overrides it. The application now has a UI
control for the separate background mode described in §13, and that control now
broadcasts a cached `background` section to connected sources as well as
rewriting the generated Browser Source URLs. It changes only `data-bg`; it does
not inject `data-fx` into a connected page.

### 12.2 Surface plan

| Element selector | Token value | Treatment |
|---|---|---|
| `:root` | `--fgc-hex-mask` holds the shared hexagon SVG data URL | the one source of truth for the mask |
| `.fgc-fx` | `position: absolute`, `inset: -360px`, `z-index: 0`, `pointer-events: none` | the overscanned, non-interactive layer behind the content |
| `.fgc-fx` | `background-color: var(--fgc-line-strong)`, `opacity: 0.16` | a uniform wash, so a repeat cannot reveal a moving gradient |
| `.fgc-fx` | `-webkit-mask-image` / `mask-image`, `mask-size: 180px 322px`, `mask-repeat: repeat`, `mask-position: center` | the shared hexagon lattice at the product's tile size |
| `.fgc-fx` | `transform: translate3d(0, -322px, 0)`, `animation: fgcCanonicalFxDrift 72s linear infinite`, `will-change: transform` | the only animated layer, moved on the compositor |
| `.overlay` | `z-index: 1` | lifts the existing content above the new sibling with no geometry change |
| `html[data-fx="off"] .fgc-fx` | `display: none` | the explicit off state |
| `@media (prefers-reduced-motion: reduce)` | `animation: none` on `.fgc-fx` | stops motion but keeps the static lattice |

The SVG namespace inside the data URL is escaped as
`http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg`. `crates/server/tests/timer_overlay.rs`
forbids the literal `http://` or `https://` in an overlay; the escaped form is a
normal data URL and renders the same mask. Only the canonical block uses the
escaped form; existing inline `<svg>` markup outside the sentinels is untouched.

The block is delimited by `<!-- === FGC CANONICAL FX START === -->` and
`<!-- === FGC CANONICAL FX END === -->`. The comments are part of the contract:
the Rust test extracts exactly what lies between them.

### 12.3 Motion design and the seam proof

The keyframes travel exactly one mask tile:

```css
@keyframes fgcCanonicalFxDrift {
  from { transform: translate3d(0, -322px, 0); }
  to { transform: translate3d(180px, 0, 0); }
}
```

The difference between the two states is `(180px, 322px)`, precisely the
`180px 322px` mask repeat unit. When the animation loops back to its start,
each hexagon occupies the previous position of its neighbour, so the frame is
identical and there is no jump. The `inset: -360px` overscan is larger than one
tile on every side, so the moving layer's edge never enters the 1920x1080
viewport. Overscan hides an edge; exact tile travel is what makes the loop
seamless.

Only `transform` and `opacity` are used. There is no `background-position`,
`filter`, `width`, `height`, `box-shadow`, `requestAnimationFrame` or per-frame
JavaScript. `will-change: transform` is set on `.fgc-fx` alone. The 72s cycle is
inside the council's 40s to 90s band and is deliberately slower than a viewer
can track.

### 12.4 URL variants and scope

- `?fx=off` sets `document.documentElement.dataset.fx = 'off'`; the computed
  `display` of `.fgc-fx` becomes `none` and no animation is reported.
- `?fx=on` forces the layer on even if the document element was already marked
  off.
- With no `fx` parameter, the layer is on unless the document element already
  carries `data-fx="off"`. That is the app-level hook.
- `@media (prefers-reduced-motion: reduce)` sets `animation: none`. The layer is
  still painted and the static lattice remains, so the composition does not
  collapse.
- The `fx` query string is read once at load, so an OBS source refresh is needed
  after changing it. The separate `bg` mode is seeded from the URL in the same
  way, but a cached `background` section can now change `data-bg` on a connected
  page without a refresh; see §13.

`timer.html` and `game-overlay.html` carry no sentinel block, no `.fgc-fx`
element and no `fx` script. Their bodies stay transparent and their existing
roots are unchanged. A future change that adds the block to either file
reverses an explicit owner decision and must be recorded in `docs/decisions.md`
first.

### 12.5 Byte-equality assertion

`crates/server/tests/overlays.rs` carries
`every_overlay_carries_the_same_fx_block`. It reads the eight carrier files,
extracts the text between the two sentinels and compares every extraction with
the first one. It asserts that the array has eight entries, so excluding a
carrier is a deliberate edit.

The test also guards the mechanism rather than only the equality: the shared
mask token, `mask-size: 180px 322px`, `inset: -360px`, `will-change: transform`,
the absence of `background-position` and `requestAnimationFrame`, the
`html[data-fx="off"] .fgc-fx` rule, the two accepted `fx` parameter values, the
reduced-motion media query and its `animation: none`. A copy that drifts from
the canonical block fails even if all copies drift together.

This is a source-text test and is known to be fragile against unrelated markup
edits. That is the accepted cost of inlining a shared layer without a bundler.
When it breaks for an unrelated reason, narrow the assertion; do not remove the
byte-equality check.

### 12.6 CDP-verifiable assertions

- On `hero-bans.html?fx=on`, exactly one `.fgc-fx` exists and computes
  `display: block`, `position: absolute`, `z-index: 0`, `pointer-events: none`
  and an inset of `-360px` on all four sides.
- Its computed `mask-size` is `180px 322px`, `mask-repeat` is `repeat`,
  `background-color` is `rgb(58, 67, 86)` (`--fgc-line-strong`) and `opacity` is
  `0.16`.
- `getAnimations()` reports one `fgcCanonicalFxDrift` at 72000ms. As with §11.5,
  `getTiming().iterations` reads `null` in JSON, which is the infinite loop, and
  `getTiming().easing` reads `linear` for the `linear` shorthand used here.
- `document.documentElement.dataset.fx` reads `on` with `?fx=on`, and the existing
  `.overlay` root computes `z-index: 1`.
- With `?fx=off`, `dataset.fx` reads `off`, `.fgc-fx` computes `display: none`
  and `getAnimations()` is empty for that element.
- With no parameter, `dataset.fx` reads `on` unless the document element was
  seeded with `data-fx="off"`.
- `matchMedia('(prefers-reduced-motion: reduce)').matches` is false in the normal
  test browser; the reduced-motion branch is guarded by the source-text test,
  not claimed as browser-emulated for A74.
- The overlay body stays `rgba(0, 0, 0, 0)` and the document stays 1920x1080.
  No external request, failed response or console error is introduced.
- `timer.html` and `game-overlay.html` contain neither the sentinel comments nor
  `.fgc-fx`, and their bodies remain transparent.

A74 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) records the browser evidence
and the explicit scope of what was verified.

## 13. Background mode toggle (2026-11-23)

### 13.1 Purpose & wire

The owner observed that only `background.html` painted an opaque plate and
asked for a toggle that turns background transparency on or off, "including
removing the hexagons". A75 implemented the first reading literally: opaque
hid the lattice. The owner then clarified the intended identity rule:
opaque mode must still show the hexagons. A77 supersedes A75 and A76 on this
point. The two modes are now explicit:

- **Transparent** is the default for the eight scene overlays that carry the
  canonical FX block. Their bodies stay `rgba(0, 0, 0, 0)` and the shared
  hexagon lattice is visible.
- **Opaque** paints a solid plate from `var(--fgc-surface)` and keeps the same
  drifting hexagon lattice on top of it. It is the same theme colour used by
  the standalone background, but it is a sibling layer inside each scene
  overlay rather than a second Browser Source. The plate sits at `z-index: 0`
  and the lattice layer remains above it.
- `background.html` keeps opaque as its own default. Its `?bg=transparent`
  variant removes the plate while leaving the glow and both drifting lattices
  in place, so a user can preview the composite either way.

`BACKGROUND_MODE_FILES` in `crates/app/src/server.rs` names the ten files that
understand the mode: `auto.html`, `background.html` and the eight FX carriers.
`overlay_url_with_background(base_url, file, opaque)` appends `?bg=opaque` or
`?bg=transparent` only to those ten. `timer.html` and `game-overlay.html` are
deliberately absent; the owner's exclusion from A74 still holds, and they
receive no background mode parameter.

The app's Overlays view holds a session-only `background_opaque` signal. The
`Background` key value reads `Opaque` or `Transparent`, and a `Switch` inside a
`ToggleRow` is labelled `Opaque background`. Its hint explains that opaque
paints a solid theme colour under the lattice and that connected Browser
Sources switch straight away. It is disabled until the overlay server is
running. Changing the switch does two things. It rewrites the Browser Source
URLs shown in the app, so a source opened later starts in the selected mode. It
also broadcasts a cached `background` section over the overlay WebSocket. Every
connected source that carries the mode handler changes `data-bg` straight away.
The cached section also arrives inside the keyed init frame, so an OBS source
refresh restores the app-selected mode instead of falling back to the URL
default. The live frame is `{"type":"background","data":{"opaque":true}}` for
opaque mode and `false` for transparent mode. The init frame is
`{"type":"init","data":{"background":{"opaque":true}}}`, or `false` for
transparent mode. A77 added the init-frame parser after the owner reported that
the toggle disappeared on refresh. The state is not persisted to `app.json` and
is not dirty state. Server injection of the separate `data-fx` hook remains
unbuilt.

`auto.html` forwards the mode from its own URL to every scene iframe. It copies
`bg` through the same normalisation as the scene overlays, so the auto Browser
Source can drive the whole scene set from one URL. Timer and game overlay
ignore the harmless parameter.

### 13.2 Surface plan

| Element selector | Token value | Treatment |
|---|---|---|
| `.fgc-bg` | `position: absolute`, `inset: 0`, `z-index: 0`, `pointer-events: none` | a full-frame sibling behind the overlay content |
| `.fgc-bg` | `display: none`, `background-color: var(--fgc-surface)` | hidden in transparent mode, painted in opaque mode |
| `html[data-bg="opaque"] .fgc-bg` | `display: block` | turns the solid plate on |
| `:root` | `--fgc-surface` | the one theme token for the plate, `#0b0d12` under Night Broadcast and `#f4f6fb` under Light |
| `html, body` | `width: 100%; height: 100%; margin: 0; overflow: hidden` | the early document guard from A78 that prevents transient scrollbars during load |
| `html[data-bg="transparent"] .background` | `background-color: transparent` | the standalone page's plate-off variant |
| `html[data-bg="transparent"], html[data-bg="transparent"] body` | `background: transparent` | removes the standalone page's document plate too |

The new selector lives inside the same sentinel comments as the canonical FX
block, so `every_overlay_carries_the_same_fx_block` now compares the plate
declarations, the remaining `data-bg` rule and the mode script as well as the
original lattice rules. The test also asserts that the sentinelled block does
not contain `html[data-bg="opaque"] .fgc-fx`, so the lattice cannot be hidden
again by a copied rule. Each carrier has exactly one `.fgc-bg` and one `.fgc-fx`
sibling before the overlay root. The sentinelled script also defines
`window.fgcUpdateBackground(data)`, which reads `payload.background.opaque`
from the init frame or `data.data.opaque` from a live frame (or a top-level
`opaque`) and writes `data-bg`; each carrier's `updateOverlay` calls that helper
for every inbound frame. `background.html` has the equivalent
`data.type === 'background' || (data.type === 'init' && d.background)` branch in
its own `updateOverlay`. A test,
`background_mode_is_absent_from_sources_without_a_background`, asserts that
`timer.html` and `game-overlay.html` contain neither `data-bg` nor `fgc-bg`.
`every_overlay_has_an_early_document_overflow_guard` asserts that all twelve
overlays declare the A78 guard before any `.fgc-fx` rule or iframe.

`background_html_supports_the_transparent_variant` checks the standalone page's
transparent rules, and `auto_overlay_forwards_the_background_mode_to_its_scenes`
checks the iframe parameter forwarding. The byte-equality extraction still
starts and ends at `<!-- === FGC CANONICAL FX START === -->` and
`<!-- === FGC CANONICAL FX END === -->`; adding the plate inside those sentinels
is what keeps the two layers from drifting apart.

### 13.3 URL variants

The inline script reads `bg` from `new URLSearchParams(location.search)` in the
eight scene carriers and in `background.html`:

- `?bg=opaque` and its alias `?bg=off` select the solid plate with the hexagons
  still visible over it.
- `?bg=transparent` and its alias `?bg=on` select the transparent lattice.
- With no `bg` parameter, a scene overlay stays transparent unless its document
  element was already seeded with `data-bg="opaque"`.
- `background.html` defaults to opaque unless a parameter says otherwise.

The `fx` parameter is independent. Opaque paints the plate but does not rewrite
or clear `data-fx`. The query string is read once when the document loads and
seeds the initial mode, but the app broadcasts a `background` section when the
operator changes the switch. The handler in every modes-aware overlay overwrites
`data-bg` on that message, so connected sources change without an OBS refresh.
A newly opened source starts from its URL and then receives the cached section
inside the init frame, which keeps it aligned with the app even after a Browser
Source refresh. A77 fixed the handler, which previously only recognised the
live `background` message and therefore let a refreshed source fall back to its
URL default. §12.1 still reserves `data-fx` as a separate, unbuilt injection
hook.

### 13.4 CDP-verifiable assertions

On `http://127.0.0.1:3001/overlays/hero-bans.html?bg=opaque`, measured in a real
Chromium tab:

- `document.documentElement.dataset.bg` reads `opaque` and `dataset.fx` still
  reads `on`.
- One `.fgc-bg` computes `display: block`, `background-color: rgb(11, 13, 18)`
  under Night Broadcast and `inset: 0px`.
- The single `.fgc-fx` computes `display: block` with
  `mask-size: 180px 322px` and `opacity: 0.16`; `getAnimations()` reports
  `fgcCanonicalFxDrift`. A77 reversed the A75 assertion that the layer was
  hidden in opaque mode.
- `document.body` computes `rgba(0, 0, 0, 0)`; the plate is the sibling layer,
  not a body background. The existing `.overlay` still computes `z-index: 1`.
- The early A78 guard computes `overflow: hidden` on both the document element
  and the body. The overscanned FX layer reports `scrollWidth: 1360` and
  `scrollHeight: 738` in a 1000x700 tab, but the scrollbar gutters are zero.
- `matchMedia('(prefers-reduced-motion: reduce)').matches` is false in the test
  browser. Reduced motion is covered by the unchanged source-text assertion,
  not claimed as browser-emulated.
- The console is empty.

On the same page with `?bg=transparent`:

- `dataset.bg` reads `transparent`.
- `.fgc-bg` computes `display: none`, even though its theme colour remains
  `rgb(11, 13, 18)`.
- `.fgc-fx` computes `display: block`, and `getAnimations()` reports one
  `fgcCanonicalFxDrift` at 72000ms.
- The body stays transparent and the console stays empty.

On `http://127.0.0.1:3001/overlays/background.html?bg=transparent`:

- `dataset.bg` reads `transparent`; the document element, body and `.background`
  all compute `rgba(0, 0, 0, 0)`.
- Two `.background__lattice` layers remain, and `getAnimations()` reports
  `glowBreathe`, `latticeDriftForward` and `latticeDriftReverse`.
- The console is empty.

The live message path was checked separately in a connected tab with no URL
parameter. Before the message, `hero-bans.html` read `bgMode: 'transparent'`,
showed one `.fgc-fx` with a drift animation and kept `.fgc-bg` hidden. Calling
`window.updateOverlay({type:'background', data:{opaque:true}})` changed
`dataset.bg` to `opaque` and turned the plate on in the same document: `.fgc-bg`
became `display: block` with `backgroundColor: rgb(11, 13, 18)`, while `.fgc-fx`
stayed `display: block` with its drift animation. A76 recorded the older
hidden-layer result; A77 supersedes it. Sending `opaque: false` hid the plate
and left the lattice running. The `background.html` tab showed the inverse
mapping from its opaque default: `opaque: false` made the document, body and
plate transparent while all three animations kept running, and `opaque: true`
restored `rgb(11, 13, 18)`. The app's `Switch` was then clicked through CDP: it
exposed `role="switch"`, `aria-label="Opaque background"`, `aria-checked` and
`data-state`, and the previously connected hero-bans tab moved to opaque mode
without a refresh. All four checks had empty consoles.

A77 added the refresh proof. With the cached state at
`background: {"opaque":true}`, a fresh `hero-bans.html?probe=1795470496591` read
`bgMode: opaque`, `.fgc-bg display: block` at `rgb(11, 13, 18)`, and `.fgc-fx
display: block` with `mask-size: 180px 322px`, `opacity: 0.16` and
`fgcCanonicalFxDrift`. Reloading the same tab kept all four values. A fresh and
reloaded `background.html` tab both read `bgMode: opaque` with the
`rgb(11, 13, 18)` plate and the same empty console. An earlier switch-off pass
in the previous app instance had proved the same persistence in the other
direction: fresh hero-bans and background tabs after switching to transparent
both stayed transparent after reload, with the FX lattice still visible on
hero-bans. The A78 guard probe covered hero-bans, background, map-pool,
game-overlay and auto: every fresh and reloaded tab reported the guard,
`overflow: hidden` on html and body, empty console and a zero scrollbar gutter.
The FX carriers reported `scrollWidth: 1360` and `scrollHeight: 738` because of
the `-360px` overscan, but the clipped document has no visible scrollbars.

Both browser tabs measured a 1000x700 viewport. That is the Review-pane tab
size, not the overlay's 1920x1080 design canvas. The computed-style and
animation assertions do not depend on the viewport size; a visual sign-off in
OBS at 1920x1080 remains an outstanding owner check.

A75 and A76 in [`docs/manual-qa.md`](manual-qa.md) and
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md) remain the historical
records for the URL-mode values, the live message and Switch proof, with their
1000x700 caveat and source-text coverage. A77 supersedes both on the lattice
question and adds refresh persistence. A78 records the early document overflow
guard and the zero-gutter browser proof. Visual sign-off at 1920x1080 in OBS
and browser-emulated reduced motion remain outstanding owner checks.

## 14. Roster overlays - wash, role icon and edge rails (A79)

### 14.1 Purpose and scope

A79 refines the paired `team-roster-home.html` and `team-roster-away.html`
overlays. It keeps the Night Broadcast tokens, the 1500px flexible card row
from A61, the rectangular substitute strip, and the hexagon separator outside
the rows. It does not change the wire protocol, filenames, fonts, assets, FX
block, background-mode handling or early overflow guard. The two files remain
byte-identical apart from `<title>` and `TEAM_SIDE`. The owner first asked for a
wider gradient, a subtle bottom glow, role icons at the bottom centre of the
portrait overlapping a taller name plate, smaller pronouns, and an experiment
with a country flag or country code. After the first implementation the owner
reversed the last point and the overlap: "Scrap the idea about the country flag
or tag for now. We'll revisit this later in the backlog. Maybe for the core
team, let's try reduceing the size of the role icon and move it up more so it
doesn't overlap over the name plate like we originally tried. On top of that,
let's add some decorative lines on the left and right side (similar design to
the divider between core and subs)." The experiment and its deferred status are
recorded in `docs/backlog.md`.

### 14.2 Surface plan

The `.roster-wash` is 760px wide and spans the full 1080px height. It no
longer has element-level opacity; its alpha lives in the token colours.
`.roster-wash::after` adds a 320px bottom glow at `bottom: -120px` with
`filter: blur(14px)`. Home anchors the wash at `left: 0`, paints
`linear-gradient(90deg, var(--fgc-wash), transparent 82%)` and puts the glow at
`22% 100%`. Away anchors at `right: 0`, mirrors the angle to 270deg and the
glow centre to `78% 100%`. The default custom properties are
`--fgc-wash: rgb(255 122 26 / 16%)` and
`--fgc-glow: rgb(255 122 26 / 20%)`.

The core row is wrapped in `.core-stage`, a 1500×600 positioned box. The
`.core-players` child fills that box and keeps the 20px gap and flexible card
widths. Two `.core-rail` siblings sit at `left: -92px` and `right: -92px`,
each 20px wide and 520px tall with `top: 40px` and `bottom: 40px`. Their
`::before` is a 1px `var(--fgc-line)` rule; their `.core-rail__hex` is an
18×20 `var(--fgc-line-strong)` hexagon using `--fgc-hex-cell`. This repeats
the `.roster-separator` construction as a vertical motif without changing the
row geometry.

The core role badge lives inside `.hero-image` and is 56×56 at
`left: 50%`, `bottom: 16px`, `transform: translateX(-50%)`. It uses
`--fgc-surface`, the shared hexagon clip, `z-index: 3`, and a centred flex
layout. Its image is 40×40 and `:empty` hides the badge when no icon exists.
The hero remains 740×474 and `position: relative`. With a 126px plate below,
the badge bottom sits 16px above the plate top. The earlier 76px badge at
`bottom: -36px` is forbidden; source-text assertions guard both numbers.

The substitute badge is unchanged by the reversal: it stays in the 72×76
`.sub-portrait` at 58×58, `bottom: 8px`, with a 44×44 image. The substitute
card, row and plate are 76px high. Core `.player-info` is 126px high with
`padding: 20px 24px` and `justify-content: center`. Core pronouns are 18px
with 22px line height; substitute pronouns are 14px with 16px line height.
Names use `overflow: hidden`, `white-space: nowrap` and `text-overflow:
ellipsis`, with the full value retained in `title`. `fitRosterText()` still
shifts from 36px to an 18px floor on core and from 24px to a 14px floor on
substitutes.

There is no name-plate code chip. The `NAME-PLATE CODE CHIP` CSS block,
`teamCode()`, `rosterCode` and every `code` parameter were removed. The
constructor signatures are `createCorePlayerCard(player, index)` and
`createSubCard(player, index, totalSubs)`. Source-text assertions require the
absence of `player-code`, `teamCode`, `data-initials` and `playerInitials`.

### 14.3 State matrix

| State | Required result |
|---|---|
| Team colours on | Header uses the team colour; `--fgc-team`, `--fgc-wash` and `--fgc-glow` are set from the hex, with alpha 0.16 and 0.22. |
| Team colours off | All three inline properties are removed; header returns to `var(--fgc-surface)`; wash and glow return to orange. |
| Role icon present | Hexagon badge renders the icon at 40px on core and 44px on substitutes. |
| Role icon absent | `:empty` hides the badge; no empty hexagon remains. |
| Long core name | Font shrinks to the 18px floor if needed, then ellipsizes; `title` keeps the full name. |
| Long substitute name | Font shrinks to the 14px floor if needed, then ellipsizes; `title` keeps the full name. |
| No substitutes | Separator and substitute strip remain in place with no rows added. |
| Fresh or reloaded source | A79 styling is intrinsic HTML/CSS/JS; no cached message is required. The A77 background init replay and A78 overflow guard remain in force. |

### 14.4 JS helpers

`escapeHtml()` and `fitText()` are unchanged. `fitRosterText()` keeps the
floors described above and runs from `document.fonts.ready`. `getRoleIcon()`
still normalises the role to lower case and returns an `<img>` only when a URL
exists. `updateRoster()` chooses the team and players from `TEAM_SIDE`, applies
or removes the three team-colour properties, filters players by
`position === 'core'` and `position === 'sub'`, and passes only the player and
row metadata to the constructors. No new payload field, helper or message type
was introduced.

### 14.5 CDP-verifiable assertions

On a fresh home probe tab at the 1920×1080 design canvas:

- `.overlay` reports `dataset.rosterSide === 'home'`; `.roster-wash` is 760×1080
  at x = 0; its computed background starts
  `linear-gradient(90deg, rgba(62, 207, 142, 0.16)`; the `::after` background
  is `radial-gradient(58% 100% at 22% 100%, rgba(62, 207, 142, 0.22)` with
  height 320px, bottom -120px and `blur(14px)`.
- Away mirrors the wash at x = 1160 with 270deg and `78% 100%`.
- `--fgc-team`, `--fgc-wash` and `--fgc-glow` match the live team colour when
  enabled and are empty after `teamColorsEnabled: false`.
- Core hero is 740×474; badge is 56×56 with a 40px icon, hex clip, centre
  offset 0 and badge-bottom minus plate-top equal to -16px; plate is 126px
  with `20px 24px` padding; pronouns are 18px.
- Sub portrait is 72×76; badge is 58×58 with 8px inset, 44px icon and 0px
  centre offset; pronouns are 14px.
- `.core-stage` is 1500×600; left and right rails are 20×520 at x = 118 and
  x = 1782, outside the card row; line and hex colours are
  `rgb(42, 49, 64)` and `rgb(58, 67, 86)`.
- Long names report ellipsis, a full `title`, and `scrollWidth >=
  clientWidth` only after the fit floor is reached.
- No `.player-code`, `teamCode`, `data-initials` or `playerInitials` exists.
- Core bottom <= separator top and separator bottom <= substitutes top.
- `innerWidth - clientWidth` and `innerHeight - clientHeight` are zero;
  html and body overflow both compute `hidden`; console is empty.

A79's real-browser evidence is in `docs/manual-qa.md` row A79, with repeatable
steps in `docs/manual-qa-runbook.md` section A79. Screenshots were captured,
but observer review failed because of an OpenCode Go usage limit, so OBS
visual sign-off at 1920×1080 remains an owner check.

## 15. Roster overlays - role order, proportion and full-width glow (A80)

### 15.1 Purpose and scope

A80 continues the paired `team-roster-home.html` and `team-roster-away.html`
refresh. The owner asked to remove the left and right decorations, fix the
squashed substitute hexagon by shrinking the role icon, force both rows into
Damage, Tank, Support order, and soften the gradient so the glow spans the whole
bottom of the window rather than following the home or away side. A80 supersedes
only the A79 side rails and the side-anchored radial glow. It keeps the A79 core
badge, taller plate, smaller pronouns, removal of the code chip, separator and
zero-gutter contract. The two files remain byte-identical apart from `<title>`
and `TEAM_SIDE`. No new asset, font, message type, payload field, FX sentinel or
filename is introduced.

### 15.2 Surface plan

The `.core-stage` wrapper and both `.core-rail` elements are gone.
`.core-players` is again the direct 1500×600 flex row with a 20px gap; the
A61 flexible card widths and the 284px substitute cards are unchanged. The
`.roster-separator` remains between core and substitutes.

The core badge stays at 56×56 with a 40×40 image, `left: 50%`, `bottom: 16px`
inside the 740×474 `.hero-image`, hexagon clipped and hidden by `:empty`. The
126px `.player-info` keeps `padding: 20px 24px`, 18px/22px pronouns, ellipsis
and the full-value `title`. Substitute cards stay 76px high. Their `.sub-portrait`
is 72×76; the badge is now 46×54 with a 34×34 image, `bottom: 8px`, zero
centre offset and the shared hexagon clip. The old 58×58 badge and 44×44 image
are forbidden.

The `.roster-wash` stays 760px wide and side-anchored: home uses 90deg from the
left, away uses 270deg from the right. Its alpha is softer. The defaults are
`--fgc-wash: rgb(255 122 26 / 12%)` and
`--fgc-glow: rgb(255 122 26 / 12%)`. When team colours are enabled the JS sets
`--fgc-wash` to `rgba(r, g, b, 0.12)` and `--fgc-glow` to
`rgba(r, g, b, 0.14)`, and removes both properties when they are disabled.
`.roster-wash::after` spans the whole 1920px canvas with `left: 50%` and
`transform: translateX(-50%)`, height 320px, `bottom: -120px`,
`filter: blur(14px)` and `linear-gradient(0deg, var(--fgc-glow), transparent
100%)`. Side-specific radial glows are removed.

### 15.3 State matrix

| State | Required result |
|---|---|
| Scrambled core and substitute input | Both rows render Damage, then Tank, then Support, with stable order inside each role. |
| Unknown role | Ranked with Damage so the player remains visible in a deterministic slot. |
| Team colours on | Header uses the team colour; wash alpha is 0.12 and glow alpha is 0.14. |
| Team colours off | Inline wash and glow properties are removed; the orange 12% defaults return. |
| Role icon present | Core badge shows a 40px icon; substitute badge shows a 34px icon in the 46×54 hexagon. |
| Role icon absent | `:empty` hides the badge. |
| Long name | Font shrinks to the core 18px or substitute 14px floor, then ellipsizes; `title` keeps the full value. |
| No substitutes | Separator and substitute strip remain in place with no rows added. |
| Fresh or reloaded source | A80 styling is intrinsic HTML/CSS/JS; A77 init replay and A78 overflow guard remain in force. |

### 15.4 JS helpers

`escapeHtml()`, `fitText()`, `fitRosterText()` and `getRoleIcon()` are unchanged.
A80 adds `ROLE_RANK` with `damage: 0`, `tank: 1`, `support: 2`, `roleRank(role)`
and `sortPlayersByRole(players)`. `roleRank()` lowercases and trims the role,
returns the mapped rank or 0 for unknown and empty roles, and
`sortPlayersByRole()` copies the array, attaches the original index and sorts by
rank then index. `updateRoster()` calls the sorter on both filtered lists before
creating cards. The incoming payload is not mutated and no new field is read.
The application model in `crates/core/src/model.rs` stores `tank`, `damage` and
`support`; the visible labels are `Tank`, `Damage` and `Support`.

### 15.5 CDP-verifiable assertions

On fresh home and away probe tabs at the 1920×1080 design canvas:

- Both `#core-players` and `#substitutes` expose names in Damage, Damage, Tank,
  Tank, Support order for the scrambled A80 fixture.
- `.core-stage`, `.core-rail` and `.core-rail__hex` counts are all zero, and
  `.core-players` measures 1500×600.
- `.player-card-sub .role-badge` measures 46×54 with an 8px inset and a 34×34
  image; its centre offset is 0. Core badge remains 56×56 with a 40×40 image.
- `.roster-wash` remains 760px; home and away keep their 90deg and 270deg side
  gradients with alpha 0.12.
- `.roster-wash::after` measures 1920×320, is centred on the window, and paints
  `linear-gradient(0deg, rgba(<team>, 0.14), rgba(0, 0, 0, 0) 100%)`.
- No `radial-gradient`, `.core-rail`, 58×58 substitute badge or 44×44 substitute
  image remains.
- Long names report ellipsis and a full `title`; the verified core name reaches
  the 18px floor with 236/552.
- Core bottom <= separator top and separator bottom <= substitutes top.
- `innerWidth - clientWidth` and `innerHeight - clientHeight` are zero; html and
  body overflow both compute `hidden`; console is empty.

A80's real-browser evidence is in `docs/manual-qa.md` row A80, with repeatable
steps in `docs/manual-qa-runbook.md` section A80. Screenshots were captured, but
the observer subagent again failed with an OpenCode Go usage limit, so OBS visual
sign-off at 1920×1080 remains an owner check.

## 16. Roster overlays - one full-width bottom gradient (A81)

### 16.1 Purpose and scope

A81 changes only the paired `team-roster-home.html` and
`team-roster-away.html` wash treatment. The owner rejected the A80 side wash
after seeing it in review: it still read as a side gradient and was cut off
before the frame edge. The new contract is one subtle glow rising from the
bottom of the overlay and stretching edge to edge across the full 1920px frame.
A81 supersedes only the gradient geometry in A80. The rail removal, substitute
hexagon proportion, Damage/Tank/Support sort, role badges, plate height,
pronoun sizes, ellipsis guard and byte-equality contract are unchanged.

### 16.2 Surface plan

`.roster-wash` is the primary layer. It is absolutely positioned with `left: 0`,
`right: 0`, `bottom: 0`, `height: 420px`, `z-index: 0` and
`pointer-events: none`. Its background is
`linear-gradient(0deg, var(--fgc-wash), transparent 100%)`, so colour is
strongest at the bottom and fades upward. It no longer has a width, top, left
or right edge that depends on `data-roster-side`.

`.roster-wash::after` is the softer second layer. It uses `left: 0`, `right: 0`,
`bottom: -100px`, `height: 300px`, `filter: blur(24px)` and
`linear-gradient(0deg, var(--fgc-glow), transparent 100%)`. The negative bottom
lets the blur soften below the visible frame; the overlay's A78 overflow guard
clips the excess. Both layers are as wide as the overlay, so no centred 1920px
child can be clipped by a 760px parent.

No `.overlay[data-roster-side="home"] .roster-wash` or
`.overlay[data-roster-side="away"] .roster-wash` rule remains. The
`data-roster-side` attribute is still assigned for diagnostics and the
byte-equality guard, but it does not select any styling.

### 16.3 State matrix

| State | Required result |
|---|---|
| Default colours | wash and glow paint the orange defaults at 8% and 9% respectively |
| Team colours on | JS sets `--fgc-wash` to `rgba(r, g, b, 0.08)` and `--fgc-glow` to `rgba(r, g, b, 0.09)` |
| Team colours off | all inline `--fgc-team`, `--fgc-wash` and `--fgc-glow` properties are removed |
| Home and away | identical wash geometry and gradient direction; only `data-roster-side` differs |
| Fresh or reloaded source | styling is intrinsic HTML and CSS; A77 init replay and A78 overflow guard remain in force |

### 16.4 JS helpers

A81 adds no helper and changes no message handler. The A80 `ROLE_RANK`,
`roleRank()` and `sortPlayersByRole()` helpers remain in both files. The team
colour branch keeps writing the two gradient custom properties; A81 changes
only the CSS that consumes them.

### 16.5 CDP-verifiable assertions

On fresh 1920×1080 probe tabs for both roster files with a loaded `teamRoster`
payload:

- `.roster-wash` reports left 0, right 0, width 1920, bottom 0, y 660 and
  height 420.
- Its computed background is `linear-gradient(0deg, rgba(<team>, 0.08),
  rgba(0, 0, 0, 0) 100%)`.
- `.roster-wash::after` reports left 0, right 0, width 1920, bottom -100, height
  300, `blur(24px)` and `linear-gradient(0deg, rgba(<team>, 0.09),
  rgba(0, 0, 0, 0) 100%)`.
- No `radial-gradient`, `linear-gradient(90deg, var(--fgc-wash)`,
  `linear-gradient(270deg, var(--fgc-wash)`, `width: 760px` or side-specific
  `.roster-wash` selector remains in either source file.
- Core and substitute names remain in Damage, Tank, Support order; the core
  badge is 56×56, the substitute badge is 46×54 with a 34px icon, and no rail
  node exists.
- Long names report ellipsis and a full `title`; gutters are zero; html and body
  overflow are `hidden`; console is empty.

A81's real-browser evidence is in `docs/manual-qa.md` row A81, with repeatable
steps in `docs/manual-qa-runbook.md` section A81. Screenshots were captured, but
visual review was unavailable in this session, so OBS sign-off at 1920×1080
remains an owner check.

## 17. Roster overlays and team colour controls (A82)

### 17.1 Purpose and scope

A82 completes the owner's 2026-11-24 roster and team-properties review. The
owner asked for spacing around uploaded logos, an animated divider between the
core and substitute rows, stored team colours to be respected, a text colour
option, and gradients that make dark blue and bright pink read with the same
visual weight. They then refined the colour controls: automatic logo colouring
must apply to Background and Text individually, both colours need recommended
values, computed contrast must change Text only, computed contrast must be
usable at the same time as automatic logo colours, and the functions must adjust
a logo-derived text colour when a manually chosen background is too similar.
A82 supersedes the single `Text colour mode` select from the in-progress A82
work. It does not supersede A80 or A81 geometry: rails stay removed, the
substitute hexagon stays 46×54 with a 34px icon, both rows stay sorted Damage,
Tank, Support, and the bottom wash stays full width and bottom only.

### 17.2 Roster surface plan

`.team-logo` gains `padding: 12px` and `background-origin: content-box` while
its background colour remains `var(--fgc-surface-inverse)`, so the panel still
fills the same frame but an uploaded logo no longer touches its edge. The
`.roster-separator` between the two rectangular rows gains three keyframe
groups: `dividerReveal` for the container, `dividerLine` for the two rules, and
`dividerHex` for the centre mark. The reveal starts after the existing 1.6s
entrance beat, runs once with `both` fill, and is disabled under
`prefers-reduced-motion: reduce`.

### 17.3 Colour propagation

`team_dto()` now writes `textColor` beside `color`, and the two protocol golden
fixtures carry the stored value. Both roster files read `teamData.textColor`
and use it directly for the team name when it parses as six-digit hex. If no
valid colour is present, they fall back to the existing Night Broadcast
contrast colour derived from the background. No new payload field is needed for
the app's automatic-colour switches because they are editing inputs, not
overlay state.

### 17.4 Luminance-balanced wash

The bottom-only gradient contracts from A81 stay fixed in geometry, but live
team colours no longer produce the same alpha for every hue. Each roster
computes sRGB relative luminance and uses
`wash = clamp(0.20 - luminance * 0.16, 0.08, 0.20)` and
`glow = clamp(wash + 0.01, 0.09, 0.21)`. A dark blue therefore gets a stronger
wash and glow than a bright pink, so the two read as equally present. The
default orange tokens remain unchanged at 8% and 9%; only the live colour path
uses the compensation.

### 17.5 Team-properties control plan

The Colours panel exposes two automatic switches and one computed switch.
`colorFromLogo` recommends a Background value from the sampled logo and disables
the Background field while on. `textColorFromLogo` recommends a Text value from
the same palette and disables the Text field while on. `computedContrast` always
changes Text only, can be enabled with either automatic switch, and uses the
Night Broadcast `#0b0d12` or `#eef2f8` colours when it is the only active text
rule. When both `textColorFromLogo` and `computedContrast` are on, the sampled
text is adjusted toward the contrast target instead of being replaced. Manual
background changes re-harmonise an automatic text colour; computed contrast uses
the stricter 4.5:1 target, while automatic text alone uses the 3:1 display-text
target. Turning off an automatic switch enables manual editing of that field.

### 17.6 CDP-verifiable assertions

- Both roster sources keep the same normalised bytes apart from `<title>` and
  `TEAM_SIDE`.
- Source assertions require the 12px logo padding, content-box background
  origin, three divider keyframe names, the reduced-motion fallback,
  `teamData.textColor`, the luminance formula, and the nested init replay of
  `teamColorsEnabled`, `roleIcons` and `showPlayerPortraits`.
- At a fresh 1920×1080 tab with `#0f3b8f`, the wash alpha is 0.19 and the glow
  alpha is 0.20; with `#ff5c7a`, they are 0.15 and 0.16.
- A stored yellow or cyan `textColor` renders as that exact RGB value on the
  team name.
- The app panel shows independent `Background from logo`, `Text from logo` and
  `Computed contrast` switches. Computed contrast changes only Text and can be
  switched on while both automatic switches remain on.
- The palette sampler reports opaque logo buckets, ignores alpha values below
  128, and keeps at most eight quantised colours. Harmonisation preserves the
  dominant channel of the sampled text while reaching the required contrast
  ratio against the actual background.

A82's real-browser evidence is in `docs/manual-qa.md` row A82, with repeatable
steps in `docs/manual-qa-runbook.md` section A82. The owner opened the app in OBS
at 1920×1080 and reported: "A82 is a pass." The visual sign-off is complete.

## 18. Map select clarity, scores and winner treatment (A83)

### 18.1 Purpose and scope

A83 refreshes `map-select.html` only. The owner chose it over game overlay and
gave five goals: continue the established theme philosophy, rework map scores,
make the winning team logo more visible with a subtle map glow, keep hero square
images for now, and make the current map unmistakable. A83 does not change the
canonical FX block, the early overflow guard, the five bundled fonts, the
Night Broadcast tokens, the projected card count, the legacy tiebreaker card or
the ban portrait source. Hexagonal hero images remain a later consideration.

### 18.2 Current-map treatment

`.map-card--current` keeps its wider `flex: 1.8` layout and gains a 2px inset
accent border. A new `.map-card__current` badge sits at top 16 and left 16 with
z-index 4. It is an accent plate with on-accent text, 14px SemiBold uppercase
type and 0.08em letter spacing. Its `::before` uses the Night Broadcast
hexagon `clip-path` as a dot. The card HTML inserts exactly one badge for the
card whose index matches `currentMapIndex`. Upcoming cards do not receive it.

### 18.3 Score rework

The old translucent 60px board and bare numeric divider are removed. Each
completed board is now a 64px surface panel at top 286 with a 1px line border,
medium radius, a 10px gap and 5px padding. It contains two `.map-card__score-team`
plates, each using its own team colour through `--team-a-color` or
`--team-b-color` with a 0.16-opacity wash. The winner plate adds a 2px coloured
border and a 14px coloured glow through `::after`. A faint colon separates the
two scores. Team tags appear above or beside the numbers so a viewer can tell
which side each score belongs to without relying on order alone.

`safeHexColor()` sanitises values before they are written to CSS variables.
Unknown or malformed colours fall back to Night Broadcast tokens instead of
breaking the board. `createMapCard()` derives `aWon` and `bWon` from completed
scores, applies the winner class and passes the winning colour to the card.

### 18.4 Winner logo and map glow

The winning logo receives `.map-card__logo--winner`, which adds a drop shadow
and a hexagon `::before` halo behind the 120×120 image. Completed map images
receive a 0.12-opacity radial winner-colour wash through `::before` while the
existing 0.16-opacity accent scrim remains on `::after`. Upcoming maps keep their
placeholder, hidden board and hidden logo. The glow is deliberately subtle and
never replaces the map artwork.

### 18.5 Square ban portraits

`createBanStrip()` continues to use `heroPortraits[heroName]` for each ban and
keeps the 58×58 square `<img>` element. This slice must not apply a hexagon
clip-path or border-radius to ban portraits. The rotated orange ban-order
diamond remains outside the image. A future slice may evaluate hexagonal hero
art when the owner is ready.

### 18.6 CDP-verifiable assertions

- One current card, one `Current map` badge and one board per projected map.
- Current card inset border is `rgb(255,122,26) 0 0 0 2px` under Night Broadcast.
- The badge measures 150×28 with orange background, dark text, 14px SemiBold and
  a hexagon `::before` clip-path.
- Completed boards measure 215×64 at top 286 with `rgb(11,13,18)`, a 1px
  `rgb(42,49,64)` border, 6px radius, 10px gap, 5px padding and z-index 3.
- Each completed board has two 86×52 score plates; the winner plate has a 2px
  coloured border, a 14px coloured glow and 0.55 opacity.
- The winner logo is 120×120 with a hexagon halo at 0.24 opacity and 10px blur;
  the completed map radial wash is 0.12 opacity.
- With `#0f3b8f` and `#ff5c7a`, the winner treatment follows the higher score and
  reverses on the next completed map.
- Ban portraits report 58×58, natural 1920×1080 art, `border-radius: 0`,
  `clip-path: none` and `object-fit: cover`.
- Overlay geometry stays 1920×1080 with hidden overflow, zero gutters and no
  console errors. The canonical FX sentinels are unchanged.

A83's real-browser evidence is in `docs/manual-qa.md` row A83, with repeatable
steps in `docs/manual-qa-runbook.md` section A83. A screenshot could not be
captured because the desktop window was not visible, so OBS visual sign-off
remains with the owner.

## 19. Map select and hero bans - Clean Canvas and targeted reveals (A84)

**Status:** **implemented and QA'd 2026-11-24.** A84 was reviewed by the owner in
the running overlays and accepted. It supersedes the chrome-heavy A83 treatment
on `map-select.html` while retaining the Night Broadcast tokens, the five
bundled faces, the early overflow guard, the canonical FX block, square ban
portraits, the projected card count and the legacy tiebreaker behaviour.

**Scope (this spec):** `map-select.html`, `hero-bans.html`, the eight canonical
FX carriers, `crates/app/src/state.rs`, the Match Controller hero picker and
`crates/server/tests/overlays.rs`.

### 19.1 Clean Canvas

Map select removes `.map-card__current`, the 2px inset accent border, the score
board, the per-card game-mode icon and the `Now playing` pill. The footer keeps
only the map type text. The current card keeps `flex: var(--current-grow, 1.8) 1
0;` while opening, then settles to 1.8 after the row reveals; a later current-
map change trades width in one handoff. No card header competes with the art.

### 19.2 Winner treatment

The winning logo is centred at `top: calc(50% - 52px)` and `left: 50%`, 112px
wide, with a dark drop shadow and a soft team-colour glow. The old giant hexagon
`::after` plate and its `-12px` inset are gone. Completed art uses
`brightness(0.52) saturate(0.45)` under a radial team-colour wash at 0.13
opacity. `safeHexColor()` validates six-digit hex values before they reach
`--winner-color`, so malformed team data cannot break the glow.

### 19.3 Opening and live reconciliation

The opening reveal is a one-shot. After `settleInitialAnimations()`, duplicate
payloads produce zero animations. Stable card identity lets the overlay call
`reconcileMaps()`, `refreshCardContent()`, `animateContentSwap()`,
`setCurrentVisual()` and `animateCardWidth()` instead of rebuilding the row.
Only changed art, type and name animate; the old text is cloned into a ghost,
faded out and removed while the new text fades in. A dark curtain covers art
swaps; empty slots update in place without a curtain. Unchanged image elements
stay mounted so unrelated updates do not reload the same art.

### 19.4 Hero bans and app controls

Hero bans reveal the changed side only. `animateBanReveal(side, firstSide)`
animates art blur/opacity, name translation and role scale over 560ms, with a
90ms stagger for the second side. `renderedHomeHero` and `renderedAwayHero`
track the last painted hero so the opening does not replay on every broadcast.
Match Controller hero pickers use native `<select>` elements so long hero lists
remain visible. `AppStore` now seeds `background_opaque` as `true`; transparent
remains an explicit override and the cached `background` broadcast is unchanged.

### 19.5 Evidence

A84's evidence is in `docs/manual-qa.md` row A84 and sign-off table, with
repeatable steps in `docs/manual-qa-runbook.md` section A84. The source guards
are `map_select_keeps_a_clean_canvas_with_centred_winners`,
`map_select_reconciles_live_data_without_replaying_the_opening`,
`hero_bans_reveals_each_new_ban_without_replaying_the_opening` and
`hero_bans_keeps_unchanged_images_when_another_ban_updates`.

## 20. Map pool - Clean Canvas and readable team-colour picks (A85)

**Status:** **implemented and QA'd 2026-11-24.** The owner reviewed the final
live render and answered "Approve A85". The overlay keeps the Night Broadcast
tokens, five bundled faces, early overflow guard, canonical FX block, grouped
mode order and existing payload contract.

### 20.1 Layout and motion

`map-pool.html` renders a centred 1768×968 grid with 56px vertical and 72px
horizontal padding, five mode columns and no `Map Pool` heading. `fitText` and
`fitMapNames` are removed. Names use `container-type: inline-size`,
`font-size: clamp(13px, 9cqw, 20px)` and a two-line clamp. Map art overscans
flex rounding by `-1px` on each side and keeps the black bottom gradient. Mode
marks are plain contain-style icons with a drop shadow; no `.mode-icon` rule or
image uses `clip-path`. Columns, cards and mode marks enter on staggered beats,
and the reduced-motion branch includes the mode marks.

### 20.2 Pick state

A picked image uses `grayscale(100%) brightness(0.56) saturate(0.45)`. A radial
winner wash sits above the art, the existing black footer gradient remains
above that, and a team-colour bottom wash, crest reveal and name rule complete
the state. Every pick colour is `rgb(var(--pick-glow-rgb, 255 122 26) / ...)`;
no static `--fgc-pick-glow*` token remains. The crest drop shadow, radial wash,
bottom wash and inset name rule all use that same derived colour. The pick
animation is 620ms, delayed 60ms, and joins the card's existing animation batch.

### 20.3 Colour derivation and init contract

`lightenedRgbTriplet()` lifts each team channel 45% toward white with
`PICK_LIGHTEN_MIX = 0.45`, preserving hue while making dark teams readable.
Only the derived pick treatment uses it; logo artwork remains untouched. The
keyed init frame wraps the scoreboard, so `updateOverlay()` now reads
`const scoreboard = d.scoreboard || d;` and forwards
`{ ...scoreboard, mapPoolData }`. Without this unwrap, the pool renders but
loses team colours and falls back to orange.

### 20.4 Live-update contract

Cards are matched by stable `dataset.slot`. Repeated payloads with the same
pool and pick signature cause no animation. Changed slots run one batch of
card, content, mode-mark and pick animations; `refreshPickedState()` reveals
the crest only when the pick key changes. Opening marks are cancelled and
removed by `settleOpening()`, and post-opening updates cannot restart the
opening. Reduced motion remains guarded.

### 20.5 CDP-verifiable assertions

- 1920×1080 document, opaque canvas, zero headings and no console errors.
- Five columns, seven fixture cards, five plain mode icons and `modeClip: none`.
- Bottom black gradient, `-1px` image margin, wrapped two-line names.
- Picked filter is `grayscale(1) brightness(0.56) saturate(0.45)`.
- Pick glow, wash, name rule and crest shadow use the same lightened team rgb.
- Live Inglorious Blasters evidence derives `127 120 149` from `#17093e`.
- Opening reports 22 CSS animations; duplicates report zero.
- A name change produces one four-animation batch.
- A pick change produces one six-animation batch including glow and crest.

A85's evidence is in `docs/manual-qa.md` row A85 and sign-off table, with
repeatable steps in `docs/manual-qa-runbook.md` section A85. The source guards
are `map_pool_applies_the_clean_canvas_and_targeted_update_contract`,
`map_pool_animates_mode_marks_and_team_pick_reveal` and
`map_pool_lifts_dark_team_colours_for_the_pick_glow`.