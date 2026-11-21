# ISS-018: M9-7 — six-caster row clips social row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from `docs/manual-qa.md`, M9-7.

The six-caster `row-casters` layout clipped the social row with long handles. Real WebView2 measured `+26.75px` right / `-96.92px` left for `@anextremelylongcastersocialhandle` and `-23.67px` left for `@longcasterhandlefive`. It was recorded as a known overlay-polish issue rather than a v1 fix.

After v1 closeout the owner approved a scoped fix that preserves the overlay protocol, `app.json` schema and 1920×1080 geometry. `.talent-subtext` is confined to the card content box, `.social-icon` stays fixed at 20px, and long handles and pronouns ellipsize while the handle span keeps the full value in `title`. Gate A53 verified 1-, 4- and 6-caster states in real WebView2: every social-row child stayed inside its card, long text ellipsized, full handles remained in `title`, card widths and document size were unchanged, and there were zero page console errors.

**Status: closed · fixed.** Verified by `crates/server/tests/overlays.rs::row_casters_social_row_stays_inside_the_card`, the full Rust gate set, `dx build --desktop`, and A53 in `docs/manual-qa.md` / `docs/manual-qa-runbook.md`.
