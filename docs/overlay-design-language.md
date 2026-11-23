# Overlay design language

This is the canonical reference for the visual language of the Fruiterra
broadcast overlays. It records what has shipped, why it looks the way it does,
which ideas are agreed but not yet built, and the risks that have to be respected
when the language is extended. It was opened on 2026-11-23 after the owner asked
for "all these overlay updates, design philosophy and ideas, and everything" to
be written down in as much detail as possible.

The document is deliberately broader than a specification. A specification says
what a file must do; this document says what the product believes. Read both:
[`docs/overlay-refresh-specs.md`](overlay-refresh-specs.md) carries the
per-overlay technical contract and the exact selectors, while this document
carries the reasoning, the shape rules, the per-overlay playbook and the open
ideas. The original design pass is kept in
[`docs/overlay-refresh-design.md`](overlay-refresh-design.md); this document is
the living language that grew out of it. Locked choices live in
[`docs/decisions.md`](decisions.md), open work in
[`docs/backlog.md`](backlog.md), the human checklist in
[`docs/overlay-qa.md`](overlay-qa.md), the evidence in
[`docs/manual-qa.md`](manual-qa.md) and the repeatable steps in
[`docs/manual-qa-runbook.md`](manual-qa-runbook.md).

Everything in the "Shipped" sections is true of the code on `main` at
`326b8fd` plus the working-tree change recorded as A74. Anything marked
"in flight" or "planned" is a decision, not a shipped feature. Dates are part
of the record so a later reader can tell a rule from a wish.

## 1. What the language is for

The overlays are Browser Sources in OBS. They are not a website, and they are
not a component library. Each one is a single self-contained HTML file with its
CSS, JavaScript, font faces and inline SVG data URLs baked into that file. There
is no bundler, no shared stylesheet URL and no runtime fetch. The files are
embedded whole by `rust_embed` through `crates/server/src/embed.rs`
(`#[folder = "overlays/"]`), so a new file is served at `/overlays/<file>` as
soon as it exists; only `OVERLAY_FILES` and `SCENE_OVERLAY_FILES` in
`crates/app/src/server.rs` decide which URLs are offered in the app and which
ones take part in the auto scene switcher.

That delivery model shapes the design language. A rule cannot be enforced by
importing a module; it has to be copied faithfully into each file and then
guarded by a test that reads the source. A design decision is therefore not
finished until the source, the test, the specification, the QA row and the
runbook section all agree. This is slower than a shared CSS file but it makes
every Browser Source independent in OBS, which matters more.

The language has four jobs:

1. Make a broadcast look like one product, not eight separate HTML pages.
2. Keep the game feed readable. Nothing an overlay does may compete with the
   play area for attention.
3. Stay cheap for a source that may be alive for three hours or more.
4. Leave room for an operator to use team colours, a light theme, reduced
   motion and a persistent background without redesigning each scene.

## 2. The shape language

### 2.1 The hexagon is the identity mark

The hexagon is the product's shape language. It is the frame around things that
say *who* or *what state*, not a texture to fill space. The shipped hero bans
overlay uses one pointy-top hexagon for the team logo badge and one for the role
badge:

```css
clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
```

The logo badge is 100×115 and the role badge is 60×68, both using that same
path. The old side-pointing role path
`polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%)` was retired in
the full-bleed rework. One shape, at different scales, is what makes the two
badges read as a family.

The hexagon is allowed on:

- the team logo badge;
- the role badge;
- status chips such as picked, banned, first and second;
- avatars and portrait frames;
- small identity plates that hold a name or a short label.

It is forbidden on:

- table rows, because the honeycomb breaks the horizontal rhythm the eye uses
  to scan a roster or a standings table;
- map thumbnails, because a hex grid over a screenshot makes the image read as
  texture rather than as a place;
- long paragraphs, values that need tabular figures, or any readout whose
  corners have to stay intact for legibility.

The distinction is about scanning, not decoration. An identity mark is looked
at; a table is scanned. Different visual machinery is needed for each.

### 2.2 The shared mask and tile

The hexagon lattice is an inline SVG data URL with a 56×100 view box and the
path `M28 66L0 50L0 16L28 0L56 16L56 50L28 66L28 100L0 84L0 50L28 34L56 50L56 84L28 100`. It is stroked in white
(`stroke='%23ffffff'`) at one unit wide and filled with none. When it is used as
a CSS mask, the stroke becomes the drawn cell. The repeating unit in CSS is
`180px 322px`, not the 56×100 view box; that is the number every drift and
overscan calculation uses.

The namespace inside the data URL is written as
`http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg` in the canonical FX block. The escaped
form is required because `crates/server/tests/timer_overlay.rs` forbids the
literal `http://` or `https://` in any overlay; even an inline SVG namespace
counts as an external URL to that test. The escaped URI is a normal data URL and
renders the same mask. Existing inline `<svg>` strings outside the FX block keep
the literal namespace, because they are not loaded as URLs.

### 2.3 Identity first, pattern second

The subtleness of the hexagon changed in A72. The hero bans wall joints dropped
from `0.5` to `0.3` opacity. The lattice copy in front of the art dropped from
`18%` and `7%` tints to `11%` and `4%`, with a `3.5%` floor wash. The pattern is
now structure behind the subject rather than a second subject. The owner's
direction was that the hexagons should be "a little more subtle still" while
keeping the creativity of the design.

