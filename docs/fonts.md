# Font decision — replacing Apotek

## Why replace it

Apotek is a commercial typeface by **Kerns & Cairns** (distributed via Adobe Fonts and Type Network). The OTFs in the repo are not licensed for redistribution or self-hosted webfont use. Bundling them in a new Rust binary would require an app/self-hosting licence. Replacing Apotek with an OFL font removes that risk and the PenPot dependency in one step.

## Requirements

- Free for commercial use, including embedding in a distributed desktop app and a local web server.
- OFL (or equivalent) so we can bundle, subset and self-host.
- Looks good in **all caps** at broadcast sizes.
- Professional esports/broadcast character — not a novelty sci-fi face.
- Enough weights to replace the five existing families: Light, Regular, Medium, SemiBold, Black.
- Ideally a width axis for long team names and tight layouts.

## Shortlist

| Font | Licence | Weights / widths | Character | All-caps | Verdict |
|---|---|---|---|---|---|
| **Saira** | OFL 1.1 | 100–900, 50–125 width (variable); 9 weights × 7 widths | Squared, technical grotesque; superellipse curves | Excellent; sturdy caps, good numerals | Initial recommendation; superseded by R1 (Archivo selected). Retained as a reversible fallback |
| **Archivo** | OFL 1.1 | 100–900; normal/semi-expanded/expanded | Neutral American grotesque; high-impact | Excellent; very professional, less "gamer" | **Selected** in R1 and shipped in v1 |
| **Barlow** | OFL 1.1 | 9 weights; normal/semi-condensed/condensed | DIN-like, technical, sporty | Very good; condensed variants suit jerseys/scoreboards | Good alternative; slightly narrower than Apotek |
| **Titillium Web** | OFL 1.1 | 200–900 | Clean, broadcast/scientific | Good; a little generic | Reliable but less distinctive |
| **Bebas Neue** | OFL 1.1 | 1 weight, all-caps display | Tall condensed display | Excellent for headlines only | Not suitable for rosters/body text; could be a secondary display face |
| **Rajdhani** | OFL 1.1 | 300–700, condensed techno | Very "esports" | Good, but dated/techy and limited weights | Not recommended as the primary face |
| **Chakra Petch** | OFL 1.1 | 6 weights, squared techno | Futuristic | Good for titles, less so for body | Not recommended as the primary face |

## Decision: Archivo selected (Saira retained as reversible fallback)

The first OBS pass (R1) found that Saira does not read like Apotek. **Archivo**
was selected and shipped in all seven v1 overlays. M9-5 is Done and A41 Pass
confirmed the shipped rendering at 1920×1080. Saira remains bundled so a revert
is a five-file swap, not because the choice is still open.

- **Licence:** SIL Open Font License 1.1 — commercial use, embedding,
  self-hosting, subsetting and modification are allowed; derivatives stay OFL;
  the font cannot be sold by itself.
- **Why Archivo:** it is a neutral American grotesque with a high-impact all-caps
  cut. It sits closer to Apotek's broad, unornamented caps than Saira's squared,
  technical character, and it holds up at broadcast sizes without looking like a
  sci-fi display face.
- **Source:** Google Fonts / Omnibus-Type for Saira, Google Fonts / Omnibus-Type
  for Archivo. `OFL.txt` (Saira) and `OFL-Archivo.txt` ship beside the files.

Saira was the M9 choice and remains the bundled fallback: a robust squared
grotesque with a huge weight/width system, strong all-caps, and a width axis for
long names. Its files and licence stay so a revert is the same five-file swap;
no A/B review remains open.

### Weight mapping

| Existing family | Archivo weight (current) | Saira weight (fallback) |
|---|---|---|
| `Apotek Light` | Archivo Light 300 | Saira Light 300 |
| `Apotek` (regular) | Archivo Regular 400 | Saira Regular 400 |
| `Apotek Medium` | Archivo Medium 500 | Saira Medium 500 |
| `Apotek Semibold` | Archivo SemiBold 600 | Saira SemiBold 600 |
| `Apotek Black` | Archivo Black 900 | Saira Black 900 |

Use static WOFF2 instances (300/400/500/600/900) for predictable rendering in
WebView2/OBS. Static files are simpler to reason about and avoid variable-font
edge cases in older WebView2 builds; bundle a variable file only if needed.

### Type scale in the overlays

The bundled overlays share one type scale, so a text role always looks the same
whichever overlay it appears in. **The alias family carries the weight and a
rule never declares a `font-weight` of its own**, because a rule that names a
family it does not own silently loses the weight it asks for.

