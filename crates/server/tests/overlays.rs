//! Static compatibility checks for the nine bundled overlays.
//!
//! These tests run against the same embedded bytes the release build serves, so
//! a missing overlay, an external network reference or an unported font fails
//! the build before the manual OBS checklist starts.

use fgc_server::{embedded_font, embedded_overlay};

struct OverlaySpec {
    file: &'static str,
    message_types: &'static [&'static str],
}

const OVERLAYS: [OverlaySpec; 9] = [
    OverlaySpec {
        file: "game-overlay.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'show'",
            "'theme'",
        ],
    },
    OverlaySpec {
        file: "hero-bans.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'heroBans'",
            "'show'",
            "'theme'",
        ],
    },
    OverlaySpec {
        file: "map-pool.html",
        message_types: &["'init'", "'scoreboard'", "'mapPool'", "'show'", "'theme'"],
    },
    OverlaySpec {
        file: "map-select.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'mapSelect'",
            "'show'",
            "'theme'",
        ],
    },
    OverlaySpec {
        file: "row-casters.html",
        message_types: &["castersData", "'show'", "'theme'"],
    },
    OverlaySpec {
        file: "row-hosts.html",
        message_types: &["castersData", "'show'", "'theme'"],
    },
    OverlaySpec {
        file: "row-interviews.html",
        message_types: &["castersData", "interviewee", "'show'", "'theme'"],
    },
    OverlaySpec {
        file: "team-roster-home.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'teamRoster'",
            "'show'",
            "'theme'",
        ],
    },
    OverlaySpec {
        file: "team-roster-away.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'teamRoster'",
            "'show'",
            "'theme'",
        ],
    },
];

/// Bundled font files are named `<prefix>-<weight>.woff2`.
///
/// The trial font is Archivo; switching back to Saira only changes this value
/// and the `@font-face` blocks.
const FONT_PREFIX: &str = "archivo";

const FONTS: [(&str, &str); 5] = [
    ("300", "FGC Display Light"),
    ("400", "FGC Display"),
    ("500", "FGC Display Medium"),
    ("600", "FGC Display SemiBold"),
    ("900", "FGC Display Black"),
];

fn overlay_text(file: &str) -> String {
    let bytes = embedded_overlay(file).expect("overlay should be embedded");
    String::from_utf8(bytes.into_owned()).expect("overlay should be UTF-8")
}

#[test]
fn overlays_are_embedded_and_subscribe_to_expected_messages() {
    for spec in &OVERLAYS {
        let text = overlay_text(spec.file);
        assert!(
            text.contains("new WebSocket("),
            "{} should open the overlay WebSocket",
            spec.file
        );
        assert!(
            text.contains("new WebSocket(`${scheme}://${target}`)"),
            "{} should connect through the origin-aware resolver",
            spec.file
        );
        assert!(
            text.contains("location.host"),
            "{} should fall back to the page origin",
            spec.file
        );
        assert!(
            text.contains("query.get('host')") && text.contains("query.get('port')"),
            "{} should keep the ?host= and ?port= overrides",
            spec.file
        );
        assert!(
            !text.contains("ws://localhost:"),
            "{} hardcodes localhost, so a remote client would dial itself",
            spec.file
        );
        for message_type in spec.message_types {
            assert!(
                text.contains(message_type),
                "{} should handle the {message_type} message",
                spec.file
            );
        }
    }
}

#[test]
fn overlays_have_no_external_network_references() {
    for spec in &OVERLAYS {
        let text = overlay_text(spec.file);
        assert!(
            !text.contains("design.penpot.app"),
            "{} still references Penpot",
            spec.file
        );
        for scheme in ["http://", "https://"] {
            for (index, _) in text.match_indices(scheme) {
                let tail: String = text[index..].chars().take(80).collect();
                assert!(
                    tail.starts_with("http://www.w3.org/2000/svg"),
                    "{} contains an external URL: {tail}",
                    spec.file
                );
            }
        }
    }
}

#[test]
fn bundled_fonts_are_woff2_and_referenced_by_every_overlay() {
    for (weight, alias) in FONTS {
        let name = format!("{FONT_PREFIX}-{weight}.woff2");
        let bytes = embedded_font(&name).expect("display font should be embedded");
        assert!(
            bytes.starts_with(b"wOF2"),
            "{name} should be a valid WOFF2 file"
        );

        for spec in &OVERLAYS {
            let text = overlay_text(spec.file);
            assert!(
                text.contains(&format!("font-family: '{alias}'")),
                "{} should declare the {alias} face",
                spec.file
            );
            assert!(
                text.contains(&format!("/assets/fonts/{name}")),
                "{} should load {name} from the font route",
                spec.file
            );
        }
    }

    for licence_name in ["OFL.txt", "OFL-Archivo.txt"] {
        let licence = embedded_font(licence_name).expect("font licence should be embedded");
        assert!(
            String::from_utf8_lossy(&licence).contains("SIL Open Font License"),
            "the bundled font licence {licence_name} should be the OFL"
        );
    }
}

#[test]
fn hero_ban_icons_are_confined_to_the_game_overlay() {
    let game = overlay_text("game-overlay.html");
    let hero_bans = overlay_text("hero-bans.html");

    assert!(
        game.contains("heroIcons"),
        "the game overlay should resolve hero ban icons"
    );
    assert!(
        hero_bans.contains("heroPortraits"),
        "the hero-bans overlay should resolve hero art"
    );
    assert!(
        !hero_bans.contains("heroIcons"),
        "the hero-bans overlay must not apply the small hero ban icons"
    );
}

#[test]
fn game_overlay_clears_stale_team_logos() {
    let text = overlay_text("game-overlay.html");

    assert!(
        text.contains("el.homeLogo.style.backgroundImage = d.teamA.logoUrl ? `url(${d.teamA.logoUrl})` : 'none';"),
        "a null team A logo must clear the live background"
    );
    assert!(
        text.contains("el.awayLogo.style.backgroundImage = d.teamB.logoUrl ? `url(${d.teamB.logoUrl})` : 'none';"),
        "a null team B logo must clear the live background"
    );
    assert!(
        text.contains("} else if (d.teamA === null) {")
            && text.contains("} else if (d.teamB === null) {"),
        "unassigned teams must also clear their logo slots"
    );
}