The rule that follows from that pass: a hexagon pattern should be noticed on a
second look. If a viewer can describe the pattern before they can name the team
or the map, the pattern is too loud.

## 3. Colour

### 3.1 The token block

Night Broadcast is the default theme. The full token block is in
[`docs/overlay-refresh-specs.md`](overlay-refresh-specs.md) §0 and is inlined in
each overlay's `<head>`. The values that carry the language are:

| Token | Value | Role |
|---|---|---|
| `--fgc-surface` | `#0b0d12` | default chrome and the background plate |
| `--fgc-surface-raised` | `#11151d` | raised panels and the hero bans wall |
| `--fgc-surface-inverse` | `#eef2f8` | score chips and high-contrast readouts |
| `--fgc-line` | `#2a3140` | quiet dividers |
| `--fgc-line-strong` | `#3a4356` | strong dividers and the lattice wash |
| `--fgc-text` | `#eef2f8` | primary text |
| `--fgc-text-muted` | `#8d99ad` | secondary text and neutral washes |
| `--fgc-accent` | `#ff7a1a` | the single system signal |
| `--fgc-ink-on-art` | `#eef2f8` | text laid over hero artwork |
| `--fgc-plate-dark` | `#0b0d12` | the plate behind that text |

The Light theme's `LIGHT_THEME_TOKENS` in `crates/protocol/src/theme.rs` sets
`--fgc-surface` to `#f4f6fb`. A `theme` message applies the new properties to
the document element and removes the ones the new set omits, so every overlay
repaints without a reload.

Orange is a system signal, not a team colour. It marks one primary state per
view: the active map, the live score, the selected caster, the first-ban
indicator. Two orange things competing in one frame means the frame has no
hierarchy. Team colours are the primary identity where identity matters.

### 3.2 Team colour enters from the outer edge

The centre of a frame is never a colour bar and never a seam. This was the
owner's correction on the hero bans redesign, and it is now a rule for the whole
language. Team colour washes in from the outer edge of the frame, over the art
and the lattice, and fades out before the middle.

The shipped hero bans rule is `.hero-half__edge`, at `z-index: 4`. The home half
uses `linear-gradient(to right, color-mix(in srgb, var(--team-a-color, var(--fgc-accent)) 46%, transparent) 0%, color-mix(... 16% ...) 18%, rgb(0 0 0 / 0%) 38%)`. The away half mirrors it with
`to left` and `--team-b-color`. In words: 46% of the team colour at the outer
edge, 16% at 18% across, nothing by 38%. The centre is left for the subject and
the game feed.

The runtime colours are set with `style.setProperty('--team-a-color', ...)` and
`--team-b-color` on the `#hero-bans` container, not on `:root`. That keeps them
away from the root theme overrides and means a colour can be removed cleanly
when a payload has no team colour. A later match cannot inherit an earlier
wash.

`color-mix()` needs Chrome 111 or later, which OBS 31 has. The project floor is
OBS 31.0.4 (CEF 116), so the wash is safe. On an older OBS the declaration would
be dropped and the edge would be bare; that is accepted, not a reason to remove
the effect.

### 3.3 Neutral ink over artwork

Text over artwork uses `--fgc-ink-on-art` (`#eef2f8`) on `--fgc-plate-dark`
(`#0b0d12`). It never uses the team colour. A team colour might be a light
yellow, a deep navy or a saturated pink; none of those is guaranteed to contrast
with a hero portrait. Neutral ink is predictable.

The ban-order label is the worked example. The markup and `updateBanOrder()`
say `First ban` and `Second ban`; CSS `text-transform: uppercase` renders
`FIRST BAN` and `SECOND BAN`. The old `1ST BAN` and `2ND BAN` literals, and the
old `color: var(--fgc-accent)`, are retired. Keeping the words in the markup and
the case in CSS means the accessibility tree and the visible frame say the same
thing.

### 3.4 Theme ownership

A theme owns the chrome. The app does not ask an overlay to pick its own colours
from a match; the app sends a `theme` message and the overlay applies it. Team
colours are a separate channel, set on a local container and cleared when
absent. Background washes may opt into team colours through an explicit variant
such as `?tone=team`; without that variant they stay neutral. The opaque plate
never changes with team colours.

## 4. Layout and the layering contract

Every scene overlay is a 1920×1080 document with a transparent body. The
`body` computes to `rgba(0, 0, 0, 0)` and the document must not scroll. The
standalone background is the one deliberate exception: it paints an opaque
plate, and it belongs below the scenes in OBS.

Within a scene overlay the stacking order is stable:

| Layer | z-index | Purpose |
|---|---|---|
| `.fgc-fx` | 0 | the optional transparent lattice, when present |
| `.overlay` or the overlay root | 1 | the overlay's own content |
| hero bans wall | 0 | structure behind the art |
| hero bans art | 1 | the subject |
| hero bans grid | 3 | the quiet lattice in front of the art |
| hero bans edge and captions | 4 | the team wash and neutral labels |

The canonical FX block lifts `.overlay` to `z-index: 1` so the existing content
stays above the new sibling without changing its geometry. The timer overlay has
no FX layer, so it keeps its own `.fgc-timer` root untouched. This matters: the
owner explicitly said there should be no background overlay attached to the
timer or the game overlay, and neither file carries the FX block or the
`.fgc-fx` element.

