//! Portable match-only library files.
//!
//! A project bundle carries everything: teams, talent, tournaments, assets and
//! settings. A match library is deliberately smaller. It carries saved matches
//! and their map pools so a schedule prepared on one machine can be moved to
//! another without replacing the rest of the project.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{CoreError, Match, MatchId, SCHEMA_VERSION};

/// Filename extension used for match library files.
pub const MATCH_LIBRARY_EXTENSION: &str = "json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchLibrary {
    schema_version: u32,
    matches: Vec<Match>,
}

/// Writes `matches` to `path` as a versioned JSON library.
///
/// The file is self-contained and human-readable; it never references asset
/// hashes or blob files, so it can be opened on a machine with a different
/// image library.
pub fn write_match_library(path: &Path, matches: &[Match]) -> Result<(), CoreError> {
    let library = MatchLibrary {
        schema_version: SCHEMA_VERSION,
        matches: matches.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&library)
        .map_err(|error| CoreError::Serialize(error.to_string()))?;
    std::fs::write(path, bytes).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads a match library and returns its matches.
///
/// Duplicate identifiers are dropped, first-to values are clamped to the
/// supported range, and the returned order is the order in the file.
pub fn read_match_library(path: &Path) -> Result<Vec<Match>, CoreError> {
    let bytes = std::fs::read(path).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let library: MatchLibrary = serde_json::from_slice(&bytes).map_err(|error| {
        CoreError::InvalidMatchLibrary(format!("could not parse {}: {error}", path.display()))
    })?;
    if library.schema_version > SCHEMA_VERSION {
        return Err(CoreError::UnsupportedSchema {
            found: library.schema_version,
            supported: SCHEMA_VERSION,
        });
    }

    let mut seen = BTreeSet::<MatchId>::new();
    let mut matches = Vec::new();
    for mut entry in library.matches {
        entry.first_to = entry.first_to.clamp(1, 4);
        if seen.insert(entry.id.clone()) {
            matches.push(entry);
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_match;

    #[test]
    fn match_library_round_trips_on_disk() {
        let dir = std::env::temp_dir().join(format!("fgc-match-lib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("matches.json");
        let mut first = new_match();
        first.title = "Grand Final".to_owned();
        first.scheduled_at = Some("2026-11-21T18:00".to_owned());
        let second = new_match();

        write_match_library(&path, &[first.clone(), second.clone()]).expect("write");
        let loaded = read_match_library(&path).expect("read");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "Grand Final");
        assert_eq!(loaded[0].scheduled_at.as_deref(), Some("2026-11-21T18:00"));
        assert_eq!(loaded[0].id, first.id);
        assert_eq!(loaded[1].id, second.id);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn import_clamps_first_to_and_drops_duplicate_ids() {
        let dir = std::env::temp_dir().join(format!("fgc-match-lib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("matches.json");
        let mut first = new_match();
        first.first_to = 9;
        let duplicate = first.clone();

        write_match_library(&path, &[first.clone(), duplicate]).expect("write");
        let loaded = read_match_library(&path).expect("read");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].first_to, 4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_newer_library_is_refused() {
        let dir = std::env::temp_dir().join(format!("fgc-match-lib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("matches.json");
        std::fs::write(
            &path,
            format!(r#"{{"schemaVersion":{},"matches":[]}}"#, SCHEMA_VERSION + 1),
        )
        .expect("write");

        let error = read_match_library(&path).expect_err("newer library");
        assert!(matches!(error, CoreError::UnsupportedSchema { .. }));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_older_library_loads_with_current_defaults() {
        let dir = std::env::temp_dir().join(format!("fgc-match-lib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("matches.json");
        std::fs::write(
            &path,
            r#"{"schemaVersion":0,"matches":[{"id":"match-old","title":"Old"}]}"#,
        )
        .expect("write");

        let loaded = read_match_library(&path).expect("read older library");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Old");
        assert_eq!(loaded[0].scheduled_at, None);
        assert_eq!(loaded[0].first_to, 3);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn malformed_json_is_reported_as_invalid_library() {
        let dir = std::env::temp_dir().join(format!("fgc-match-lib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("matches.json");
        std::fs::write(&path, "{not valid json").expect("write");

        let error = read_match_library(&path).expect_err("invalid library");
        assert!(matches!(error, CoreError::InvalidMatchLibrary(_)));

        std::fs::remove_dir_all(&dir).ok();
    }
}
