use fgc_core::{Asset, AssetCategory, AssetId};

/// An in-memory lookup over bundled and user assets.
///
/// The server merges the embedded bundled manifest with `AppState::assets`
/// into one catalogue. Projection only reads this catalogue; it never touches
/// the filesystem.
#[derive(Clone, Debug, Default)]
pub struct AssetCatalog {
    assets: Vec<Asset>,
}

impl AssetCatalog {
    /// Builds a catalogue from any iterator of assets.
    #[must_use]
    pub fn new(assets: impl IntoIterator<Item = Asset>) -> Self {
        Self {
            assets: assets.into_iter().collect(),
        }
    }

    /// Returns every asset in insertion order.
    #[must_use]
    pub fn assets(&self) -> &[Asset] {
        &self.assets
    }

    /// Looks up an asset by identifier.
    #[must_use]
    pub fn get(&self, id: &AssetId) -> Option<&Asset> {
        self.assets.iter().find(|asset| &asset.id == id)
    }

    /// Finds the first asset in `category` whose name matches case-insensitively.
    ///
    /// Empty names never match, mirroring the original guard against blank
    /// map slots.
    #[must_use]
    pub fn find_by_name(&self, category: AssetCategory, name: &str) -> Option<&Asset> {
        if name.is_empty() {
            return None;
        }
        self.assets
            .iter()
            .find(|asset| asset.category == category && asset.name.eq_ignore_ascii_case(name))
    }

    /// Finds the first asset in `category` whose name matches exactly.
    #[must_use]
    pub fn find_exact(&self, category: AssetCategory, name: &str) -> Option<&Asset> {
        if name.is_empty() {
            return None;
        }
        self.assets
            .iter()
            .find(|asset| asset.category == category && asset.name == name)
    }

    /// Iterates assets in one category, in insertion order.
    pub fn in_category(&self, category: AssetCategory) -> impl Iterator<Item = &Asset> {
        self.assets
            .iter()
            .filter(move |asset| asset.category == category)
    }

    /// Number of assets in the catalogue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns true when the catalogue holds no assets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}
