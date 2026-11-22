//! Content packs: distributable artwork that adds to a project.
//!
//! A project bundle replaces the whole project. A content pack does not: it
//! carries artwork only, and importing it adds those assets to the current
//! project without touching teams, players, talent, tournaments, settings or
//! the current match. That makes a pack safe to publish, share and re-import.
//!
//! A pack is a folder:
//!
//! ```text
//! pack.json                 the manifest
//! blobs/<xx>/<sha256>.<ext> the artwork, content addressed
//! ```
//!
//! Every blob is verified against its SHA-256 before it is written, exactly
//! like a project bundle, so a pack can never install a file that does not
//! match its manifest.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    AppState, Asset, AssetAttachment, AssetCategory, AssetId, AssetOrigin, AssetStore, CoreError,
    MAX_ASSET_BYTES, Role, SCHEMA_VERSION, VERSION, blob_path, detect_extension, extensions_match,
    sanitize_svg, sha256_hex,
};

/// Name of the manifest file inside a pack folder.
pub const PACK_MANIFEST_NAME: &str = "pack.json";

/// Value of the `kind` field that marks a manifest as a content pack.
pub const PACK_KIND: &str = "fgc-content-pack";

/// Version of the pack format this build understands.
pub const PACK_VERSION: u32 = 1;

/// Human-readable details a pack carries about itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    /// Display name.
    pub name: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// Author or organisation.
    #[serde(default)]
    pub author: String,
    /// Licence name or URL.
    #[serde(default)]
    pub license: String,
    /// Project homepage.
    #[serde(default)]
    pub homepage: String,
}

/// One asset inside a pack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAsset {
    /// Stable identity. Importing the same pack twice installs nothing new.
    pub id: AssetId,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Category.
    pub category: AssetCategory,
    /// SHA-256 of the file bytes.
    pub hash: String,
    /// File extension.
    pub ext: String,
    /// Original filename, kept for diagnostics.
    #[serde(default)]
    pub original_filename: Option<String>,
    /// Hero icon asset.
    #[serde(default)]
    pub icon: Option<AssetId>,
    /// Hero portrait asset.
    #[serde(default)]
    pub portrait: Option<AssetId>,
    /// Transparent cut-out art for the hero bans overlay.
    #[serde(default)]
    pub transparent: Option<AssetId>,
    /// Hexagonal badge art for the hero bans overlay.
    #[serde(default)]
    pub badge: Option<AssetId>,
    /// Role of the hero.
    #[serde(default)]
    pub role: Option<Role>,
    /// Game-mode asset for a map.
    #[serde(default)]
    pub game_mode: Option<AssetId>,
    /// Attached media owned by another asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AssetAttachment>,
}

/// The manifest at the root of a pack folder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    /// Always [`PACK_KIND`].
    pub kind: String,
    /// Pack format version.
    pub version: u32,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// Author or organisation.
    #[serde(default)]
    pub author: String,
    /// Licence name or URL.
    #[serde(default)]
    pub license: String,
    /// Project homepage.
    #[serde(default)]
    pub homepage: String,
    /// App version that produced the pack.
    #[serde(default)]
    pub app_version: String,
    /// Schema version of the assets.
    #[serde(default)]
    pub schema_version: u32,
    /// The artwork in the pack.
    pub assets: Vec<PackAsset>,
}

/// Totals for an export or an import.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackReport {
    /// Assets described by the pack.
    pub assets: usize,
    /// Assets added to the project.
    pub added: usize,
    /// Assets that were already installed and were left alone.
    pub skipped: usize,
    /// Assets that were already installed but whose links the pack repaired.
    pub repaired: usize,
    /// Blob files written.
    pub files: u64,
    /// Total bytes written.
    pub bytes: u64,
}

/// What installing a content pack changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackInstall {
    /// Assets that are new to the project.
    pub added: Vec<Asset>,
    /// Assets that were already installed but whose links the pack repaired.
    pub repaired: Vec<Asset>,
    /// Totals for the operation.
    pub report: PackReport,
}

