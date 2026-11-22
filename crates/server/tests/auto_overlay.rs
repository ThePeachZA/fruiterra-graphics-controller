//! Static checks for the post-v1 auto-overlay scene switcher.
//!
//! These are separate from the frozen-overlay checks so this page cannot
//! weaken the v1 font, network or message-subscription assertions.

use fgc_server::embedded_overlay;

const SCENES: [&str; 7] = [
    "game-overlay.html",
    "map-select.html",
    "hero-bans.html",
    "map-pool.html",
    "team-roster-home.html",
    "team-roster-away.html",
    "row-casters.html",
];

#[test]
fn auto_overlay_is_embedded_and_accepts_only_bundled_scenes() {
    let bytes = embedded_overlay("auto.html").expect("auto overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("auto overlay should be UTF-8");

    assert!(text.contains("<!DOCTYPE html>"));
    assert!(text.contains("const OVERLAYS = Object.freeze(["));
    assert!(text.contains("const ALLOWED = new Set(OVERLAYS);"));
    assert!(text.contains("ALLOWED.has(file)"));
    assert!(text.contains("included.includes(candidate)"));
    for scene in SCENES {
        assert!(
            text.contains(&format!("'{scene}'")),
            "auto page should allow-list {scene}"
        );
    }
}

#[test]
fn auto_overlay_reads_the_auto_overlay_section_and_applies_show() {
    let bytes = embedded_overlay("auto.html").expect("auto overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("auto overlay should be UTF-8");

    assert!(text.contains("message.data.autoOverlay"));
    assert!(text.contains("message.type === 'autoOverlay'"));
    assert!(text.contains("message.type === 'show'"));
    assert!(text.contains("for (const frame of frames.values())"));
    assert!(text.contains("frame.dataset.visible = 'false';"));
    assert!(text.contains("if (visible && file === safeActive) frame.dataset.visible = 'true';"));
    assert!(text.contains("new WebSocket(`${scheme}://${target}`)"));
    assert!(text.contains("location.host"));
    assert!(text.contains("query.get('host')") && text.contains("query.get('port')"));
}

#[test]
fn auto_overlay_suppresses_the_favicon_request() {
    let bytes = embedded_overlay("auto.html").expect("auto overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("auto overlay should be UTF-8");

    assert!(
        text.contains(r#"<link rel="icon" href="data:,">"#),
        "the auto page should suppress the favicon request"
    );
}

#[test]
fn auto_overlay_has_no_external_sources_or_localhost_socket() {
    let bytes = embedded_overlay("auto.html").expect("auto overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("auto overlay should be UTF-8");

    for scheme in ["http://", "https://"] {
        assert!(
            !text.contains(scheme),
            "auto page must not contain an external URL"
        );
    }
    assert!(
        !text.contains("ws://localhost:"),
        "auto page must not hardcode a localhost WebSocket"
    );
    assert!(
        !text.contains("srcdoc"),
        "auto page must only load allow-listed overlay files"
    );
}
