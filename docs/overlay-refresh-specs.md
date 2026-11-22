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