Full-bleed framing is preferred for moment overlays. The hero bans halves tile
the full 1920 width, 0–960 and 960–1920, and reach the full 1080 height. There
is no `.hero-bans__centre`, no banner and no seam. A centre gap reads as two
slides next to each other; a full-bleed frame reads as one broadcast moment.

Dense overlays keep their tables and rails rectangular. The hexagon belongs in
the chips and plates around a table, not in the rows themselves. The council
decision of 2026-11-23 is explicit: hexagons on identity marks, never on table
rows or map thumbnails.

## 5. The cut-out technique

The transparent hero files supplied by the content pack are full 1920×1080
canvases with the hero standing on the bottom of the canvas. Fitting that whole
canvas into a half with `object-fit: contain` letterboxed the subject; using
`cover` cropped it. Neither was acceptable for a full-bleed ban screen.

`crates/server/overlays/hero-bans.html` now places the cut-out by script in two
steps:

1. `subjectBox(img)` draws the loaded image into an offscreen canvas and scans
   the alpha channel. A pixel with alpha above `8` counts as part of the hero.
   The scan returns the bounding box of the opaque subject, not the bounding box
   of the file.
2. `placeCutout(img, box)` computes `const scale = box.height / subject.height;`
   where `subject.height` is the height of the half. It applies a single
   transform to the image, centred on the subject box and anchored to the
   bottom of the half.

The `<img>` inside `.hero-half__art--cutout` keeps `width: auto; height: auto;`,
is absolutely positioned and uses `transform-origin: bottom center`. The
transform alone moves and scales it, so the image is not repainted on every
frame. A canvas that cannot be read gets `.hero-half__art--cutout-fallback`,
which falls back to `object-fit: cover` with `object-position: center bottom`.

The accepted caveat is recorded in A72: the alpha bounding box includes a
weapon or an outstretched arm, so the box centre can sit about 19px, roughly 1%
of the half width, away from the visual centre of the hero. The QA accepted this
rather than adding a hand-tuned offset that would break on the next hero.

Reuse of this locator across overlays is planned, not shipped. The next overlay
that needs a transparent subject should copy the same two functions rather than
inventing a second alpha threshold. If more than three overlays need it, the
duplication should be revisited with a byte-identical block test, as was done
for the FX layer.

## 6. Motion

### 6.1 Durations and easing

Motion is slow enough to read through. The shared token block declares
`--fgc-duration-micro: 120ms`, `--fgc-duration-state: 240ms`,
`--fgc-duration-reveal: 480ms`, `--fgc-duration-scene: 900ms` and
`--fgc-ease-out: cubic-bezier(0.22, 1, 0.36, 1)`. The hero bans reveal uses a
second curve, `cubic-bezier(0.16, 1, 0.3, 1)`, for the halves and the first
curve for the captions.

Ambient motion is much slower again. The background glow breathes on 84s, the
ambient lattice drifts on 88s and the lit lattice on 72s. The canonical FX layer
drifts on 72s. A Browser Source can run for an entire broadcast, so an ambient
effect is measured in tens of seconds, not seconds.

### 6.2 The reveal delay and the stinger hand-off

A moment overlay does not appear the instant a scene changes. `--hero-reveal-delay:
0.8s` on `:root` gives a stinger or a scene transition time to finish. A72
measured the live animation: the home half starts `heroReveal` at 0.8s and runs
0.9s; the away half starts at 0.92s; the captions start at 1.15s and 1.27s and
run 0.7s. The animation uses `both` fill mode, so the element stays hidden
before the delay and stays at the final state afterwards.

The council rule for the rest of the product: 0.8s for moment overlays (hero
bans and the game overlay) and 0.35s for dense overlays such as the roster and
map pool, where the operator wants information on screen quickly. The shorter
delay is planned as `--fgc-reveal-local`; it is not shipped.

The background-tab gotcha is real. CSS animations can be throttled or paused
when a page is not visible, so a reveal that depends on an animation finishing
in real time can look different after a tab is restored. The fill mode keeps the
end state, but QA should never treat an animation callback as a data boundary.

### 6.3 Compositor rules

Only `transform` and `opacity` may animate. No keyframe may touch
`background-position`, `filter`, `width`, `height` or `box-shadow`. There is no
`requestAnimationFrame` and no per-frame JavaScript in any overlay. `will-change:
transform` is set on the drifting layer only; putting it on every layer creates
extra compositor surfaces for no benefit.

`translate3d()` is used for drift so the browser keeps the layer on the GPU.
The transform values are chosen to be sub-pixel-slow at 60 fps. The aim is
continuous presence, not visible movement.

## 7. The animated background

### 7.1 Standalone first

`crates/server/overlays/background.html` shipped in `9600a65` as a separate
persistent Browser Source. It is first in `OVERLAY_FILES` and deliberately
absent from `SCENE_OVERLAY_FILES`, so the auto scene switcher and its number
hotkeys still hold nine scenes. The background stays on screen for the whole
broadcast while scenes replace one another.

The page paints a **solid, opaque plate**, not a transparent wash. `html`,
`body` and `.background` all use `var(--fgc-surface)`: `rgb(11, 13, 18)` under
Night Broadcast and `rgb(244, 246, 251)` under Light. A `theme` message repaints
all three. Because the plate is opaque, the OBS source must sit **below** the
scene sources; above them it would hide the show.

