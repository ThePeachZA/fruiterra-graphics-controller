//! Static checks for the post-v1 standalone timer overlay.
//!
//! Kept separate from the frozen-overlay checks so the new timer cannot weaken
//! the v1 font, network or message-subscription assertions.

use fgc_server::embedded_overlay;

#[test]
fn timer_overlay_is_embedded_and_renders_match_timer_ticks() {
    let bytes = embedded_overlay("timer.html").expect("timer overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("timer overlay should be UTF-8");

    assert!(text.contains("<!DOCTYPE html>"));
    assert!(text.contains("00:00:00"));
    assert!(text.contains("rel=\"icon\" href=\"data:,\""));
    assert!(text.contains("message.data.matchTimer"));
    assert!(text.contains("message.type === 'matchTimer'"));
    assert!(text.contains("message.data.action === 'reset'"));
    assert!(text.contains("settings.leadingZeroes !== false"));
    assert!(text.contains("settings.showHours !== false"));
    assert!(text.contains("hours * 60 + minutes"));
    assert!(text.contains("lastSettings = data.settings"));
    assert!(text.contains("aria-live=\"off\""));
}

#[test]
fn timer_overlay_applies_show_and_uses_the_origin_aware_socket() {
    let bytes = embedded_overlay("timer.html").expect("timer overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("timer overlay should be UTF-8");

    assert!(text.contains("message.type === 'show'"));
    assert!(text.contains("typeof message.visible === 'boolean'"));
    assert!(text.contains("fgc-timer--hidden"));
    assert!(text.contains("new WebSocket(`${scheme}://${target}`)"));
    assert!(text.contains("location.host"));
    assert!(text.contains("query.get('host')") && text.contains("query.get('port')"));
    assert!(text.contains("setTimeout(connect, 2000)"));
}

#[test]
fn timer_overlay_is_plain_and_has_no_external_sources() {
    let bytes = embedded_overlay("timer.html").expect("timer overlay should be embedded");
    let text = String::from_utf8(bytes.into_owned()).expect("timer overlay should be UTF-8");

    for scheme in ["http://", "https://"] {
        assert!(
            !text.contains(scheme),
            "timer overlay must not contain an external URL"
        );
    }
    assert!(
        !text.contains("ws://localhost:"),
        "timer overlay must not hardcode a localhost WebSocket"
    );
    assert!(
        !text.contains("setInterval") && !text.contains("Date.now"),
        "timer overlay must render broadcast ticks rather than run its own clock"
    );
    assert!(
        !text.contains("srcdoc"),
        "timer overlay must not load nested HTML"
    );
}
