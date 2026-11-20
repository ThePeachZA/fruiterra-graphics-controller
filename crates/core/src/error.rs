use std::path::PathBuf;

/// Errors surfaced by the core domain.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A generic validation failure.
    #[error("invalid data: {0}")]
    Invalid(String),

    /// A value failed to deserialize, with the JSON path to the field.
    #[error("invalid data at {path}: {message}")]
    InvalidField {
        /// JSON path such as `teams[0].players[0].role`.
        path: String,
        /// Underlying serde message.
        message: String,
    },

    /// The file was written by a newer build.
    #[error("state schema version {found} is newer than the supported version {supported}")]
    UnsupportedSchema {
        /// Version found in the file.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },

    /// `schemaVersion` was not a non-negative integer.
    #[error("schemaVersion must be a non-negative integer")]
    InvalidSchemaVersion,

    /// A filesystem operation failed.
    #[error("failed to access {}: {source}", path.display())]
    Io {
        /// Path that could not be accessed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The state file could not be parsed.
    #[error("failed to parse {}: {message}", path.display())]
    Parse {
        /// Path that failed to parse.
        path: PathBuf,
        /// Parser message.
        message: String,
    },

    /// The state could not be serialized.
    #[error("failed to serialise state: {0}")]
    Serialize(String),

    /// The in-memory store lock was poisoned.
    #[error("state store lock was poisoned")]
    StorePoisoned,

    /// The platform data directory could not be resolved.
    #[error("could not resolve the application data directory")]
    NoDataDir,

    /// An asset hash was not 64 lowercase hexadecimal characters.
    #[error("asset hash must be exactly 64 lowercase hexadecimal characters")]
    InvalidAssetHash,

    /// An asset extension was not on the allow-list.
    #[error("asset extension {0:?} is not allowed")]
    InvalidAssetExtension(String),

    /// A user asset exceeded the import size limit.
    #[error("asset is {size} bytes; the maximum is {max} bytes")]
    AssetTooLarge {
        /// Size of the rejected asset in bytes.
        size: u64,
        /// Maximum accepted size in bytes.
        max: u64,
    },

    /// The bytes were not one of the supported image formats.
    #[error("unsupported asset type; expected PNG, JPEG, WebP, GIF or SVG")]
    UnsupportedAssetType,

    /// The SVG contained a construct that is unsafe to serve.
    #[error("unsafe SVG: {0}")]
    UnsafeSvg(String),

    /// A referenced user blob was not present in the asset store.
    #[error("asset blob {hash}.{ext} is missing")]
    AssetBlobMissing {
        /// Content hash of the missing blob.
        hash: String,
        /// Extension of the missing blob.
        ext: String,
    },

    /// A blob's bytes did not match the content hash that addressed it.
    #[error("asset blob hash mismatch: expected {expected}, got {actual}")]
    AssetHashMismatch {
        /// Hash recorded in the asset metadata or bundle.
        expected: String,
        /// Hash computed from the bytes on disk.
        actual: String,
    },

    /// A project bundle was malformed or incomplete.
    #[error("invalid project bundle: {0}")]
    InvalidBundle(String),

    /// A match library file was malformed or incomplete.
    #[error("invalid match library: {0}")]
    InvalidMatchLibrary(String),

    /// A schema migration failed.
    #[error("migration from schema version {from} failed: {message}")]
    Migration {
        /// Version being migrated from.
        from: u32,
        /// Failure detail.
        message: String,
    },
}
