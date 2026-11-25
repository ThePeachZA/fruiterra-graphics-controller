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
fn pick_overlays_show_a_tag_placeholder_for_empty_logo_slots() {
    let hero_bans = overlay_text("hero-bans.html");
    assert!(
        hero_bans.contains("function teamLogoPlaceholderText(team)")
            && hero_bans.contains("function setTeamLogoPlaceholder(art, team)")
            && hero_bans.contains("--fgc-tag-ink: #eef2f8;")
            && hero_bans.contains(".hero-half__logo-art--placeholder {")
            && hero_bans.contains(
                "color: var(--fgc-tag-ink);\n      text-shadow: 0 2px 8px rgb(0 0 0 / 65%);"
            )
            && hero_bans.contains("setTeamLogoPlaceholder(el.homeLogoArt, d.teamA);")
            && hero_bans.contains("setTeamLogoPlaceholder(el.awayLogoArt, d.teamB);"),
        "hero bans should render the team tag inside the hex badge when the logo is empty"
    );

    let map_select = overlay_text("map-select.html");
    assert!(
        map_select.contains("function teamLogoPlaceholderText(team)")
            && map_select.contains("--fgc-tag-ink: #eef2f8;")
            && map_select.contains(".map-card__logo--placeholder {")
            && map_select.contains(
                "background: linear-gradient(to bottom, rgb(11 13 18 / 86%), rgb(11 13 18 / 64%));"
            )
            && map_select.contains(
                "color: var(--fgc-tag-ink);\n      text-shadow: 0 2px 10px rgb(0 0 0 / 70%);"
            )
            && map_select.contains("tag: logoUrl ? '' : teamLogoPlaceholderText(winnerTeam),")
            && map_select.contains("logoText = escapeHtml(winnerDetailsForCard.tag);")
            && map_select.contains("logo.textContent = details.tag;"),
        "map select should render the winning team's tag when its logo is empty"
    );

    let map_pool = overlay_text("map-pool.html");
    assert!(
        map_pool.contains("function teamLogoPlaceholderText(team)")
            && map_pool.contains("function chosenByTag(mapData)")
            && map_pool.contains("--fgc-tag-ink: #eef2f8;")
            && map_pool.contains(".map-logo-overlay__tag {")
            && map_pool.contains(
                "background: linear-gradient(180deg, rgb(11 13 18 / 82%), rgb(11 13 18 / 58%));"
            )
            && map_pool.contains(
                "color: var(--fgc-tag-ink);\n      text-shadow: 0 2px 10px rgb(0 0 0 / 70%),"
            )
            && map_pool.contains(
                "const isPicked = !!(mapData.chosenBy && (mapData.chosenByLogo || tag));"
            )
            && map_pool.contains("chosenByTag: chosenByTag(map)"),
        "map pool should mark a pick and render the team tag when the logo is empty"
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
        text.contains(".mode-item--active,\n    .mode-item--active-first,\n    .mode-item--active-last,\n    .mode-item--active-single {")
            && text.contains("font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;")
            && text.contains("font-weight: 600;"),
        "only the selected game mode should use the heavier face"
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
fn game_overlay_uses_position_side_gradients_without_resizing_bars() {
    let text = overlay_text("game-overlay.html");

    assert!(
        text.contains("--fgc-bar-base: #0b0d12;")
            && text.contains("--fgc-bar-ink: #eef2f8;")
            && text.contains("--fgc-team-left: #2f6feb;")
            && text.contains("--fgc-team-left-deep: #173d78;")
            && text.contains("--fgc-team-right: #df4450;")
            && text.contains("--fgc-team-right-deep: #792430;"),
        "the score bar needs fixed broadcast side colours"
    );
    assert!(
        text.contains(".team-bar--home {")
            && text.contains(".team-bar--away {")
            && text.contains(".team-bar--home.swapped {")
            && text.contains(".team-bar--away.swapped {"),
        "the side treatment must be expressible for both teams in both orders"
    );
    assert!(
        text.contains("linear-gradient(270deg, var(--fgc-team-left) 0%, var(--fgc-team-left-deep) 38%, var(--fgc-bar-base) 100%)")
            && text.contains("linear-gradient(90deg, var(--fgc-team-right) 0%, var(--fgc-team-right-deep) 38%, var(--fgc-bar-base) 100%)"),
        "default bars should be strongest at the inner score end"
    );
    assert!(
        text.contains(".team-bar--home.swapped {\n      background: linear-gradient(90deg, var(--fgc-team-right)")
            && text.contains(".team-bar--away.swapped {\n      background: linear-gradient(270deg, var(--fgc-team-left)"),
        "swapping sides must keep blue on the left and red on the right"
    );
    assert!(
        text.contains(".team-bar--home.team-colored {\n      background: linear-gradient(270deg, color-mix(in srgb, var(--team-a-color,")
            && text.contains(".team-bar--away.team-colored {\n      background: linear-gradient(90deg, color-mix(in srgb, var(--team-b-color,")
            && text.contains(".team-bar--home.swapped.team-colored {\n      background: linear-gradient(90deg, color-mix(in srgb, var(--team-a-color,")
            && text.contains(".team-bar--away.swapped.team-colored {\n      background: linear-gradient(270deg, color-mix(in srgb, var(--team-b-color,")
            && !text.contains("background: var(--team-a-color, var(--fgc-surface));")
            && !text.contains("background: var(--team-b-color, var(--fgc-surface));"),
        "team colours must be a reversed wash rather than a flat fill"
    );
    assert!(
        text.contains(".team-bar__logo {\n      --logo-fade: to right;\n      --logo-clip: inset(12px 0 17px 28px);\n      width: 75px;")
            && text.contains("background-color: transparent;")
            && text.contains("filter: drop-shadow(0 3px 5px rgb(0 0 0 / 62%));")
            && !text.contains("--team-logo-bg"),
        "the logo must sit directly on the bar without a solid tile"
    );
    assert!(
        text.contains(".team-bar > .team-bar__logo {\n      clip-path: none;")
            && text.contains(".team-bar__logo::after {")
            && text.contains(".team-bar__logo::before {")
            && text.contains("width: 134px;")
            && text.contains("height: 84px;")
            && text.contains("top: -12px;")
            && text.contains("left: -28px;")
            && text.matches("clip-path: var(--logo-clip);").count() == 2
            && text.contains("background: linear-gradient(var(--logo-fade), rgb(3 3 6 / 94%) 0%, rgb(3 3 6 / 74%) 46%, transparent 82%);")
            && text.contains("-webkit-mask-image: linear-gradient(var(--logo-fade), #000 0%, #000 88%, transparent 100%);"),
        "the oversized logo art must be clipped on all sides and fade toward the name"
    );
    assert!(
        text.contains(".team-bar__logo {\n      --logo-fade: to right;")
            && text.contains(".team-bar--away .team-bar__logo,\n    .team-bar--home.swapped .team-bar__logo {\n      --logo-fade: to left;\n      --logo-clip: inset(12px 28px 17px 0);")
            && text.contains(".team-bar--away.swapped .team-bar__logo {\n      --logo-fade: to right;\n      --logo-clip: inset(12px 0 17px 28px);")
            && text.contains(".team-bar--away .team-bar__logo::before,\n    .team-bar--away .team-bar__logo::after,")
            && text.contains(".team-bar--away.swapped .team-bar__logo::before,\n    .team-bar--away.swapped .team-bar__logo::after {")
            && text.contains("right: -28px;"),
        "the overscan, crop and fade must mirror on right-side logos"
    );
    assert!(
        text.contains(".center-ban {\n      position: absolute;\n      top: 12px;\n      width: 47px;\n      height: 50px;")
            && text.contains(".center-ban--home {\n      left: 762px;\n    }")
            && text.contains(".center-ban--away {\n      right: 762px;\n    }")
            && text.contains(".center-ban.hidden {")
            && text.contains("width: 35px;\n      height: 42px;\n      border-radius: 5px;")
            && text.contains(
                "box-shadow: inset 0 0 0 1px rgb(238 242 248 / 18%), 0 2px 6px rgb(0 0 0 / 45%);"
            )
            && text.contains("filter: saturate(0.92) contrast(1.06);")
            && text.contains(".team-bar__ban-icon::after {")
            && text.contains(".team-bar__ban-icon::before {")
            && text.contains("--ban-accent: var(--fgc-team-right);")
            && text.contains("clip: rect(0, 0, 0, 0);")
            && !text.contains("font-size: 9px;")
            && !text.contains("letter-spacing: 0.06em;"),
        "the bans must be compact chips flanking the centre status bar in fixed boxes"
    );
    assert!(
        text.contains("@keyframes banChipIn {")
            && text.contains("opacity: 0;\n        transform: translateY(8px) scale(0.88);")
            && text.contains("opacity: 1;\n        transform: none;")
            && text.contains("z-index: 2;\n      opacity: 0;")
            && text.contains(
                ".center-ban.animate-ban {\n      animation: banChipIn 0.36s cubic-bezier(0.16, 1, 0.3, 1) var(--ban-delay, 2.5s) both;\n    }"
            )
            && text.contains("function revealBanChips(delayMs) {")
            && text.contains("chip.style.setProperty('--ban-delay', `${delayMs + index * 120}ms`);")
            && text.contains("revealBanChips(animationsTriggered ? 0 : 2500);"),
        "the centre ban chips must animate after the other entrance animations"
    );
    assert!(
        text.contains(".team-bar {\n      position: absolute;\n      top: 52px;\n      width: 525px;\n      height: 55px;")
            && text.contains("the oversized logo artwork. */\n      overflow: hidden;\n      clip-path: inset(0 round 6px);")
            && text.contains(".team-bar--home {\n      left: 43px;")
            && text.contains(".team-bar--away {\n      left: 1348px;"),
        "the team bar bounding boxes and final crop must not change"
    );
    assert!(
        text.contains(
            ".top-bar {\n      position: absolute;\n      top: 13px;\n      height: 25px;"
        ) && text.contains(".top-bar--left {\n      left: 43px;\n      width: 525px;")
            && text.contains(".top-bar--right {\n      left: 1348px;\n      width: 525px;"),
        "the two top bars must keep their exact footprint"
    );
    assert!(
        text.contains(".match-bug {\n      position: absolute;\n      top: 12px;\n      left: 821px;\n      width: 278px;\n      height: 50px;"),
        "the centre status bar must keep its exact footprint"
    );
    assert!(
        text.contains(".title-text {\n      flex: 1 1 0;\n      min-width: 0;\n      display: block;\n      overflow: hidden;\n      text-overflow: ellipsis;\n      white-space: nowrap;")
            && text.contains(".subtitle-text {\n      flex: 1 1 0;\n      min-width: 0;\n      display: block;\n      overflow: hidden;\n      text-overflow: ellipsis;\n      white-space: nowrap;"),
        "long title and subtitle text must ellipsise inside the single-line info bar"
    );
    assert!(
        text.contains(".top-bar--left::before {")
            && text.contains("background-image: linear-gradient(180deg, var(--fgc-accent-strong), var(--fgc-accent));")
            && text.contains(".match-bug::before {")
            && text.contains("background: linear-gradient(90deg, transparent, var(--fgc-accent), transparent);"),
        "the top bars should carry restrained orange datums"
    );
    assert!(
        !text.contains("transition: all 0.2s;"),
        "the mode segments must use explicit token transitions"
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
            && text.contains(".team-bar--home .team-bar__score { order: 3; }")
            && text.contains(".team-bar--away .team-bar__score { order: 2; }")
            && text.contains(".team-bar--away .team-bar__logo { order: 4; }")
            && !text.contains(".team-bar--home .team-bar__ban { order:")
            && !text.contains(".team-bar--away .team-bar__ban { order:"),
        "the logo and score keep their order; bans no longer occupy the bars"
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
fn hero_bans_reveals_each_new_ban_without_replaying_the_opening() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains("function animateBanReveal(side, firstSide)")
            && text.contains("function prefersReducedMotion()")
            && text.contains("const BAN_REVEAL_DURATION = 560;")
            && text.contains("const BAN_REVEAL_STAGGER = 90;")
            && text.contains("let renderedHomeHero = '';")
            && text.contains("let renderedAwayHero = '';"),
        "each ban should have its own tracked reveal instead of replaying the scene"
    );
    assert!(
        text.contains("[art, [{ opacity: 0, filter: 'blur(8px)' }, { opacity: 1, filter: 'blur(0px)' }]]")
            && text.contains("[name, [{ opacity: 0, transform: 'translateY(16px)' }, { opacity: 1, transform: 'translateY(0)' }]]")
            && text.contains("[role, [{ opacity: 0, transform: 'scale(0.8)' }, { opacity: 1, transform: 'scale(1)' }]]"),
        "the hero art, name and role icon should all animate in"
    );
    assert!(
        text.contains("const baseDelay = side === firstSide ? 0 : BAN_REVEAL_STAGGER;")
            && text.contains(
                "animation.finished.then(() => animation.cancel(), () => animation.cancel());"
            ),
        "the first ban should lead and each reveal should clean up its own fill"
    );
    assert!(
        text.contains("if (nextHomeHero && renderedHomeHero !== nextHomeHero) {\n            animateBanReveal('home', firstSide);\n          }")
            && text.contains("if (nextAwayHero && renderedAwayHero !== nextAwayHero) {\n            animateBanReveal('away', firstSide);\n          }")
            && text.contains("if (opening) {\n          triggerAnimations();\n          animationsTriggered = true;\n        }"),
        "updates should reveal only changed bans while the opening stays a one-shot"
    );
}

#[test]
fn hero_bans_keeps_unchanged_images_when_another_ban_updates() {
    let text = overlay_text("hero-bans.html");

    assert!(
        text.contains("const currentImage = art.querySelector('img');")
            && text.contains("if (!currentImage || currentImage.getAttribute('src') !== source) {")
            && text.contains("if (roleIcon.getAttribute('src') !== icon) roleIcon.src = icon;"),
        "unchanged hero art and role icons should stay mounted across ban updates"
    );
    assert!(
        text.contains("// Keep an image whose source is unchanged so unrelated ban picks do not")
            && text.contains("// make the browser reload the same art."),
        "the overlay should explain why unchanged images are preserved"
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
        text.contains("flex: 0 0 22px;"),
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
            text.contains("const DARK_TEXT = '#0b0d12';")
                && text.contains("const LIGHT_TEXT = '#eef2f8';")
                && text.contains("function contrastTextColor(rgb) {")
                && text.contains("return luminance > 0.5 ? DARK_TEXT : LIGHT_TEXT;"),
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
fn roster_overlays_use_hexagon_identity_without_touching_the_rows() {
    for file in ["team-roster-home.html", "team-roster-away.html"] {
        let text = overlay_text(file);

        assert!(
            text.contains(
                "--fgc-hex-cell: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);"
            ),
            "{file} should declare one pointy-top hexagon cell shape"
        );
        assert!(
            text.contains(".player-card-core .role-badge {")
                && text.contains(".player-card-sub .role-badge {")
                && text.contains("clip-path: var(--fgc-hex-cell);")
                && text.contains("bottom: 16px;")
                && text.contains("width: 56px;")
                && text.contains("height: 56px;")
                && text.contains("width: 40px;")
                && text.contains("height: 40px;")
                && text.contains(".player-card-sub .sub-portrait {")
                && text.contains("bottom: 8px;")
                && text.contains("left: 50%;")
                && text.contains("transform: translateX(-50%);")
                && text.contains("width: 46px;")
                && text.contains("height: 54px;")
                && text.contains("width: 34px;")
                && text.contains("height: 34px;"),
            "{file} should centre both clipped role badges and use a proportional sub hexagon"
        );
        assert!(
            !text.contains(".core-stage")
                && !text.contains(".core-rail")
                && !text.contains("core-rail--left")
                && !text.contains("core-rail--right"),
            "{file} should not decorate the sides of the core row"
        );
        assert!(
            text.contains("const ROLE_RANK = { damage: 0, tank: 1, support: 2 };")
                && text.contains("function sortPlayersByRole(list)")
                && text.contains("const corePlayersList = sortPlayersByRole(")
                && text.contains("const subs = sortPlayersByRole("),
            "{file} should order both rows Damage, Tank, Support and keep order inside a role"
        );
        assert!(
            !text.contains("data-initials")
                && !text.contains("playerInitials")
                && !text.contains("teamCode")
                && !text.contains("player-code"),
            "{file} should leave the name plate plain and defer country or tag codes to the backlog"
        );
        assert!(
            text.contains("text-overflow: ellipsis;")
                && text.contains("white-space: nowrap;")
                && text
                    .matches("title=\"${escapeHtml(player.name || 'PLAYER')}\"")
                    .count()
                    == 2,
            "{file} should truncate long names and keep the full value available"
        );
        assert!(
            text.contains("height: 126px;")
                && text.contains("padding: 20px 24px;")
                && text.contains("font-size: 18px;\n      line-height: 22px;")
                && text.contains("font-size: 14px;\n      line-height: 16px;"),
            "{file} should grow the name plate and reduce both pronoun lines"
        );

        assert!(
            text.contains(".roster-wash {")
                && text
                    .contains("left: 0;\n      right: 0;\n      bottom: 0;\n      height: 420px;")
                && text.contains("linear-gradient(0deg, var(--fgc-wash), transparent 100%);")
                && text.contains(".roster-wash::after {")
                && text.contains("bottom: -100px;")
                && text.contains("height: 300px;")
                && text.contains("filter: blur(24px);")
                && text.contains("linear-gradient(0deg, var(--fgc-glow), transparent 100%);")
                && !text.contains("radial-gradient(")
                && !text.contains("width: 760px;")
                && !text.contains("linear-gradient(90deg, var(--fgc-wash)")
                && !text.contains("linear-gradient(270deg, var(--fgc-wash)")
                && !text.contains(".overlay[data-roster-side=\"home\"] .roster-wash")
                && !text.contains(".overlay[data-roster-side=\"away\"] .roster-wash")
                && text.contains("--fgc-wash: rgb(255 122 26 / 8%);")
                && text.contains("--fgc-glow: rgb(255 122 26 / 9%);")
                && text.contains("rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alphas.wash})")
                && text.contains("rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alphas.glow})")
                && text.contains(
                    "el.overlay.dataset.rosterSide = TEAM_SIDE === 'A' ? 'home' : 'away';"
                ),
            "{file} should use a full-width bottom glow with no side wash or side-anchored gradient"
        );
        assert!(
            text.contains("el.overlay.style.setProperty('--fgc-team', teamData.color);")
                && text.contains("el.overlay.style.setProperty('--fgc-wash'")
                && text.contains("el.overlay.style.setProperty('--fgc-glow'")
                && text.contains("el.overlay.style.removeProperty('--fgc-team');")
                && text.contains("el.overlay.style.removeProperty('--fgc-wash');")
                && text.contains("el.overlay.style.removeProperty('--fgc-glow');"),
            "{file} should drive wash and glow from the team colour and reset with the theme"
        );

        let core = text
            .find("id=\"core-players\"")
            .unwrap_or_else(|| panic!("{file} should keep the core player root"));
        let separator = text
            .find("class=\"roster-separator\"")
            .unwrap_or_else(|| panic!("{file} should place a hexagon separator"));
        let subs = text
            .find("id=\"substitutes\"")
            .unwrap_or_else(|| panic!("{file} should keep the substitute root"));
        assert!(
            core < separator && separator < subs,
            "{file} should place the separator between the two rectangular rows"
        );

        assert!(
            text.contains(".player-card-core .player-name {\n      width: 100%;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;")
                && text.contains(".player-card-core .player-pronouns {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;"),
            "{file} should keep the existing name and pronoun type rules"
        );
    }
}

#[test]
fn roster_overlays_apply_a82_team_bar_divider_and_colour_options() {
    for file in ["team-roster-home.html", "team-roster-away.html"] {
        let text = overlay_text(file);

        assert!(
            text.contains("@keyframes dividerReveal {")
                && text.contains("@keyframes dividerLine {")
                && text.contains("@keyframes dividerHex {")
                && text.contains("animation: dividerReveal 360ms ease-out 1.6s both;")
                && text.contains("animation: dividerLine 480ms var(--fgc-ease-out) 1.6s both;")
                && text.contains("animation: dividerHex 480ms var(--fgc-ease-out) 1.7s both;")
                && text.contains("@media (prefers-reduced-motion: reduce) {")
                && text.contains("animation: none !important;"),
            "{file} should animate the core/sub divider and respect reduced motion"
        );

        assert!(
            text.contains(".team-logo {")
                && text.contains("padding: 12px;")
                && text.contains("background-origin: content-box;")
                && text.contains("background-color: var(--fgc-surface-inverse);"),
            "{file} should inset uploaded logos inside the full team-logo panel"
        );

        assert!(
            text.contains("function parseHexColor(value) {")
                && text.contains("/^[0-9a-fA-F]{6}$/.test(hex)")
                && text.contains("function relativeLuminance(rgb) {")
                && text.contains(
                    "0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)"
                )
                && text.contains("function gradientAlphas(rgb) {")
                && text.contains("0.20 - luminance * 0.16")
                && text.contains("clamp(0.20 - luminance * 0.16, 0.08, 0.20)")
                && text.contains("clamp(wash + 0.01, 0.09, 0.21)"),
            "{file} should compensate gradient strength by perceived luminance"
        );

        assert!(
            text.contains("const rgb = parseHexColor(teamData.color);")
                && text.contains("const selectedText = parseHexColor(teamData.textColor);")
                && text.contains(
                    "el.teamName.style.color = selectedText ? teamData.textColor : contrastTextColor(rgb);"
                )
                && text.contains("const alphas = gradientAlphas(rgb);")
                && text.contains("rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alphas.wash})")
                && text.contains("rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alphas.glow})")
                && !text.contains("rgba(${r}, ${g}, ${b}, 0.08)")
                && !text.contains("rgba(${r}, ${g}, ${b}, 0.09)"),
            "{file} should honour the selected team text colour and use compensated alphas"
        );

        assert!(
            text.contains("const roster = Object.assign({}, d.teamRoster);")
                && text.contains(
                    "for (const key of ['teamColorsEnabled', 'roleIcons', 'showPlayerPortraits']) {"
                )
                && text.contains("if (d[key] !== undefined) roster[key] = d[key];")
                && text.contains("return updateOverlay({ type: 'teamRoster', data: roster });")
                && !text
                    .contains("return updateOverlay({ type: 'teamRoster', data: d.teamRoster });"),
            "{file} should carry saved display settings through init replay"
        );
    }
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
        text.contains("rgb(var(--pick-glow-rgb, 255 122 26) / 62%)")
            && text.contains("rgb(var(--pick-glow-rgb, 255 122 26) / 34%)"),
        "the chosen map should be lit in the choosing team's colour"
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
                "animation: columnReveal var(--pool-anim-duration) var(--fgc-ease-out) both;"
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
            && text.contains("rgb(var(--pick-glow-rgb, 255 122 26) / 62%)")
            && text.contains("linear-gradient("),
        "the picked map should carry a team-colour gradient wash above the artwork"
    );
    assert!(
        text.contains(".map-card.picked .map-name {")
            && text.contains("box-shadow: inset 0 2px 0 rgb(var(--pick-glow-rgb, 255 122 26));"),
        "the picked map should keep an accent rule in the choosing team's colour"
    );
    assert!(
        !text.contains(".map-image.chosen::after"),
        "the old flat wash behind the greyscale filter should be gone"
    );
}

#[test]
fn map_pool_applies_the_clean_canvas_and_targeted_update_contract() {
    let text = overlay_text("map-pool.html");

    assert!(
        !text.contains("map-pool-header") && !text.contains("fitMapNames"),
        "map-pool should drop the giant title and the old one-line fitting pass"
    );
    assert!(
        text.contains(".map-image::after {")
            && text.contains("rgba(0, 0, 0, 0.78) 100%")
            && text.contains("margin-inline: -1px;"),
        "map art should carry the same bottom gradient and seam overscan as map-select"
    );
    assert!(
        text.contains("container-type: inline-size;")
            && text.contains("font-size: clamp(13px, 9cqw, 20px);")
            && text.contains("-webkit-line-clamp: 2;"),
        "long map names should shrink and wrap rather than truncate"
    );
    assert!(
        text.contains(".map-card__swap {")
            && text.contains("<div class=\"map-card__swap\"></div>")
            && text.contains(".map-card__text-ghost {")
            && text.contains("nameGhost.animate("),
        "a live map change should be covered once and crossfade the old name out"
    );
    assert!(
        text.contains("delay: index === 0 ? 0 : 60,"),
        "art should lead while the name crossfade shares one beat"
    );
    assert!(
        text.contains("--pick-glow-rgb")
            && text.contains("data?.teamA?.color")
            && text.contains("data?.teamB?.color"),
        "the picked map should glow in the choosing team's colour"
    );
    assert!(
        text.contains("function reconcilePool(")
            && text.contains("function refreshCard(")
            && text.contains("function settleOpening(")
            && text.contains("openingSettled")
            && text.contains("if (signature === renderedSignature) return;"),
        "live frames should reconcile changed cards instead of replaying the opening"
    );
    assert!(
        !text.contains("el.mapPool.innerHTML = '';"),
        "a pool update should never rebuild every column from scratch"
    );
}

#[test]
fn map_pool_animates_mode_marks_and_team_pick_reveal() {
    let text = overlay_text("map-pool.html");

    assert!(
        text.contains("@keyframes modeMarkReveal")
            && text.contains(".animate-mode {")
            && text.contains("column.querySelectorAll('.mode-icon, .mode-name')")
            && text.contains(".animate-column, .animate-card, .animate-mode"),
        "mode marks should enter on their own staggered beat with the columns"
    );
    assert!(
        !text.contains("clip-path: polygon("),
        "game mode icons should be plain marks without a hexagon frame"
    );
    assert!(
        text.contains("grayscale(100%) brightness(0.56) saturate(0.45);"),
        "a picked map should sit darker behind the winner wash"
    );
    assert!(
        text.contains("rgb(var(--pick-glow-rgb, 255 122 26) / 62%) 0%")
            && text.contains("rgb(var(--pick-glow-rgb, 255 122 26) / 48%) 0%"),
        "the pick gradient should carry a stronger team-colour bottom wash"
    );
    assert!(
        text.contains("@keyframes pickWashReveal")
            && text.contains(".map-card.animate-pick .map-image::before")
            && text.contains("card.classList.add('animate-pick')"),
        "the radial winner wash should fade up when a team takes a map"
    );
    assert!(
        text.contains("const pickChanged = pickKey(previous) !== pickKey(nextMap);")
            && text.contains("glow.animate(")
            && text.contains("refreshPickedState(card, nextMap, colors, animations, pickChanged)"),
        "a pick change should join the existing one-batch card update"
    );
    assert!(
        text.contains("const scoreboard = d.scoreboard || d;")
            && text.contains("data: { ...scoreboard, mapPoolData }"),
        "the keyed init frame should unwrap the scoreboard so team colours survive"
    );
}

#[test]
fn map_pool_lifts_dark_team_colours_for_the_pick_glow() {
    let text = overlay_text("map-pool.html");

    assert!(
        text.contains("const PICK_LIGHTEN_MIX = 0.45;")
            && text.contains("function lightenedRgbTriplet(color)")
            && text.contains("A: lightenedRgbTriplet(data?.teamA?.color)")
            && text.contains("B: lightenedRgbTriplet(data?.teamB?.color)"),
        "the pick glow should lift dark team colours toward white while keeping their hue"
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
        "the card and text panel should sit on the raised surface"
    );
    assert!(
        text.contains(
            ".map-card__image--placeholder {\n      background: var(--fgc-surface-muted);\n    }"
        ) && !text.contains("linear-gradient(180deg"),
        "the placeholder image should be a solid muted surface"
    );
    assert!(
        !text.contains("background: var(--fgc-accent-soft);"),
        "the map art should not carry an accent scrim"
    );
    assert!(
        text.contains(".map-card__image--complete {")
            && text.contains("brightness(0.52) saturate(0.45);"),
        "completed map art should fade back behind the winner crest"
    );
    assert!(
        text.contains(".map-card__image::after {")
            && text.contains("background: linear-gradient(")
            && text.contains("to bottom,")
            && text.contains("rgba(0, 0, 0, 0) 0%,")
            && text.contains("rgba(0, 0, 0, 0.28) 48%,")
            && text.contains("rgba(0, 0, 0, 0.78) 100%"),
        "map art should carry a black gradient that darkens towards its end"
    );
    assert!(
        text.contains(".map-card__type {\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 14px;\n      color: var(--fgc-accent);"),
        "the map mode label should carry the orange accent"
    );
    assert!(
        text.contains(".map-card__chosen-by {\n      width: 100%;\n      font-family: 'FGC Display', 'Apotek', sans-serif;\n      font-size: 12px;\n      color: var(--fgc-text-muted);"),
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
fn map_select_shows_the_ban_icons_on_the_map() {
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
        text.contains("const iconUrl = heroIcons[heroName];")
            && !text.contains("heroPortraits")
            && !text.contains("heroBadges"),
        "the bans should use the hero ban icons, not portraits or badge artwork"
    );
    assert!(
        text.contains(
            "<div class=\"${visualClass}\">\n          <div class=\"${imageClass}\" id=\"image-${index}\" style=\"${imageStyle}\"></div>\n          <div class=\"map-card__swap\"></div>\n          ${createBanStrip(bans)}\n        </div>"
        ),
        "the ban strip should sit over the map image"
    );
    assert!(
        text.contains(
            ".map-card__bans {\n      position: absolute;\n      left: 50%;\n      bottom: 16px;\n      transform: translateX(-50%);\n      display: flex;\n      gap: 18px;"
        ),
        "the ban icons should align to the bottom of the map image"
    );
    assert!(
        text.contains(
            ".map-card__ban {\n      position: relative;\n      width: 52px;\n      height: 60px;"
        ) && text
            .contains("clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);"),
        "each ban icon should sit inside a hexagon mask"
    );
    assert!(
        text.contains(".map-card__ban::before {")
            && text.contains("background: var(--fgc-accent);"),
        "each hex ban should carry an accent ring"
    );
    assert!(
        text.contains(".map-card__ban img {")
            && text.contains("object-fit: contain;")
            && text.contains("inset: 2px;"),
        "each ban icon should fill the inner hex with a small ring gap"
    );
    assert!(
        text.contains(
            ".map-card__ban-number {\n      position: absolute;\n      top: -9px;\n      right: -8px;\n      width: 26px;\n      height: 30px;\n      background: var(--fgc-accent);\n      clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);"
        ) && text.contains(".map-card__ban-number::before {")
            && text.contains("inset: 2px;")
            && !text.contains("border-radius: 10px;")
            && !text.contains("transform: rotate(45deg);"),
        "the ban order should use the same upright hexagon ring as the ban icon"
    );
    assert!(
        text.contains("@keyframes banReveal {")
            && text.contains("animation: banReveal 420ms var(--ease-out-expo) both;")
            && text.contains("style=\"--ban-delay: ${banDelay}\""),
        "ban icons should reveal in a stagger"
    );
}

#[test]
fn map_select_keeps_a_clean_canvas_with_centred_winners() {
    let text = overlay_text("map-select.html");

    assert!(
        text.contains(
            ".map-card--current {\n      flex: var(--current-grow, 1.8) 1 0;\n    }"
        ) && !text.contains("map-card__current")
            && !text.contains("box-shadow: inset 0 0 0 2px var(--fgc-accent);")
            && !text.contains("Now playing")
            && text.contains(".map-card--current.animate-card {\n      --current-grow: 1;")
            && text.contains("@keyframes mapCardFocus {")
            && text.contains("mapCardFocus 760ms var(--ease-out-expo) forwards;")
            && text.contains(
                "var(--focus-delay, calc(var(--card-delay, var(--anim-delay)) + 160ms));"
            )
            && text.contains(
                "card.style.setProperty('--focus-delay', `calc(var(--anim-delay) + ${settleAfter}s)`);"
            )
            && text.contains("0.1 + (mapsToShow - 1) * 0.1 + 1.25;"),
        "the current map should widen only after the rest of the row has settled"
    );
    assert!(
        text.contains("function safeHexColor(value, fallback) {")
            && text.contains("/^#?([0-9a-fA-F]{6})$/")
            && text.contains("--winner-color: ${winnerColor};")
            && text.contains("card.dataset.winner = aWon ? 'a' : bWon ? 'b' : 'none';"),
        "team colours should be validated before they reach the winner glow"
    );
    assert!(
        text.contains(
            ".map-card__logo {\n      position: absolute;\n      top: calc(50% - 52px);\n      left: 50%;\n      transform: translate(-50%, -50%);\n      width: 112px;"
        ) && text.contains("drop-shadow(0 6px 18px rgba(0, 0, 0, 0.65))")
            && text.contains("drop-shadow(0 0 16px var(--winner-color, var(--fgc-accent)))")
            && text.contains(".map-card__logo--winner::before {")
            && text.contains("border-radius: 50%;")
            && !text.contains(".map-card__logo--winner::after")
            && !text.contains("inset: -12px;"),
        "the winner logo should sit at the image centre without the old giant hex plate"
    );
    assert!(
        !text.contains("filter: grayscale")
            && !text.contains("background: var(--fgc-accent-soft);")
            && text.contains(".map-card__image--complete {")
            && text.contains("brightness(0.52) saturate(0.45);")
            && text.contains(".map-card__visual--complete::after {")
            && text.contains("radial-gradient(")
            && text.contains("var(--winner-color, var(--fgc-accent)),")
            && text.contains("opacity: 0.13;"),
        "completed art should fade back under a subtle team-colour gradient glow"
    );
    assert!(
        !text.contains("gameModeIcons")
            && !text.contains("map-card__mode-icon")
            && text.contains(".map-card__meta {")
            && text.contains("<span class=\"map-card__type\">${escapeHtml(mapType || '')}</span>"),
        "the footer should keep the map type text without a game mode icon"
    );
    assert!(
        !text.contains("map-card__board")
            && !text.contains("map-card__score-team")
            && !text.contains("map-card__score-side"),
        "map select should not show score slabs"
    );
    assert!(
        text.contains("@keyframes winnerLogoReveal {")
            && text.contains("@keyframes footerReveal {")
            && text.contains("animation: footerReveal 420ms var(--ease-out-expo) both;")
            && text.contains("@media (prefers-reduced-motion: reduce) {")
            && text.contains(".map-card__ban,")
            && text.contains(
                ".map-card--current.animate-card {\n        --current-grow: 1.8;\n      }"
            ),
        "minor components should animate in and respect reduced motion"
    );
}

#[test]
fn map_select_reconciles_live_data_without_replaying_the_opening() {
    let text = overlay_text("map-select.html");

    assert!(
        text.contains("margin-inline: -1px;")
            && text
                .contains("/* Flex rounding can leave a one-pixel gutter at either card edge. */"),
        "map art should overscan flex rounding at both card edges"
    );
    assert!(
        text.contains("/* A dark curtain covers map swaps so no empty surface can flash through. */")
            && text.contains(".map-card__swap {\n      position: absolute;\n      inset: -1px;\n      z-index: 2;\n      opacity: 0;\n      pointer-events: none;\n      background:\n        linear-gradient(")
            && text.contains("<div class=\"map-card__swap\"></div>")
            && text.contains("swapLayer.style.opacity = '1';\n      applyUpdate();")
            && text.contains("inAnimations.push(\n        swapLayer.animate(")
            && text.contains("// Art leads; the old text fades out as type and name reveal together.")
            && text.contains("delay: index === 0 ? 0 : 60,")
            && !text.contains("CONTENT_OUT_DURATION")
            && !text.contains("CURTAIN_OUT_DELAY"),
        "map swaps should cover, swap once and reveal text as one fade in a single pass"
    );
    assert!(
        text.contains("container-type: inline-size;")
            && text.contains(
                ".map-card__name {\n      width: 100%;\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;\n      font-size: clamp(16px, 8cqw, 24px);\n      color: var(--fgc-text);\n      text-transform: uppercase;\n      text-align: center;\n      line-height: 1.1;\n      white-space: normal;\n      overflow-wrap: anywhere;\n      display: -webkit-box;\n      -webkit-box-orient: vertical;\n      -webkit-line-clamp: 2;\n      overflow: hidden;\n    }"
            ),
        "long map names should shrink with the card and then wrap to two lines"
    );
    assert!(
        text.contains(
            ".map-card--settled .map-card__ban,\n    .map-card--settled .map-card__text,\n    .map-card--settled .map-card__logo--winner {\n      animation: none !important;\n      opacity: 1;\n    }"
        ) && text.contains(".map-card--settled .map-card__logo--winner {\n      transform: translate(-50%, -50%);\n    }"),
        "post-opening updates should stop the CSS reveal animations from replaying"
    );
    assert!(
        text.contains("function reconcileMaps(")
            && text.contains("function refreshCardContent(")
            && text.contains("function animateContentSwap(")
            && text.contains("function setCurrentVisual(")
            && text.contains("function animateCardWidth(")
            && text.contains("function prefersReducedMotion()")
            && text.contains("} else if (animationsTriggered) {\n          reconcileMaps(")
            && text.contains("if (!openingSettled) {\n        settleInitialAnimations();\n        openingSettled = true;\n      }")
            && text.contains("renderedCurrentIndex = currentMapIndex;")
            && !text.contains("updateMapsContainer(d.maps, d.teamA, d.teamB);"),
        "live updates should reconcile existing cards instead of rebuilding the row"
    );
    assert!(
        text.contains("animateCardWidth(previousCard, 1.8, 1);")
            && text.contains("animateCardWidth(nextCard, 1, 1.8, () => {")
            && text.contains("[{ flexGrow: String(from) }, { flexGrow: String(to) }]")
            && text.contains("animation.finished.then(finish, finish);"),
        "a new current map should trade width with the old card in one handoff"
    );
    assert!(
        text.contains("const identityChanged =")
            && text.contains("!sameValue(previousMap.name, mapNameRaw)")
            && text.contains("!sameValue(previousMap.type, mapTypeRaw)")
            && text.contains("!sameValue(previousMap.mapImageUrl, mapImage)")
            && text.contains("animateContentSwap(card, [image, type, name], applyMapContent);"),
        "assigned map art and text should animate only when their own data changes"
    );
    assert!(
        text.contains("const applyMapContent = () => {")
            && text.contains("// Empty slots have no art to wipe, so a curtain only makes them flicker.")
            && text.contains("if (hasArt) {\n          animateContentSwap(card, [image, type, name], applyMapContent);\n        } else {\n          cancelCardSwaps(card);\n          applyMapContent();\n        }"),
        "empty map slots should update in place instead of fading through a curtain"
    );
    assert!(
        text.contains(
            ".map-card__text-ghost {\n      position: absolute;\n      box-sizing: border-box;\n      z-index: 4;\n      pointer-events: none;\n      animation: none !important;\n    }"
        )
            && text.contains("textGhost = text.cloneNode(true);")
            && text.contains("textGhost.classList.add('map-card__text-ghost');")
            && text.contains("// Art leads; the old text fades out as type and name reveal together.")
            && text.contains("if (textGhost) textGhost.remove();")
            && text.contains(
                "card.querySelectorAll('.map-card__text-ghost').forEach((ghost) => ghost.remove());"
            ),
        "old map text should crossfade out while the new text fades in"
    );
    assert!(
        text.contains("if (previousDetails.logoUrl !== details.logoUrl) {\n          animateLogoIn(logo, index);\n        }")
            && text.contains("function animateLogoIn(logo, index)"),
        "completion should reveal only the winning team logo"
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
        text.contains(".talent-text {")
            && text.contains("background: linear-gradient(180deg, var(--fgc-surface-raised), var(--fgc-surface));"),
        "the caster text panel should sit on the raised surface"
    );
    assert!(
        text.contains(".talent-name {\n      width: 100%;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;\n      font-size: 44px;\n      line-height: 1.02;\n      color: var(--fgc-text);"),
        "the caster name should use the Night Broadcast text token"
    );
    assert!(
        text.contains(".social-handle {\n      font-family: 'FGC Display Light', 'Apotek Light', sans-serif;\n      font-size: 22px;\n      color: var(--fgc-text-muted);"),
        "social handles should use muted text"
    );
    assert!(
        text.contains(".social-icon path {\n      fill: var(--fgc-text-muted);\n    }"),
        "social icons should take the muted tier from a stylesheet rule"
    );
    assert!(
        text.contains(".talent-image:not([src]),\n    .talent-image[src=\"\"] {")
            && !text.contains("background-image"),
        "a missing portrait should hide the image, and portraits should never be painted as CSS backgrounds"
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
fn talent_rows_keep_portraits_unobstructed() {
    for file in ["row-casters.html", "row-hosts.html", "row-interviews.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains("The photograph stays unobstructed")
                && text.contains("<img class=\"talent-image\" alt=\"\" />"),
            "{file} should render a real image with no decorative cover"
        );
        for retired in [
            ".talent-image-frame",
            ".caster-card::before",
            ".talent-media::after",
            "--wash-angle",
            ".talent-role",
            "background-image",
            "talentFrameReveal",
        ] {
            assert!(
                !text.contains(retired),
                "{file} still carries the retired talent overlay decoration {retired}"
            );
        }
    }
}

#[test]
fn talent_rows_reconcile_cards_by_stable_slot() {
    for (file, slot) in [
        ("row-casters.html", "caster:${index}"),
        ("row-hosts.html", "talent:${index}"),
        ("row-interviews.html", "interviewer:${index}"),
    ] {
        let text = overlay_text(file);
        assert!(
            text.contains("const renderedCards = new Map();")
                && text.contains("card.dataset.slot = item.slot;")
                && text.contains("card.dataset.signature = signature;")
                && text.contains("if (card.dataset.signature === signature) return false;"),
            "{file} should reuse cards and ignore repeated frames"
        );
        assert!(
            text.contains(slot),
            "{file} should key cards by a stable slot"
        );
        assert!(
            text.contains("if (items.length && !openingPlayed)")
                && text.contains("} else if (changed.length) {"),
            "{file} should play the opening once and animate only changed slots afterwards"
        );
    }

    let text = overlay_text("row-interviews.html");
    assert!(
        text.contains("slot: 'guest'") && text.contains("item.kind === 'guest'"),
        "the interview guest should keep its own stable slot"
    );
    assert!(
        text.contains(".talent-card--guest .talent-name {\n      color: var(--fgc-accent);\n    }")
            && text.contains("entry.subtext"),
        "the guest should keep the accent name and supplied subtext"
    );
}

#[test]
fn talent_rows_replay_load_in_when_shown() {
    for file in ["row-casters.html", "row-hosts.html", "row-interviews.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains("let overlayVisible = true;")
                && text.contains("function replayLoadIn(startDelay = 0) {")
                && text.contains("if (visible && !wasVisible) replayLoadIn();")
                && text.contains("card.__fgcAnimationTimer"),
            "{file} should replay the load-in only on a hidden-to-visible transition"
        );
    }
}

#[test]
fn talent_rows_delay_the_opening_load_in() {
    for file in ["row-casters.html", "row-hosts.html", "row-interviews.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains("const LOAD_IN_START_DELAY = 500;")
                && text.contains("replayLoadIn(LOAD_IN_START_DELAY);")
                && text.contains("LOAD_IN_START_DELAY + 1100 + items.length * 110")
                && text.contains("animateCard(card, startDelay + index * 110)")
                && text.contains("const startDelay = openingSettled ? 0 : LOAD_IN_START_DELAY;")
                && text.contains("animateCard(card, startDelay + index * 90)"),
            "{file} should hold the first load-in card until the startup delay elapses"
        );
    }
}

#[test]
fn talent_rows_fit_transparent_portraits_by_subject_box() {
    for file in ["row-casters.html", "row-hosts.html", "row-interviews.html"] {
        let text = overlay_text(file);
        assert!(
            text.contains("function subjectBox(img)")
                && text.contains("pixels[(y * width + x) * 4 + 3] > 8")
                && text.contains(
                    ".talent-image.portrait--cutout {\n      object-fit: contain;\n    }"
                )
                && text.contains("img.addEventListener('load', () => fitPortrait(img));")
                && text.contains("fitText(name, 44, 22)")
                && !text.contains("fitText(name, 48, 24)"),
            "{file} should centre transparent portraits on their visible subject"
        );
    }
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
fn every_overlay_has_an_early_document_overflow_guard() {
    const GUARD: &str =
        "    html, body { width: 100%; height: 100%; margin: 0; overflow: hidden; }";

    for file in ALL_OVERLAYS {
        let text = overlay_text(file).replace('\r', "");
        let guard = text
            .find(GUARD)
            .unwrap_or_else(|| panic!("{file} should contain the early document overflow guard"));

        for later in [".fgc-fx {", "<iframe"] {
            if let Some(index) = text.find(later) {
                assert!(
                    guard < index,
                    "{file} should declare the document guard before {later}"
                );
            }
        }
    }
}

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
            && !canonical.contains("html[data-bg=\"opaque\"] .fgc-fx"),
        "the canonical block should carry the opaque plate and keep the lattice in that mode"
    );
    assert!(
        canonical.contains("requestedBg === 'off' ? 'opaque'")
            && canonical.contains("requestedBg === 'on' ? 'transparent'")
            && canonical.contains("root.dataset.bg = bgMode"),
        "the canonical block should read the background mode from the URL"
    );
    assert!(
        canonical.contains(
            "} else if (root.dataset.bg !== 'opaque' && root.dataset.bg !== 'transparent') {\n        root.dataset.bg = 'opaque';"
        ),
        "the canonical block should default to opaque while preserving a host override"
    );
    assert!(
        canonical.contains("window.fgcUpdateBackground = (data) => {")
            && canonical.contains("payload.background")
            && canonical.contains("candidate.opaque"),
        "the canonical block should accept the background section from both live messages and init"
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
        text.contains("bgMode === 'opaque' || bgMode === 'transparent' ? bgMode : 'opaque'"),
        "the standalone background should keep its opaque default"
    );
    assert!(
        text.contains("data.type === 'background'")
            && text.contains("data.type === 'init' && d.background")
            && text.contains("payload.opaque"),
        "the standalone background should accept the background section from both live messages and init"
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
    // Numeric labels are Strong everywhere.
    assert!(
        overlay_text("map-select.html").contains(
            ".map-card__ban-number span {\n      position: relative;\n      z-index: 1;\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;"
        ),
        "the map select ban order should be SemiBold"
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
            ".mode-name {\n      display: inline-block;\n      font-family: 'FGC Display Medium', 'Apotek Medium', sans-serif;"
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

    // A person name is Medium; the game overlay team name is the heavier face.
    assert!(
        overlay_text("game-overlay.html")
            .contains(".team-bar__name {\n      flex: 1;\n      min-width: 0;\n      font-family: 'FGC Display SemiBold', 'Apotek Semibold', sans-serif;\n      font-size: 27px;")
            && overlay_text("game-overlay.html").contains("fitText(el.homeName, 27, 18);")
            && overlay_text("game-overlay.html").contains("fitText(el.awayName, 27, 18);"),
        "the game overlay team name should be SemiBold and slightly smaller"
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
