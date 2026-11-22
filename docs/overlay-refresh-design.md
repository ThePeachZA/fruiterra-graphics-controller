# Overlay refresh — design pass

Status: **philosophy and token system approved 2026-11-22**. The
**game-overlay pilot is implemented and QA'd** (A57): `game-overlay.html` now
uses the Night Broadcast tokens, keeps its filenames, wire shapes, 1920×1080
canvas and geometry, and retires the old white/black/purple palette. The
**rollout is complete** (A58): `hero-bans.html`, `map-pool.html`,
`map-select.html`, `row-casters.html`, `team-roster-home.html`,
`team-roster-away.html` and the standalone `timer.html` now carry the same
tokens, so no bundled overlay uses the old palette. `auto.html` remains a pure
iframe host and was verified separately (A59), which also fixed its missing
favicon link. Per-overlay targets and the one recorded deviation are in
[`docs/overlay-refresh-specs.md`](overlay-refresh-specs.md).

Owner direction: draw the most inspiration from the **OWWC27** examples, without
copying them 1:1 and without copying official league/team marks. Team colours
are the primary identity where possible; orange is the system signal. The game
overlay does not duplicate game-rendered objective, timer or player hero cards.

This document is the first deliverable of the refresh. It defines the shared
design philosophy and token system first, then the layout concepts. The
owner’s review answers are recorded in §8; the game-overlay pilot applied them
to one file and is the reference for the rollout.

Input: `F:\Graphics Controller Projects\Example Overlays\REFERENCE_NOTES.md`
(textual notes from the owner’s example screenshots).

---

## 1. Step-by-step process

1. **Lock the contracts.** Filenames, wire message shapes, 1920×1080 canvas,
   transparent background and the existing layout geometry stay fixed. This
   pass changes surface, colour, type and states — not the protocol or the
   boxes.
2. **Write the philosophy and tokens.** Done in this document.
3. **Review the direction.** Confirm the palette, type scale, motif and layout
   concepts before any HTML is edited.
4. **Build a shared token block.** One copy-paste `:root` block and a small
   shared component vocabulary, inlined into each overlay exactly as the
   `@font-face` rules are today.
5. **Pilot one overlay.** Done 2026-11-22 — game overlay, because it carries the
   score bar, team identity, map rail and centre bug — the most reusable
   grammar.
6. **Real QA on the pilot.** Done 2026-11-22 (A57) — WebView2 over CDP at
   1920×1080 with DOM and computed-style evidence for team colours on and off.
   Long names, missing logos, null teams, empty states and reduced motion stay
   on the rollout checklist.
7. **Roll out overlay by overlay.** Reuse the pilot’s tokens and components;
   do not invent a new skin per file.
8. **Full QA and docs.** Re-run the overlay checks, update the wiki/protocol
   docs where the visual contract is described, and record the refresh in the
   backlog.

### Non-goals for this pass

- No new overlay files and no new wire message types.
- No protocol, schema or persistence changes.
- No official Overwatch/OWL/OWCS/OWWC logos, shields, wordmarks or sponsor
  locks.
- No remote fonts, images, CDNs or network dependencies.
- No change to the 1920×1080 canvas, the transparent background or the
  existing layout geometry.
- No per-overlay “creative” deviations from the shared system.

---

## 2. Design philosophy — Fruiterra Night Broadcast

Working style name: **Night Broadcast**. It takes the OWWC27 lessons — dark
canvas, one precious metal-like accent, restrained geometry, typography as the
identity — and re-tunes them to Fruiterra’s own brand: near-black blue-charcoal,
Fruiterra orange, high-contrast near-white type, and the existing Archivo
broadcast face.

### Principles

1. **The game is the picture.** The overlay is chrome, not content. Dark
   surfaces sit only where information lives; gameplay stays visible. No full
   opaque panels, no large gradients, no centre-frame decoration. The game
   overlay must not duplicate what the game capture already renders: the
   objective, the match timer, the player hero cards and the kill feed belong
   to the game, not to the overlay.
