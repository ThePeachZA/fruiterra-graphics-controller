mod support;

use fgc_core::{CoreError, JsonStore, MemoryStore, Store, migrate};
use std::fs;
use tempfile::tempdir;

fn changed_state() -> fgc_core::AppState {
    let mut state = support::sample_state();
    state.teams[0].name = "Changed".to_owned();
    state
}

#[test]
fn json_store_round_trips() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("data").join("app.json");
    let store = JsonStore::new(&path);
    let state = support::sample_state();

    store.save(&state).expect("save should succeed");
    let loaded = store.load().expect("load should succeed");

    assert_eq!(loaded, state);
}

#[test]
fn missing_file_loads_default() {
    let directory = tempdir().expect("temporary directory");
    let store = JsonStore::new(directory.path().join("missing.json"));

    let loaded = store.load().expect("missing file should load defaults");

    assert_eq!(loaded, fgc_core::AppState::default());
}

#[test]
fn backup_rotates_previous_state() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("app.json");
    let store = JsonStore::new(&path);
    let first = support::sample_state();
    let second = changed_state();
    let mut third = second.clone();
    third.teams[0].name = "Third".to_owned();

    store.save(&first).expect("first save");
    store.save(&second).expect("second save");
    store.save(&third).expect("third save");

    let backup_bytes = fs::read(store.backup_path()).expect("backup should exist");
    let backup_value: serde_json::Value =
        serde_json::from_slice(&backup_bytes).expect("backup should be valid JSON");
    let backup = migrate(backup_value).expect("backup should load");

    assert_eq!(backup, second);
    assert_eq!(store.load().expect("live state should load"), third);
}

#[test]
fn corrupt_primary_falls_back_to_backup() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("app.json");
    let store = JsonStore::new(&path);
    let first = support::sample_state();
    let second = changed_state();

    store.save(&first).expect("first save");
    store.save(&second).expect("second save");
    fs::write(&path, b"{ not valid json").expect("corrupt primary");

    let loaded = store.load().expect("backup should be used");

    assert_eq!(loaded, first);
}

#[test]
fn failed_atomic_save_leaves_live_file_intact() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("app.json");
    let store = JsonStore::new(&path);
    let first = support::sample_state();
    let second = changed_state();

    store.save(&first).expect("first save");
    let temp_path = store.temp_path();
    fs::create_dir(&temp_path).expect("block the temporary path");

    let error = store.save(&second).expect_err("save should fail");
    assert!(matches!(error, CoreError::Io { .. }));

    assert_eq!(store.load().expect("live state should load"), first);
    assert!(temp_path.is_dir(), "failed temp artifact should remain");
}

#[test]
fn memory_store_round_trips() {
    let store = MemoryStore::default();
    let state = support::sample_state();

    store.save(&state).expect("save should succeed");

    assert_eq!(store.load().expect("load should succeed"), state);
}