| Tier | Weight | Alias | Used for |
|---|---|---|---|
| Display | 900 | `FGC Display Black` | The biggest headline on a screen: the map pool header, the map select title, a roster team name, the hero bans hero name |
| Strong | 600 | `FGC Display SemiBold` | A score, a count or a map name: the game overlay score and title, the map select name and score, the map pool map name, the timer value |
| Body | 500 | `FGC Display Medium` | A person's or team's name, and every label: the game overlay name, the hero bans ban order and role tag, the roster and talent names, social role tags |
| Secondary | 400 | `FGC Display` | Small supporting text that should recede |
| Quiet | 300 | `FGC Display Light` | Handles, pronouns and separators |

This replaced a set of rules that asked for a weight their family did not
have. The game overlay's bar name and bar score, the title, the subtitle, the
map number and the format label all inherited `body { font-family: 'FGC
Display' }`, which only ships the 400 face, so every one of them rendered at
Regular whatever the rule asked for. The timer overlay had the same problem
inverted: its `@font-face` declared the regular alias at weight 600, so it
hijacked the alias every other overlay used for the 400 face. It now declares
`'FGC Display SemiBold'`.

Four static guards in `crates/server/tests/overlays.rs` keep the scale honest:
no `font-weight` may appear outside an `@font-face`, every declared face must
be a tier and load the matching Archivo file, the nine match overlays must
ship all five faces, and each role must resolve to the same weight in every
overlay that uses it.

### CSS migration

1. Inline the five `@font-face` rules in each overlay, pointing at stable
   `/assets/fonts/archivo-<weight>.woff2` paths. No shared `fonts.css` in v1, so
   each overlay remains independently replaceable.
2. Rename the families to neutral aliases: `FGC Display Light`, `FGC Display`,
   `FGC Display Medium`, `FGC Display SemiBold`, `FGC Display Black`.
3. Update every `font-family` declaration to the matching alias. Fallback stack:
   `'FGC Display', 'Apotek', sans-serif`.
4. For user-supplied overlay overrides, document the family names. Existing
   copies that still reference `Apotek` fall back to Archivo only if an `Apotek`
   alias rule is added; v1 does not ship that alias.
5. Reverting to Saira, or trying Barlow, means replacing the five WOFF2 files
   and the `@font-face` `src:` URLs only; the family aliases and all layout CSS
   stay unchanged.

See `docs/assets.md` §4 for the exact `@font-face` block and file list.

### UI font

Bundle **Inter** (OFL) locally for the control panel and remove the Google Fonts `@import`. Alternatively use a system stack (`Segoe UI Variable`, `Segoe UI`, `system-ui`) to avoid shipping a second family. Recommendation: bundle Inter — it keeps the current UI look and is OFL.

## Visual QA checklist

Signed off by R1/M9-5 and A41; R2 covered the offline item. The boxes are kept
as the reusable acceptance checklist.

- [ ] All 7 overlays at 1920×1080, transparent background, in OBS.
- [ ] Long team names and tags: no overflow or clipping; test names up to the
      32-character team / 24-character player limits. Text shrinks to a floor,
      then ellipsizes; it never wraps.
- [ ] Scores and map numbers: tabular numerals line up (`font-feature-settings: "tnum"`).
- [ ] All-caps headings: cap height and letter-spacing match the current design closely.
- [ ] Rosters: player names, roles, pronouns; portrait layouts unaffected.
- [ ] Hero bans and map pool: mode names and map names fit their cards.
- [ ] Timer (once built): digits do not jitter; leading-zero setting works.
- [ ] Offline test: disconnect the network, restart OBS, confirm fonts still load.
- [ ] Licence file present in the bundle and referenced in About/README.

## Review outcome

R1 selected **Archivo** over Saira; M9-5 is Done and A41 Pass signed off all
seven overlays at 1920×1080. Both WOFF2 sets and licence files stay in the repo
so a revert is reproducible. The overlay test in `crates/server/tests/overlays.rs`
pins the active prefix (`FONT_PREFIX`), so reverting is a one-line change plus
the `@font-face` `src:` URLs; the neutral aliases mean no layout CSS change.
**Barlow** remains an untried third option, not an open checkpoint.

## Fallback plan

If a future change needs a different face, revert to **Saira** (already bundled)
or try **Barlow** (sportier, more condensed). Either migration is the same: swap the
five WOFF2 files, update the five `@font-face` `src:` URLs and the
`FONT_PREFIX` test constant, and keep the family aliases. Do not ship Apotek
unless a redistribution licence is purchased.