/// Parses and validates a pack manifest.
pub fn parse_pack_manifest(bytes: &[u8]) -> Result<PackManifest, CoreError> {
    let manifest: PackManifest = serde_json::from_slice(bytes).map_err(|error| {
        CoreError::InvalidPack(format!("invalid {PACK_MANIFEST_NAME}: {error}"))
    })?;
    if manifest.kind != PACK_KIND {
        return Err(CoreError::InvalidPack(format!(
            "expected a {PACK_KIND} manifest but found {:?}",
            manifest.kind
        )));
    }
    if manifest.version > PACK_VERSION {
        return Err(CoreError::InvalidPack(format!(
            "pack format version {} is newer than the supported version {PACK_VERSION}",
            manifest.version
        )));
    }
    Ok(manifest)
}

/// Reads the manifest of the pack in `directory`.
pub fn read_pack_manifest(directory: &Path) -> Result<PackManifest, CoreError> {
    let path = directory.join(PACK_MANIFEST_NAME);
    let bytes = fs::read(&path).map_err(|source| {
        CoreError::InvalidPack(format!("could not read {}: {source}", path.display()))
    })?;
    parse_pack_manifest(&bytes)
}

/// Builds the pack manifest for a set of assets.
fn manifest_for(assets: &[&Asset], info: &PackInfo) -> PackManifest {
    PackManifest {
        kind: PACK_KIND.to_owned(),
        version: PACK_VERSION,
        name: info.name.clone(),
        description: info.description.clone(),
        author: info.author.clone(),
        license: info.license.clone(),
        homepage: info.homepage.clone(),
        app_version: VERSION.to_owned(),
        schema_version: SCHEMA_VERSION,
        assets: assets
            .iter()
            .map(|asset| PackAsset {
                id: asset.id.clone(),
                name: asset.name.clone(),
                category: asset.category,
                hash: asset.hash.clone(),
                ext: asset.ext.clone(),
                original_filename: asset.original_filename.clone(),
                icon: asset.icon.clone(),
                portrait: asset.portrait.clone(),
                transparent: asset.transparent.clone(),
                badge: asset.badge.clone(),
                role: asset.role,
                game_mode: asset.game_mode.clone(),
                attachment: asset.attachment.clone(),
            })
            .collect(),
    }
}

impl AssetStore {
    /// Writes a content pack with every user asset in `state`.
    ///
    /// Every blob is read and hash-verified before the pack folder is created,
    /// so a corrupt source store cannot leave a partial pack behind.
    pub fn export_content_pack(
        &self,
        state: &AppState,
        directory: &Path,
        info: &PackInfo,
    ) -> Result<PackReport, CoreError> {
        let assets: Vec<&Asset> = state
            .assets
            .iter()
            .filter(|asset| asset.origin == AssetOrigin::User)
            .collect();

        let mut pending = Vec::new();
        let mut seen = BTreeSet::new();
        for asset in &assets {
            if !seen.insert((asset.hash.clone(), asset.ext.clone())) {
                continue;
            }
            let source_path = blob_path(self.root(), &asset.hash, &asset.ext)?;
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
            pending.push((asset.hash.clone(), asset.ext.clone(), bytes));
        }

        fs::create_dir_all(directory).map_err(|source| CoreError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let pack_blobs = directory.join("blobs");
        fs::create_dir_all(&pack_blobs).map_err(|source| CoreError::Io {
            path: pack_blobs.clone(),
            source,
        })?;

        let manifest = manifest_for(&assets, info);
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| CoreError::Serialize(error.to_string()))?;
        fs::write(directory.join(PACK_MANIFEST_NAME), manifest_bytes).map_err(|source| {
            CoreError::Io {
                path: directory.join(PACK_MANIFEST_NAME),
                source,
            }
        })?;

        let mut report = PackReport {
            assets: assets.len(),
            ..PackReport::default()
        };
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
            report.files += 1;
            report.bytes += bytes.len() as u64;
        }