The `show` message toggles both the artwork and the plate: `.background` goes
between `block` and `none`, and `html` gains or loses `background-hidden`. The
rule `html.background-hidden, html.background-hidden body { background: transparent; }`
means a hidden source leaves no solid colour behind. That is the only moment
the background plate is transparent.

### 7.2 Layers

Three layers sit over the plate:

- `.background__glow` at `inset: -12%`, with two radial gradients at `4% 54%`
  and `96% 38%`, breathing on 84s.
- `.background__lattice--ambient`, painted with `--fgc-line-strong` at opacity
  `0.13`, drifting forward on 88s.
- `.background__lattice--lit`, two radial gradients weighted to `-6% 52%` and
  `106% 38%` at opacity `0.68`, drifting in reverse on 72s.

Both lattices use the shared hexagon mask at `mask-size: 180px 322px` and
`inset: -180px -220px`, so a drifting edge never enters the frame. The layers
are weighted to the outer edges, leaving the centre comparatively plain for a
game feed.

The neutral aliases are `--background-a: var(--fgc-text-muted)` (`#8d99ad`) and
`--background-b: var(--fgc-line-strong)` (`#3a4356`). Under `?tone=team` they
become the live team colours with those greys as fallbacks. The aliases live on
`.background`, not on `:root`, so team colours cannot collide with the root
theme overrides.

### 7.3 Variants

OBS preserves a query string on a Browser Source URL, so the variants are
parameters rather than extra pages:

- `?motion=still` adds `motion-still` to `<html>` and sets `animation: none`,
  keeping a composed first frame. `?motion=drift` is not a separate mode; any
  other value leaves the drift running.
- `?tone=team` adds `tone-team` and swaps the two wash aliases. The plate stays
  the theme surface colour.
- `?bg=transparent` adds `data-bg="transparent"` and removes the plate while
  keeping the glow and both lattices. Opaque is this page's default; the
  `off` / `on` aliases normalise to opaque / transparent as they do in the
  scene overlays.
- `?host=` and `?port=` retarget the WebSocket, as in the other overlays.
- `@media (prefers-reduced-motion: reduce)` freezes the motion independently of
  the query string. The OS preference wins.

The query string is read once at load. Changing a variant needs a reload or an
OBS source refresh. The `bg` mode is the exception: it can also be changed live
by the cached `background` section described in §8.6.

## 8. The canonical FX block

### 8.1 Why it is copied

The owner asked for the background treatment to be "built into the existing
ones" as well as shipped separately. There is no shared CSS URL in this
project, so one canonical block is inlined in each target overlay between two
sentinel comments:

```html
<!-- === FGC CANONICAL FX START === -->
<!-- === FGC CANONICAL FX END === -->
```

`crates/server/tests/overlays.rs` has
`every_overlay_carries_the_same_fx_block`, which extracts the block from each
file and compares it byte for byte. The test also asserts that the block carries
the shared mask, the exact one-tile drift, the overscan, `will-change`, the
toggle and the reduced-motion rule. Source-text tests are fragile against
unrelated markup edits; that is the known cost of avoiding a bundler. A change
to one copy must be a change to all copies, and the test is the gate.

### 8.2 The rules

The canonical layer is a transparent sibling of the overlay root:

```css
.fgc-fx {
  position: absolute;
  inset: -360px;
  z-index: 0;
  pointer-events: none;
  background-color: var(--fgc-line-strong);
  opacity: 0.16;
  -webkit-mask-image: var(--fgc-hex-mask);
  mask-image: var(--fgc-hex-mask);
  -webkit-mask-size: 180px 322px;
  mask-size: 180px 322px;
  -webkit-mask-repeat: repeat;
  mask-repeat: repeat;
  -webkit-mask-position: center;
  mask-position: center;
  transform: translate3d(0, -322px, 0);
  animation: fgcCanonicalFxDrift 72s linear infinite;
  will-change: transform;
}
```

The block also sets `.overlay { z-index: 1; }` so the existing content stays
above the layer. It does not paint a plate. The body of every overlay that
carries it remains `rgba(0, 0, 0, 0)`, so OBS can still composite the scene over
the standalone background.

The `inset: -360px` overscan exceeds one tile on every side. The colour is a
uniform wash, not a repeating gradient, because a gradient would move visibly
even when the lattice re-aligned.

### 8.3 Seamless drift

The keyframe travel is exactly one mask tile:

```css
@keyframes fgcCanonicalFxDrift {
  from { transform: translate3d(0, -322px, 0); }
  to { transform: translate3d(180px, 0, 0); }
}
```

The difference between the two states is `(180px, 322px)`, precisely the
`180px 322px` repeat unit. When the animation jumps back to the start, every
hexagon is in the place where its neighbour was, so the frame is identical.
Changing either number creates a visible seam. Changing the mask tile size
without changing both numbers creates a seam. The overscan hides the layer
edge; it does not make the drift seamless.

### 8.4 Toggle and reduced motion

The tiny inline script reads `new URLSearchParams(location.search).get('fx')`.
`?fx=off` sets `data-fx="off"` on `<html>` and
`html[data-fx="off"] .fgc-fx` sets `display: none`. `?fx=on` forces it on. With
no parameter, the layer is on unless the document element already carries
`data-fx="off"`. Server injection of that attribute is still the reserved hook
for the future app-level FX switch; the server does not inject it today.

