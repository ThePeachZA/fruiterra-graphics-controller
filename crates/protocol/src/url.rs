use base64::Engine as _;
use fgc_core::Asset;

/// Resolves an asset's content identity to a URL the overlay can load.
///
/// Implementations must not perform I/O unless the caller explicitly chose the
/// data-URL mode.
pub trait AssetUrlResolver: Send + Sync {
    /// Returns the URL for `asset`, or `None` when the bytes are unavailable.
    fn url(&self, asset: &Asset) -> Option<String>;
}

/// Serves assets from `GET /assets/<hash>.<ext>` on the overlay server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServedAssetResolver {
    base_url: String,
}

impl ServedAssetResolver {
    /// Creates a resolver for `base_url`, which must be absolute.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: trim_base_url(&base_url.into()),
        }
    }

    /// The normalised base URL without a trailing slash.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl AssetUrlResolver for ServedAssetResolver {
    fn url(&self, asset: &Asset) -> Option<String> {
        Some(format!(
            "{}/assets/{}.{}",
            self.base_url, asset.hash, asset.ext
        ))
    }
}

/// Produces `data:` URLs by calling a caller-supplied byte loader.
///
/// The loader is where the server reads blob files; projection itself performs
/// no filesystem access. A loader that returns `None` yields a `null` URL,
/// matching the original's "image not found" behaviour.
pub struct DataUrlResolver<F> {
    load: F,
}

impl<F> DataUrlResolver<F>
where
    F: Fn(&Asset) -> Option<Vec<u8>>,
{
    /// Creates a resolver backed by `load`.
    #[must_use]
    pub fn new(load: F) -> Self {
        Self { load }
    }
}

impl<F> AssetUrlResolver for DataUrlResolver<F>
where
    F: Fn(&Asset) -> Option<Vec<u8>> + Send + Sync,
{
    fn url(&self, asset: &Asset) -> Option<String> {
        let bytes = (self.load)(asset)?;
        Some(data_url_for(asset, &bytes))
    }
}

/// Builds a base64 `data:` URL for `asset` from its bytes.
#[must_use]
pub fn data_url_for(asset: &Asset, bytes: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime_for_extension(&asset.ext), encoded)
}

/// MIME type for an allow-listed asset extension.
#[must_use]
pub fn mime_for_extension(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "image/png",
    }
}

/// Stable URL for a bundled overlay font.
///
/// Fonts keep a readable path (`/assets/fonts/<name>.woff2`) rather than a
/// content hash so the CSS aliases stay stable across builds. Returns `None`
/// for anything that is not a plain `.woff2` file name.
#[must_use]
pub fn font_url(base_url: &str, file_name: &str) -> Option<String> {
    if file_name.is_empty()
        || file_name.contains(['/', '\\'])
        || file_name.contains("..")
        || !file_name.ends_with(".woff2")
    {
        return None;
    }
    Some(format!(
        "{}/assets/fonts/{}",
        trim_base_url(base_url),
        file_name
    ))
}

fn trim_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_owned()
}