#[test]
fn game_overlay_shows_a_tag_placeholder_for_empty_logo_slots() {
    let text = overlay_text("game-overlay.html");

    assert!(
        text.contains("function teamLogoPlaceholderText(team)")
            && text.contains("function setTeamLogoPlaceholder(logo, placeholder, team)"),
        "the game overlay should derive and render an empty-logo placeholder"
    );
    assert!(
        text.contains("const tag = typeof team.tag === 'string' ? team.tag.trim() : '';")
            && text.contains("if (tag) return tag;"),
        "the team tag must be preferred for the placeholder"
    );
    assert!(
        text.contains("placeholder.textContent = text;")
            && text.contains("placeholder.classList.toggle('team-bar__logo-placeholder--visible', Boolean(text));"),
        "the placeholder must be shown only when the logo slot is empty"
    );
    assert!(
        text.contains("setTeamLogoPlaceholder(el.homeLogo, el.homeLogoPlaceholder, d.teamA);")
            && text
                .contains("setTeamLogoPlaceholder(el.awayLogo, el.awayLogoPlaceholder, d.teamB);"),
        "scoreboard frames must update both placeholders"
    );
    assert!(
        text.contains("setTeamLogoPlaceholder(el.homeLogo, el.homeLogoPlaceholder, null);")
            && text.contains("setTeamLogoPlaceholder(el.awayLogo, el.awayLogoPlaceholder, null);"),
        "unassigned teams must clear their placeholders"
    );
}

#[test]
fn game_overlay_uses_night_broadcast_tokens() {
    let text = overlay_text("game-overlay.html");

    assert!(
        text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-accent: #ff7a1a;")
            && text.contains("--fgc-surface-inverse: #eef2f8;"),
        "the game overlay should inline the Night Broadcast token block"
    );
    assert!(
        text.contains("background: var(--fgc-surface-inverse)"),
        "the score chip should use the inverse surface"
    );
    assert!(
        text.contains(".mode-item--active {")
            && text.contains("background: var(--fgc-accent);")
            && text.contains("color: var(--fgc-on-accent);"),
        "the active map segment should be orange on dark"
    );
    assert!(
        !text.contains("#ac46f7") && !text.contains("rgba(172, 70, 247"),
        "the old purple accent should be retired from the game overlay"
    );
    assert!(
        !text.contains("#090909") && !text.contains("#eeeeee") && !text.contains("#ffffff"),
        "the old flat greys should be retired from the game overlay"
    );
    assert!(
        text.contains("return luminance > 0.5 ? '#0b0d12' : '#eef2f8';"),
        "contrast text should use the Night Broadcast surfaces"
    );
}

#[test]
fn game_overlay_drops_the_outline_rings_and_moves_the_logo_outward() {
    let text = overlay_text("game-overlay.html");

    assert!(
        !text.contains("inset 0 0 0 1px var(--fgc-line)"),
        "the game overlay must not draw the hairline outline rings"
    );
    assert!(
        text.contains("box-shadow: 0 6px 22px rgb(0 0 0 / 38%);")
            && text.contains("box-shadow: 0 4px 16px rgb(0 0 0 / 32%);"),
        "the panels should read as floating surfaces instead of outlined boxes"
    );
    assert!(
        text.contains(".team-bar--home .team-bar__logo { order: 1; }")
            && text.contains(".team-bar--home .team-bar__ban { order: 3; }")
            && text.contains(".team-bar--home .team-bar__score { order: 4; }")
            && text.contains(".team-bar--away .team-bar__ban { order: 3; }")
            && text.contains(".team-bar--away .team-bar__logo { order: 5; }"),
        "the team logo belongs on the outer end and the ban next to the score"
    );
    assert!(
        text.contains("if (teamAbbreviations && team.tag) return team.tag;")
            && text.contains("if (d.teamAbbreviations !== undefined) {"),
        "the overlay should show the team tag when abbreviations are switched on"
    );
}

#[test]
fn map_select_card_count_follows_projected_maps() {
    let text = overlay_text("map-select.html");

    assert!(
        text.contains("function projectedMapCardCount(maps)")
            && text.contains("return isLegacyTiebreaker ? TIEBREAKER_INDEX + 1 : maps.length;"),
        "map-select should render the projected maps array"
    );
    assert!(
        !text.contains("getMapsToShow") && !text.contains("firstTo"),
        "map-select must not derive its card count from first-to"
    );
    assert!(
        text.contains("maps.length === TIEBREAKER_INDEX")
            && text.contains("currentMapIndex === TIEBREAKER_INDEX"),
        "map-select should keep the legacy tiebreaker card"
    );
}