2. **Two colour roles.** Team colours are the primary identity wherever they
   are relevant — team bars, roster cards, winner screens and map picks. The
   Fruiterra orange `#FF7A1A` is the *system signal* for active, live,
   selected or important states. When team colours are off, or the information
   is neutral, the surface falls back to the neutral dark chrome. The old
   purple `#AC46F7` accent is retired.
3. **Type is the identity.** Archivo does the talking: condensed caps for
   labels and names, large tabular numerals for scores and clocks, small
   uppercase meta text for seeds, stages and times. No new display font.
4. **Geometry is a contract.** The existing boxes stay where they are: 43px
   outer margin, 525×55 team bars, 25px top bars, 278×50 centre bug, 6px
   radii. The refresh changes what those boxes look like, not where they are.
5. **Information lives in the same zones on every overlay.** Top score band,
   left/right team zones, centre map/format bug, bottom map band, lower
   thirds. A viewer should never have to re-learn the layout.
6. **Restraint is the premium.** One motif, one signal accent, thin hairlines,
   quiet surfaces. If an element does not help read the match, it goes.
7. **States are designed, not debugged.** Default, active, past, empty,
   missing logo, long name, team-coloured and reduced-motion states are part
   of the system from the start.
8. **Local by default.** Fonts, colours and fallbacks must work offline. Team
   logos and colours are user assets and always have a safe neutral fallback.

### What “Night Broadcast” is not

- Not a copy of OWWC27’s gold-on-navy or its hex motif.
- Not a return to the current black/white/purple v1 skin.
- Not a light “clean esports” skin; the owner’s preference is the darker,
  more premium OWWC direction.
- Not a decorative skin per overlay. All seven overlays plus the timer share
  one philosophy.

---

## 3. Token system

These tokens are the single source of truth for the refresh. They are written
with the same `--fgc-*` naming as the app theme so the control panel and the
overlays stay visibly related. Each overlay inlines the block, exactly as it
inlines its own `@font-face` rules today.

### 3.1 Colour

```css
:root {
  /* Canvas */
  --fgc-canvas: transparent;

  /* Surfaces */
  --fgc-surface: #0b0d12;          /* near-black blue-charcoal */
  --fgc-surface-raised: #11151d;   /* cards, roster, schedule rows */
  --fgc-surface-inverse: #eef2f8;  /* score chips, high-contrast plates */
  --fgc-surface-muted: #151922;    /* recessed wells, roster columns */

  /* Lines */
  --fgc-line: #2a3140;
  --fgc-line-strong: #3a4356;
  --fgc-line-on-inverse: #0b0d12;

  /* Text */
  --fgc-text: #eef2f8;
  --fgc-text-muted: #8d99ad;
  --fgc-text-faint: #6b7688;
  --fgc-text-inverse: #0b0d12;

  /* Brand */
  --fgc-accent: #ff7a1a;
  --fgc-accent-strong: #ff9346;
  --fgc-accent-soft: rgb(255 122 26 / 16%);
  --fgc-on-accent: #0b0d12;

  /* Semantic */
  --fgc-success: #3ecf8e;
  --fgc-warning: #f5b84b;
  --fgc-danger: #ef5b5b;

  /* Dynamic team colours (user assets) */
  --fgc-team-a: #eef2f8;
  --fgc-team-b: #eef2f8;
  --fgc-team-a-text: #0b0d12;
  --fgc-team-b-text: #0b0d12;
}
```

Usage rules:

- `--fgc-surface` is the default chrome. `--fgc-surface-inverse` is reserved
  for score chips and other high-contrast readouts.
- `--fgc-accent` is the system signal. Use it for one primary state per view:
  the active map, the live score, the selected caster, the current stage. Use
  `--fgc-accent-soft` for fills and `--fgc-accent` for text/rules.
