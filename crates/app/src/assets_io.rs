//! Native dialogs, URL fetching and Windows shell integration for assets.
//!
//! This module is the only place in the UI that talks to `rfd`, `reqwest` or
//! `explorer.exe`. Keeping it isolated makes the state layer easy to test and
//! keeps every filesystem/network failure on a blocking task.

use std::path::{Path, PathBuf};
use std::time::Duration;

use fgc_core::MAX_ASSET_BYTES;
use reqwest::Url;

/// Timeout applied to a single URL import.
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout applied to a content pack download.
///
/// A pack carries whole artwork libraries, so it is far larger than a single
/// image and gets a much longer budget.
const PACK_FETCH_TIMEOUT: Duration = Duration::from_secs(600);

/// Hard size cap for a downloaded content pack.
const MAX_PACK_BYTES: u64 = 512 * 1024 * 1024;

/// Opens the native file picker for one or more allow-listed images.
#[must_use]
pub fn pick_image_files() -> Vec<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import images")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "svg"])
        .pick_files()
        .unwrap_or_default()
}

/// Opens the native file picker for one allow-listed image.
#[must_use]
pub fn pick_image_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose image")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "svg"])
        .pick_file()
}

/// Opens the native folder picker used for project bundles.
#[must_use]
pub fn pick_bundle_folder(title: &str) -> Option<PathBuf> {
    rfd::FileDialog::new().set_title(title).pick_folder()
}

/// Opens a save dialog for a match library file.
#[must_use]
pub fn pick_match_library_save() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export matches")
        .add_filter("Match library", &[fgc_core::MATCH_LIBRARY_EXTENSION])
        .set_file_name("matches.json")
        .save_file()
}

/// Opens an open dialog for a match library file.
#[must_use]
pub fn pick_match_library_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import matches")
        .add_filter("Match library", &[fgc_core::MATCH_LIBRARY_EXTENSION])
        .pick_file()
}

/// Opens the native file picker for an overlay theme file.
#[must_use]
pub fn pick_theme_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import overlay theme")
        .add_filter("Overlay theme", &["json"])
        .pick_file()
}

/// Validates a URL before any network request is made.
///
/// Only absolute `http`/`https` URLs without embedded credentials are
/// accepted. Relative URLs, `file:` URLs and other schemes are rejected.
pub fn validate_import_url(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Enter an image URL.".to_owned());
    }

    let url = Url::parse(trimmed).map_err(|_| "That is not a valid URL.".to_owned())?;
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "Only http and https URLs can be imported, not {scheme}:."
            ));
        }
    }
    if url.host_str().is_none() {
        return Err("The URL must include a host name.".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URLs containing credentials are not supported.".to_owned());
    }
    Ok(url)
}

/// Fetches an image with a timeout and a hard size cap.
///
/// The cap is enforced both from `Content-Length` and while streaming, so a
/// server that lies about its length cannot exhaust memory.
pub async fn fetch_image(url: Url) -> Result<(Vec<u8>, Option<String>), String> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|error| format!("Could not create the HTTP client: {error}"))?;

    let mut response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|error| format!("Could not fetch the image: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("The server returned {}.", response.status()));
    }
    if let Some(length) = response.content_length()
        && length > MAX_ASSET_BYTES
    {
        return Err(format!(
            "The image is larger than the {} limit.",
            format_bytes(MAX_ASSET_BYTES)
        ));
    }

    let filename = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned);

    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Could not read the image: {error}"))?
    {
        if bytes.len() as u64 + chunk.len() as u64 > MAX_ASSET_BYTES {
            return Err(format!(
                "The image is larger than the {} limit.",
                format_bytes(MAX_ASSET_BYTES)
            ));
        }
        bytes.extend_from_slice(&chunk);
    }

    Ok((bytes, filename))
}

/// Downloads a content pack archive.
///
/// The size cap is enforced from `Content-Length` and while streaming, so a
/// server that lies about the length cannot exhaust memory.
pub async fn fetch_pack(url: Url) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(PACK_FETCH_TIMEOUT)
        .build()
        .map_err(|error| format!("Could not create the HTTP client: {error}"))?;

    let mut response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|error| format!("Could not download the pack: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("The server returned {}.", response.status()));
    }
    if let Some(length) = response.content_length()
        && length > MAX_PACK_BYTES
    {
        return Err(format!(
            "The pack is larger than the {} limit.",
            format_bytes(MAX_PACK_BYTES)
        ));
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Could not read the pack: {error}"))?
    {
        if bytes.len() as u64 + chunk.len() as u64 > MAX_PACK_BYTES {
            return Err(format!(
                "The pack is larger than the {} limit.",
                format_bytes(MAX_PACK_BYTES)
            ));
        }
        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
}

/// Extracts a zipped content pack into `directory`.
///
/// Entries that try to escape the target directory are skipped, and the
/// folder that holds `pack.json` is returned. A zip that wraps the pack in a
/// single folder is accepted as well.
pub fn extract_pack_archive(bytes: Vec<u8>, directory: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create {}: {error}", directory.display()))?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|error| format!("That is not a readable zip archive: {error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Could not read the archive: {error}"))?;
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let target = directory.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        }
        let mut file = std::fs::File::create(&target)
            .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
        std::io::copy(&mut entry, &mut file)
            .map_err(|error| format!("Could not write {}: {error}", target.display()))?;
    }

    pack_manifest_folder(directory)
        .ok_or_else(|| "The archive does not contain a content pack.".to_owned())
}

