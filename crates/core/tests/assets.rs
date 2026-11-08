use fgc_core::{
    ALLOWED_EXTENSIONS, AssetCategory, AssetOrigin, CoreError, blob_path, blob_relative_path,
    bundled_asset_id, is_allowed_extension, is_valid_hash, validate_asset_identity,
};
use std::path::Path;

const HASH: &str = "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa99887766554433221100";

#[test]
fn valid_hash_and_extensions_are_accepted() {
    assert!(is_valid_hash(HASH));
    for extension in ALLOWED_EXTENSIONS {
        assert!(is_allowed_extension(extension));
    }
    assert!(validate_asset_identity(HASH, "png").is_ok());
}

#[test]
fn hash_validation_rejects_traversal_and_malformed_input() {
    for hash in [
        "",
        "abc",
        "../../etc/passwd",
        "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa9988776655443322110",
        "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa998877665544332211000",
        "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa9988776655443322110g",
        "3FA1C9D2E4B5A6978877665544332211FFEEDDCCBBAA99887766554433221100",
    ] {
        assert!(!is_valid_hash(hash), "hash should be rejected: {hash}");
        assert!(matches!(
            validate_asset_identity(hash, "png"),
            Err(CoreError::InvalidAssetHash)
        ));
    }
}

#[test]
fn extension_validation_rejects_unknown_and_traversal() {
    for extension in ["", "exe", "PNG", "../png", "png/../../", "png ", "jpeg.png"] {
        assert!(
            !is_allowed_extension(extension),
            "extension should be rejected: {extension}"
        );
        assert!(matches!(
            validate_asset_identity(HASH, extension),
            Err(CoreError::InvalidAssetExtension(_))
        ));
    }
}

#[test]
fn blob_path_is_content_addressed() {
    let relative = blob_relative_path(HASH, "png").expect("valid identity");
    assert_eq!(
        relative,
        Path::new("blobs").join("3f").join(format!("{HASH}.png"))
    );

    let absolute = blob_path(Path::new("C:/data/assets"), HASH, "webp").expect("valid identity");
    assert!(absolute.ends_with(Path::new("3f").join(format!("{HASH}.webp"))));
}

#[test]
fn blob_path_rejects_invalid_identity() {
    assert!(blob_relative_path("../../escape", "png").is_err());
    assert!(blob_relative_path(HASH, "../png").is_err());
}

#[test]
fn bundled_ids_are_deterministic() {
    let first = bundled_asset_id(HASH).expect("valid hash");
    let second = bundled_asset_id(HASH).expect("valid hash");

    assert_eq!(first, second);
    assert_eq!(first.as_str(), format!("b:{HASH}"));
    assert!(bundled_asset_id("not-a-hash").is_err());
}

#[test]
fn asset_constructors_validate_and_set_identity() {
    let user = fgc_core::Asset::new_user(
        "Peach",
        AssetCategory::Logo,
        HASH,
        "png",
        Some("peach.png".to_owned()),
    )
    .expect("valid user asset");

    assert_eq!(user.origin, AssetOrigin::User);
    assert_eq!(user.hash, HASH);
    assert_eq!(user.ext, "png");

    let bundled = fgc_core::Asset::new_bundled("Ilios", AssetCategory::Map, HASH, "webp")
        .expect("valid bundled asset");

    assert_eq!(bundled.origin, AssetOrigin::Bundled);
    assert_eq!(bundled.id.as_str(), format!("b:{HASH}"));

    assert!(fgc_core::Asset::new_user("Bad", AssetCategory::Logo, "bad", "png", None).is_err());
    assert!(fgc_core::Asset::new_bundled("Bad", AssetCategory::Map, HASH, "exe").is_err());
}
