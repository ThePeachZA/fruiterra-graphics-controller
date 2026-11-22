use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::ProjectDirs;

use crate::{APP_NAME, CoreError};

/// Environment variable that overrides the application data root.
pub const DATA_DIR_ENV: &str = "FGC_DATA_DIR";

/// Explicit data root, set once at start-up.
static DATA_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Points the application at a different data root.
///
/// The desktop app calls this when `--data-dir` is passed. Every path helper
/// resolves through [`app_data_dir`], so the whole project moves with it.
pub fn set_data_root(path: PathBuf) {
    let _ = DATA_ROOT.set(path);
}

/// Returns the explicitly configured data root, if there is one.
#[must_use]
pub fn data_root_override() -> Option<PathBuf> {
    DATA_ROOT.get().cloned()
}

/// Resolves the platform project directories for the application.
pub fn project_dirs() -> Result<ProjectDirs, CoreError> {
    ProjectDirs::from("com", "Fruiterra", APP_NAME).ok_or(CoreError::NoDataDir)
}

/// `%APPDATA%\Fruiterra\Fruiterra Graphics Controller` on Windows.
fn release_data_dir() -> Result<PathBuf, CoreError> {
    let dirs = project_dirs()?;
    let data = dirs.data_dir();
    // `ProjectDirs::data_dir` appends a `data` component on Windows. The app
    // owns that name itself (`<base>\data\app.json`), so use the parent there
    // to avoid a nested `data\data` layout. Other platforms already return the
    // base directory.
    #[cfg(target_os = "windows")]
    {
        data.parent()
            .map(Path::to_path_buf)
            .ok_or(CoreError::NoDataDir)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(data.to_path_buf())
    }
}

/// The application data root.
///
/// The root is resolved in this order:
///
/// 1. An explicit override set with [`set_data_root`].
/// 2. The [`DATA_DIR_ENV`] environment variable.
/// 3. A `Fruiterra Graphics Controller (dev)` sibling in a debug build, so a
///    development build keeps its own project and never overwrites the
///    shipped build's.
/// 4. The platform project directory.
pub fn app_data_dir() -> Result<PathBuf, CoreError> {
    if let Some(root) = data_root_override() {
        return Ok(root);
    }
    match std::env::var_os(DATA_DIR_ENV) {
        Some(root) if !root.is_empty() => return Ok(PathBuf::from(root)),
        _ => {}
    }
    let base = release_data_dir()?;
    if cfg!(debug_assertions) {
        return Ok(base.with_file_name(format!("{APP_NAME} (dev)")));
    }
    Ok(base)
}

/// Describes where the project lives, for the settings page.
#[must_use]
pub fn data_root_description() -> String {
    app_data_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|error| format!("Could not be resolved: {error}"))
}

/// Directory containing `app.json` and its backup.
pub fn data_dir() -> Result<PathBuf, CoreError> {
    Ok(app_data_dir()?.join("data"))
}

/// Root of the user asset store.
pub fn assets_root() -> Result<PathBuf, CoreError> {
    Ok(app_data_dir()?.join("assets"))
}

/// Directory containing content-addressed blobs.
pub fn blobs_dir() -> Result<PathBuf, CoreError> {
    Ok(assets_root()?.join("blobs"))
}

/// Optional per-file overlay overrides.
pub fn overlays_override_dir() -> Result<PathBuf, CoreError> {
    Ok(app_data_dir()?.join("overlays"))
}

/// Log directory.
pub fn logs_dir() -> Result<PathBuf, CoreError> {
    Ok(app_data_dir()?.join("logs"))
}

/// Path to the persisted state file.
pub fn state_file() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join("app.json"))
}

/// Path to the previous good state file.
pub fn backup_file() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join("app.json.bak"))
}

/// Moves an unreadable state file aside so the application can start clean
/// without destroying a file the operator may still need.
///
/// The moved file keeps a `.broken` suffix and a counter, so repeated
/// failures never overwrite an earlier copy. Returns the new paths.
pub fn quarantine_state_files() -> Vec<PathBuf> {
    let Ok(directory) = data_dir() else {
        return Vec::new();
    };
    quarantine_state_files_in(&directory)
}

/// Moves `app.json` and `app.json.bak` out of `directory` and returns the new
/// paths.
pub fn quarantine_state_files_in(directory: &Path) -> Vec<PathBuf> {
    let mut moved = Vec::new();
    for name in ["app.json", "app.json.bak"] {
        let source = directory.join(name);
        if !source.is_file() {
            continue;
        }
        let Some(stem) = source.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        for index in 0..100 {
            let target = directory.join(format!("{stem}.broken-{index}"));
            if target.exists() {
                continue;
            }
            if fs::rename(&source, &target).is_ok() {
                moved.push(target);
            }
            break;
        }
    }
    moved
}

/// Creates every directory the application expects under its data root.
pub fn ensure_app_dirs() -> Result<(), CoreError> {
    ensure_dirs(&app_data_dir()?)
}

/// Creates the expected directory layout beneath `root`.
pub fn ensure_dirs(root: &Path) -> Result<(), CoreError> {
    for directory in [
        root.join("data"),
        root.join("assets").join("blobs"),
        root.join("overlays"),
        root.join("logs"),
    ] {
        fs::create_dir_all(&directory).map_err(|source| CoreError::Io {
            path: directory,
            source,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_dirs_creates_the_expected_layout() {
        let root = tempfile::tempdir().expect("temp dir");
        ensure_dirs(root.path()).expect("create dirs");
        for relative in ["data", "assets/blobs", "overlays", "logs"] {
            assert!(root.path().join(relative).is_dir(), "{relative} missing");
        }
    }

    #[test]
    fn app_data_dir_ends_with_the_application_name() {
        // Regression: `ProjectDirs::data_dir` appends `data` on Windows, which
        // would have produced `<base>\data\data` for the state file.
        //
        // A debug build adds a `(dev)` suffix so it keeps its own project.
        let directory = app_data_dir().expect("app data dir");
        let name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        assert!(
            name == APP_NAME || name == format!("{APP_NAME} (dev)"),
            "{name}"
        );
    }

    #[test]
    fn quarantine_moves_the_state_files_aside() {
        let root = tempfile::tempdir().expect("temp dir");
        let directory = root.path().join("data");
        fs::create_dir_all(&directory).expect("create data dir");
        fs::write(directory.join("app.json"), "{}").expect("write state");
        fs::write(directory.join("app.json.bak"), "{}").expect("write backup");

        let moved = quarantine_state_files_in(&directory);

        assert_eq!(moved.len(), 2, "{moved:?}");
        assert!(!directory.join("app.json").exists());
        assert!(!directory.join("app.json.bak").exists());
        for path in &moved {
            assert!(path.exists(), "{}", path.display());
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            assert!(name.contains(".broken-"), "{name}");
        }
    }

    #[test]
    fn quarantine_keeps_earlier_copies() {
        let root = tempfile::tempdir().expect("temp dir");
        let directory = root.path().join("data");
        fs::create_dir_all(&directory).expect("create data dir");

        fs::write(directory.join("app.json"), "{}").expect("write state");
        let first = quarantine_state_files_in(&directory);
        fs::write(directory.join("app.json"), "{}").expect("write state");
        let second = quarantine_state_files_in(&directory);

        assert_eq!(first.len(), 1, "{first:?}");
        assert_eq!(second.len(), 1, "{second:?}");
        assert_ne!(first[0], second[0]);
        assert!(first[0].exists(), "the first copy is still there");
    }
}
