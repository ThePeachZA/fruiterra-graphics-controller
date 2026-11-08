use fgc_core::{
    app_data_dir, assets_root, backup_file, blobs_dir, data_dir, ensure_dirs, logs_dir,
    overlays_override_dir, state_file,
};
use tempfile::tempdir;

#[test]
fn paths_have_the_documented_leaf_names() {
    let root = app_data_dir().expect("data directory should resolve");
    assert!(root.is_absolute());
    assert!(
        root.to_string_lossy().to_lowercase().contains("fruiterra"),
        "unexpected data directory: {}",
        root.display()
    );

    assert!(data_dir().expect("data dir").ends_with("data"));
    assert!(assets_root().expect("assets dir").ends_with("assets"));
    assert!(blobs_dir().expect("blobs dir").ends_with("blobs"));
    assert!(
        overlays_override_dir()
            .expect("overlays dir")
            .ends_with("overlays")
    );
    assert!(logs_dir().expect("logs dir").ends_with("logs"));
    assert!(state_file().expect("state file").ends_with("app.json"));
    assert!(
        backup_file()
            .expect("backup file")
            .ends_with("app.json.bak")
    );
}

#[test]
fn ensure_dirs_creates_the_expected_layout() {
    let directory = tempdir().expect("temporary directory");

    ensure_dirs(directory.path()).expect("directory layout");

    assert!(directory.path().join("data").is_dir());
    assert!(directory.path().join("assets").join("blobs").is_dir());
    assert!(directory.path().join("overlays").is_dir());
    assert!(directory.path().join("logs").is_dir());
}
