use std::path::{Path, PathBuf};

use crate::{Asset, AssetCategory, AssetId, AssetOrigin, CoreError};

/// Extensions accepted by the content-addressed asset store.
pub const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "svg"];

/// Maximum accepted size for a user-imported asset, in bytes.
pub const MAX_ASSET_BYTES: u64 = 20 * 1024 * 1024;

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

/// Detects an allow-listed extension from a file's magic bytes.
///
/// The detected extension is authoritative for imports: a file named `.png`
/// that actually contains a JPEG is stored as a JPEG.
#[must_use]
pub fn detect_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("jpg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && bytes[8..].starts_with(b"WEBP") {
        return Some("webp");
    }
    if looks_like_svg(bytes) {
        return Some("svg");
    }
    None
}

/// Returns true when a declared extension is compatible with detected content.
///
/// `jpg` and `jpeg` are treated as the same format because both are accepted
/// in stored metadata.
#[must_use]
pub fn extensions_match(declared: &str, detected: &str) -> bool {
    declared == detected || matches!((declared, detected), ("jpg" | "jpeg", "jpg" | "jpeg"))
}

/// Rejects SVG content that could execute script or fetch external resources.
///
/// The check is deliberately conservative: a false positive is preferable to
/// serving active content into an OBS Browser Source. Element and attribute
/// names are matched case-insensitively, and fragments such as `url(#gradient)`
/// remain valid.
pub fn sanitize_svg(bytes: &[u8]) -> Result<(), CoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| unsafe_svg("the SVG is not valid UTF-8".to_owned()))?;
    let lower = text.to_ascii_lowercase();

    for marker in ["<!doctype", "<!entity"] {
        if lower.contains(marker) {
            return Err(unsafe_svg(format!("{marker}> is not allowed")));
        }
    }

    for element in [
        "script",
        "foreignobject",
        "iframe",
        "embed",
        "object",
        "animate",
        "animatetransform",
        "animatemotion",
        "set",
        "handler",
        "listener",
    ] {
        if contains_element(&lower, element) {
            return Err(unsafe_svg(format!("<{element}> elements are not allowed")));
        }
    }

    if has_event_handler(&lower) {
        return Err(unsafe_svg(
            "event handler attributes are not allowed".to_owned(),
        ));
    }

    for scheme in ["javascript:", "vbscript:", "data:", "file:"] {
        if contains_ignoring_whitespace(&lower, scheme) {
            return Err(unsafe_svg(format!("{scheme} URLs are not allowed")));
        }
    }

    for marker in ["@import", "expression(", "-moz-binding"] {
        if lower.contains(marker) {
            return Err(unsafe_svg(format!("{marker} is not allowed")));
        }
    }

    if has_external_url_attribute(&lower) {
        return Err(unsafe_svg(
            "external href/src references are not allowed".to_owned(),
        ));
    }
    if has_external_css_url(&lower) {
        return Err(unsafe_svg(
            "external url() references are not allowed".to_owned(),
        ));
    }

    Ok(())
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(4096)];
    let text = String::from_utf8_lossy(head);
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<svg") || (lower.starts_with("<?xml") && lower.contains("<svg"))
}

fn unsafe_svg(reason: String) -> CoreError {
    CoreError::UnsafeSvg(reason)
}

fn contains_element(haystack: &str, name: &str) -> bool {
    let mut rest = haystack;
    while let Some(index) = rest.find('<') {
        let after = &rest[index + 1..];
        let after = after.trim_start_matches('/');
        if let Some(tail) = after.strip_prefix(name)
            && tail.starts_with(|character: char| {
                character.is_ascii_whitespace() || character == '>' || character == '/'
            })
        {
            return true;
        }
        rest = after;
    }
    false
}