The shipped app control is the separate background toggle described in §8.6.
It rewrites Browser Source URLs with a `bg` parameter and broadcasts a cached
`background` section that changes `data-bg` on connected pages. It does not
inject `data-fx`; that hook remains unbuilt.

`@media (prefers-reduced-motion: reduce)` sets `animation: none` on `.fgc-fx`.
The static lattice stays, so the composition still holds and the operator does
not get an empty overlay. OBS may not honour the OS preference in every build,
which is why the explicit `?fx=off` variant exists.

### 8.5 Scope on 2026-11-23

The block currently ships in eight overlays: `hero-bans.html`, `map-pool.html`,
`map-select.html`, `row-casters.html`, `row-hosts.html`, `row-interviews.html`,
`team-roster-home.html` and `team-roster-away.html`.

It does **not** ship in `timer.html` or `game-overlay.html`. The owner stated on
2026-11-23 that there should be no background overlay attached to either of
those. It also does not ship in `auto.html`, which is a preview host with its
own iframes, or in `background.html`, which already owns the opaque plate and
its richer three-layer motion.

The first native carrier is hero bans, because it already had the hexagon wall
and a proven release path. Calling it a pilot is a statement about rollout
order, not about adding FX to the timer.

### 8.6 Background mode

The owner then pointed out that only `background.html` had an opaque background
and asked for a toggle that turns transparency on or off, "including removing
the hexagons". A75 implemented that literally. The owner later clarified that
opaque mode should still show the hexagons, so A77 supersedes A75 and A76 on
this rule. Each of the eight carriers now has a second sibling inside the
canonical block:

```css
.fgc-bg {
  position: absolute;
  inset: 0;
  z-index: 0;
  pointer-events: none;
  display: none;
  background-color: var(--fgc-surface);
}

html[data-bg="opaque"] .fgc-bg { display: block; }
```

Transparent is the normal mode for a scene overlay: the body stays clear and
the drifting lattice is visible. Opaque mode paints the same theme plate that
the standalone background owns and keeps the hexagons on top of it. A77 removed
the `html[data-bg="opaque"] .fgc-fx { display: none; }` rule; a byte-equality
assertion now forbids its return. The plate is a sibling layer rather than a
body background, so the body continues to read `rgba(0, 0, 0, 0)` in both modes.
This keeps the overlay compositing contract intact and avoids coupling the new
mode to the theme's body rules.

The inline script reads `bg` from the query string as well as `fx`. The accepted
values are `opaque` and `transparent`, with `off` and `on` as aliases. Scene
overlays default to transparent; `background.html` defaults to opaque and gets
the inverse `?bg=transparent` variant, which drops its plate but keeps the glow
and both lattices. `auto.html` forwards the mode to its scene iframes. The two
attributes are independent: opaque adds the plate under the lattice but does
not rewrite `data-fx`.

The app's Overlays view has a session-only `Background` switch in a `ToggleRow`.
It uses `overlay_url_with_background()` to append `?bg=opaque` or
`?bg=transparent` to the ten URLs in `BACKGROUND_MODE_FILES`: the eight carriers,
the standalone background and the auto host. Timer and game overlay are not in
that list. Changing the switch also broadcasts a cached `background` section,
`{"type":"background","data":{"opaque":true}}` or its `false` form. Every
modes-aware overlay registers a handler inside its canonical block, or an
equivalent branch in `background.html`, that writes the boolean to `data-bg` on
the document element. Connected Browser Sources therefore switch straight away,
and a source opened later receives the cached section in its init frame. A77
fixed the refresh gap: the live frame is
`{"type":"background","data":{"opaque":true}}`, while the keyed init frame is
`{"type":"init","data":{"background":{"opaque":true}}}`. The handler reads
`payload.background.opaque` from the init frame and `data.data.opaque` from the
live frame, so a Browser Source refresh restores the app-selected mode. URL
rewriting is kept as the starting state for new sources, not as the only
delivery mechanism.

Adding the plate inside the sentinel comments means the byte-equality test now
covers both layers. The test also asserts exactly one `.fgc-bg` and one
`.fgc-fx` per carrier, the remaining `data-bg` rule, the aliases, the init-aware
handler and the absence of the old hide rule.
Separate source-text tests keep the mode out of timer and game overlay, give
the standalone page its transparent variant, and check the auto forwarding.

A78 added one document-level rule to all twelve overlays, immediately after the
first `<style>` and outside the canonical sentinels:

```css
/* Critical document guard: contain overscanned FX/lattice layers during load so transient scrollbars never appear. */
html, body { width: 100%; height: 100%; margin: 0; overflow: hidden; }
```

The FX layer overscans by `-360px`. Before A78, the fixed-size `.overlay` had
`overflow: hidden`, but the document itself did not, so a Browser Source
refresh could show scrollbars for a split second. The early guard clips the
document from first paint. `every_overlay_has_an_early_document_overflow_guard`
asserts that the rule is present in every overlay before any `.fgc-fx` rule or
iframe, so it cannot be lost to a copy edit.

## 9. Per-overlay playbook

The order below is the council's rollout order from 2026-11-23. Each item is a
separate change with its own QA row and runbook section. The FX column describes
the shared transparent lattice, not the opaque standalone background. Every FX
carrier can also take the §8.6 background mode: transparent by default, or
opaque with a plate under the same hexagons. `background.html` starts opaque and
can be made transparent. Timer and game overlay have no background mode at all.