#[test]
fn hero_bans_fills_the_screen_with_two_hero_halves() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains(
            ".hero-bans {\n      position: absolute;\n      inset: 0;\n      display: flex;"
        ) && text.contains(".hero-half {\n      position: relative;\n      flex: 1 1 0;"),
        "hero bans should render two full-bleed halves"
    );
    assert!(
        text.contains("id=\"home-half\"")
            && text.contains("id=\"away-half\"")
            && text.contains("id=\"home-art\"")
            && text.contains("id=\"away-art\""),
        "each half should carry its own hero art"
    );
    assert!(
        text.contains("id=\"home-logo\"")
            && text.contains("id=\"away-logo\"")
            && text.contains(
                ".hero-half--home .hero-half__logo {\n      left: var(--fgc-margin);\n    }"
            )
            && text.contains(
                ".hero-half--away .hero-half__logo {\n      right: var(--fgc-margin);\n    }"
            ),
        "the team logo should sit in the outer top corner of each half"
    );
    assert!(
        text.contains("id=\"home-role\"")
            && text.contains("id=\"away-role\"")
            && text.contains("id=\"home-role-icon\"")
            && text.contains("id=\"away-role-icon\""),
        "each half should carry a role hexagon"
    );
    assert!(
        text.contains("id=\"home-name\"")
            && text.contains("id=\"away-name\"")
            && text.contains("id=\"home-order\"")
            && text.contains("id=\"away-order\""),
        "each half should carry the hero name and the ban order"
    );
    assert!(
        text.contains("el.homeOrder.textContent = 'First ban';")
            && text.contains("el.awayOrder.textContent = 'Second ban';")
            && text.contains("el.homeOrder.textContent = 'Second ban';")
            && text.contains("el.awayOrder.textContent = 'First ban';"),
        "the team that banned first should carry the First ban label"
    );
    assert!(
        text.contains(".hero-half__wall {")
            && text.contains(".hero-half__wall::before {")
            && text.contains(".hero-half__wall::after {")
            && text.contains(".hero-half__grid {")
            && text.contains(".hero-half--away .hero-half__grid {")
            && text.contains("mask-size: 180px 322px;")
            && text.contains("radial-gradient("),
        "the halves should carry a honeycomb wall behind the art and a lit copy in front of it"
    );
    assert!(
        !text.contains("hero-half__texture"),
        "the old accent texture should be gone"
    );
    assert!(
        text.contains("id=\"home-logo-art\"")
            && text.contains("id=\"away-logo-art\"")
            && text.contains(".hero-half__logo-art {"),
        "the team logo artwork should sit in the band of the hexagonal badge"
    );
    assert!(
        text.matches("clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);")
            .count()
            == 2,
        "the logo badge and the role badge should share one hexagon shape"
    );
    assert!(
        text.contains(".hero-half__art--cutout img {")
            && text.contains("transform-origin: bottom center;")
            && !text.contains("object-fit: contain;\n      object-position: center bottom;"),
        "a transparent cut-out should be placed by script so the hero fills the half"
    );
    assert!(
        text.contains("function subjectBox(img)")
            && text.contains("function placeCutout(img, box)")
            && text.contains("const scale = box.height / subject.height;"),
        "the cut-out should be measured before it is placed"
    );
    assert!(
        text.contains(".hero-half__edge {")
            && text.contains(".hero-half--home .hero-half__edge {")
            && text.contains(".hero-half--away .hero-half__edge {")
            && text.contains("var(--team-a-color")
            && text.contains("var(--team-b-color"),
        "the team colour should wash in from the outer edge of each half"
    );
    assert!(
        !text.contains("hero-bans__centre"),
        "the seam down the middle should be gone"
    );
    assert!(
        text.contains("--hero-reveal-delay:") && text.contains("var(--hero-reveal-delay)"),
        "the reveal should wait so a scene transition can finish first"
    );
    assert!(
        !text.contains("map-strip")
            && !text.contains("hero-panel")
            && !text.contains("first-to-text")
            && !text.contains("id=\"header\""),
        "the header, the map strip and the old panels should be gone"
    );
}

#[test]
fn hero_bans_prefers_the_transparent_art_and_reads_the_role() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains("const cutout = heroName ? heroTransparent[heroName] : null;")
            && text.contains("const fallback = heroName ? heroPortraits[heroName] : null;")
            && text.contains("const source = cutout || fallback;"),
        "the transparent cut-out should win, with the regular art as the fallback"
    );
    assert!(
        text.contains("const slug = heroName ? heroRoles[heroName] : null;")
            && text.contains("const icon = slug ? roleIcons[slug] : null;"),
        "the role hexagon should resolve the hero role through the role icons"
    );
    assert!(
        text.contains("if (d.heroTransparent) {")
            && text.contains("if (d.heroRoles) {")
            && text.contains("if (d.roleIcons) {"),
        "the overlay should read the transparent art, the roles and the icons"
    );
    assert!(
        text.contains("el.heroBans.style.setProperty('--team-a-color', d.teamA.color")
            && text.contains("el.heroBans.style.setProperty('--team-b-color', d.teamB.color")
            && !text.contains("applyTeamColour"),
        "the team colour should wash in from the outer edge, not colour the label"
    );
}

#[test]
fn row_casters_social_row_stays_inside_the_card() {
    let text = overlay_text("row-casters.html");

    assert!(
        text.contains(".talent-subtext {\n      display: flex;\n      align-items: center;\n      gap: 12px;\n      max-width: 100%;\n      min-width: 0;\n    }"),
        "the social row must be constrained to the card's content box"
    );
    assert!(
        text.contains("flex: 0 0 20px;"),
        "the social icon must not shrink away"
    );
    assert!(
        text.contains(".social-handle,\n    .pronouns {\n      min-width: 0;\n      overflow: hidden;\n      text-overflow: ellipsis;\n      white-space: nowrap;\n    }"),
        "long handles and pronouns must ellipsize instead of widening the row"
    );
    assert!(
        text.contains("class=\"social-handle\" title="),
        "the full handle must remain available on the rendered span"
    );
}

#[test]
fn every_overlay_declares_exactly_the_five_display_faces() {
    for spec in &OVERLAYS {
        let text = overlay_text(spec.file);
        let faces = text.matches("@font-face").count();
        assert_eq!(
            faces,
            FONTS.len(),
            "{} should declare {} faces, found {faces}",
            spec.file,
            FONTS.len()
        );
        assert!(
            !text.contains("font-family: 'Apotek"),
            "{} should not declare an Apotek face",
            spec.file
        );
    }
}

