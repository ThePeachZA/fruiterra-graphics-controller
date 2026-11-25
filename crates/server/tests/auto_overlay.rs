//! Static checks for the post-v1 auto-overlay scene switcher.
//!
//! These are separate from the frozen-overlay checks so this page cannot
//! weaken the v1 font, network or message-subscription assertions.

use fgc_server::embedded_overlay;

const SCENES: [&str; 9] = [
    "game-overlay.html",
    "map-select.html",
    "hero-bans.html",
    "map-pool.html",
    "team-roster-home.html",
    "team-roster-away.html",
    "row-casters.html",
    "row-hosts.html",
    "row-interviews.html",
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
    assert!(text.contains("void incoming.offsetWidth;"));
    assert!(text.contains("requestAnimationFrame(() => {"));
    assert!(text.contains("incoming.dataset.visible = 'true';"));
    assert!(text.contains("if (displayed === target) return;"));
    assert!(text.contains("function replaceFrame(file)"));
    assert!(text.contains("previous.remove();") && text.contains("frames.delete(file);"));
    assert!(text.contains("const incoming = target ? replaceFrame(target) : null;"));
    assert!(
        text.contains("if (file === displayed) continue;")
            && text.contains("frames.delete(file);")
            && text.contains("}, SCENE_FADE_MS);")
    );
    assert!(text.contains("new WebSocket(`${scheme}://${target}`)"));
    assert!(text.contains("location.host"));
    assert!(text.contains("query.get('host')") && text.contains("query.get('port')"));
}

#[test]
fn auto_overlay_crossfades_scene_changes_over_six_hundred_milliseconds() {
    let bytes = embedded_overlay("auto.html").expect("auto overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("auto overlay should be UTF-8");

    assert!(
        text.contains("const SCENE_FADE_MS = 600;"),
        "auto scene changes must use a 600ms fade"
    );
    assert!(
        text.contains("transition: opacity 600ms ease, visibility 0s linear 600ms;")
            && text.contains("transition: opacity 600ms ease, visibility 0s;"),
        "both outgoing and incoming scenes must fade over 600ms"
    );
    assert!(
        text.contains("root.dataset.transition = 'fade';")
            && text.contains("root.dataset.transition = 'idle';")
            && text.contains("}, SCENE_FADE_MS);"),
        "the fade state must be observable and clear after 600ms"
    );
    assert!(
        text.contains("data-transition=\"idle\""),
        "the auto root should start with an idle transition state"
    );
    for forbidden in [
        "stinger",
        "fgcReplayStinger",
        "fgcStingerCut",
        "stingerFrames",
        "ensureStingerFrame",
    ] {
        assert!(
            !text.to_lowercase().contains(&forbidden.to_lowercase()),
            "the auto overlay should no longer contain stinger wiring: {forbidden}"
        );
    }
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