| Overlay | Purpose | Hexagon use | Colour rule | Motion | FX state |
|---|---|---|---|---|---|
| `background.html` | persistent backdrop beneath every scene | two drifting lattices, the product's mask alone | solid opaque `--fgc-surface` plate; `?tone=team` only for the washes | glow 84s, forward 88s, reverse 72s | not applicable; it is the richer standalone treatment |
| `hero-bans.html` | full-bleed ban moment | logo 100×115, role 60×68, wall and grid | outer-edge team wash, neutral ink on art, no seam | 0.8s reveal, 0.9s halves | shipped 2026-11-23 |
| `timer.html` | non-game countdown and lower-third bug | no FX hexagon; planned hex plate | transparent body, theme text | no reveal delay, no per-frame clock | explicitly excluded by owner |
| `game-overlay.html` | chrome around the game capture | planned logo chips and score plate; no centre bar | dual outer-edge washes, neutral centre | 0.8s reveal when refreshed | explicitly excluded by owner |
| `map-pool.html` | map grid by mode | planned status chips, never on thumbnails | outer-edge wash, staggered rows | 0.35s reveal planned | shipped transparent FX |
| `map-select.html` | vertical map slots and ban thumbnails | planned name plate and vote chips; thumbnails stay rectangular | bottom outer-corner washes | 0.35s reveal planned | shipped transparent FX |
| `row-casters.html` | caster row | planned hexagon portrait frames and role chips | per-card outer-edge wash | 0.35s reveal planned | shipped transparent FX |
| `row-hosts.html` | host and analyst row | planned portrait frames and role chips | per-card outer-edge wash | 0.35s reveal planned | shipped transparent FX |
| `row-interviews.html` | interviewer and guest row | planned portrait frames and role chips | per-card outer-edge wash | 0.35s reveal planned | shipped transparent FX |
| `team-roster-home.html` | home core and substitutes, ordered Damage/Tank/Support | 56×56 core role hexagon inside the portrait; 46×54 substitute hexagon with a 34px icon; no side rails | one full-width bottom gradient from the window edge to edge at alpha 0.08, with a blurred alpha 0.09 glow layer; no side-origin wash | 0.35s reveal planned | shipped transparent FX and A81 refinement |
| `team-roster-away.html` | away core and substitutes, ordered Damage/Tank/Support | mirrored 56×56 role hexagon and the same 46×54 substitute hexagon; no side rails | the same full-width bottom gradient and blurred glow; no home or away colour anchoring | 0.35s reveal planned | shipped transparent FX and A81 refinement |
| `auto.html` | preview host for the scene switcher | planned hexagon preview cards | previews load `?fx=off` | scene swaps only | excluded; previews default FX off |

### 9.1 Planned per-overlay refinements

These are agreed ideas, not changes in the tree:

- **Game overlay.** Replace the centre bar with a hexagon score plate and logo
  chips, keep the centre clear, and wash team colour in from both outer edges.
- **Map pool.** Keep the rectangular table, add hexagon status chips, an outer
  wash and staggered rows. Do not clip the map thumbnails.
- **Map select.** Add a hexagon name plate, hexagon vote chips and a hexagon
  clipping only if it does not cut the corners off the thumbnail. Put both team
  washes at the bottom outer corners.
- **Talent rows.** Add hexagon portrait frames and role chips, a per-card outer
  wash, and reuse `subjectBox()` where a transparent portrait is supplied.
- **Timer.** Add a hexagon plate and keep tabular figures; no per-frame
  JavaScript and no FX layer.
- **Auto.** Add hexagon preview cards and load every preview with `?fx=off`, so
  the preview and live source never render the effect twice.

## 10. Broadcast safety and accessibility

### 10.1 Platform floor

The floor is OBS 31.0.4 with CEF 116. That is the reason `color-mix()` is safe
for the hero bans edge wash and why `mask-image` and `translate3d()` are used
directly. A feature that needs a newer Chromium cannot be added without raising
the floor and recording that in the decisions log.

### 10.2 Compositor cost

A Browser Source may run for hours. Every ambient effect must be cheap in the
steady state: CSS animation on `transform` or `opacity`, no per-frame
JavaScript, no layout thrash, no growing timer or event list. `will-change` is a
hint, not a substitute for a cheap animation, and belongs only on the element
that moves.

The standalone background renders the whole plate and lattice once; the FX
layer adds a second lattice over a scene. Preview and live sources can both
render it, which is why the planned `auto.html` previews load `?fx=off`.

### 10.3 Reduced motion

Both motion systems freeze on `@media (prefers-reduced-motion: reduce)` while
keeping their static lattice. `?motion=still` does the same for the background
and `?fx=off` removes the FX layer completely. The explicit variants matter
because OBS may not pass the OS preference through to a Browser Source in every
configuration. Reduced motion must never mean "empty overlay".

### 10.4 Contrast

Text over artwork is neutral and sits on a plate. Team colour is reserved for
edges and identity marks. Orange marks one state. The Light theme is a first
class theme, not an afterthought; the background plate was verified in both
themes in A73. Any new text treatment has to be checked against both the dark
and light token sets before it ships.

### 10.5 Long-broadcast hygiene

