use std::path::{Path, PathBuf};

use crate::{Asset, AssetCategory, AssetId, AssetOrigin, CoreError};

/// Extensions accepted by the content-addressed asset store.
pub const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "svg"];

/// Returns true when `hash` is exactly 64 lowercase hexadecimal characters.
#[must_use]
pub fn is_valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Returns true when `ext` is on the asset allow-list.
#[must_use]
pub fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.contains(&ext)
}

/// Validates the hash and extension pair used to address a blob.
pub fn validate_asset_identity(hash: &str, ext: &str) -> Result<(), CoreError> {
    if !is_valid_hash(hash) {
        return Err(CoreError::InvalidAssetHash);
    }
    if !is_allowed_extension(ext) {
        return Err(CoreError::InvalidAssetExtension(ext.to_owned()));
    }
    Ok(())
}

/// Builds the deterministic identifier used by bundled assets.
pub fn bundled_asset_id(hash: &str) -> Result<AssetId, CoreError> {
    if !is_valid_hash(hash) {
        return Err(CoreError::InvalidAssetHash);
    }
    Ok(AssetId::from_raw(format!("b:{hash}")))
}

/// Relative blob path: `blobs/<first-two-hex>/<hash>.<ext>`.
pub fn blob_relative_path(hash: &str, ext: &str) -> Result<PathBuf, CoreError> {
    validate_asset_identity(hash, ext)?;
    let prefix = hash.get(..2).ok_or(CoreError::InvalidAssetHash)?;
    Ok(PathBuf::from("blobs")
        .join(prefix)
        .join(format!("{hash}.{ext}")))
}

/// Absolute blob path beneath a blob root.
pub fn blob_path(root: &Path, hash: &str, ext: &str) -> Result<PathBuf, CoreError> {
    Ok(root.join(blob_relative_path(hash, ext)?))
}

impl Asset {
    /// Creates user asset metadata after validating its content identity.
    pub fn new_user(
        name: impl Into<String>,
        category: AssetCategory,
        hash: impl Into<String>,
        ext: impl Into<String>,
        original_filename: Option<String>,
    ) -> Result<Self, CoreError> {
        let hash = hash.into();
        let ext = ext.into();
        validate_asset_identity(&hash, &ext)?;
        Ok(Self {
            id: AssetId::new(),
            name: name.into(),
            category,
            hash,
            ext,
            origin: AssetOrigin::User,
            original_filename,
            added_at: None,
            icon: None,
            portrait: None,
            game_mode: None,
        })
    }

    /// Creates bundled asset metadata with a deterministic `b:<hash>` identity.
    pub fn new_bundled(
        name: impl Into<String>,
        category: AssetCategory,
        hash: impl Into<String>,
        ext: impl Into<String>,
    ) -> Result<Self, CoreError> {
        let hash = hash.into();
        let ext = ext.into();
        validate_asset_identity(&hash, &ext)?;
        Ok(Self {
            id: bundled_asset_id(&hash)?,
            name: name.into(),
            category,
            hash,
            ext,
            origin: AssetOrigin::Bundled,
            original_filename: None,
            added_at: None,
            icon: None,
            portrait: None,
            game_mode: None,
        })
    }
}