#[test]
fn roster_overlays_use_night_broadcast_tokens() {
    let home = overlay_text("team-roster-home.html");
    let away = overlay_text("team-roster-away.html");

    for (file, text) in [
        ("team-roster-home.html", &home),
        ("team-roster-away.html", &away),
    ] {
        assert!(
            text.contains("--fgc-surface: #0b0d12;")
                && text.contains("--fgc-surface-raised: #11151d;")
                && text.contains("--fgc-surface-inverse: #eef2f8;")
                && text.contains("--fgc-surface-muted: #151922;")
                && text.contains("--fgc-text: #eef2f8;")
                && text.contains("--fgc-text-muted: #8d99ad;")
                && text.contains("--fgc-accent: #ff7a1a;"),
            "{file} should inline the canonical Night Broadcast token block"
        );
        for retired in [
            "#fff", "#ffffff", "#090909", "#eeeeee", "#eee", "#444444", "#2d2d2d",
        ] {
            assert!(
                !text.contains(retired),
                "{file} should have retired the old colour {retired}"
            );
        }
        assert!(
            text.contains("const textColor = luminance > 0.5 ? '#0b0d12' : '#eef2f8';"),
            "{file} should resolve team-name contrast to the Night Broadcast palette"
        );
        assert!(
            text.contains(r#"<link rel="icon" href="data:,">"#),
            "{file} should suppress the favicon request"
        );
    }

    let home_norm = home
        .replace("<title>Team Roster - Home</title>", "<title></title>")
        .replace("const TEAM_SIDE = 'A';", "const TEAM_SIDE = '';");
    let away_norm = away
        .replace("<title>Team Roster - Away</title>", "<title></title>")
        .replace("const TEAM_SIDE = 'B';", "const TEAM_SIDE = '';");
    assert_eq!(
        home_norm, away_norm,
        "the roster pair must stay byte-identical apart from <title> and TEAM_SIDE"
    );
}

#[test]
fn timer_overlay_uses_night_broadcast_tokens_and_keeps_hour_rollup() {
    let text = overlay_text("timer.html");

    assert!(
        text.contains("--fgc-text: #eef2f8;") && text.contains("color: var(--fgc-text);"),
        "the timer value should use the Night Broadcast text token"
    );
    assert!(
        !text.contains("#fff"),
        "the timer should have retired the old flat white"
    );
    assert!(
        text.contains("text-shadow: 0 2px 12px rgba(0, 0, 0, 0.65);"),
        "the timer keeps its legibility shadow"
    );
    assert!(
        text.contains("const showHours = settings.showHours !== false;")
            && text
                .contains("const parts = showHours ? [hours, minutes] : [hours * 60 + minutes];"),
        "the showHours roll-up logic must stay intact"
    );
    assert!(
        text.contains("let lastSettings = {};")
            && text.contains("if (data.settings && typeof data.settings === 'object') lastSettings = data.settings;"),
        "reset frames must keep reusing the last settings"
    );
    assert!(
        text.contains(r#"aria-live="off""#)
            && text.contains("message.data.matchTimer")
            && text.contains("message.data.action === 'reset'"),
        "the timer keeps its live-region and reset handling"
    );
}

#[test]
fn map_pool_overlay_uses_night_broadcast_tokens() {
    let text = overlay_text("map-pool.html");

    assert!(
        text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-surface-raised: #11151d;")
            && text.contains("--fgc-surface-inverse: #eef2f8;")
            && text.contains("--fgc-surface-muted: #151922;")
            && text.contains("--fgc-text: #eef2f8;")
            && text.contains("--fgc-text-muted: #8d99ad;")
            && text.contains("--fgc-line: #2a3140;")
            && text.contains("--fgc-accent: #ff7a1a;"),
        "map-pool should inline the canonical Night Broadcast token block"
    );
    assert!(
        text.contains(".mode-header {")
            && text.contains(".map-name {\n      background: var(--fgc-surface);"),
        "the mode header and map name bar should sit on the base surface"
    );
    assert!(
        text.contains("background-color: var(--fgc-surface-muted);")
            && text.contains(": 'background-color: var(--fgc-surface-muted)';"),
        "empty map art should fall back to the muted surface in CSS and JS"
    );
    assert!(
        text.contains("var(--fgc-pick-glow)") && text.contains("var(--fgc-pick-glow-mid)"),
        "the chosen map should be lit by the pick glow gradient"
    );
    assert!(
        text.contains("color: var(--fgc-text);"),
        "map-pool labels should use the Night Broadcast text token"
    );
    for retired in [
        "#fff", "#ffffff", "#090909", "#151515", "#eeeeee", "#eee", "#ac46f7", "#444444",
        "#2d2d2d", "#1a1a1a", "#666",
    ] {
        assert!(
            !text.contains(retired),
            "map-pool should have retired the old colour {retired}"
        );
    }
    assert!(
        !text.contains("mix-blend-mode: darken"),
        "map-pool should not blend map art against the dark surfaces"
    );
    assert!(
        text.contains(r#"<link rel="icon" href="data:,">"#),
        "map-pool should suppress the favicon request"
    );
    assert_eq!(
        text.matches("@font-face").count(),
        FONTS.len(),
        "map-pool should keep exactly five display faces"
    );
    assert!(
        text.contains("ws.send(JSON.stringify({ type: 'requestInit' }));"),
        "map-pool must keep asking the server for its initial frame"
    );
}

#[test]
fn map_pool_staggers_each_mode_column() {
    let text = overlay_text("map-pool.html");

    assert!(
        text.contains("@keyframes columnReveal {")
            && text.contains(".animate-column {")
            && text.contains(
                "animation: columnReveal var(--pool-anim-duration) var(--fgc-ease-out) forwards;"
            )
            && text.contains("animation-delay: var(--column-delay, var(--pool-anim-delay));"),
        "map-pool should reveal each mode column with its own animation"
    );
    assert!(
        text.contains("column.style.setProperty(")
            && text.contains("'--column-delay',")
            && text.contains("COLUMN_STAGGER_SECONDS"),
        "each mode column should get its own delay"
    );
    assert!(
        !text.contains(".animate-container {") && !text.contains("@keyframes fadeIn {"),
        "map-pool should not fade the whole pool as one block any more"
    );
    assert!(
        text.contains("column.style.opacity = '1';"),
        "a later pool update should show the new columns straight away"
    );
}

#[test]
fn map_pool_lights_the_picked_map() {
    let text = overlay_text("map-pool.html");

    assert!(
        text.contains(".map-image.chosen {") && text.contains("filter: grayscale(100%)"),
        "the picked map should stay greyscale"
    );
    assert!(
        text.contains(".map-pick-glow {")
            && text.contains("var(--fgc-pick-glow)")
            && text.contains("linear-gradient("),
        "the picked map should carry a gradient wash above the artwork"
    );
    assert!(
        text.contains(".map-card.picked .map-name {")
            && text.contains("box-shadow: inset 0 2px 0 var(--fgc-accent);"),
        "the picked map should keep an accent rule under the artwork"
    );
    assert!(
        !text.contains(".map-image.chosen::after"),
        "the old flat wash behind the greyscale filter should be gone"
    );
}

#[test]
fn map_select_overlay_uses_night_broadcast_tokens() {
    let text = overlay_text("map-select.html");

    assert!(
        text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-surface-raised: #11151d;")
            && text.contains("--fgc-surface-inverse: #eef2f8;")
            && text.contains("--fgc-surface-muted: #151922;")
            && text.contains("--fgc-text: #eef2f8;")
            && text.contains("--fgc-text-muted: #8d99ad;")
            && text.contains("--fgc-text-inverse: #0b0d12;")
            && text.contains("--fgc-line: #2a3140;")
            && text.contains("--fgc-accent: #ff7a1a;"),
        "map-select should inline the canonical Night Broadcast token block"
    );
    assert!(
        text.contains("--anim-delay: 0.8s;")
            && text.contains("--anim-duration: 0.5s;")
            && text.contains("--ease-out-expo: cubic-bezier(0.16, 1, 0.3, 1);"),
        "the existing animation variables must survive inside the merged token block"
    );
    assert_eq!(
        text.matches("background: var(--fgc-surface-raised);")
            .count(),
        2,
        "both the card and its text panel should sit on the raised surface"
    );
    assert!(
        text.contains(
            ".map-card__image--placeholder {\n      background: var(--fgc-surface-muted);\n    }"
        ) && !text.contains("linear-gradient(180deg"),
        "the placeholder image should be a solid muted surface"
    );
    assert!(
        text.contains("background: var(--fgc-accent-soft);"),
        "the completed map scrim should use the accent-soft token"
    );
    assert!(
        text.contains("background: rgba(11, 13, 18, 0.6);"),
        "the score board should be a token-derived translucent surface"
    );
    assert!(
        text.contains(".map-card__type {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 18px;\n      color: var(--fgc-accent);"),
        "the map mode label should carry the orange accent"
    );
    assert!(
        text.contains(".map-card__chosen-by {\n      font-family: 'FGC Display', 'Apotek', sans-serif;\n      font-size: 14px;\n      color: var(--fgc-text-muted);"),
        "the chosen-by tag should use muted text"
    );
    assert!(
        text.contains(".watermark {\n      height: 48px;\n      min-width: 366px;\n      background: var(--fgc-surface-inverse);")
            && text.contains(".watermark__text {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 28px;\n      color: var(--fgc-text-inverse);"),
        "the watermark band should keep the inverse plate with inverse text"
    );
    assert!(
        !text.contains("mix-blend-mode: darken"),
        "map-select should not blend map art against the dark surfaces"
    );
    for retired in [
        "#fff", "#ffffff", "#090909", "#151515", "#eeeeee", "#eee", "#ac46f7", "#444444",
        "#2d2d2d", "#1a1a1a", "#666",
    ] {
        assert!(
            !text.contains(retired),
            "map-select should have retired the old colour {retired}"
        );
    }
    assert!(
        text.contains(r#"<link rel="icon" href="data:,">"#),
        "map-select should suppress the favicon request"
    );
    assert_eq!(
        text.matches("@font-face").count(),
        FONTS.len(),
        "map-select should keep exactly five display faces"
    );
    assert!(
        !text.contains("requestInit"),
        "map-select must not gain an init request it never sent"
    );
}

#[test]
fn map_select_shows_the_banned_heroes_on_the_map() {
    let text = overlay_text("map-select.html");

    assert!(
        text.contains("function createBanStrip(bans)")
            && text.contains("function orderedBans(bans)"),
        "map-select should build the ban strip from the projected bans"
    );
    assert!(
        text.contains("bans.firstBanTeam === 'B' ? [...teamB, ...teamA] : [...teamA, ...teamB]"),
        "the ban order should follow the team that banned first"
    );
    assert!(
        text.contains("heroPortraits[heroName]"),
        "the ban portraits should come from the projected hero portraits"
    );
    assert!(
        text.contains(
            "<div class=\"map-card__visual\">\n          <div class=\"${imageClass}\" id=\"image-${index}\" style=\"${imageStyle}\"></div>\n          ${createBanStrip(bans)}\n        </div>"
        ),
        "the ban strip should sit beside the map image, not inside it"
    );
    assert!(
        text.contains(
            ".map-card__bans {\n      position: absolute;\n      left: 14px;\n      bottom: 14px;"
        ),
        "the ban strip should overlay the bottom of the map image"
    );
    assert!(
        text.contains(".map-card__ban-number {\n      position: absolute;\n      top: -8px;\n      right: -8px;\n      width: 24px;\n      height: 24px;\n      transform: rotate(45deg);"),
        "the ban order should be a small diamond in the corner"
    );
    assert!(
        text.contains(".map-card__ban-number span {\n      transform: rotate(-45deg);"),
        "the ban number should stay upright inside the diamond"
    );
}

#[test]
fn row_casters_overlay_uses_night_broadcast_tokens() {
    let text = overlay_text("row-casters.html");

    assert!(
        text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-surface-raised: #11151d;")
            && text.contains("--fgc-surface-muted: #151922;")
            && text.contains("--fgc-text: #eef2f8;")
            && text.contains("--fgc-text-muted: #8d99ad;")
            && text.contains("--fgc-line: #2a3140;")
            && text.contains("--fgc-accent: #ff7a1a;"),
        "row-casters should inline the canonical Night Broadcast token block"
    );
    assert!(
        text.contains(".talent-text {\n      background: var(--fgc-surface-raised);"),
        "the caster text panel should sit on the raised surface"
    );
    assert!(
        text.contains(".talent-name {\n      width: 100%;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 48px;\n      color: var(--fgc-text);"),
        "the caster name should use the Night Broadcast text token"
    );
    assert!(
        text.contains(".social-handle {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;\n      font-size: 24px;\n      color: var(--fgc-text-muted);"),
        "social handles should use muted text"
    );
    assert!(
        text.contains(".social-icon path {\n      fill: var(--fgc-text-muted);\n    }"),
        "social icons should take the muted tier from a stylesheet rule"
    );
    assert!(
        text.contains(": 'background-color: var(--fgc-surface-muted)';"),
        "a missing portrait should fall back to the muted surface in JS"
    );
    for retired in [
        "#fff", "#ffffff", "#090909", "#151515", "#eeeeee", "#eee", "#ac46f7", "#444444",
        "#2d2d2d", "#1a1a1a", "#666",
    ] {
        assert!(
            !text.contains(retired),
            "row-casters should have retired the old colour {retired}"
        );
    }
    assert!(
        text.contains(r#"<link rel="icon" href="data:,">"#),
        "row-casters should suppress the favicon request"
    );
    assert_eq!(
        text.matches("@font-face").count(),
        FONTS.len(),
        "row-casters should keep exactly five display faces"
    );
    assert!(
        text.contains("ws.send(JSON.stringify({ type: 'requestInit' }));"),
        "row-casters must keep asking the server for its initial frame"
    );
}

#[test]
fn hero_bans_overlay_uses_night_broadcast_tokens() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-surface-inverse: #eef2f8;")
            && text.contains("--fgc-text: #eef2f8;")
            && text.contains("--fgc-text-inverse: #0b0d12;")
            && text.contains("--fgc-text-muted: #8d99ad;")
            && text.contains("--fgc-accent: #ff7a1a;")
            && text.contains("--fgc-on-accent: #0b0d12;")
            && text.contains("--fgc-line: #2a3140;"),
        "hero-bans should inline the canonical Night Broadcast token block"
    );
    assert!(
        text.contains(".hero-half__role {\n      width: 60px;\n      height: 68px;\n      display: flex;\n      align-items: center;\n      justify-content: center;\n      background: var(--fgc-plate-dark);"),
        "the role hexagon should sit on the dark plate so the white icon reads"
    );
    assert!(
        text.contains(".hero-half__name {\n      width: 86%;\n      font-family: 'FGC Display Black', 'Apotek Black', sans-serif;\n      font-size: 92px;\n      line-height: 1.02;\n      color: var(--fgc-ink-on-art);"),
        "the hero name should hover over the art in the near-white ink"
    );
    assert!(
        text.contains(".hero-half__order {\n      max-width: 70%;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 30px;\n      line-height: 1;\n      letter-spacing: 0.22em;\n      text-transform: uppercase;\n      color: var(--fgc-ink-on-art);"),
        "the ban order should use the neutral near-white ink so it reads on any art"
    );
    assert!(
        text.contains(".hero-half__logo {\n      position: absolute;\n      top: var(--fgc-margin);\n      z-index: 4;\n      width: 100px;\n      height: 115px;\n      background-color: var(--fgc-plate-dark);"),
        "the team logo should sit on the dark hexagonal plate so the white glyph reads"
    );
    assert!(
        text.contains("--fgc-plate-dark: #0b0d12;") && text.contains("--fgc-ink-on-art: #eef2f8;"),
        "the overlay should declare the local plate and ink tokens"
    );
    assert!(
        text.contains(
            "background: linear-gradient(\n        to top,\n        rgb(0 0 0 / 78%) 0%,"
        ),
        "the halves should carry a scrim so the caption stays legible"
    );
    for retired in [
        "#fff", "#ffffff", "#090909", "#151515", "#eeeeee", "#eee", "#ac46f7", "#444444",
        "#2d2d2d", "#1a1a1a", "#666",
    ] {
        assert!(
            !text.contains(retired),
            "hero-bans should have retired the old colour {retired}"
        );
    }
    assert!(
        text.contains(r#"<link rel="icon" href="data:,">"#),
        "hero-bans should suppress the favicon request"
    );
    assert_eq!(
        text.matches("@font-face").count(),
        FONTS.len(),
        "hero-bans should keep exactly five display faces"
    );
    assert!(
        !text.contains("requestInit"),
        "hero-bans must not gain an init request it never sent"
    );
}

#[test]
fn every_overlay_applies_the_theme_message() {
    for spec in OVERLAYS {
        let bytes = embedded_overlay(spec.file).expect("overlay should be embedded");
        let text = String::from_utf8(bytes.into_owned()).expect("overlay should be UTF-8");

        assert!(
            text.contains("function applyTheme(payload)"),
            "{} should define applyTheme",
            spec.file
        );
        assert!(
            text.contains("root.style.setProperty(name, String(tokens[name]));"),
            "{} should apply the theme tokens",
            spec.file
        );
        assert!(
            text.contains("root.style.removeProperty(name);"),
            "{} should clear the tokens of an earlier theme",
            spec.file
        );
        assert!(
            text.contains("'theme'"),
            "{} should listen for the theme message",
            spec.file
        );
    }
}

#[test]
fn the_timer_overlay_applies_the_theme_message() {
    let bytes = embedded_overlay("timer.html").expect("timer overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("timer overlay should be UTF-8");

    assert!(text.contains("function applyTheme(payload)"));
    assert!(text.contains("root.style.setProperty(name, String(tokens[name]));"));
    assert!(text.contains("message.type === 'theme'"));
    assert!(text.contains("message.data.theme"));
}

#[test]
fn the_light_theme_only_names_tokens_the_overlays_declare() {
    for spec in OVERLAYS {
        let bytes = embedded_overlay(spec.file).expect("overlay should be embedded");
        let text = String::from_utf8(bytes.into_owned()).expect("overlay should be UTF-8");
        let start = text
            .find(":root {")
            .expect("overlay should declare :root tokens");
        let rest = &text[start..];
        let end = rest.find('}').expect("the :root block should close");
        let root = &rest[..=end];

        for (name, _value) in fgc_protocol::LIGHT_THEME_TOKENS {
            assert!(
                root.contains(&format!("{name}:")),
                "{} should declare the {name} token",
                spec.file
            );
        }
    }
}

/// The shared type scale.
///
/// The alias family carries the weight, so a rule never declares a
/// `font-weight` of its own. That is what keeps the overlays consistent: the
/// same role gets the same weight in every file, and a rule cannot ask for a
/// weight its family does not have.
const TYPE_SCALE: [(&str, &str); 5] = [
    ("300", "FGC Display Light"),
    ("400", "FGC Display"),
    ("500", "FGC Display Medium"),
    ("600", "FGC Display SemiBold"),
    ("900", "FGC Display Black"),
];

/// Every bundled overlay page, including the ones the shared spec does not
/// cover.
const ALL_OVERLAYS: [&str; 12] = [
    "auto.html",
    "background.html",
    "game-overlay.html",
    "hero-bans.html",
    "map-pool.html",
    "map-select.html",
    "row-casters.html",
    "row-hosts.html",
    "row-interviews.html",
    "team-roster-away.html",
    "team-roster-home.html",
    "timer.html",
];

#[test]
fn every_overlay_carries_the_same_fx_block() {
    const START: &str = "<!-- === FGC CANONICAL FX START === -->";
    const END: &str = "<!-- === FGC CANONICAL FX END === -->";
    const FILES: [&str; 8] = [
        "hero-bans.html",
        "map-pool.html",
        "map-select.html",
        "row-casters.html",
        "team-roster-home.html",
        "team-roster-away.html",
        "row-hosts.html",
        "row-interviews.html",
    ];

    let mut canonical: Option<String> = None;
    for file in FILES {
        let text = overlay_text(file);
        assert_eq!(
            text.matches(START).count(),
            1,
            "{file} should contain exactly one canonical FX block"
        );
        assert_eq!(
            text.matches("class=\"fgc-bg\"").count(),
            1,
            "{file} should contain exactly one background plate sibling"
        );
        assert_eq!(
            text.matches("class=\"fgc-fx\"").count(),
            1,
            "{file} should contain exactly one lattice sibling"
        );
        assert!(
            text.contains("if (window.fgcUpdateBackground) window.fgcUpdateBackground(data);"),
            "{file} should forward section messages to the live background handler"
        );
        let start = text.find(START).expect("FX start sentinel should exist");
        let end = text[start..]
            .find(END)
            .expect("FX end sentinel should exist")
            + start
            + END.len();
        let block = &text[start..end];

        if let Some(expected) = &canonical {
            assert_eq!(
                block, expected,
                "{file} must carry the canonical FX block byte-for-byte"
            );
        } else {
            canonical = Some(block.to_owned());
        }
    }

    let canonical = canonical.expect("at least one overlay should carry the FX block");
    assert!(
        canonical.contains("--fgc-hex-mask:")
            && canonical.contains("mask-size: 180px 322px")
            && canonical.contains("translate3d(0, -322px, 0)")
            && canonical.contains("translate3d(180px, 0, 0)"),
        "the canonical block should contain the shared mask and one-tile drift"
    );
    assert!(
        canonical.contains("inset: -360px")
            && canonical.contains("will-change: transform")
            && !canonical.contains("background-position")
            && !canonical.contains("requestAnimationFrame"),
        "the canonical block should overscan and animate only on the compositor"
    );
    assert!(
        canonical.contains("html[data-fx=\"off\"] .fgc-fx")
            && canonical.contains("requestedFx === 'on' || requestedFx === 'off'")
            && canonical.contains("@media (prefers-reduced-motion: reduce)")
            && canonical.contains("animation: none"),
        "the canonical block should carry the toggle and the reduced-motion freeze"
    );
    assert!(
        canonical.contains(".fgc-bg {")
            && canonical.contains("background-color: var(--fgc-surface)")
            && canonical.contains("html[data-bg=\"opaque\"] .fgc-bg")
            && canonical.contains("html[data-bg=\"opaque\"] .fgc-fx"),
        "the canonical block should carry the opaque plate and hide the lattice in that mode"
    );
    assert!(
        canonical.contains("requestedBg === 'off' ? 'opaque'")
            && canonical.contains("requestedBg === 'on' ? 'transparent'")
            && canonical.contains("root.dataset.bg = bgMode"),
        "the canonical block should read the background mode from the URL"
    );
    assert!(
        canonical.contains("window.fgcUpdateBackground = (data) => {")
            && canonical.contains("payload.opaque"),
        "the canonical block should accept a live background section message"
    );
}

#[test]
fn background_mode_is_absent_from_sources_without_a_background() {
    for file in ["timer.html", "game-overlay.html"] {
        let text = overlay_text(file);
        assert!(
            !text.contains("data-bg") && !text.contains("fgc-bg"),
            "{file} must not gain a background treatment"
        );
    }
}

#[test]
fn background_html_supports_the_transparent_variant() {
    let text = overlay_text("background.html");
    assert!(
        text.contains("html[data-bg=\"transparent\"]")
            && text.contains("html[data-bg=\"transparent\"] .background")
            && text.contains("background-color: transparent"),
        "the standalone background should be able to drop its opaque plate"
    );
    assert!(
        text.contains("requestedBg === 'off' ? 'opaque'") && text.contains("root.dataset.bg"),
        "the standalone background should read the same URL toggle"
    );
    assert!(
        text.contains("data.type === 'background'") && text.contains("d.opaque"),
        "the standalone background should accept a live background section message"
    );
}

#[test]
fn auto_overlay_forwards_the_background_mode_to_its_scenes() {
    let text = overlay_text("auto.html");
    assert!(
        text.contains("const bg = query.get('bg');")
            && text.contains("bg === 'opaque' || bg === 'transparent'")
            && text.contains("params.set('bg', bg)"),
        "the auto overlay should pass the background mode to every scene frame"
    );
}

#[test]
fn background_overlay_is_broadcast_safe_and_supports_url_variants() {
    let text = overlay_text("background.html");

    assert!(
        text.contains(".background__lattice {")
            && text.contains(".background__lattice--ambient {")
            && text.contains(".background__lattice--lit {")
            && text.contains("mask-size: 180px 322px;"),
        "the background should carry the shared hexagon lattice and two drift layers"
    );
    assert!(
        text.contains("@keyframes latticeDriftForward")
            && text.contains("@keyframes latticeDriftReverse")
            && text.contains("@keyframes glowBreathe"),
        "the lattice should drift in opposite directions while the glow breathes"
    );

    let keyframes_start = text
        .find("@keyframes latticeDriftForward")
        .expect("background should define its lattice animation");
    let keyframes_end = text
        .find("    .background__glow,")
        .expect("background keyframes should end before the layer declarations");
    let keyframes = &text[keyframes_start..keyframes_end];
    for property in [
        "background-position:",
        "filter:",
        "width:",
        "height:",
        "box-shadow:",
    ] {
        assert!(
            !keyframes.contains(property),
            "background keyframes must not animate paint-heavy {property}"
        );
    }
    for declaration in keyframes.split(';') {
        let Some((property, _value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property
            .rsplit('{')
            .next()
            .expect("a keyframe declaration should have a property")
            .trim();
        assert!(
            property == "transform" || property == "opacity",
            "background keyframes should animate only transform and opacity, found {property}"
        );
    }

    assert!(
        text.contains("@media (prefers-reduced-motion: reduce)")
            && text.contains(".motion-still .background__glow")
            && text.contains("animation: none;"),
        "reduced-motion and motion=still should leave a static composition"
    );
    assert!(
        text.contains("query.get('motion')")
            && text.contains("query.get('tone')")
            && text.contains("=== 'still'")
            && text.contains("=== 'team'"),
        "OBS URL query parameters should select the motion and tone variants"
    );
    assert!(
        text.contains("=== NIGHT BROADCAST TOKENS ===")
            && text.contains("--fgc-surface: #0b0d12;")
            && text.contains("--fgc-ink-on-art: #eef2f8;"),
        "the background should inline the shared Night Broadcast tokens"
    );
    assert!(
        !text.contains("requestAnimationFrame"),
        "the background must not run a per-frame JavaScript animation loop"
    );
    assert!(
        text.contains("background-color: var(--fgc-surface);")
            && text.contains("background: var(--fgc-surface);")
            && !text.contains("rgba(0, 0, 0, 0)"),
        "the background should paint a solid, opaque plate from the theme tokens"
    );
    assert!(
        text.contains("html.background-hidden")
            && text.contains("root.classList.toggle('background-hidden', !visible);"),
        "hiding the source should take the solid plate with it"
    );
}

/// Extracts the `<style>` block of an overlay.
fn style_block(text: &str) -> &str {
    let start = text
        .find("<style>")
        .expect("overlay should have a style block");
    let end = text[start..]
        .find("</style>")
        .expect("overlay should close its style block");
    &text[start..start + end]
}

/// The tiers an overlay declares as faces, as `(weight, family)` pairs.
fn declared_faces(text: &str) -> Vec<(&'static str, &'static str)> {
    TYPE_SCALE
        .iter()
        .filter(|(weight, family)| {
            text.contains(&format!(
                "font-family: '{family}';\n      font-weight: {weight};"
            ))
        })
        .copied()
        .collect()
}

#[test]
fn every_overlay_uses_the_shared_type_scale() {
    for file in ALL_OVERLAYS {
        let text = overlay_text(file);
        let css = style_block(&text);

        // The alias family carries the weight, so no rule declares one.
        let without_faces = css.replace("@font-face", "");
        assert!(
            !without_faces.contains("font-weight:"),
            "{file} must not declare a font-weight outside an @font-face block"
        );

        // Every face is one of the five tiers and loads the matching file.
        let faces = declared_faces(&text);
        if file == "auto.html" {
            // The auto overlay only hosts the other pages in frames, so it
            // paints no text of its own and loads no face.
            assert!(
                faces.is_empty(),
                "auto.html should not declare a display face"
            );
            return;
        }
        assert!(
            !faces.is_empty(),
            "{file} should declare at least one display face"
        );
        for (weight, family) in &faces {
            assert!(
                text.contains(&format!(
                    "src: url('/assets/fonts/{FONT_PREFIX}-{weight}.woff2')"
                )),
                "{file} should load the {weight} face for {family}"
            );
        }

        // Every family a rule names is a tier this overlay declares.
        for (_, family) in TYPE_SCALE {
            if text.contains(&format!("font-family: '{family}'")) {
                assert!(
                    faces.iter().any(|(_, declared)| *declared == family),
                    "{file} uses {family} without declaring it as a face"
                );
            }
        }
    }
}

#[test]
fn the_match_overlays_all_ship_the_five_display_faces() {
    for file in ALL_OVERLAYS {
        if file == "timer.html" || file == "auto.html" || file == "background.html" {
            // The background paints no text and is deliberately asset-free.
            continue;
        }
        let text = overlay_text(file);
        assert_eq!(
            declared_faces(&text).len(),
            TYPE_SCALE.len(),
            "{file} should declare the five display faces"
        );
    }
}

#[test]
fn the_timer_overlay_uses_the_semi_bold_alias() {
    let text = overlay_text("timer.html");

    assert!(
        text.contains("font-family: 'FGC Display SemiBold';\n      font-weight: 600;"),
        "the timer should load the SemiBold face under the shared alias"
    );
    assert!(
        text.contains("font-family: 'FGC Display SemiBold', 'Apotek Semibold', Arial, sans-serif;"),
        "the timer value should use the shared SemiBold alias"
    );
    assert!(
        !text.contains("font-family: 'FGC Display';\n      font-weight: 600;"),
        "the timer must not hijack the regular alias for the SemiBold face"
    );
}

#[test]
fn the_type_scale_gives_each_role_one_weight() {
    // Scores are Strong everywhere.
    assert!(
        overlay_text("map-select.html")
            .contains(".map-card__score {\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;"),
        "the map select score should be SemiBold"
    );
    assert!(
        overlay_text("game-overlay.html")
            .contains(".team-bar__score {\n      width: 55px;\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;"),
        "the game overlay score should be SemiBold too"
    );

    // The name of a map is Strong everywhere.
    for file in ["map-pool.html", "map-select.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains("font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;"),
            "{file} should render the map name in SemiBold"
        );
    }

    // The game mode is a label, so it is Medium everywhere.
    assert!(
        overlay_text("map-pool.html").contains(
            ".mode-name {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;"
        ),
        "the map pool mode name should be Medium"
    );
    assert!(
        overlay_text("map-select.html")
            .contains(".map-card__type {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;"),
        "the map select map type should be Medium too"
    );

    // Pronouns are the quietest text on the element, so they are Light
    // everywhere.
    for file in ["team-roster-home.html", "team-roster-away.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains(".player-card-core .player-pronouns {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;"),
            "{file} should render the core pronouns in Light"
        );
        assert!(
            text.contains(".player-card-sub .player-pronouns {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;"),
            "{file} should render the substitute pronouns in Light too"
        );
    }
    for file in ["row-casters.html", "row-hosts.html", "row-interviews.html"] {
        assert!(
            overlay_text(file).contains(
                ".pronouns {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;"
            ),
            "{file} should render the pronouns in Light"
        );
    }

    // A team or person name is Medium everywhere.
    assert!(
        overlay_text("game-overlay.html")
            .contains(".team-bar__name {\n      flex: 1;\n      min-width: 0;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;"),
        "the game overlay team name should be Medium"
    );
    assert!(
        overlay_text("team-roster-home.html").contains(
            ".player-card-core .player-name {\n      width: 100%;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;"
        ),
        "the roster player name should be Medium"
    );

    // The hero name on the hero bans screen is the headline of that screen,
    // so it is the one name that is Display.
    assert!(
        overlay_text("hero-bans.html").contains(
            ".hero-half__name {\n      width: 86%;\n      font-family: 'FGC Display Black', 'Apotek Black', sans-serif;"
        ),
        "the hero bans hero name should be Display"
    );
}
