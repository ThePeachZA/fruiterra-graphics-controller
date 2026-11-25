//! Content-addressed user asset storage, garbage collection and project
//! bundle export/import.
//!
//! Blobs live at `<root>/blobs/<first-two-hex>/<sha256>.<ext>`. The hash is of
//! the file bytes, so identical imports share one file. Bundled assets are
//! embedded in the binary and never appear on disk.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    AppState, Asset, AssetCategory, AssetOrigin, CoreError, MAX_ASSET_BYTES, SCHEMA_VERSION,
    VERSION, blob_path, detect_extension, extensions_match, is_allowed_extension, is_valid_hash,
    migrate, sanitize_svg,
};

/// Content-addressed asset store rooted at the application's `assets` folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetStore {
    root: PathBuf,
}

/// Result of inserting bytes into the store.
#[derive(Clone, Debug, PartialEq)]
pub struct InsertOutcome {
    /// New logical asset metadata.
    pub asset: Asset,
    /// True when this call created a new blob file.
    pub created_blob: bool,
}

/// One blob found by [`AssetStore::scan`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlobEntry {
    /// Absolute path to the blob.
    pub path: PathBuf,
    /// Content hash.
    pub hash: String,
    /// File extension.
    pub ext: String,
    /// File size in bytes.
    pub bytes: u64,
}

/// Totals for a set of blobs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlobReport {
    /// Number of blobs.
    pub files: u64,
    /// Total size in bytes.
    pub bytes: u64,
}

/// Unreferenced blob totals shown in Settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnusedAssets {
    /// Number of unreferenced blobs.
    pub files: u64,
    /// Total size of unreferenced blobs in bytes.
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundleManifest {
    schema_version: u32,
    app_version: String,
}

impl AssetStore {
    /// Creates a store rooted at `root`, which contains the `blobs` directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Root directory passed to [`Self::new`].
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directory containing content-addressed blobs.
    #[must_use]
    pub fn blobs_root(&self) -> PathBuf {
        self.root.join("blobs")
    }

    /// Validates, hashes and stores `bytes`, returning new logical metadata.
    ///
    /// The detected content type is authoritative; `name` is only a display
    /// label. Identical bytes are written once and reused.
    pub fn insert(
        &self,
        name: impl Into<String>,
        category: AssetCategory,
        bytes: &[u8],
        original_filename: Option<String>,
    ) -> Result<InsertOutcome, CoreError> {
        let size = bytes.len() as u64;
        if size > MAX_ASSET_BYTES {
            return Err(CoreError::AssetTooLarge {
                size,
                max: MAX_ASSET_BYTES,
            });
        }
        let ext = detect_extension(bytes).ok_or(CoreError::UnsupportedAssetType)?;
        if ext == "svg" {
            sanitize_svg(bytes)?;
        }

        let hash = sha256_hex(bytes);
        let created_blob = self.write_blob(&hash, ext, bytes)?;
        let mut asset = Asset::new_user(name, category, hash, ext, original_filename)?;
        asset.added_at = now_rfc3339();
        Ok(InsertOutcome {
            asset,
            created_blob,
        })
    }

