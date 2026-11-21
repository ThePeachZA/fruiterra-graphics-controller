# IDE-018: Overlays and presentation roadmap (§C)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from the unnumbered `docs/backlog.md` §C catalogue. Bullets are grouped by section; no item was invented beyond the source text.

- Further overlay polish (§C1): fit, spacing and long-name handling without changing design or protocol.
- Animate-in/out and show/hide API beyond `display` toggling; replay animations on demand.
- Live preview panel inside the app (iframe/WebView) for each overlay.
- Overlay theme/template system; scoreboard skins.
- Timer overlay with countup/countdown/target, alignment, leading zeroes and standalone mode (see B5).
- Tournament/bracket overlay (see P4).
- Lower-thirds, player intro, matchup, winner screens, ad/BRB loops.
- Per-show asset packs and overlay overrides folder.
- Font manager: user-selectable fonts, local by default.
- Localisation of overlay labels (for example FIRST TO, MAP).

**Status: triaged.** This is an umbrella record for the section; split into individual Doctrine items when a bullet is promoted to design or implementation.

Items already promoted separately (for example IMP-003, the D8 series, P1–P4 and B4/B5) are listed here only as cross-references.