No animation may depend on a wall clock, `Date.now` or a `setInterval` that
accumulates state. The timer renders broadcast ticks from WebSocket messages.
Theme and team colours are always removed when absent from a later payload.
The WebSocket reconnects with a bounded `setTimeout`, not a tight loop.

## 11. How a change is verified

A visual change is not done when the HTML looks right in one browser. The
discipline established during this pass has four artefacts:

1. A technical section in `docs/overlay-refresh-specs.md`, using the pattern of
   §1.13, §1.14, §11 and §12: purpose, wire behaviour, surface plan, motion or
   interaction, URL variants and CDP-verifiable assertions.
2. A row in `docs/manual-qa.md` §7.1 and a sign-off entry in §7.2. The frozen
   v1.0.0 sign-off in §5 is never edited for post-v1 work.
3. A runbook section in `docs/manual-qa-runbook.md` with Covers, Steps, Gotchas,
   Evidence and Result, marked Pass only for what was actually observed.
4. A source-text test in `crates/server/tests/overlays.rs` or the relevant
   focused test file. The test mirrors the rule that cannot be checked by
   rendering, such as byte equality of the copied FX block.

Browser checks use the real local overlay server on `127.0.0.1:3001` and read
computed styles and `getAnimations()` through in-page JavaScript. Values from
`getTiming()` need care: `iterations` reads `null` in JSON, which is the
infinite loop, and `easing` reads `linear` when `ease-in-out` was declared in
the animation shorthand, because that easing lands on the keyframes rather than
on the effect timing. Both details are recorded so a later QA pass does not
report a false failure.

Source-text tests are brittle by design. They break when unrelated markup is
reformatted, and that is the price of guarding a copied block without a build
step. When a test breaks for that reason, fix the selector or the assertion
narrowly; do not weaken the equality rule.

## 12. Open ideas, risks and roadmap

### 12.1 Ordered roadmap

1. Inline the shared token block in every overlay. Done.
2. Ship the canonical FX block and the standalone background. The background is
   done; the FX block is in eight overlays as of 2026-11-23.
3. Guard the copies with the byte-equality test. Done.
4. Pilot the FX in the first native carrier, hero bans. Done.
5. Refresh game overlay and hero bans together visually. Planned; game overlay
   keeps no FX layer.
6. Roll the FX to the rest of the overlays. Done for the eight listed in §8.5;
   timer and game overlay stay excluded and auto stays off.
7. Reuse `subjectBox()` in other overlays that receive transparent artwork.
   Planned.
8. Run the per-overlay design refreshes from §9.1, one overlay per change.
   Rosters are done in A79 and refined again in A80; the remaining overlays are
   still planned.
9. The app-level background toggle - done as a URL rewrite plus a cached
   `background` section. It appends `?bg=` to the ten modes-aware Browser Source
   URLs for new sources, broadcasts `data-bg` changes to connected pages, and
   replays the cached mode in the keyed init frame, so no OBS refresh is needed
   and a refresh cannot reset the mode. Opaque keeps the hexagons over the
   plate. The separate app-level FX toggle that injects `data-fx` is still
   planned.
10. The early document overflow guard - done for all twelve overlays. It stops
    the transient scrollbars seen on Browser Source refresh (A78).
11. Keep this document, the specs, the checklist, the QA register, the runbook,
    the decisions and the backlog in step with each change. Continuous.

### 12.2 Risk register

| Risk | State today | Mitigation |
|---|---|---|
| OBS GPU cost over a long broadcast | two lattice layers at most, one per source | transform/opacity only, 72s+ cycles, `will-change` on one layer, `?fx=off` |
| A visible drift seam if the SVG does not tile on 180×322 | mitigated, not hypothetical | drift travels exactly one tile; overscan hides the layer edge; equality test guards the numbers |
| OBS may ignore `prefers-reduced-motion` | accepted platform risk | `?motion=still` and `?fx=off` give an explicit operator control |
| The alpha-scan locator is fragile against soft edges and drop shadows | accepted for the current transparent art | threshold is recorded as alpha above 8; `object-fit: cover` fallback for an unreadable canvas |
| Neutral ink on very light artwork | plausible | `--fgc-ink-on-art` sits on `--fgc-plate-dark`, never directly on the art |
| Hexagon clipping cuts the corners off a logo | plausible in the planned refreshes | keep the inscribed band; inspect at 100×115 and 60×68 before rollout |
| Table readability | guarded by the shape rule | no hexagons on table rows or map thumbnails |
| A core role badge crosses back into the name plate | fixed by A79, retained in A80 | the badge is 56×56 at `bottom: 16px`, the plate is 126px, and source-text plus browser probes require a negative overlap; the old 76px badge at `bottom: -36px` is forbidden |
| A substitute hexagon looks squashed | fixed by A80 | the badge is 46×54 with a 34×34 icon and an 8px inset inside the 72×76 portrait; source-text assertions require those exact dimensions, and the browser probe measured a 0.852 width/height ratio with 0px centre offset |
| Divider-style rails return around the card row | removed by A80 | neither `.core-stage` nor any `.core-rail` remains; source-text assertions forbid all four rail selectors, and the browser probe found zero rail nodes |
| Roster role order drifts away from the game's role list | fixed by A80 | `ROLE_RANK` orders damage, tank, support; `sortPlayersByRole()` applies a stable index tiebreak to both filtered rows, and the browser probe read DAMAGE, TANK, SUPPORT in core and sub order |
| The gradient returns to a side wash or gets clipped short of the frame edge | fixed by A81 | `.roster-wash` is now `left: 0; right: 0; bottom: 0; height: 420px` with `linear-gradient(0deg, var(--fgc-wash), transparent 100%)`; `::after` uses the same full-width edges, extends 100px below the frame and blurs 24px. Source-text assertions forbid the 760px wash, 90/270deg side gradients and side-specific wash rules, while browser probes measured 1920px-wide wash and glow boxes on both sides |
| Country flag or team tag code returns as a half-finished identity chip | deferred by the owner in A79 | no `player-code` or `teamCode` remains; source-text assertions forbid them, and `docs/backlog.md` records the idea for a later decision |
| Source-text tests break on unrelated markup edits | known cost | narrow the assertion; never remove the equality check |
| Copy-paste drift between duplicated blocks | guarded | sentinel comments and `every_overlay_carries_the_same_fx_block` |
| Preview and live sources render FX twice | planned | `auto.html` previews load `?fx=off` |
| App-level toggle does not reach a connected source | mitigated for `bg`, open for `data-fx` | the `bg` switch broadcasts a cached section so connected and newly connected sources switch live; A77 also replays it in the keyed init frame so a refresh keeps the mode; the future `data-fx` injection must use the same pattern |
| Opaque mode loses the product's hexagon identity | fixed by A77 | no hide rule inside the canonical block; the byte-equality test forbids `html[data-bg="opaque"] .fgc-fx` from returning |
| Transient scrollbars appear during a Browser Source refresh | fixed by A78 | the early `html, body { ... overflow: hidden; }` guard clips the document from first paint; a source-text test and a zero-gutter browser probe guard it |
| OBS version floor | 31.0.4 | no feature that needs a newer Chromium without a decision |
| Schedule realism | open | one overlay or one mechanism per change, each with its four artefacts |

