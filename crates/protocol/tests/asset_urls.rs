//! Asset URL, catalogue and font-URL tests.

mod support;

use fgc_core::{AssetCategory, AssetId};
use fgc_protocol::{
    AssetCatalog, AssetUrlResolver, DataUrlResolver, ServedAssetResolver, data_url_for, font_url,
    mime_for_extension,
};

#[test]
fn served_urls_are_hash_addressed_and_absolute() {
    let resolver = ServedAssetResolver::new("http://127.0.0.1:3001");
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-map-ilios"))
        .expect("Ilios asset");

    let url = resolver.url(asset).expect("served URL");

    assert_eq!(
        url,
        format!("http://127.0.0.1:3001/assets/{}.png", support::hash(0x21))
    );
}

#[test]
fn served_resolver_trims_trailing_slashes() {
    let resolver = ServedAssetResolver::new("http://127.0.0.1:3001///");

    assert_eq!(resolver.base_url(), "http://127.0.0.1:3001");
}

#[test]
fn served_resolver_keeps_the_extension() {
    let resolver = ServedAssetResolver::new("http://127.0.0.1:3001");
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-ana"))
        .expect("Ana asset");

    let url = resolver.url(asset).expect("served URL");

    assert!(url.ends_with(&format!(".{}", asset.ext)));
}

#[test]
fn data_urls_encode_bytes_with_the_right_mime_type() {
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-map-ilios"))
        .expect("Ilios asset");

    let url = data_url_for(asset, b"\x89PNG");

    assert_eq!(url, "data:image/png;base64,iVBORw==");
}

#[test]
fn data_url_resolver_uses_the_supplied_loader() {
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-map-ilios"))
        .expect("Ilios asset");
    let resolver = DataUrlResolver::new(|_: &fgc_core::Asset| Some(b"\x89PNG".to_vec()));

    let url = resolver.url(asset).expect("data URL");

    assert_eq!(url, "data:image/png;base64,iVBORw==");
}

#[test]
fn data_url_resolver_returns_none_when_bytes_are_missing() {
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-map-ilios"))
        .expect("Ilios asset");
    let resolver = DataUrlResolver::new(|_: &fgc_core::Asset| None);

    assert_eq!(resolver.url(asset), None);
}

#[test]
fn mime_types_match_the_original_allow_list() {
    assert_eq!(mime_for_extension("png"), "image/png");
    assert_eq!(mime_for_extension("jpg"), "image/jpeg");
    assert_eq!(mime_for_extension("jpeg"), "image/jpeg");
    assert_eq!(mime_for_extension("gif"), "image/gif");
    assert_eq!(mime_for_extension("webp"), "image/webp");
    assert_eq!(mime_for_extension("svg"), "image/svg+xml");
    assert_eq!(mime_for_extension("bmp"), "image/png");
}

#[test]
fn font_urls_are_stable_and_readable() {
    let url = font_url("http://127.0.0.1:3001/", "saira-400.woff2");

    assert_eq!(
        url.as_deref(),
        Some("http://127.0.0.1:3001/assets/fonts/saira-400.woff2")
    );
}

#[test]
fn font_urls_reject_path_tricks_and_other_files() {
    let base = "http://127.0.0.1:3001";

    assert_eq!(font_url(base, ""), None);
    assert_eq!(font_url(base, "nested/saira-400.woff2"), None);
    assert_eq!(font_url(base, "nested\\saira-400.woff2"), None);
    assert_eq!(font_url(base, "..\\saira-400.woff2"), None);
    assert_eq!(font_url(base, "saira-400.png"), None);
}

#[test]
fn catalogue_get_returns_the_matching_asset() {
    let catalog = support::fixture_catalog();
    let asset = catalog
        .get(&AssetId::from_raw("asset-logo-a"))
        .expect("logo asset");

    assert_eq!(asset.name, "Alpha");
    assert_eq!(asset.category, AssetCategory::Logo);
}

#[test]
fn catalogue_get_returns_none_for_unknown_ids() {
    let catalog = support::fixture_catalog();

    assert!(catalog.get(&AssetId::from_raw("missing")).is_none());
}

#[test]
fn catalogue_find_by_name_is_case_insensitive() {
    let catalog = support::fixture_catalog();

    let lower = catalog
        .find_by_name(AssetCategory::Map, "ilios")
        .expect("lower-case match");
    let upper = catalog
        .find_by_name(AssetCategory::Map, "ILIOS")
        .expect("upper-case match");

    assert_eq!(lower.id, upper.id);
}

#[test]
fn catalogue_find_by_name_ignores_empty_names() {
    let catalog = support::fixture_catalog();

    assert!(catalog.find_by_name(AssetCategory::Map, "").is_none());
}

#[test]
fn catalogue_find_exact_is_case_sensitive() {
    let catalog = support::fixture_catalog();

    assert!(catalog.find_exact(AssetCategory::Logo, "Peach").is_some());
    assert!(catalog.find_exact(AssetCategory::Logo, "peach").is_none());
}

#[test]
fn catalogue_in_category_preserves_insertion_order() {
    let catalog = support::fixture_catalog();
    let modes: Vec<&str> = catalog
        .in_category(AssetCategory::GameMode)
        .map(|asset| asset.name.as_str())
        .collect();

    assert_eq!(modes, ["Control", "Escort", "Hybrid", "Flashpoint", "Push"]);
}

#[test]
fn catalogue_reports_linked_icons() {
    let catalog = support::fixture_catalog();

    assert!(catalog.is_linked_icon(&AssetId::from_raw("asset-ana-icon")));
    assert!(!catalog.is_linked_icon(&AssetId::from_raw("asset-ana")));
    assert!(!catalog.is_linked_icon(&AssetId::from_raw("missing")));
}

#[test]
fn catalogue_len_and_is_empty() {
    let empty = AssetCatalog::default();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);

    let catalog = support::fixture_catalog();
    assert!(!catalog.is_empty());
    assert_eq!(catalog.len(), catalog.assets().len());
}
