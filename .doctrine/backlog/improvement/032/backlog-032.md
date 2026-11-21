# IMP-032: Placeholder TAG image for empty slots

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from the deferred overlay-refresh note in `docs/backlog.md` (A9 section).

When no logo is set, show a placeholder image containing the team’s TAG or another abbreviation so the slot is never empty. This was explicitly not implemented in the A9 fix.

After v1 closeout the owner approved this as the next scoped overlay polish. The game overlay now renders a text placeholder in its existing 75×55 logo slot when a team has no logo: the tag is used first, blank tags fall back to a short name abbreviation, long tags ellipsize with the full value in `title`, and the colour contrasts with the logo background. The placeholder hides as soon as a real logo is supplied and clears for unassigned teams. No protocol, schema, persisted state or asset changed.

Gate A54 verified the live server `init` plus seven replayed states in real WebView2 at 1920×1080: `T3`/`T1` from the fixture, partial-frame preservation, logo-present hiding, `LONGCASTERTAG` ellipsis at 63/227px, `NS` and `王小` fallbacks, null-team clearing, and white/dark contrast colours. Document and logo geometry stayed unchanged with zero page console errors.

**Status: closed · done.** Verified by `crates/server/tests/overlays.rs::game_overlay_shows_a_tag_placeholder_for_empty_logo_slots`, the full Rust gate set, `dx build --desktop`, and A54 in `docs/manual-qa.md` / `docs/manual-qa-runbook.md`.
