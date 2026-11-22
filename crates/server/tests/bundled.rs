//! Bundled manifest integrity and catalogue-link tests.

use std::collections::BTreeSet;
use std::path::Path;

use fgc_core::{AssetCategory, is_allowed_extension, is_valid_hash};
use fgc_server::{bundled_asset_bytes, bundled_assets, bundled_catalog};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn manifest_is_sorted_unique_and_matches_the_embedded_bytes() {
    let assets = bundled_assets();
    assert!(!assets.is_empty(), "the bundled manifest is empty");

    let names: Vec<&str> = assets.iter().map(|asset| asset.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "bundled assets must be name-sorted");
    assert_eq!(
        names.iter().collect::<BTreeSet<_>>().len(),
        names.len(),
        "bundled asset names must be unique"
    );

    for asset in assets {
        assert!(
            is_valid_hash(asset.hash),
            "{} has an invalid hash",
            asset.name
        );
        assert!(
            is_allowed_extension(asset.ext),
            "{} has an unsupported extension",
            asset.name
        );
        assert_eq!(asset.id, format!("b:{}", asset.hash));

        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(asset.path);
        let bytes = std::fs::read(&path).expect("bundled file should exist");
        assert_eq!(sha256(&bytes), asset.hash, "{} hash mismatch", asset.name);

        let embedded = bundled_asset_bytes(asset.path).expect("embedded bytes");
        assert_eq!(embedded.as_ref(), bytes.as_slice());
    }
}

#[test]
fn catalog_contains_every_category_and_resolves_links() {
    let catalog = bundled_catalog();
    assert_eq!(catalog.len(), bundled_assets().len());

    let ids: BTreeSet<String> = catalog
        .iter()
        .map(|asset| asset.id.as_str().to_owned())
        .collect();
    let categories: Vec<AssetCategory> = catalog.iter().map(|asset| asset.category).collect();
    // The application ships only the placeholder watermark. Game-mode, role
    // and side art belongs to a content pack, so the binary stays generic.
    assert!(
        categories.contains(&AssetCategory::Logo),
        "the placeholder watermark must be bundled"
    );
    for category in [
        AssetCategory::GameMode,
        AssetCategory::Role,
        AssetCategory::Side,
    ] {
        assert!(
            !categories.contains(&category),
            "{category:?} art belongs to a content pack, not the binary"
        );
    }

    for asset in &catalog {
        for link in [&asset.icon, &asset.portrait, &asset.game_mode]
            .into_iter()
            .flatten()
        {
            assert!(
                ids.contains(link.as_str()),
                "{} links to unknown asset {}",
                asset.name,
                link
            );
        }
    }
}