- Team colours are the primary identity where they are relevant: team bars,
  roster cards, winner screens, map picks and other team-owned surfaces. Use
  `--fgc-team-a` / `--fgc-team-b` for those fills, with the matching
  `--fgc-team-a-text` / `--fgc-team-b-text` colour.
- When team colours are off, or the information is neutral rather than
  team-owned, fall back to `--fgc-surface` / `--fgc-surface-raised` with
  `--fgc-text`. Team colour never spills into neutral chrome.
- `--fgc-success`, `--fgc-warning` and `--fgc-danger` are for semantic states
  only, never decoration.

### 3.2 Type

Font family stays `'FGC Display', 'Apotek', sans-serif` with the existing
Archivo weights. The refresh changes sizes and roles, not the family.

| Token | Weight | Size | Use |
|---|---|---|---|
| `--fgc-font-display` | 900 | 96–220px | winner, matchup, countdown, map-select title |
| `--fgc-font-title` | 600 | 40–64px | overlay titles, standings/schedule headers |
| `--fgc-font-team` | 500 | 28–32px | team names in bars |
| `--fgc-font-player` | 500 | 14–18px | player handles |
| `--fgc-font-score` | 600 | 34–42px | scores, map numbers |
| `--fgc-font-meta` | 500 | 10–14px | seeds, regions, times, pronouns |
| `--fgc-font-map` | 900 | 20–28px | map names on cards |

Rules:

- All broadcast labels and names are uppercase.
- Scores, clocks and map numbers use `font-variant-numeric: tabular-nums`.
- Team and player names ellipsize; they never wrap.
- Meta text uses `letter-spacing: 0.06–0.12em`.
- No italics in the overlay face; the OWWC examples use italic display type,
  but Archivo’s character is strong enough without it. If a future pass
  wants italic energy, it should be a deliberate brand decision, not a
  default.

### 3.3 Space, radius, line

Keep the current geometry values so the refresh does not move boxes.

| Token | Value | Use |
|---|---|---|
| `--fgc-space-1` | 4px | inline gaps |
| `--fgc-space-2` | 8px | chip padding |
| `--fgc-space-3` | 12px | card padding |
| `--fgc-space-4` | 16px | bar padding |
| `--fgc-space-5` | 24px | section gaps |
| `--fgc-radius-sm` | 4px | inputs, small tags |
| `--fgc-radius-md` | 6px | team bars, cards, chips (current) |
| `--fgc-radius-lg` | 10px | large panels, roster card |
| `--fgc-radius-pill` | 999px | meta pills |
| `--fgc-line` | 1px | hairlines |
| `--fgc-margin` | 43px | outer page margin (current) |

### 3.4 Depth and motion

```css
:root {
  --fgc-shadow-panel: 0 8px 24px rgb(0 0 0 / 32%);
  --fgc-duration-micro: 120ms;
  --fgc-duration-state: 240ms;
  --fgc-duration-reveal: 480ms;
  --fgc-duration-scene: 900ms;
  --fgc-ease-out: cubic-bezier(0.22, 1, 0.36, 1);
}
```

- Animate `transform` and `opacity` only.
- One orchestrated reveal per overlay, not a cascade on every element.
- Respect `prefers-reduced-motion`: reveals become instant or a simple fade.
- No bounce, no spin, no glow pulses.

### 3.5 Component vocabulary

Every overlay is built from the same small set of parts:

- **Team bar** — 525×55, dark surface, 1px line, radius 6px. Children: ban
  chip, name, logo, score chip, side icon. When team colours are on, the bar
  fill is the team colour and the text uses the matching contrast colour.
- **Meta bar** — 525×25, dark surface, radius 6px. Left title/subtitle, right
  map/mode rail.
- **Centre bug** — 278×50, dark surface, radius 6px. Map number and format
  only; it does not carry the objective or match timer, which are rendered by
  the game.
- **Score chip** — 55×55, inverse surface, dark numeral, tabular nums.
- **Player card** — roster-only column: portrait, handle, pronouns. The game
  overlay does not render player hero cards.
