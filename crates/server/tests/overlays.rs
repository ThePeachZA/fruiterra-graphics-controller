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
        message_types: &["'init'", "'scoreboard'", "'gameOverlay'", "'show'"],
    },
    OverlaySpec {
        file: "hero-bans.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'heroBans'",
            "'show'",
        ],
    },
    OverlaySpec {
        file: "map-pool.html",
        message_types: &["'init'", "'scoreboard'", "'mapPool'", "'show'"],
    },
    OverlaySpec {
        file: "map-select.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'mapSelect'",
            "'show'",
        ],
    },
    OverlaySpec {
        file: "row-casters.html",
        message_types: &["castersData", "'show'"],
    },
    OverlaySpec {
        file: "row-hosts.html",
        message_types: &["castersData", "'show'"],
    },
    OverlaySpec {
        file: "row-interviews.html",
        message_types: &["castersData", "interviewee", "'show'"],
    },
    OverlaySpec {
        file: "team-roster-home.html",
        message_types: &[
            "'init'",
            "'scoreboard'",
            "'gameOverlay'",
            "'teamRoster'",
            "'show'",
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
fn hero_bans_crops_portraits_inside_their_slots() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains(".hero-ban-portrait {\n      flex: 1;\n      min-width: 0;\n      min-height: 0;\n      overflow: hidden;"),
        "hero ban slots should clip images to their flex dimensions"
    );
    assert!(
        text.contains("object-fit: cover"),
        "hero ban portraits should be cropped rather than stretched"
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
        text.contains(".team-score {\n      width: 80px;\n      height: 80px;\n      display: flex;\n      align-items: center;\n      justify-content: center;\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;\n      font-size: 48px;\n      color: var(--fgc-text-inverse);\n      text-transform: uppercase;\n      background: var(--fgc-surface-inverse);"),
        "the score chip should keep the inverse plate with inverse text"
    );
    assert!(
        text.contains(".ban-order {\n      width: 100px;\n      height: 80px;\n      background: var(--fgc-accent);"),
        "the ban order marker should carry the orange accent"
    );
    assert!(
        text.contains(".ban-order-text {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 24px;\n      color: var(--fgc-on-accent);"),
        "the ban order text should sit on the accent with inverse text"
    );
    assert!(
        text.contains(".map-info-type {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 24px;\n      color: var(--fgc-accent);"),
        "the map type label should carry the orange accent"
    );
    assert!(
        text.contains("return luminance > 0.5 ? '#0b0d12' : '#eef2f8';"),
        "team name contrast should follow the shared surface tokens"
    );
    assert!(
        text.contains("el.homeSection.style.backgroundColor = 'var(--fgc-surface)';")
            && text.contains("el.awaySection.style.backgroundColor = 'var(--fgc-surface)';"),
        "the neutral team sections should fall back to the base surface"
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
