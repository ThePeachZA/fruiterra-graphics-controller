use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::{APP_NAME, CoreError};

/// Resolves the platform project directories for the application.
pub fn project_dirs() -> Result<ProjectDirs, CoreError> {
    ProjectDirs::from("com", "Fruiterra", APP_NAME).ok_or(CoreError::NoDataDir)
}

/// `%APPDATA%\Fruiterra\Fruiterra Graphics Controller` on Windows.
pub fn app_data_dir() -> Result<PathBuf, CoreError> {
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
        let directory = app_data_dir().expect("app data dir");
        assert!(directory.ends_with(APP_NAME), "{}", directory.display());
    }
}