        Ok(report)
    }

    /// Reads the artwork of a content pack so it can be added to a project.
    ///
    /// The pack never replaces the project. `existing` is the project's
    /// current asset list; assets that are already installed are skipped, so
    /// installing the same pack twice changes nothing. Links to assets that
    /// neither the pack nor the project provides are dropped rather than left
    /// dangling. The returned assets are the ones to add to the project.
    pub fn import_content_pack(
        &self,
        existing: &[Asset],
        directory: &Path,
    ) -> Result<PackInstall, CoreError> {
        let manifest = read_pack_manifest(directory)?;
        let mut report = PackReport {
            assets: manifest.assets.len(),
            ..PackReport::default()
        };

        // The assets the project already uses, keyed by id.
        let existing: BTreeMap<String, &Asset> = existing
            .iter()
            .map(|asset| (asset.id.as_str().to_owned(), asset))
            .collect();

        // Decide what to install before touching the store, so a bad blob
        // cannot leave half a pack behind.
        let mut installs: Vec<(AssetId, &PackAsset)> = Vec::new();
        let mut already: Vec<&PackAsset> = Vec::new();
        for asset in &manifest.assets {
            match existing.get(asset.id.as_str()) {
                Some(current) if current.hash == asset.hash => {
                    already.push(asset);
                }
                Some(_) => {
                    // The id is taken by different content. Give the pack's
                    // asset a fresh identity instead of overwriting anything.
                    installs.push((AssetId::new(), asset));
                }
                None => installs.push((asset.id.clone(), asset)),
            }
        }

        let mut pending = Vec::new();
        for (_, asset) in &installs {
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
                return Err(CoreError::InvalidPack(format!(
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

        for (hash, ext, bytes) in pending {
            self.write_blob(&hash, &ext, &bytes)?;
            report.files += 1;
            report.bytes += bytes.len() as u64;
        }

        // The id each declared pack id is installed under. A link has to be
        // rewritten to point at the id the pack actually installs.
        let installed_ids: BTreeMap<String, AssetId> = installs
            .iter()
            .map(|(id, asset)| (asset.id.as_str().to_owned(), id.clone()))
            .collect();

        // A link survives when its target is installed by this pack or is
        // already in the project.
        let resolve = |link: &Option<AssetId>| -> Option<AssetId> {
            let link = link.as_ref()?;
            if let Some(id) = installed_ids.get(link.as_str()) {
                return Some(id.clone());
            }
            if existing.contains_key(link.as_str()) {
                return Some(link.clone());
            }
            None
        };

        let mut added = Vec::new();
        for (id, asset) in &installs {
            let attachment = match &asset.attachment {
                Some(attachment) => {
                    let Some(owner) = resolve(&Some(attachment.owner.clone())) else {
                        // An attachment whose owner is missing would appear in
                        // the library as a selectable hero, so leave it out.
                        continue;
                    };
                    Some(AssetAttachment {
                        owner,
                        kind: attachment.kind,
                    })
                }
                None => None,
            };

            added.push(Asset {
                id: id.clone(),
                name: asset.name.clone(),
                category: asset.category,
                hash: asset.hash.clone(),
                ext: asset.ext.clone(),
                origin: AssetOrigin::User,
                original_filename: asset.original_filename.clone(),
                added_at: None,
                icon: resolve(&asset.icon),
                badge: resolve(&asset.badge),
                portrait: resolve(&asset.portrait),
                transparent: resolve(&asset.transparent),
                role: asset.role,
                game_mode: resolve(&asset.game_mode),
                attachment,
            });
        }

        // An asset that is already installed is left alone, except that a link
        // which no longer resolves is repaired from the pack. That is how a
        // project whose links pointed at the old bundled icons picks up the
        // pack's own icons without losing a link the operator set themselves.
        let mut repaired = Vec::new();
        for asset in &already {
            let Some(current) = existing.get(asset.id.as_str()) else {
                continue;
            };
            let mut fixed = (*current).clone();
            let mut changed = false;

            let repair = |slot: &mut Option<AssetId>, pack_link: &Option<AssetId>| {
                let dangling = match slot {
                    None => true,
                    Some(id) => !existing.contains_key(id.as_str()),
                };
                if !dangling {
                    return false;
                }
                match resolve(pack_link) {
                    Some(link) => {
                        *slot = Some(link);
                        true
                    }
                    None => false,
                }
            };

            changed |= repair(&mut fixed.icon, &asset.icon);
            changed |= repair(&mut fixed.badge, &asset.badge);
            changed |= repair(&mut fixed.portrait, &asset.portrait);
            changed |= repair(&mut fixed.transparent, &asset.transparent);
            changed |= repair(&mut fixed.game_mode, &asset.game_mode);

            if let (None, Some(role)) = (fixed.role, asset.role) {
                fixed.role = Some(role);
                changed = true;
            }

            if changed {
                repaired.push(fixed);
            } else {
                report.skipped += 1;
            }
        }

        report.added = added.len();
        report.repaired = repaired.len();

        Ok(PackInstall {
            added,
            repaired,
            report,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, AssetOrigin};

    const PNG_A: &[u8] = b"\x89PNG\r\n\x1a\npack-a";
    const PNG_B: &[u8] = b"\x89PNG\r\n\x1a\npack-b";

    fn store(root: &Path) -> AssetStore {
        AssetStore::new(root.join("assets"))
    }

    /// Imports a pack into a project the way the app does.
    fn install(target: &AssetStore, installed: &mut AppState, pack: &Path) -> PackReport {
        let outcome = target
            .import_content_pack(&installed.assets, pack)
            .expect("import");
        for asset in &outcome.repaired {
            if let Some(current) = installed.assets.iter_mut().find(|a| a.id == asset.id) {
                *current = asset.clone();
            }
        }
        installed.assets.extend(outcome.added);
        outcome.report
    }

    /// A project with one logo, one hero and a team.
    fn project(root: &Path) -> (AssetStore, AppState) {
        let asset_store = store(root);
        let mut state = AppState::default();
        let logo = asset_store
            .insert("Blasters", AssetCategory::Logo, PNG_A, None)
            .expect("logo")
            .asset;
        let hero = asset_store
            .insert("Ana", AssetCategory::Hero, PNG_B, None)
            .expect("hero")
            .asset;
        state.assets.push(logo);
        state.assets.push(hero);
        state.current_match = Some(crate::new_match());
        (asset_store, state)
    }

    #[test]
    fn a_pack_round_trips_its_assets() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());
        let pack = root.path().join("pack");
        let info = PackInfo {
            name: "Fruiterra Overwatch".to_owned(),
            description: "Test pack".to_owned(),
            author: "Tester".to_owned(),
            license: "CC0".to_owned(),
            homepage: "https://example.invalid".to_owned(),
        };

        let exported = asset_store
            .export_content_pack(&state, &pack, &info)
            .expect("export");
        assert_eq!(exported.assets, 2);
        assert_eq!(exported.files, 2);

        let manifest = read_pack_manifest(&pack).expect("manifest");
        assert_eq!(manifest.kind, PACK_KIND);
        assert_eq!(manifest.name, "Fruiterra Overwatch");
        assert_eq!(manifest.assets.len(), 2);

        let target = AssetStore::new(root.path().join("other"));
        let mut installed = AppState::default();
        let report = install(&target, &mut installed, &pack);
        assert_eq!(report.added, 2);
        assert_eq!(report.skipped, 0);
        assert_eq!(installed.assets.len(), 2);
        assert!(
            installed
                .assets
                .iter()
                .all(|a| a.origin == AssetOrigin::User)
        );
        assert!(
            installed
                .assets
                .iter()
                .any(|a| a.name == "Ana" && a.category == AssetCategory::Hero)
        );
    }

    #[test]
    fn importing_a_pack_twice_adds_nothing() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());
        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&state, &pack, &PackInfo::default())
            .expect("export");

        let target = AssetStore::new(root.path().join("other"));
        let mut installed = AppState::default();
        install(&target, &mut installed, &pack);
        let report = install(&target, &mut installed, &pack);
        assert_eq!(report.added, 0);
        assert_eq!(report.skipped, 2);
        assert_eq!(installed.assets.len(), 2);
    }

    #[test]
    fn a_pack_never_touches_the_rest_of_the_project() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());
        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&state, &pack, &PackInfo::default())
            .expect("export");

        let target = AssetStore::new(root.path().join("other"));
        let installed = AppState {
            current_match: Some(crate::new_match()),
            ..AppState::default()
        };
        let match_before = installed.current_match.clone();

        target
            .import_content_pack(&installed.assets, &pack)
            .expect("import");
        assert_eq!(installed.current_match, match_before);
        assert!(installed.teams.is_empty());
    }

    #[test]
    fn a_tampered_blob_is_rejected_and_nothing_is_installed() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());
        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&state, &pack, &PackInfo::default())
            .expect("export");

        // Tamper with one blob so its hash no longer matches the manifest.
        let manifest = read_pack_manifest(&pack).expect("manifest");
        let target_asset = &manifest.assets[0];
        let blob = blob_path(&pack, &target_asset.hash, &target_asset.ext).expect("blob path");
        fs::write(&blob, PNG_B).expect("tamper");

        let target = AssetStore::new(root.path().join("other"));
        let installed = AppState::default();
        let error = target
            .import_content_pack(&installed.assets, &pack)
            .expect_err("tampered pack must fail");
        assert!(matches!(error, CoreError::AssetHashMismatch { .. }));
        assert!(installed.assets.is_empty());
    }

    #[test]
    fn a_link_to_a_missing_asset_is_dropped() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());

        // Point the hero at an asset that is not in the pack.
        let hero_id = state.assets[1].id.clone();
        let mut state = state;
        state.assets[1].icon = Some(AssetId::new());
        let _ = hero_id;

        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&state, &pack, &PackInfo::default())
            .expect("export");

        let target = AssetStore::new(root.path().join("other"));
        let mut installed = AppState::default();
        install(&target, &mut installed, &pack);
        assert!(installed.assets.iter().all(|asset| asset.icon.is_none()));
    }

    #[test]
    fn a_pack_repairs_a_link_that_no_longer_resolves() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());

        // The pack links the hero to the logo.
        let logo_id = state.assets[0].id.clone();
        let mut with_link = state.clone();
        with_link.assets[1].icon = Some(logo_id.clone());

        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&with_link, &pack, &PackInfo::default())
            .expect("export");

        // The project already holds the same hero, but its link points at an
        // asset that is not there, the way a project that pointed at the old
        // bundled icons did.
        let mut installed = state.clone();
        installed.assets[1].icon = Some(AssetId::new());

        let target = AssetStore::new(root.path().join("other"));
        let report = install(&target, &mut installed, &pack);
        assert_eq!(report.added, 0, "both assets are already installed");
        assert_eq!(report.repaired, 1, "the hero link is repaired");
        assert_eq!(installed.assets[1].icon, Some(logo_id));
    }

    #[test]
    fn a_manifest_that_is_not_a_pack_is_rejected() {
        let root = tempfile::tempdir().expect("temp dir");
        let pack = root.path().join("pack");
        fs::create_dir_all(&pack).expect("pack dir");
        fs::write(
            pack.join(PACK_MANIFEST_NAME),
            br#"{"kind":"something-else"}"#,
        )
        .expect("write");
        let error = read_pack_manifest(&pack).expect_err("must fail");
        assert!(matches!(error, CoreError::InvalidPack(_)));
    }

    #[test]
    fn an_asset_in_the_pack_keeps_its_identity() {
        let root = tempfile::tempdir().expect("temp dir");
        let (asset_store, state) = project(root.path());
        let pack = root.path().join("pack");
        asset_store
            .export_content_pack(&state, &pack, &PackInfo::default())
            .expect("export");

        let target = AssetStore::new(root.path().join("other"));
        let mut installed = AppState::default();
        install(&target, &mut installed, &pack);
        for source in &state.assets {
            let matching = installed
                .assets
                .iter()
                .find(|asset| asset.hash == source.hash)
                .expect("installed asset");
            assert_eq!(matching.id, source.id);
            let _: &Asset = matching;
        }
    }
}