- **Map slot** — vertical card; image, name bar, active border, pick overlay.
- **Map card** — grid card; image, name, mode/pick state.
- **Lower third** — bottom band; match score rows, last-map thumbnail.
- **Info card** — roster, schedule, standings; dark raised surface.
- **Full-screen moment** — winner/matchup; huge display type over muted game
  frame.

---

## 4. Layout concepts

These are the reusable layout ideas. They describe structure, not final
visuals. The existing geometry is preserved.

### 4.1 Score band (game overlay, hero bans, map select)

```
┌──────────────────────────────────────────────────────────────────────┐
│ [ban] TEAM A      [logo] [0] │      MAP 1 / FT3      │ [0] [logo]      TEAM B [ban] │
└──────────────────────────────────────────────────────────────────────┘
```

- Team bars stay at the existing 43px / 525px / 55px geometry.
- Score chips stay inverse plates so the score reads instantly.
- The centre bug keeps map number + format only. The objective, match timer
  and player hero cards are rendered by the game capture, not by this overlay.
- Orange appears on the active map, the live system state, or the leading
  score — never on all three.

### 4.2 Map rail (top-right meta bar)

```
┌─────────────────────────────────────────────┐
│  MAP 1  │  MAP 2  │  MAP 3  │  MAP 4  │ MAP 5 │
│  active │  dark   │  dark   │  dark   │ dark  │
└─────────────────────────────────────────────┘
```

- Five equal segments in the existing 525×25 bar.
- Active segment: orange fill, dark text.
- Past segments: muted opacity; future segments: faint.
- Map numbers stay tabular.

### 4.3 Map select (vertical slots)

```
┌────────┬────────┬────────┬────────┬────────┐
│ MAP 1  │ MAP 2  │ MAP 3  │ MAP 4  │ MAP 5  │
│ active │ dark   │ dark   │ dark   │ empty  │
│ image  │ image  │ image  │ image  │ slot   │
│■■■■■■■■│        │        │        │        │
│ OASIS  │   -    │   -    │   -    │   -    │
└────────┴────────┴────────┴────────┴────────┘
```

- Active slot: bright image, orange header/bar, 2px orange border.
- Decided/picked slots: dimmed image, team logo overlay, no border.
- Empty slots: dark surface with a faint watermark.
- Bans sit inside the map image as small chips, not in a separate panel.

### 4.4 Map pool (dark grid)

```
┌──────┬──────┬──────┬──────┬──────┐
│ mode │ mode │ mode │ mode │ mode │
├──────┼──────┼──────┼──────┼──────┤
│ map  │ map  │ map  │ map  │ map  │
│ map  │ map  │ map  │ map  │ map  │
│ map  │ map  │ map  │ map  │ map  │
└──────┴──────┴──────┴──────┴──────┘
```

- Dark raised surface, 5×3 grid, 1px lines, 6px radii.
- Mode icon header row in muted white.
- Picked map: team logo overlay + dimmed image, or a small orange “pick” tag.
- Map names in Archivo Black, bottom-left or bottom-centre.

### 4.5 Info card (roster, schedule, standings)

```
┌─────────────────────────────────────────────┐
│ [logo] TEAM NAME                             │
├────────┬────────┬────────┬────────┬────────┤
│ player │ player │ player │ player │ player │
│ role   │ role   │ role   │ role   │ role   │
├────────┴────────┴────────┴────────┴────────┤
│ subs / meta                                 │
└─────────────────────────────────────────────┘
```

- Roster: five equal columns, dark raised surface, role icon, name, pronouns.
- Schedule/standings: rows on a dark raised surface, orange header tab, thin
  row dividers, right-aligned times/records.
- One accent per screen: the header tab or the cutoff rule.

### 4.6 Caster row

```
┌──────────────┬──────────────┬──────────────┐
│   camera     │   camera     │   camera     │
├──────────────┴──────────────┴──────────────┤
│ NAME  pronouns · handle                    │
└────────────────────────────────────────────┘
```