/// Finds the folder that holds a pack manifest.
///
/// The manifest normally sits at the root of the pack, but a zip that wraps
/// the whole pack in one folder is accepted too.
#[must_use]
pub fn pack_manifest_folder(directory: &Path) -> Option<PathBuf> {
    let manifest = fgc_core::PACK_MANIFEST_NAME;
    if directory.join(manifest).is_file() {
        return Some(directory.to_path_buf());
    }
    let mut found = None;
    for entry in std::fs::read_dir(directory).ok()? {
        let Ok(entry) = entry else { continue };
        if !entry.path().is_dir() {
            continue;
        }
        if entry.path().join(manifest).is_file() {
            if found.is_some() {
                // More than one candidate: the archive is not a pack.
                return None;
            }
            found = Some(entry.path());
        }
    }
    found
}

/// Formats a byte count for display in Settings and toasts.
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let value = bytes as f64;
    if value >= MIB {
        format!("{:.1} MB", value / MIB)
    } else if value >= KIB {
        format!("{:.1} KB", value / KIB)
    } else {
        format!("{bytes} B")
    }
}

/// Creates the per-file overlay override directory and reveals it in Explorer.
pub fn open_overlays_folder() -> Result<PathBuf, String> {
    let directory = fgc_core::overlays_override_dir()
        .map_err(|error| format!("Could not resolve it: {error}"))?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create {}: {error}", directory.display()))?;
    reveal(&directory)?;
    Ok(directory)
}

/// Opens an app-generated overlay URL in the default browser.
///
/// The URL comes from `overlay_url`, but it is still passed straight to the
/// child process instead of through a shell so no argument is re-parsed.
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Could not open the browser: {error}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
        Err("Opening the browser is only supported on Windows.".to_owned())
    }
}

#[cfg(target_os = "windows")]
fn reveal(directory: &Path) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open the overlays folder: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn reveal(directory: &Path) -> Result<(), String> {
    let _ = directory;
    Err("Opening the overlays folder is only supported on Windows.".to_owned())
}

/// Creates a timestamped bundle directory beneath `parent`.
///
/// Exporting into a fresh folder means a previous export can never leave stale
/// blobs behind in the new bundle.
#[must_use]
pub fn unique_bundle_directory(parent: &Path) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let base = parent.join(format!("fruiterra-project-{stamp}"));
    if !base.exists() {
        return base;
    }
    for suffix in 1..1000 {
        let candidate = parent.join(format!("fruiterra-project-{stamp}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    base
}

/// Creates a timestamped content pack directory beneath `parent`.
///
/// A pack folder always starts empty so a previous export can never leave
/// stale blobs behind.
#[must_use]
pub fn unique_pack_directory(parent: &Path) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let base = parent.join(format!("fruiterra-pack-{stamp}"));
    if !base.exists() {
        return base;
    }
    for suffix in 1..1000 {
        let candidate = parent.join(format!("fruiterra-pack-{stamp}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_urls() {
        assert!(validate_import_url("https://example.com/logo.png").is_ok());
        assert!(validate_import_url("http://example.com/logo.png").is_ok());
        assert!(validate_import_url("  https://example.com/a.webp  ").is_ok());
    }

    #[test]
    fn rejects_relative_and_non_http_urls() {
        assert!(validate_import_url("logo.png").is_err());
        assert!(validate_import_url("file:///C:/logo.png").is_err());
        assert!(validate_import_url("javascript:alert(1)").is_err());
        assert!(validate_import_url("data:image/png;base64,AAAA").is_err());
        assert!(validate_import_url("").is_err());
    }

    #[test]
    fn rejects_urls_with_credentials() {
        assert!(validate_import_url("https://user:secret@example.com/a.png").is_err());
        assert!(validate_import_url("https://user@example.com/a.png").is_err());
    }

    #[test]
    fn formats_bytes_for_display() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(MAX_ASSET_BYTES), "20.0 MB");
    }

    #[test]
    fn unique_bundle_directories_do_not_collide() {
        let parent = tempfile::tempdir().expect("temp dir");
        let first = unique_bundle_directory(parent.path());
        std::fs::create_dir_all(&first).expect("create first bundle");
        let second = unique_bundle_directory(parent.path());
        assert_ne!(first, second);
    }
}