### 12.3 Ideas not yet scheduled

- A stinger transition that hands off to the 0.8s reveal without a visible gap.
- A flythrough moment overlay for match introductions.
- A single app switch for every overlay's FX, backed by `data-fx` injection.
  The shipped `bg` toggle already uses the cached-section pattern, but it only
  changes `data-bg` and does not cover this.
- A country flag or country code in the roster name plate. The owner asked to
  experiment with it in A79, then said to scrap it and revisit it later in the
  backlog. No country or nationality field exists in `Player`, `PlayerDto` or
  `TeamDto`; a real implementation needs a new payload field and a separate
  decision with its own QA gate.
- A locator helper shared by more than one cut-out overlay.
- A visual regression capture per overlay, if a stable OBS capture harness can be
  built without making CI slow.

Ideas stay in the backlog until they have a QA row and a runbook section. The
purpose of this document is to keep a good idea from being lost, not to pretend
it has shipped.

## 13. Change log

| Date | Change |
|---|---|
| 2026-11-22 | Original Night Broadcast design pass and the resolved owner review recorded in `docs/overlay-refresh-design.md`. |
| 2026-11-23 | Hero bans cut-outs, edge wash, subtler hexagons, neutral labels and the 0.8s reveal (A72, commit `2ba5650`). |
| 2026-11-23 | Standalone animated background with the solid theme plate and three CSS animations (A73, commit `9600a65`). |
| 2026-11-23 | Ten locked design decisions, the rollout plan and the full risk list written into `docs/decisions.md` and `docs/backlog.md`. |
| 2026-11-23 | Canonical transparent FX block added to eight overlays with a byte-equality test; timer and game overlay explicitly excluded by the owner (A74). |
| 2026-11-23 | This document opened as the canonical design-language reference after the owner asked for maximum written detail. |
| 2026-11-23 | Background mode toggle added: the first implementation hid the hexagons in opaque mode, `background.html` gained the inverse transparent variant, `auto.html` forwards the mode, and the app rewrites mode-aware Browser Source URLs (A75). |
| 2026-11-23 | The mode toggle became live: a cached `background` section changes `data-bg` on connected sources, the app control became an accessible `Switch` in a `ToggleRow`, and no OBS source refresh is needed (A76). |
| 2026-11-23 | A77 superseded A75/A76 on the lattice rule: opaque keeps the hexagons over the plate, and the cached `background` section is parsed from the keyed init frame so a Browser Source refresh restores the selected mode. |
| 2026-11-23 | A78 added an early `html, body` overflow guard to all twelve overlays, stopping the transient scrollbars seen on refresh; a source-text test and browser gutter probe record the result. |
| 2026-11-23 | A79 refined both rosters together: wider wash and bottom glow, smaller core role badge fully inside the portrait, divider-style vertical rails outside the card row, taller plate with smaller pronouns, and removal of the country/tag code chip experiment. |
| 2026-11-23 | A80 superseded A79's rails and side-anchored glow: both rosters dropped the rails, used a 46×54 substitute hexagon with a 34px icon, ordered core and substitutes Damage/Tank/Support, softened the wash to 12% and the glow to 14%, and stretched the glow across the full 1920px bottom of the frame. |
| 2026-11-23 | A81 replaced the remaining A80 side wash with one full-width bottom gradient, then softened it again after the owner said it looked harsh: wash alpha 0.08, glow alpha 0.09, 420px gradient and a 300px blurred glow layer. |
