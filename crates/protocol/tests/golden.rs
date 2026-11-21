//! Parsed-JSON golden tests for the wire contract.
//!
//! Fixtures live in `tests/fixtures/` and are derived field-by-field from the
//! original Electron projection (`src/views/MatchController.tsx`). Comparison
//! is on parsed JSON: key order is irrelevant, but every field and value must
//! match exactly.

mod support;

use std::fs;
use std::path::PathBuf;

use fgc_core::AppState;
use fgc_protocol::{
    AssetCatalog, Message, Projector, SectionCache, TimerActionDto, auto_overlay, show,
    timer_action, timer_tick,
};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read fixture {}: {error}", path.display());
    });
    serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!("failed to parse fixture {}: {error}", path.display());
    })
}

fn assert_fixture(name: &str, actual: &Value) {
    let expected = fixture(name);
    assert_eq!(&expected, actual, "fixture mismatch: {name}");
}

fn full_projector_parts() -> (AppState, AssetCatalog, fgc_protocol::ServedAssetResolver) {
    (
        support::fixture_state(),
        support::fixture_catalog(),
        support::resolver(),
    )
}

#[test]
fn scoreboard_full() {
    let (state, catalog, resolver) = full_projector_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let actual = serde_json::to_value(Message::new(
        "scoreboard",
        projector.scoreboard(match_state),
    ))
    .expect("scoreboard JSON");

    assert_fixture("scoreboard_full.json", &actual);
}

#[test]
fn scoreboard_minimal() {
    let state = AppState::default();
    let catalog = AssetCatalog::default();
    let resolver = support::resolver();
    let projector = Projector::new(&state, &catalog, &resolver);

    let actual = serde_json::to_value(Message::new(
        "scoreboard",
        projector.scoreboard(&support::minimal_match()),
    ))
    .expect("minimal scoreboard JSON");

    assert_fixture("scoreboard_minimal.json", &actual);
}

#[test]
fn init_after_broadcast() {
    let (state, catalog, resolver) = full_projector_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let cache = support::fixture_cache(&state, &projector);

    let actual = serde_json::to_value(cache.init()).expect("init JSON");

    assert_fixture("init_after_broadcast.json", &actual);
}

#[test]
fn init_empty() {
    let actual = serde_json::to_value(SectionCache::new().init()).expect("empty init JSON");

    assert_fixture("init_empty.json", &actual);
}

#[test]
fn casters() {
    let (state, catalog, resolver) = full_projector_parts();
    let projector = Projector::new(&state, &catalog, &resolver);
    let match_state = state.current_match.as_ref().expect("fixture match");

    let actual =
        serde_json::to_value(projector.casters_message(match_state)).expect("casters JSON");

    assert_fixture("casters.json", &actual);
}

#[test]
fn timer_tick_message() {
    let (state, _, _) = full_projector_parts();
    let actual =
        serde_json::to_value(timer_tick(&state.settings.timer, 0, 12, 34)).expect("tick JSON");

    assert_fixture("timer_tick.json", &actual);
}

#[test]
fn timer_tick_can_hide_hours() {
    let (mut state, _, _) = full_projector_parts();
    state.settings.timer.show_hours = false;
    let actual = serde_json::to_value(timer_tick(&state.settings.timer, 1, 12, 0))
        .expect("hidden-hour tick JSON");

    assert_eq!(
        actual,
        serde_json::json!({
            "type": "matchTimer",
            "data": {
                "hours": 1,
                "minutes": 12,
                "seconds": 0,
                "settings": {
                    "alignment": "center",
                    "leadingZeroes": true,
                    "showHours": false,
                    "standalone": false,
                },
            },
        })
    );
}

#[test]
fn timer_action_start() {
    let actual = serde_json::to_value(timer_action(TimerActionDto::start())).expect("start JSON");

    assert_fixture("timer_action_start.json", &actual);
}

#[test]
fn timer_action_pause() {
    let actual = serde_json::to_value(timer_action(TimerActionDto::pause())).expect("pause JSON");

    assert_fixture("timer_action_pause.json", &actual);
}

#[test]
fn timer_action_reset() {
    let actual =
        serde_json::to_value(timer_action(TimerActionDto::reset(0, 0, 0))).expect("reset JSON");

    assert_fixture("timer_action_reset.json", &actual);
}

#[test]
fn show_message() {
    let actual = serde_json::to_value(show(true)).expect("show JSON");

    assert_fixture("show.json", &actual);
}

#[test]
fn auto_overlay_message() {
    let actual = serde_json::to_value(auto_overlay(
        vec!["game-overlay.html".to_owned(), "map-select.html".to_owned()],
        Some("map-select.html".to_owned()),
    ))
    .expect("autoOverlay JSON");

    assert_fixture("auto_overlay.json", &actual);
}

#[test]
fn auto_overlay_allows_no_active_scene() {
    let actual =
        serde_json::to_value(auto_overlay(Vec::new(), None)).expect("empty autoOverlay JSON");

    assert_eq!(
        actual,
        serde_json::json!({ "type": "autoOverlay", "data": { "included": [], "active": null } })
    );
}

#[test]
fn show_hides_overlay() {
    let actual = serde_json::to_value(show(false)).expect("show JSON");

    assert_eq!(
        actual,
        serde_json::json!({ "type": "show", "visible": false })
    );
}
