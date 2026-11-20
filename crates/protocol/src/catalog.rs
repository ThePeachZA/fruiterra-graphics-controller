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

    /// Returns the linked game-mode name for a map name, or an empty string.
    ///
    /// Map slots keep a snapshot of this value so the overlay does not need to
    /// resolve assets while rendering. Editing a link must refresh that
    /// snapshot; this lookup is the shared source of truth.
    #[must_use]
    pub fn game_mode_for_map_name(&self, name: &str) -> String {
        self.find_by_name(AssetCategory::Map, name)
            .and_then(|asset| asset.game_mode.as_ref())
            .and_then(|id| self.get(id))
            .map_or_else(String::new, |asset| asset.name.clone())
    }

    /// Returns true when `id` is linked as another asset's icon.
    ///
    /// Linked icons are hidden from hero pickers and ban lists so an icon
    /// never appears as a selectable hero.
    #[must_use]
    pub fn is_linked_icon(&self, id: &AssetId) -> bool {
        self.assets
            .iter()
            .any(|asset| asset.icon.as_ref() == Some(id))
    }

    /// Returns true when `asset` can be offered as a selectable hero.
    ///
    /// Attached ban icons are hidden, as are legacy icons linked from another
    /// hero. Neither should appear as a second hero with the same artwork.
    #[must_use]
    pub fn is_selectable_hero(&self, asset: &Asset) -> bool {
        asset.category == AssetCategory::Hero
            && !asset.is_attachment()
            && !self.is_linked_icon(&asset.id)
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
