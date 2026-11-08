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
| **Saira** | OFL 1.1 | 100–900, 50–125 width (variable); 9 weights × 7 widths | Squared, technical grotesque; superellipse curves | Excellent; sturdy caps, good numerals | **Recommended** — closest to Apotek's robust, slightly squared feel, and the width axis is useful for long tags/names |
| **Archivo** | OFL 1.1 | 100–900; normal/semi-expanded/expanded | Neutral American grotesque; high-impact | Excellent; very professional, less "gamer" | Safe alternative if Saira feels too technical |
| **Barlow** | OFL 1.1 | 9 weights; normal/semi-condensed/condensed | DIN-like, technical, sporty | Very good; condensed variants suit jerseys/scoreboards | Good alternative; slightly narrower than Apotek |
| **Titillium Web** | OFL 1.1 | 200–900 | Clean, broadcast/scientific | Good; a little generic | Reliable but less distinctive |
| **Bebas Neue** | OFL 1.1 | 1 weight, all-caps display | Tall condensed display | Excellent for headlines only | Not suitable for rosters/body text; could be a secondary display face |
| **Rajdhani** | OFL 1.1 | 300–700, condensed techno | Very "esports" | Good, but dated/techy and limited weights | Not recommended as the primary face |
| **Chakra Petch** | OFL 1.1 | 6 weights, squared techno | Futuristic | Good for titles, less so for body | Not recommended as the primary face |

## Decision: Saira (confirmed)

- **Licence:** SIL Open Font License 1.1 — commercial use, embedding, self-hosting, subsetting and modification are allowed; derivatives stay OFL; the font cannot be sold by itself.
- **Why:** it is a robust, squared grotesque with a huge weight/width system. It reads as technical and competitive without being a sci-fi cliché, and it holds up in all caps at 1920×1080. The width axis lets us tighten long team names without switching families.
- **Source:** Google Fonts / Omnibus-Type (`github.com/Omnibus-Type/Saira`). Ship `OFL.txt` alongside the font files.

### Weight mapping

| Existing family | Saira weight |
|---|---|
| `Apotek Light` | Saira Light 300 |
| `Apotek` (regular) | Saira Regular 400 |
| `Apotek Medium` | Saira Medium 500 |
| `Apotek Semibold` | Saira SemiBold 600 |
| `Apotek Black` | Saira Black 900 |

Use static WOFF2 instances (300/400/500/600/900) for predictable rendering in WebView2/OBS, or a single variable WOFF2 with `font-weight: 100 900` and `font-stretch: 50% 125%`. Static files are simpler to reason about and avoid variable-font edge cases in older WebView2 builds; bundle both only if needed.

### CSS migration

1. Inline the five `@font-face` rules in each overlay, pointing at stable `/assets/fonts/saira-<weight>.woff2` paths. No shared `fonts.css` in v1, so each overlay remains independently replaceable.
2. Rename the families to neutral aliases: `FGC Display Light`, `FGC Display`, `FGC Display Medium`, `FGC Display SemiBold`, `FGC Display Black`.
3. Update every `font-family` declaration to the matching alias. Fallback stack: `'FGC Display', 'Apotek', sans-serif`.
4. For user-supplied overlay overrides, document the family names. Existing copies that still reference `Apotek` fall back to Saira only if an `Apotek` alias rule is added; v1 does not ship that alias.
5. Swapping Saira for Archivo or Barlow later means replacing the five WOFF2 files only; no CSS change.

See `docs/assets.md` §4 for the exact `@font-face` block and file list.

### UI font

Bundle **Inter** (OFL) locally for the control panel and remove the Google Fonts `@import`. Alternatively use a system stack (`Segoe UI Variable`, `Segoe UI`, `system-ui`) to avoid shipping a second family. Recommendation: bundle Inter — it keeps the current UI look and is OFL.

## Visual QA checklist

- [ ] All 7 overlays at 1920×1080, transparent background, in OBS.
- [ ] Long team names and tags: no overflow or clipping; test 20+ character names.
- [ ] Scores and map numbers: tabular numerals line up (`font-feature-settings: "tnum"`).
- [ ] All-caps headings: cap height and letter-spacing match the current design closely.
- [ ] Rosters: player names, roles, pronouns; portrait layouts unaffected.
- [ ] Hero bans and map pool: mode names and map names fit their cards.
- [ ] Timer (once built): digits do not jitter; leading-zero setting works.
- [ ] Offline test: disconnect the network, restart OBS, confirm fonts still load.
- [ ] Licence file present in the bundle and referenced in About/README.

## Review checkpoint

Once the rebuilt overlay HTML is rendering in OBS, compare **Saira**, **Archivo** and **Barlow** side by side at 1920×1080. Keep all three specimen sheets and licence files in the repo so the comparison is reproducible. If Saira does not fit, switching is a five-file WOFF2 swap; the neutral aliases mean no CSS change.

## Fallback plan

If Saira does not fit visually after the QA pass, try **Archivo** (neutral, professional) or **Barlow** (sportier, more condensed). The migration is the same: swap the five WOFF2 files and keep the family aliases. Do not ship Apotek unless a redistribution licence is purchased.