fn has_event_handler(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index + 2 < bytes.len() {
        if bytes[index] == b'o'
            && bytes[index + 1] == b'n'
            && bytes[index + 2].is_ascii_alphabetic()
        {
            let mut end = index + 2;
            while end < bytes.len() && bytes[end].is_ascii_alphanumeric() {
                end += 1;
            }
            let mut next = end;
            while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                next += 1;
            }
            if next < bytes.len() && bytes[next] == b'=' {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn contains_ignoring_whitespace(haystack: &str, needle: &str) -> bool {
    let compact: String = haystack
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '\0')
        .collect();
    compact.contains(needle)
}

fn has_external_url_attribute(lower: &str) -> bool {
    for name in ["href", "src", "srcset"] {
        let mut rest = lower;
        while let Some(index) = rest.find(name) {
            let after = &rest[index + name.len()..];
            let after = after.trim_start();
            if let Some(value) = after.strip_prefix('=') {
                let value = value.trim_start();
                let value = value
                    .strip_prefix('"')
                    .or_else(|| value.strip_prefix('\''))
                    .unwrap_or(value);
                let value = value.trim_start();
                if !value.is_empty() && !value.starts_with('#') {
                    return true;
                }
            }
            rest = &rest[index + name.len()..];
        }
    }
    false
}

fn has_external_css_url(lower: &str) -> bool {
    let mut rest = lower;
    while let Some(index) = rest.find("url(") {
        let after = &rest[index + 4..];
        let Some(end) = after.find(')') else {
            return true;
        };
        let target = after[..end]
            .trim()
            .trim_matches(|character| character == '"' || character == '\'')
            .trim();
        if !target.is_empty() && !target.starts_with('#') {
            return true;
        }
        rest = &after[end + 1..];
    }
    false
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
            transparent: None,
            role: None,
            game_mode: None,
            attachment: None,
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
            transparent: None,
            role: None,
            game_mode: None,
            attachment: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAFE_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><defs><linearGradient id="g"><stop offset="0" stop-color="#fff"/></linearGradient></defs><rect width="10" height="10" fill="url(#g)"/></svg>"##;

    #[test]
    fn detects_supported_formats() {
        assert_eq!(detect_extension(b"\x89PNG\r\n\x1a\nrest"), Some("png"));
        assert_eq!(detect_extension(&[0xff, 0xd8, 0xff, 0xe0]), Some("jpg"));
        assert_eq!(detect_extension(b"GIF89a...."), Some("gif"));
        assert_eq!(
            detect_extension(b"RIFF\x00\x00\x00\x00WEBPVP8 "),
            Some("webp")
        );
        assert_eq!(detect_extension(SAFE_SVG), Some("svg"));
    }

    #[test]
    fn rejects_unknown_formats() {
        assert_eq!(detect_extension(b"hello world"), None);
        assert_eq!(detect_extension(b""), None);
    }

    #[test]
    fn jpeg_and_jpg_are_interchangeable() {
        assert!(extensions_match("jpg", "jpeg"));
        assert!(extensions_match("jpeg", "jpg"));
        assert!(!extensions_match("png", "jpg"));
    }

    #[test]
    fn accepts_safe_svg() {
        sanitize_svg(SAFE_SVG).expect("safe SVG");
    }

    #[test]
    fn rejects_script_svg() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#;
        assert!(matches!(sanitize_svg(svg), Err(CoreError::UnsafeSvg(_))));
    }

    #[test]
    fn rejects_event_handler_attributes() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect onload="alert(1)"/></svg>"#;
        assert!(matches!(sanitize_svg(svg), Err(CoreError::UnsafeSvg(_))));
    }

    #[test]
    fn rejects_external_references() {
        let image =
            br#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.com/a.png"/></svg>"#;
        assert!(matches!(sanitize_svg(image), Err(CoreError::UnsafeSvg(_))));

        let css = br#"<svg xmlns="http://www.w3.org/2000/svg"><style>rect{fill:url(https://example.com/a.svg)}</style></svg>"#;
        assert!(matches!(sanitize_svg(css), Err(CoreError::UnsafeSvg(_))));
    }

    #[test]
    fn rejects_doctype_and_entities() {
        let doctype = br#"<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>"#;
        assert!(matches!(
            sanitize_svg(doctype),
            Err(CoreError::UnsafeSvg(_))
        ));

        let entity = br#"<!ENTITY xxe SYSTEM "file:///etc/passwd"><svg xmlns="http://www.w3.org/2000/svg"/>"#;
        assert!(matches!(sanitize_svg(entity), Err(CoreError::UnsafeSvg(_))));
    }
}
