use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Value, json};

use crate::{AppState, CoreError, SCHEMA_VERSION, migrate};

/// Loads and saves the application state.
pub trait Store: Send + Sync {
    /// Loads the current state, returning defaults on first run.
    fn load(&self) -> Result<AppState, CoreError>;

    /// Persists the state.
    fn save(&self, state: &AppState) -> Result<(), CoreError>;
}

/// JSON-file store with atomic replacement and a single previous-good backup.
#[derive(Clone, Debug)]
pub struct JsonStore {
    path: PathBuf,
    backup_path: PathBuf,
}

impl JsonStore {
    /// Creates a store for `path`; the backup is `<path>.bak`.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let backup_path = backup_path_for(&path);
        Self { path, backup_path }
    }

    /// Primary state file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Previous-good state file.
    #[must_use]
    pub fn backup_path(&self) -> &Path {
        &self.backup_path
    }

    /// Temporary file used during a save.
    #[must_use]
    pub fn temp_path(&self) -> PathBuf {
        temp_path_for(&self.path)
    }

    fn load_path(&self, path: &Path) -> Result<AppState, CoreError> {
        let bytes = fs::read(path).map_err(|source| CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| CoreError::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        migrate(value)
    }
}

impl Store for JsonStore {
    fn load(&self) -> Result<AppState, CoreError> {
        if !self.path.exists() {
            return Ok(AppState::default());
        }

        match self.load_path(&self.path) {
            Ok(state) => Ok(state),
            Err(primary_error) => {
                if !self.backup_path.exists() {
                    return Err(primary_error);
                }

                match self.load_path(&self.backup_path) {
                    Ok(state) => {
                        tracing::warn!(
                            path = %self.path.display(),
                            backup = %self.backup_path.display(),
                            error = %primary_error,
                            "state file was unreadable; loaded the backup"
                        );
                        Ok(state)
                    }
                    Err(backup_error) => {
                        tracing::error!(
                            path = %self.path.display(),
                            backup = %self.backup_path.display(),
                            primary_error = %primary_error,
                            backup_error = %backup_error,
                            "state file and backup were both unreadable"
                        );
                        Err(primary_error)
                    }
                }
            }
        }
    }

    fn save(&self, state: &AppState) -> Result<(), CoreError> {
        let parent = self.path.parent().ok_or_else(|| {
            CoreError::Invalid("state file path has no parent directory".to_owned())
        })?;
        fs::create_dir_all(parent).map_err(|source| CoreError::Io {
            path: parent.to_path_buf(),
            source,
        })?;

        let mut value =
            serde_json::to_value(state).map_err(|error| CoreError::Serialize(error.to_string()))?;
        if let Some(object) = value.as_object_mut() {
            object.insert("schemaVersion".to_owned(), json!(SCHEMA_VERSION));
        }
        let bytes = serde_json::to_vec_pretty(&value)
            .map_err(|error| CoreError::Serialize(error.to_string()))?;

        let temp_path = self.temp_path();
        remove_stale_temp(&temp_path)?;
        write_and_sync(&temp_path, &bytes)?;

        if self.path.is_file() {
            let backup_temp = temp_path_for(&self.backup_path);
            fs::copy(&self.path, &backup_temp).map_err(|source| CoreError::Io {
                path: backup_temp.clone(),
                source,
            })?;
            fs::rename(&backup_temp, &self.backup_path).map_err(|source| CoreError::Io {
                path: self.backup_path.clone(),
                source,
            })?;
        }

        fs::rename(&temp_path, &self.path).map_err(|source| CoreError::Io {
            path: self.path.clone(),
            source,
        })?;

        Ok(())
    }
}

/// In-memory store used by tests and pure logic.
#[derive(Debug, Default)]
pub struct MemoryStore {
    state: Mutex<AppState>,
}

impl MemoryStore {
    /// Creates a store holding `state`.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self {
            state: Mutex::new(state),
        }
    }
}

impl Store for MemoryStore {
    fn load(&self) -> Result<AppState, CoreError> {
        self.state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| CoreError::StorePoisoned)
    }

    fn save(&self, state: &AppState) -> Result<(), CoreError> {
        let mut guard = self.state.lock().map_err(|_| CoreError::StorePoisoned)?;
        *guard = state.clone();
        Ok(())
    }
}

fn backup_path_for(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".bak");
    PathBuf::from(value)
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".tmp");
    PathBuf::from(value)
}

fn remove_stale_temp(path: &Path) -> Result<(), CoreError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(CoreError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), CoreError> {
    let mut file = fs::File::create(path).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(bytes).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.sync_all().map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })
}