    /// Reads a user blob by content identity.
    pub fn read_blob(&self, hash: &str, ext: &str) -> Result<Vec<u8>, CoreError> {
        let path = blob_path(&self.root, hash, ext)?;
        fs::read(&path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                CoreError::AssetBlobMissing {
                    hash: hash.to_owned(),
                    ext: ext.to_owned(),
                }
            } else {
                CoreError::Io { path, source }
            }
        })
    }

    /// Lists every valid blob beneath `blobs/`, sorted by path.
    pub fn scan(&self) -> Result<Vec<BlobEntry>, CoreError> {
        let mut entries = Vec::new();
        collect_blobs(&self.blobs_root(), &mut entries)?;
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    /// Totals for blobs whose hash is not in `state`.
    pub fn unused(&self, state: &AppState) -> Result<UnusedAssets, CoreError> {
        self.unused_hashes(&referenced_hashes(state))
    }

    /// Totals for blobs whose hash is not in `keep`.
    pub fn unused_hashes(&self, keep: &BTreeSet<String>) -> Result<UnusedAssets, CoreError> {
        let mut totals = UnusedAssets::default();
        for blob in self.scan()? {
            if !keep.contains(&blob.hash) {
                totals.files += 1;
                totals.bytes += blob.bytes;
            }
        }
        Ok(totals)
    }

    /// Deletes blobs not referenced by `state`.
    pub fn collect_garbage(&self, state: &AppState) -> Result<BlobReport, CoreError> {
        self.collect_garbage_hashes(&referenced_hashes(state))
    }

    /// Deletes blobs whose hash is not in `keep`.
    ///
    /// Callers pass the union of the saved snapshot and the in-memory state so
    /// an unsaved import is never collected.
    pub fn collect_garbage_hashes(&self, keep: &BTreeSet<String>) -> Result<BlobReport, CoreError> {
        let mut report = BlobReport::default();
        let mut parents = BTreeSet::new();
        for blob in self.scan()? {
            if keep.contains(&blob.hash) {
                continue;
            }
            match fs::remove_file(&blob.path) {
                Ok(()) => {
                    report.files += 1;
                    report.bytes += blob.bytes;
                    if let Some(parent) = blob.path.parent() {
                        parents.insert(parent.to_path_buf());
                    }
                }
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(CoreError::Io {
                        path: blob.path,
                        source,
                    });
                }
            }
        }
        for parent in parents {
            // Only removes the two-hex prefix directory when it is empty.
            let _ = fs::remove_dir(parent);
        }
        Ok(report)
    }

    /// Writes `app.json`, `export.json` and the referenced user blobs into
    /// `directory`.
    ///
    /// Every blob is read and verified before the bundle directory is created,
    /// so a corrupt source store cannot leave a partial bundle behind.
    pub fn export_bundle(
        &self,
        state: &AppState,
        directory: &Path,
    ) -> Result<BlobReport, CoreError> {
        let mut report = BlobReport::default();
        let mut exported = BTreeSet::new();
        let mut pending = Vec::new();
        for asset in state
            .assets
            .iter()
            .filter(|asset| asset.origin == AssetOrigin::User)
        {
            if !exported.insert((asset.hash.clone(), asset.ext.clone())) {
                continue;
            }

            let source_path = blob_path(&self.root, &asset.hash, &asset.ext)?;
            let bytes = fs::read(&source_path).map_err(|source| {
                if source.kind() == std::io::ErrorKind::NotFound {
                    CoreError::AssetBlobMissing {
                        hash: asset.hash.clone(),
                        ext: asset.ext.clone(),
                    }
                } else {
                    CoreError::Io {
                        path: source_path,
                        source,
                    }
                }
            })?;
            let actual = sha256_hex(&bytes);
            if actual != asset.hash {
                return Err(CoreError::AssetHashMismatch {
                    expected: asset.hash.clone(),
                    actual,
                });
            }

            report.files += 1;
            report.bytes += bytes.len() as u64;
            pending.push((asset.hash.clone(), asset.ext.clone(), bytes));
        }

        fs::create_dir_all(directory).map_err(|source| CoreError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let bundle_blobs = directory.join("blobs");
        fs::create_dir_all(&bundle_blobs).map_err(|source| CoreError::Io {
            path: bundle_blobs.clone(),
            source,
        })?;

        write_state_json(&directory.join("app.json"), state)?;
        let manifest = BundleManifest {
            schema_version: SCHEMA_VERSION,
            app_version: VERSION.to_owned(),
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| CoreError::Serialize(error.to_string()))?;
        fs::write(directory.join("export.json"), manifest_bytes).map_err(|source| {
            CoreError::Io {
                path: directory.join("export.json"),
                source,
            }
        })?;

        for (hash, ext, bytes) in pending {
            let target_path = blob_path(directory, &hash, &ext)?;
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|source| CoreError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&target_path, &bytes).map_err(|source| CoreError::Io {
                path: target_path,
                source,
            })?;
        }

        Ok(report)
    }

    /// Validates and imports a project bundle.
    ///
    /// Every user blob is read, type-checked and hash-verified before anything
    /// is written to the local store. The returned state is already migrated
    /// and repaired.
    pub fn import_bundle(&self, directory: &Path) -> Result<(AppState, BlobReport), CoreError> {
        let manifest_path = directory.join("export.json");
        let manifest_bytes = fs::read(&manifest_path).map_err(|source| {
            CoreError::InvalidBundle(format!(
                "could not read {}: {source}",
                manifest_path.display()
            ))
        })?;
        let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| CoreError::InvalidBundle(format!("invalid export.json: {error}")))?;
        if manifest.schema_version > SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchema {
                found: manifest.schema_version,
                supported: SCHEMA_VERSION,
            });
        }

        let state_path = directory.join("app.json");
        let state_bytes = fs::read(&state_path).map_err(|source| {
            CoreError::InvalidBundle(format!("could not read {}: {source}", state_path.display()))
        })?;
        let value: Value = serde_json::from_slice(&state_bytes)
            .map_err(|error| CoreError::InvalidBundle(format!("invalid app.json: {error}")))?;
        let state = migrate(value)?;

        let mut pending = Vec::new();
        let mut seen = BTreeSet::new();
        for asset in state
            .assets
            .iter()
            .filter(|asset| asset.origin == AssetOrigin::User)
        {
            if !seen.insert((asset.hash.clone(), asset.ext.clone())) {
                continue;
            }

            let path = blob_path(directory, &asset.hash, &asset.ext)?;
            let bytes = fs::read(&path).map_err(|_| CoreError::AssetBlobMissing {
                hash: asset.hash.clone(),
                ext: asset.ext.clone(),
            })?;
            let size = bytes.len() as u64;
            if size > MAX_ASSET_BYTES {
                return Err(CoreError::AssetTooLarge {
                    size,
                    max: MAX_ASSET_BYTES,
                });
            }

            let detected = detect_extension(&bytes).ok_or(CoreError::UnsupportedAssetType)?;
            if !extensions_match(&asset.ext, detected) {
                return Err(CoreError::InvalidBundle(format!(
                    "blob {}.{} contains {detected} data",
                    asset.hash, asset.ext
                )));
            }
            if detected == "svg" {
                sanitize_svg(&bytes)?;
            }

            let actual = sha256_hex(&bytes);
            if actual != asset.hash {
                return Err(CoreError::AssetHashMismatch {
                    expected: asset.hash.clone(),
                    actual,
                });
            }
            pending.push((asset.hash.clone(), asset.ext.clone(), bytes));
        }

        let mut report = BlobReport::default();
        for (hash, ext, bytes) in pending {
            self.write_blob(&hash, &ext, &bytes)?;
            report.files += 1;
            report.bytes += bytes.len() as u64;
        }

        Ok((state, report))
    }

    pub(crate) fn write_blob(
        &self,
        hash: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<bool, CoreError> {
        let path = blob_path(&self.root, hash, ext)?;
        if path.is_file()
            && fs::read(&path)
                .map(|existing| sha256_hex(&existing) == hash)
                .unwrap_or(false)
        {
            return Ok(false);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| CoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let temp = path.with_extension(format!("{ext}.tmp-{}", Uuid::new_v4()));
        write_and_sync(&temp, bytes)?;
        if let Err(source) = fs::rename(&temp, &path) {
            let _ = fs::remove_file(&temp);
            return Err(CoreError::Io { path, source });
        }
        Ok(true)
    }
}

/// Hashes every asset recorded in `state`.
///
/// Bundled hashes are included for safety; they never have on-disk blobs, so
/// including them cannot keep a user blob alive accidentally.
#[must_use]
pub fn referenced_hashes(state: &AppState) -> BTreeSet<String> {
    state
        .assets
        .iter()
        .map(|asset| asset.hash.clone())
        .collect()
}

/// SHA-256 of `bytes` as 64 lowercase hexadecimal characters.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn now_rfc3339() -> Option<String> {
    OffsetDateTime::now_utc().format(&Rfc3339).ok()
}

