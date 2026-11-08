//! Domain model, validation, persistence and asset metadata for Fruiterra
//! Graphics Controller.
//!
//! This crate is deliberately free of async runtimes, HTTP servers and UI
//! frameworks. The full domain model arrives in milestone M1.

/// User-facing application name.
pub const APP_NAME: &str = "Fruiterra Graphics Controller";

/// Executable name.
pub const BINARY_NAME: &str = "fruiterra-gc";

/// Application version from the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Errors surfaced by the core domain.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A value failed validation.
    #[error("invalid data: {0}")]
    Invalid(String),
}

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
        assert!(!VERSION.is_empty());
    }
}
