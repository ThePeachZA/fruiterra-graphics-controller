# CHR-004: Font A/B review checkpoint

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/backlog.md` §D, “Font review (after the R1 re-run)”.

R1 selected Archivo over Saira. M9-5 is Done and A41 Pass signed off all seven
overlays at 1920×1080; R2 confirmed offline rendering. Saira stays bundled as a
reversible fallback, and Barlow remains untried. A future swap is five WOFF2
files and `src:` URLs plus the `FONT_PREFIX` test constant; no layout CSS change.

**Status: closed · done.** The checkpoint is resolved. `docs/fonts.md`,
`docs/backlog.md` §D, `docs/manual-qa.md` (R1-11 and the sign-off table),
`docs/assets.md`, `docs/architecture.md`, `docs/decisions.md`,
`docs/implementation-plan.md` and `docs/wiki/FAQ.md` now describe Archivo as
selected rather than a trial or open A/B review. No font files, overlays,
protocol or server code changed.
