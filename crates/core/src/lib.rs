//! Domain model, validation, persistence and asset metadata for Fruiterra
//! Graphics Controller.
//!
//! This crate is deliberately free of async runtimes, HTTP servers and UI
//! frameworks. It owns the persisted domain model, the versioned JSON store
//! and the validation rules that keep user data loadable.

mod asset_store;
mod assets;
mod error;
mod match_control;
mod match_library;
mod migrate;
mod model;
mod paths;
mod references;
mod store;
mod tournament;
mod validation;

pub use asset_store::{
    AssetStore, BlobEntry, BlobReport, InsertOutcome, UnusedAssets, referenced_hashes, sha256_hex,
};
pub use assets::{
    ALLOWED_EXTENSIONS, MAX_ASSET_BYTES, blob_path, blob_relative_path, bundled_asset_id,
    detect_extension, extensions_match, is_allowed_extension, is_valid_hash, sanitize_svg,
    validate_asset_identity,
};
pub use error::CoreError;
pub use match_control::*;
pub use match_library::{MATCH_LIBRARY_EXTENSION, read_match_library, write_match_library};
pub use migrate::migrate;
pub use model::*;
pub use paths::{
    app_data_dir, assets_root, backup_file, blobs_dir, data_dir, ensure_app_dirs, ensure_dirs,
    logs_dir, overlays_override_dir, state_file,
};
pub use references::{
    TeamReference, clear_talent_references, remove_team_references, team_references,
};
pub use store::{JsonStore, MemoryStore, Store};
pub use tournament::*;
pub use validation::{
    MAX_PLAYER_NAME_CHARS, MAX_TALENT_NAME_CHARS, MAX_TEAM_NAME_CHARS, Repair, RepairKind,
    validate_and_repair,
};

/// User-facing application name.
pub const APP_NAME: &str = "Fruiterra Graphics Controller";

/// Executable name.
pub const BINARY_NAME: &str = "fruiterra-gc";

/// Application version from the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Current persisted schema version.
pub const SCHEMA_VERSION: u32 = 2;

/// Returns the application name.
#[must_use]
pub fn app_name() -> &'static str {
    APP_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_is_stable() {
        assert_eq!(APP_NAME, "Fruiterra Graphics Controller");
        assert_eq!(BINARY_NAME, "fruiterra-gc");
        assert_eq!(SCHEMA_VERSION, 2);
        assert!(!VERSION.is_empty());
    }
}
