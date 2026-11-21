# IDE-021: Assets and data roadmap (§C)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from the unnumbered `docs/backlog.md` §C catalogue. Bullets are grouped by section; no item was invented beyond the source text.

- Further image optimisation (WebP/AVIF) beyond the shipped hash-keyed lazy thumbnail cache.
- Relative/portable asset storage; per-show asset packs.
- SQLite option if JSON becomes a bottleneck (probably not needed).
- Backup rotation, restore points, cloud/network sync.
- Schema migrations with downgrade protection.

**Status: triaged.** This is an umbrella record for the section; split into individual Doctrine items when a bullet is promoted to design or implementation.

Items already promoted separately (for example IMP-003, the D8 series, P1–P4 and B4/B5) are listed here only as cross-references.
