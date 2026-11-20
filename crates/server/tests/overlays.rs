//! Static compatibility checks for the seven bundled overlays.
//!
//! These tests run against the same embedded bytes the release build serves, so
//! a missing overlay, an external network reference or an unported font fails
//! the build before the manual OBS checklist starts.

use fgc_server::{embedded_font, embedded_overlay};

struct OverlaySpec {
    file: &'static str,
    message_types: &'static [&'static str],
}

const OVERLAYS: [OverlaySpec; 7] = [
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