fn collect_blobs(directory: &Path, entries: &mut Vec<BlobEntry>) -> Result<(), CoreError> {
    let read = match fs::read_dir(directory) {
        Ok(read) => read,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(CoreError::Io {
                path: directory.to_path_buf(),
                source,
            });
        }
    };

    for entry in read {
        let entry = entry.map_err(|source| CoreError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| CoreError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_blobs(&path, entries)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some((hash, ext)) = parse_blob_name(&path) else {
            continue;
        };
        let bytes = entry
            .metadata()
            .map_err(|source| CoreError::Io {
                path: path.clone(),
                source,
            })?
            .len();
        entries.push(BlobEntry {
            path,
            hash,
            ext,
            bytes,
        });
    }
    Ok(())
}

fn parse_blob_name(path: &Path) -> Option<(String, String)> {
    let name = path.file_name()?.to_str()?;
    let (hash, ext) = name.rsplit_once('.')?;
    if is_valid_hash(hash) && is_allowed_extension(ext) {
        Some((hash.to_owned(), ext.to_owned()))
    } else {
        None
    }
}

fn write_state_json(path: &Path, state: &AppState) -> Result<(), CoreError> {
    let mut value =
        serde_json::to_value(state).map_err(|error| CoreError::Serialize(error.to_string()))?;
    if let Some(object) = value.as_object_mut() {
        object.insert("schemaVersion".to_owned(), json!(SCHEMA_VERSION));
    }
    let bytes = serde_json::to_vec_pretty(&value)
        .map_err(|error| CoreError::Serialize(error.to_string()))?;
    fs::write(path, bytes).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, Team, TeamId};

    const PNG_A: &[u8] = b"\x89PNG\r\n\x1a\nasset-a";
    const PNG_B: &[u8] = b"\x89PNG\r\n\x1a\nasset-b";
    const SAFE_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"><rect width="1" height="1"/></svg>"#;

    fn store() -> (tempfile::TempDir, AssetStore) {
        let directory = tempfile::tempdir().expect("temp dir");
        let store = AssetStore::new(directory.path());
        (directory, store)
    }

    fn team_with_logo(logo: Option<AssetId>) -> Team {
        Team {
            id: TeamId::new(),
            name: "Peaches".to_owned(),
            tag: "PCH".to_owned(),
            color: "#000000".to_owned(),
            text_color: "#ffffff".to_owned(),
            color_from_logo: false,
            text_color_from_logo: false,
            computed_contrast: false,
            aux_color: "#090909".to_owned(),
            logo,
            logo_background: true,
            players: Vec::new(),
        }
    }

    #[test]
    fn insert_dedupes_identical_bytes() {
        let (_directory, store) = store();
        let first = store
            .insert(
                "Logo A",
                AssetCategory::Logo,
                PNG_A,
                Some("a.png".to_owned()),
            )
            .expect("first insert");
        let second = store
            .insert(
                "Logo B",
                AssetCategory::Logo,
                PNG_A,
                Some("b.png".to_owned()),
            )
            .expect("second insert");

        assert!(first.created_blob);
        assert!(!second.created_blob);
        assert_eq!(first.asset.hash, second.asset.hash);
        assert_ne!(first.asset.id, second.asset.id);
        assert_eq!(store.scan().expect("scan").len(), 1);
    }

    #[test]
    fn replacing_a_reference_keeps_the_old_asset_valid() {
        let (_directory, store) = store();
        let first = store
            .insert("Old", AssetCategory::Logo, PNG_A, None)
            .expect("old insert");
        let second = store
            .insert("New", AssetCategory::Logo, PNG_B, None)
            .expect("new insert");

        let mut state = AppState::default();
        state.assets.push(first.asset.clone());
        state.assets.push(second.asset.clone());
        state
            .teams
            .push(team_with_logo(Some(first.asset.id.clone())));
        // Replacing a reference only changes the team's AssetId.
        state.teams[0].logo = Some(second.asset.id.clone());

        let report = store.collect_garbage(&state).expect("gc");
        assert_eq!(report.files, 0);
        assert_eq!(
            store
                .read_blob(&first.asset.hash, &first.asset.ext)
                .expect("old blob"),
            PNG_A
        );
        assert_eq!(
            store
                .read_blob(&second.asset.hash, &second.asset.ext)
                .expect("new blob"),
            PNG_B
        );
    }

    #[test]
    fn gc_removes_only_unreferenced_user_blobs() {
        let (directory, store) = store();
        let referenced = store
            .insert("Referenced", AssetCategory::Logo, PNG_A, None)
            .expect("referenced insert");
        let unused = store
            .insert("Unused", AssetCategory::Logo, PNG_B, None)
            .expect("unused insert");

        let mut state = AppState::default();
        state.assets.push(referenced.asset.clone());

        let before = store.unused(&state).expect("unused");
        assert_eq!(before.files, 1);
        assert_eq!(before.bytes, PNG_B.len() as u64);

        let report = store.collect_garbage(&state).expect("gc");
        assert_eq!(report.files, 1);
        assert!(
            store
                .read_blob(&referenced.asset.hash, &referenced.asset.ext)
                .is_ok()
        );
        assert!(matches!(
            store.read_blob(&unused.asset.hash, &unused.asset.ext),
            Err(CoreError::AssetBlobMissing { .. })
        ));

        // Files outside `blobs/` are never candidates for collection.
        let sentinel = directory.path().join("embedded.png");
        fs::write(&sentinel, PNG_A).expect("sentinel");
        store.collect_garbage(&AppState::default()).expect("gc");
        assert!(sentinel.is_file());
    }

    #[test]
    fn export_import_round_trips_on_a_second_directory() {
        let (source_directory, source) = store();
        let inserted = source
            .insert(
                "Portrait",
                AssetCategory::Portrait,
                PNG_A,
                Some("a.png".to_owned()),
            )
            .expect("insert");
        let mut state = AppState::default();
        state.assets.push(inserted.asset.clone());
        state
            .teams
            .push(team_with_logo(Some(inserted.asset.id.clone())));

        let bundle = tempfile::tempdir().expect("bundle dir");
        let exported = source.export_bundle(&state, bundle.path()).expect("export");
        assert_eq!(exported.files, 1);
        assert!(bundle.path().join("app.json").is_file());
        assert!(bundle.path().join("export.json").is_file());
        assert!(
            blob_path(bundle.path(), &inserted.asset.hash, &inserted.asset.ext)
                .expect("exported blob path")
                .is_file()
        );
        assert!(!bundle.path().join("blobs").join("blobs").exists());

        let (target_directory, target) = store();
        let (imported, report) = target.import_bundle(bundle.path()).expect("import");
        assert_eq!(report.files, 1);
        assert_eq!(imported.assets, state.assets);
        assert_eq!(
            target
                .read_blob(&inserted.asset.hash, &inserted.asset.ext)
                .expect("imported blob"),
            PNG_A
        );

        assert_no_temp_directory(source_directory.path());
        assert_no_temp_directory(target_directory.path());
        assert_no_temp_directory(bundle.path());
    }

    #[test]
    fn export_rejects_a_hash_mismatch_before_writing_anything() {
        let (source_directory, source) = store();
        let inserted = source
            .insert("Logo", AssetCategory::Logo, PNG_A, None)
            .expect("insert");
        let mut state = AppState::default();
        state.assets.push(inserted.asset.clone());

        let blob = blob_path(
            source_directory.path(),
            &inserted.asset.hash,
            &inserted.asset.ext,
        )
        .expect("blob path");
        fs::write(&blob, PNG_B).expect("tamper");

        let bundle = tempfile::tempdir().expect("bundle dir");
        assert!(matches!(
            source.export_bundle(&state, bundle.path()),
            Err(CoreError::AssetHashMismatch { .. })
        ));
        assert!(!bundle.path().join("app.json").exists());
        assert!(!bundle.path().join("export.json").exists());
        assert!(!bundle.path().join("blobs").exists());

        assert_no_temp_directory(source_directory.path());
    }

    #[test]
    fn import_rejects_a_hash_mismatch_before_writing_anything() {
        let (source_directory, source) = store();
        let inserted = source
            .insert("Logo", AssetCategory::Logo, PNG_A, None)
            .expect("insert");
        let mut state = AppState::default();
        state.assets.push(inserted.asset.clone());

        let bundle = tempfile::tempdir().expect("bundle dir");
        source.export_bundle(&state, bundle.path()).expect("export");

        let blob =
            blob_path(bundle.path(), &inserted.asset.hash, &inserted.asset.ext).expect("blob path");
        fs::write(&blob, PNG_B).expect("tamper");

        let (_target_directory, target) = store();
        assert!(matches!(
            target.import_bundle(bundle.path()),
            Err(CoreError::AssetHashMismatch { .. })
        ));
        assert!(target.scan().expect("scan").is_empty());

        assert_no_temp_directory(source_directory.path());
    }

    #[test]
    fn rejects_oversize_assets() {
        let (_directory, store) = store();
        let bytes = vec![0_u8; (MAX_ASSET_BYTES + 1) as usize];
        assert!(matches!(
            store.insert("Huge", AssetCategory::Logo, &bytes, None),
            Err(CoreError::AssetTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_asset_types() {
        let (_directory, store) = store();
        assert!(matches!(
            store.insert("Text", AssetCategory::Logo, b"not an image", None),
            Err(CoreError::UnsupportedAssetType)
        ));
    }

    #[test]
    fn rejects_unsafe_svg() {
        let (_directory, store) = store();
        let unsafe_svg =
            br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#;
        assert!(matches!(
            store.insert("Unsafe", AssetCategory::Logo, unsafe_svg, None),
            Err(CoreError::UnsafeSvg(_))
        ));
    }

    #[test]
    fn accepts_safe_svg() {
        let (_directory, store) = store();
        let inserted = store
            .insert("Safe", AssetCategory::Logo, SAFE_SVG, None)
            .expect("safe SVG");
        assert_eq!(inserted.asset.ext, "svg");
    }

    fn assert_no_temp_directory(root: &Path) {
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            let read = match fs::read_dir(&directory) {
                Ok(read) => read,
                Err(_) => continue,
            };
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    assert_ne!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("_temp"),
                        "unexpected _temp directory at {}",
                        path.display()
                    );
                    pending.push(path);
                }
            }
        }
    }
}