- Camera panels first; names below, not over the video.
- One shared lower bar for the desk; an up-next block can sit at the right.
- Pronouns and handles are meta text, not display type.

### 4.7 Lower third / full-screen moment

```
┌─────────────────────────────────────────────┐
│ MATCH SCORE   TEAM A 2-1   TEAM B 0-2  │LAST│
│                                           │MAP│
└─────────────────────────────────────────────┘
```

- Lower third: bottom band, dark surface, two score rows, last-map thumbnail.
- Winner/matchup: huge display type over a muted game frame, one team mark,
  one meta line. No extra panels.

---

## 5. Overlay-by-overlay intent (concepts only)

These are the first concepts, derived from the shared system. They are not
final designs and no HTML has been changed.

| Overlay | Core concept | Primary accent use |
|---|---|---|
| `game-overlay.html` | Score band + team identity + centre map/format bug + map/mode rail; no game-rendered objective, timer or player hero cards | Active map / leading score |
| `map-select.html` | Vertical slots, one active map, bans in-image | Active slot header/border |
| `map-pool.html` | Dark art grid, mode header, picked overlay | Pick tag / active mode |
| `hero-bans.html` | Two large art panels, ordinal tags, map strip | Active ban / first-ban tag |
| `team-roster-home.html` / `-away.html` | Five-column dark card, role icons, subs footer | Team colour as primary identity |
| `row-casters.html` | Camera panels + shared lower bar, pronouns/handles | Live caster or up-next block |
| `timer.html` | Standalone clock for starting soon, intermission, lower-third bugs, schedule and other non-game contexts; not part of the game overlay | Running/paused state |
| Auto/winner/matchup (future) | Huge type over muted game frame | Winner team colour |

---

## 6. States to design

Every component needs these states, with the same names across overlays:

- **default** — neutral dark surface, white type.
- **active** — orange fill or 2px orange border, dark text on orange.
- **past** — reduced opacity, no border.
- **empty** — faint watermark or dash, no error styling.
- **missing logo** — monogram/tag fallback, never a broken image.
- **long name** — ellipsis at the existing limits; no wrap, no overflow.
- **team-coloured** — team colour fill as the primary identity where relevant,
  with a readable matching text colour; neutral fallback when colours are off
  or the surface is not team-owned.
- **reduced motion** — instant state changes, no reveals.

---

## 7. QA and acceptance plan

The refresh is not done until it is seen in a real browser.

- Pilot `game-overlay.html` in WebView2 over CDP at 1920×1080.
- Evidence: DOM geometry, computed styles, transparency, font loading,
  network/console cleanliness, long names, missing logos, null teams, empty
  states, active/past map states, team-colour on/off and reduced motion.
- Confirm the game overlay does not add objective, match-timer or player hero
  cards; those are supplied by the game capture.
- No screenshots are published.
- Roll out only after the pilot passes; then repeat the same evidence per
  overlay.
- Keep the frozen v1 tag and source untouched; the refresh is post-v1 work.

---

## 8. Resolved decisions (owner review, 2026-11-22)

1. **Team bars.** Use the recommended Night Broadcast default: dark surface
   with inverse score chips. The old white bar is not retained.
2. **Colour roles.** Team colours are the primary identity where possible.
   Orange remains the system signal for active/live/selected states. When team
   colours are off, or the information is neutral, fall back to the neutral
   dark chrome.
3. **Game overlay scope.** The objective, match timer and player hero cards are
   rendered by the game capture, not by the overlay. The game overlay stays
   chrome: team bars, centre map/format bug, map/mode rail and score. The
   standalone timer is for non-game contexts — starting soon, intermission,
   lower-third bugs, schedule and similar overlays — not for the game itself.
4. **Delivery order.** Refresh the seven existing overlays plus the timer first.
   Schedule, standings, winner, matchup and other presentation overlays come
   later, bundled with the app features they need.
