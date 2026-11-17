//! Overlay override lookup and path validation.
//!
//! Override names are validated before any filesystem access. Only plain
//! `.html` file names made of ASCII alphanumerics, `-`, `_` and `.` are
//! accepted; separators, parent components and non-HTML extensions are
//! rejected.

use std::path::Path;

/// Validates an overlay file name.
///
/// Returns the name unchanged when it is safe to join beneath the override
/// directory. Rejects empty names, separators, parent components and anything
/// that is not a plain `.html` file.
#[must_use]
pub fn validate_overlay_name(name: &str) -> Option<&str> {
    if name.is_empty() || name.len() > 128 || !name.ends_with(".html") {
        return None;
    }
    let stem = name.strip_suffix(".html")?;
    if stem.is_empty() || name.starts_with('.') || name.contains("..") {
        return None;
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return None;
    }
    Some(name)
}

/// Validates a bundled font file name.
///
/// Fonts use stable names (`/assets/fonts/<name>.woff2`), so the same path
/// rules apply as for overlays, with a `.woff2` extension.
#[must_use]
pub fn validate_font_name(name: &str) -> Option<&str> {
    if name.is_empty() || name.len() > 128 || !name.ends_with(".woff2") {
        return None;
    }
    let stem = name.strip_suffix(".woff2")?;
    if stem.is_empty() || name.starts_with('.') || name.contains("..") {
        return None;
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return None;
    }
    Some(name)
}

/// Splits `/assets/<hash>.<ext>` into its hash and extension.
#[must_use]
pub fn split_asset_file(file: &str) -> Option<(&str, &str)> {
    file.rsplit_once('.')
}

/// Reads a per-file overlay override when one exists.
///
/// The name is validated again here so a caller that forgets to check cannot
/// escape the override directory. An empty or non-UTF-8 file is treated as
/// missing so a broken override cannot leave an overlay blank.
#[must_use]
pub fn read_override(dir: Option<&Path>, name: &str) -> Option<Vec<u8>> {
    let dir = dir?;
    let name = validate_overlay_name(name)?;
    let path = dir.join(name);
    match std::fs::read(&path) {
        Ok(bytes) if bytes.is_empty() => {
            tracing::warn!(
                path = %path.display(),
                "overlay override is empty; falling back to the embedded file"
            );
            None
        }
        Ok(bytes) if std::str::from_utf8(&bytes).is_err() => {
            tracing::warn!(
                path = %path.display(),
                "overlay override is not valid UTF-8; falling back to the embedded file"
            );
            None
        }
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                %error,
                "failed to read the overlay override; falling back to the embedded file"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bundled_overlay_names() {
        for name in [
            "game-overlay.html",
            "hero-bans.html",
            "map-pool.html",
            "map-select.html",
            "row-casters.html",
            "team-roster-away.html",
            "team-roster-home.html",
        ] {
            assert_eq!(validate_overlay_name(name), Some(name));
        }
    }

    #[test]
    fn rejects_traversal_and_non_html_names() {
        for name in [
            "",
            "..",
            "../secret.html",
            "..%2fsecret.html",
            "nested/game-overlay.html",
            "nested\\game-overlay.html",
            "game-overlay.txt",
            ".html",
            "..html",
            "game overlay.html",
            "game-overlay.html.bak",
        ] {
            assert_eq!(validate_overlay_name(name), None, "accepted {name:?}");
        }
    }

    #[test]
    fn accepts_font_names() {
        assert_eq!(
            validate_font_name("saira-display-300.woff2"),
            Some("saira-display-300.woff2")
        );
        assert_eq!(validate_font_name("../saira.woff2"), None);
        assert_eq!(validate_font_name("saira.woff"), None);
        assert_eq!(validate_font_name("fonts/saira.woff2"), None);
    }

    #[test]
    fn splits_asset_paths_at_the_last_dot() {
        assert_eq!(split_asset_file("3fa1c9e2.png"), Some(("3fa1c9e2", "png")));
        assert_eq!(split_asset_file("no-extension"), None);
    }

    #[test]
    fn override_lookup_never_escapes_its_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("game-overlay.html"), b"override").expect("write");
        std::fs::write(dir.path().join("secret.txt"), b"secret").expect("write");

        assert_eq!(
            read_override(Some(dir.path()), "game-overlay.html"),
            Some(b"override".to_vec())
        );
        assert_eq!(read_override(Some(dir.path()), "secret.txt"), None);
        assert_eq!(read_override(Some(dir.path()), "../secret.txt"), None);
    }

    #[test]
    fn empty_and_non_utf8_overrides_fall_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("game-overlay.html");

        std::fs::write(&path, b"").expect("write");
        assert_eq!(read_override(Some(dir.path()), "game-overlay.html"), None);

        std::fs::write(&path, [0xff, 0xfe, 0x00]).expect("write");
        assert_eq!(read_override(Some(dir.path()), "game-overlay.html"), None);

        std::fs::write(&path, b"<!doctype html>").expect("write");
        assert_eq!(
            read_override(Some(dir.path()), "game-overlay.html"),
            Some(b"<!doctype html>".to_vec())
        );
    }
}
