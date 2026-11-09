//! Keyed `init` cache tests.

mod support;

use fgc_protocol::{Projector, SCOREBOARD_TYPES, SectionCache};
use serde_json::json;

#[test]
fn empty_cache_produces_an_empty_keyed_map() {
    let cache = SectionCache::new();
    let message = cache.init();

    assert_eq!(message.kind, "init");
    assert!(message.data.is_empty());
    assert_eq!(
        serde_json::to_value(message).expect("init JSON"),
        json!({ "type": "init", "data": {} })
    );
}

#[test]
fn cache_stores_the_last_payload_per_type() {
    let mut cache = SectionCache::new();
    cache.set_value("scoreboard", json!({ "title": "First" }));
    cache.set_value("scoreboard", json!({ "title": "Second" }));

    assert_eq!(cache.get("scoreboard"), Some(&json!({ "title": "Second" })));
    assert_eq!(cache.len(), 1);
}

#[test]
fn cache_contains_and_remove() {
    let mut cache = SectionCache::new();
    assert!(!cache.contains("casters"));

    cache.set_value("casters", json!({ "castersData": [] }));
    assert!(cache.contains("casters"));
    assert_eq!(cache.remove("casters"), Some(json!({ "castersData": [] })));
    assert!(!cache.contains("casters"));
}

#[test]
fn cache_clear_removes_every_section() {
    let mut cache = SectionCache::new();
    cache.set_value("scoreboard", json!({}));
    cache.set_value("casters", json!({}));

    cache.clear();

    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn init_after_a_full_broadcast_has_every_overlay_key() {
    let state = support::fixture_state();
    let catalog = support::fixture_catalog();
    let resolver = support::resolver();
    let projector = Projector::new(&state, &catalog, &resolver);
    let cache = support::fixture_cache(&state, &projector);
    let message = cache.init();

    assert_eq!(message.kind, "init");
    assert_eq!(message.data.len(), SCOREBOARD_TYPES.len() + 2);

    for kind in SCOREBOARD_TYPES {
        assert!(message.data.contains_key(kind), "missing {kind}");
    }
    assert!(message.data.contains_key("casters"));
    assert!(message.data.contains_key("matchTimer"));
    assert!(!message.data.contains_key("show"));
}

#[test]
fn init_scoreboard_keys_share_the_same_payload() {
    let state = support::fixture_state();
    let catalog = support::fixture_catalog();
    let resolver = support::resolver();
    let projector = Projector::new(&state, &catalog, &resolver);
    let cache = support::fixture_cache(&state, &projector);
    let message = cache.init();

    let first = &message.data[SCOREBOARD_TYPES[0]];
    for kind in &SCOREBOARD_TYPES[1..] {
        assert_eq!(&message.data[*kind], first);
    }
}

#[test]
fn init_casters_key_holds_the_casters_wrapper() {
    let state = support::fixture_state();
    let catalog = support::fixture_catalog();
    let resolver = support::resolver();
    let projector = Projector::new(&state, &catalog, &resolver);
    let cache = support::fixture_cache(&state, &projector);
    let message = cache.init();

    let casters = message.data.get("casters").expect("casters key");
    assert!(casters.get("castersData").is_some());
    assert!(casters.get("teamA").is_none());
}

#[test]
fn init_reflects_the_most_recent_broadcast() {
    let mut cache = SectionCache::new();
    cache.set_value("scoreboard", json!({ "title": "Old" }));
    cache.set_value("scoreboard", json!({ "title": "New" }));

    let message = cache.init();

    assert_eq!(message.data["scoreboard"]["title"], "New");
}
