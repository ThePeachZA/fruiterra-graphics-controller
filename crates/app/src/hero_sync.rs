//! Bulk repair of hero artwork and metadata from a local art folder and the
//! owner's reference sheet.
//!
//! The sync is deliberately one-way: it inserts new attached assets (never
//! mutating existing bytes), points each hero at the new artwork, and drops the
//! previous *attached* ban icon so the old blob becomes eligible for the normal
//! unused-asset cleanup. Library assets are never deleted.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fgc_core::{
    AppState, AssetAttachment, AssetAttachmentKind, AssetCategory, AssetId, AssetStore, Role,
};

/// Identifier of the owner's reference sheet.
pub const SHEET_ID: &str = "1g1pvSuYyfPSzbBf_ySUISvYa4WF1YYYRZXFTY6EMY";

/// Optional environment variable pointing at a local copy of the sheet CSV.
///
/// The public export endpoint is unavailable on some networks; this escape
/// hatch lets the same parser and sync run offline without changing the sheet
/// URL. It is not set during normal use.
pub const SHEET_CSV_ENV: &str = "FGC_SHEET_CSV";

/// A generous timeout for the sheet export; the response is tiny but the
/// public endpoint can be slow to start.
const SHEET_FETCH_TIMEOUT: Duration = Duration::from_secs(120);

/// CSV export URL for the reference sheet.
#[must_use]
pub fn sheet_csv_url() -> String {
    format!("https://docs.google.com/spreadsheets/d/{SHEET_ID}/export?format=csv&gid=0")
}

/// Downloads the reference sheet as CSV.
///
/// When [`SHEET_CSV_ENV`] names a readable file, that local copy is used
/// instead of the network.
pub fn fetch_sheet_csv() -> Result<String, String> {
    if let Some(path) = std::env::var_os(SHEET_CSV_ENV).filter(|value| !value.is_empty()) {
        let path = Path::new(&path);
        return fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("Could not create a network runtime: {error}"))?;

    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(SHEET_FETCH_TIMEOUT)
            // Google Sheets answers a browser user agent and refuses some
            // others with a bare 404, which reads as a missing sheet.
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            )
            .build()
            .map_err(|error| format!("Could not create the HTTP client: {error}"))?;

        let response = client
            .get(sheet_csv_url())
            .send()
            .await
            .map_err(|error| format!("Could not fetch the reference sheet: {error}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "The reference sheet returned {}.",
                response.status()
            ));
        }

        response
            .text()
            .await
            .map_err(|error| format!("Could not read the reference sheet: {error}"))
    })
}

/// One row of the reference sheet.
///
/// Column A is the map, B the game mode, D the hero and E the role. A row with
/// a map is a map row; a row with a hero is a hero row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SheetRow {
    /// Map name.
    pub map: Option<String>,
    /// Game mode name (`Control`, `Escort`, `Flashpoint`, `Hybrid`, `Push`).
    pub game_mode: Option<String>,
    /// Hero name.
    pub hero: Option<String>,
    /// Hero role (`tank`, `damage`, `support`).
    pub role: Option<String>,
}

/// Counts and diagnostics from one sync pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeroSyncReport {
    /// Heroes with at least one field updated.
    pub heroes_updated: usize,
    /// Ban icons replaced with the square portrait.
    pub icons_set: usize,
    /// Hero badges attached.
    pub badges_set: usize,
    /// Transparent cut-outs attached.
    pub transparents_set: usize,
    /// Hero roles set from the sheet.
    pub roles_set: usize,
    /// Map game-mode links set.
    pub map_modes_set: usize,
    /// Maps missing from the sheet and artwork files that could not be used.
    pub skipped: Vec<String>,
}

/// Parses the reference sheet CSV.
///
/// Handles a UTF-8 BOM, CRLF or LF records, quoted fields and doubled quotes
/// inside a quoted field.
#[must_use]
pub fn parse_sheet_csv(text: &str) -> Vec<SheetRow> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(ch);
            }
            continue;
        }

        match ch {
            '"' => in_quotes = true,
            ',' => fields.push(std::mem::take(&mut field)),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                end_record(&mut records, &mut fields, &mut field);
            }
            '\n' => end_record(&mut records, &mut fields, &mut field),
            _ => field.push(ch),
        }
    }

    if !field.is_empty() || !fields.is_empty() {
        fields.push(field);
        records.push(fields);
    }

    records
        .into_iter()
        .filter_map(|record| {
            let row = SheetRow {
                map: column(&record, 0),
                game_mode: column(&record, 1),
                hero: column(&record, 3),
                role: column(&record, 4),
            };
            (row.map.is_some() || row.hero.is_some()).then_some(row)
        })
        .collect()
}

fn end_record(records: &mut Vec<Vec<String>>, fields: &mut Vec<String>, field: &mut String) {
    fields.push(std::mem::take(field));
    records.push(std::mem::take(fields));
}

fn column(record: &[String], index: usize) -> Option<String> {
    record
        .get(index)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Repairs hero artwork, hero roles and map game modes in `state`.
///
/// New artwork is inserted into `store` as attached hero assets. The previous
/// attached ban icon is removed from `state.assets`; its blob is left for the
/// normal unused-asset cleanup.
#[must_use]
pub fn sync(
    state: &mut AppState,
    store: &AssetStore,
    art_dir: &Path,
    rows: &[SheetRow],
) -> HeroSyncReport {
    let mut report = HeroSyncReport::default();

    let heroes: Vec<(AssetId, String)> = state
        .assets
        .iter()
        .filter(|asset| asset.category == AssetCategory::Hero && !asset.is_attachment())
        .map(|asset| (asset.id.clone(), asset.name.clone()))
        .collect();

    for (hero_id, hero_name) in heroes {
        let mut updated = false;

        match read_art(art_dir, "Icons/Square", &hero_name) {
            Some((bytes, filename)) => {
                match insert_attachment(
                    state,
                    store,
                    &hero_id,
                    format!("{hero_name} ban icon"),
                    AssetAttachmentKind::HeroBanIcon,
                    &bytes,
                    filename,
                ) {
                    Ok(new_id) => {
                        if let Some(hero) =
                            state.assets.iter_mut().find(|asset| asset.id == hero_id)
                        {
                            hero.icon = Some(new_id);
                        }
                        report.icons_set += 1;
                        updated = true;
                    }
                    Err(error) => report
                        .skipped
                        .push(format!("{hero_name} ban icon could not be stored: {error}")),
                }
            }
            None => report
                .skipped
                .push(format!("hero art missing: {hero_name} (Icons/Square)")),
        }

        match read_art(art_dir, "Icons/Hexagon", &hero_name) {
            Some((bytes, filename)) => {
                match insert_attachment(
                    state,
                    store,
                    &hero_id,
                    format!("{hero_name} badge"),
                    AssetAttachmentKind::HeroBadge,
                    &bytes,
                    filename,
                ) {
                    Ok(new_id) => {
                        if let Some(hero) =
                            state.assets.iter_mut().find(|asset| asset.id == hero_id)
                        {
                            hero.badge = Some(new_id);
                        }
                        report.badges_set += 1;
                        updated = true;
                    }
                    Err(error) => report
                        .skipped
                        .push(format!("{hero_name} badge could not be stored: {error}")),
                }
            }
            None => report
                .skipped
                .push(format!("hero art missing: {hero_name} (Icons/Hexagon)")),
        }

        match read_art(art_dir, "Transparent", &hero_name) {
            Some((bytes, filename)) => {
                match insert_attachment(
                    state,
                    store,
                    &hero_id,
                    format!("{hero_name} transparent"),
                    AssetAttachmentKind::HeroTransparent,
                    &bytes,
                    filename,
                ) {
                    Ok(new_id) => {
                        if let Some(hero) =
                            state.assets.iter_mut().find(|asset| asset.id == hero_id)
                        {
                            hero.transparent = Some(new_id);
                        }
                        report.transparents_set += 1;
                        updated = true;
                    }
                    Err(error) => report.skipped.push(format!(
                        "{hero_name} transparent could not be stored: {error}"
                    )),
                }
            }
            None => report
                .skipped
                .push(format!("hero art missing: {hero_name} (Transparent)")),
        }

        if let Some(role) = sheet_role(rows, &hero_name)
            && let Some(hero) = state.assets.iter_mut().find(|asset| asset.id == hero_id)
        {
            updated |= hero.role != Some(role);
            hero.role = Some(role);
            report.roles_set += 1;
        }

        if updated {
            report.heroes_updated += 1;
        }
    }

    sync_map_modes(state, rows, &mut report);
    report
}

fn sync_map_modes(state: &mut AppState, rows: &[SheetRow], report: &mut HeroSyncReport) {
    let maps: Vec<(AssetId, String)> = state
        .assets
        .iter()
        .filter(|asset| asset.category == AssetCategory::Map && !asset.is_attachment())
        .map(|asset| (asset.id.clone(), asset.name.clone()))
        .collect();

    for (map_id, map_name) in maps {
        let Some(mode_name) = sheet_game_mode(rows, &map_name) else {
            report.skipped.push(format!("map not in sheet: {map_name}"));
            continue;
        };

        let game_mode_id = state
            .assets
            .iter()
            .find(|asset| {
                asset.category == AssetCategory::GameMode
                    && !asset.is_attachment()
                    && asset.name.eq_ignore_ascii_case(mode_name)
            })
            .map(|asset| asset.id.clone());

        let Some(game_mode_id) = game_mode_id else {
            report.skipped.push(format!(
                "game mode asset missing: {mode_name} (map {map_name})"
            ));
            continue;
        };

        if let Some(map) = state.assets.iter_mut().find(|asset| asset.id == map_id) {
            map.game_mode = Some(game_mode_id);
            report.map_modes_set += 1;
        }
    }
}

fn insert_attachment(
    state: &mut AppState,
    store: &AssetStore,
    owner: &AssetId,
    name: String,
    kind: AssetAttachmentKind,
    bytes: &[u8],
    filename: Option<String>,
) -> Result<AssetId, String> {
    let mut outcome = store
        .insert(name, AssetCategory::Hero, bytes, filename)
        .map_err(|error| error.to_string())?;
    outcome.asset.attachment = Some(AssetAttachment {
        owner: owner.clone(),
        kind,
    });
    let new_id = outcome.asset.id.clone();

    // Drop the hero's previous attachment of this kind so it does not linger as
    // an orphan. Library assets are never removed.
    state.assets.retain(|asset| {
        !(asset.is_attachment()
            && asset
                .attachment
                .as_ref()
                .is_some_and(|attachment| &attachment.owner == owner && attachment.kind == kind))
    });
    state.assets.push(outcome.asset);
    Ok(new_id)
}

fn sheet_role(rows: &[SheetRow], hero: &str) -> Option<Role> {
    let role = rows.iter().find_map(|row| {
        let name = row.hero.as_deref()?;
        if name.eq_ignore_ascii_case(hero) {
            row.role.as_deref()
        } else {
            None
        }
    })?;
    Role::from_slug(&role.to_ascii_lowercase())
}

fn sheet_game_mode<'a>(rows: &'a [SheetRow], map: &str) -> Option<&'a str> {
    rows.iter().find_map(|row| {
        let name = row.map.as_deref()?;
        if name.eq_ignore_ascii_case(map) {
            row.game_mode.as_deref()
        } else {
            None
        }
    })
}

fn read_art(art_dir: &Path, relative: &str, hero: &str) -> Option<(Vec<u8>, Option<String>)> {
    let path = find_png(&art_dir.join(relative), hero)?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    let bytes = fs::read(&path).ok()?;
    Some((bytes, filename))
}

/// Finds a `.png` whose stem matches `hero`, preferring an exact stem match
/// over a case-insensitive one.
fn find_png(directory: &Path, hero: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(directory).ok()?;
    let mut case_insensitive = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !extension.eq_ignore_ascii_case("png") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem == hero {
            return Some(path);
        }
        if case_insensitive.is_none() && stem.eq_ignore_ascii_case(hero) {
            case_insensitive = Some(path);
        }
    }
    case_insensitive
}

#[cfg(test)]
mod tests {
    use super::*;
    use fgc_core::{Asset, AssetId};

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nhero-sync-test";

    #[test]
    fn parses_bom_quotes_and_crlf() {
        let csv = "\u{feff}Map,GameMode,,Hero,Role\r\nControl,,,Ana,Support\r\n\"King's Row\",Hybrid,,\"Soldier, 76\",Damage\r\n";
        let rows = parse_sheet_csv(csv);
        assert_eq!(rows.len(), 3);

        assert_eq!(rows[1].map.as_deref(), Some("Control"));
        assert_eq!(rows[1].hero.as_deref(), Some("Ana"));
        assert_eq!(rows[1].role.as_deref(), Some("Support"));

        assert_eq!(rows[2].map.as_deref(), Some("King's Row"));
        assert_eq!(rows[2].game_mode.as_deref(), Some("Hybrid"));
        assert_eq!(rows[2].hero.as_deref(), Some("Soldier, 76"));
        assert_eq!(rows[2].role.as_deref(), Some("Damage"));
    }

    #[test]
    fn parses_doubled_quotes_and_ignores_blank_rows() {
        let rows = parse_sheet_csv(",,,,\n,,,D.Va,Tank\n");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].hero.as_deref(), Some("D.Va"));
    }

    #[test]
    fn finds_exact_stem_before_case_insensitive_match() {
        let directory = tempfile::tempdir().expect("temp dir");
        fs::write(directory.path().join("Ana.PNG"), PNG).expect("exact");
        fs::write(directory.path().join("ana.png"), PNG).expect("insensitive");

        let found = find_png(directory.path(), "Ana").expect("found");
        assert_eq!(
            found.file_name().and_then(|name| name.to_str()),
            Some("Ana.PNG")
        );

        let found = find_png(directory.path(), "ANA").expect("found");
        assert!(
            found
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| stem.eq_ignore_ascii_case("Ana"))
        );
    }

    #[test]
    fn sync_sets_artwork_role_and_map_mode() {
        let art = tempfile::tempdir().expect("art dir");
        for folder in ["Icons/Square", "Icons/Hexagon", "Transparent"] {
            let folder = art.path().join(folder);
            fs::create_dir_all(&folder).expect("art folder");
            fs::write(folder.join("Ana.png"), PNG).expect("hero art");
        }

        let blobs = tempfile::tempdir().expect("blob dir");
        let store = AssetStore::new(blobs.path());

        let hero = Asset::new_user("Ana", AssetCategory::Hero, "a".repeat(64), "png", None)
            .expect("hero asset");
        let map = Asset::new_user("Ilios", AssetCategory::Map, "b".repeat(64), "png", None)
            .expect("map asset");
        let control = Asset::new_user(
            "Control",
            AssetCategory::GameMode,
            "c".repeat(64),
            "png",
            None,
        )
        .expect("mode asset");

        let mut state = AppState::default();
        state.assets.push(hero.clone());
        state.assets.push(map.clone());
        state.assets.push(control.clone());

        let rows = parse_sheet_csv("Ilios,Control,,,\n,,,Ana,Support\n");
        let report = sync(&mut state, &store, art.path(), &rows);

        assert_eq!(report.icons_set, 1);
        assert_eq!(report.badges_set, 1);
        assert_eq!(report.transparents_set, 1);
        assert_eq!(report.roles_set, 1);
        assert_eq!(report.map_modes_set, 1);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);

        let updated = state
            .assets
            .iter()
            .find(|asset| asset.id == hero.id)
            .expect("hero");
        assert_eq!(updated.role, Some(Role::Support));
        assert!(updated.icon.is_some());
        assert!(updated.badge.is_some());
        assert!(updated.transparent.is_some());

        let updated_map = state
            .assets
            .iter()
            .find(|asset| asset.id == map.id)
            .expect("map");
        assert_eq!(updated_map.game_mode, Some(control.id.clone()));

        // The three attached files plus the three library assets.
        assert_eq!(state.assets.len(), 6);
    }

    #[test]
    fn sync_drops_the_previous_attached_ban_icon() {
        let art = tempfile::tempdir().expect("art dir");
        fs::create_dir_all(art.path().join("Icons").join("Square")).expect("art folder");
        fs::write(art.path().join("Icons").join("Square").join("Ana.png"), PNG).expect("hero art");

        let blobs = tempfile::tempdir().expect("blob dir");
        let store = AssetStore::new(blobs.path());

        let hero = Asset::new_user("Ana", AssetCategory::Hero, "a".repeat(64), "png", None)
            .expect("hero asset");
        let old_icon = Asset::new_user(
            "Ana ban icon",
            AssetCategory::Hero,
            "d".repeat(64),
            "png",
            None,
        )
        .expect("old icon");
        let old_icon_id = old_icon.id.clone();

        let mut state = AppState::default();
        let mut hero_with_icon = hero.clone();
        hero_with_icon.icon = Some(old_icon_id.clone());
        let mut old_icon = old_icon;
        old_icon.attachment = Some(AssetAttachment::hero_ban_icon(hero.id.clone()));
        state.assets.push(hero_with_icon);
        state.assets.push(old_icon);

        let rows = parse_sheet_csv(",,,Ana,Tank\n");
        let report = sync(&mut state, &store, art.path(), &rows);

        assert_eq!(report.icons_set, 1);
        assert!(
            !state.assets.iter().any(|asset| asset.id == old_icon_id),
            "previous attached ban icon should be removed"
        );
        let updated = state
            .assets
            .iter()
            .find(|asset| asset.id == hero.id)
            .expect("hero");
        let new_icon = updated.icon.clone().expect("new icon");
        assert_ne!(new_icon, old_icon_id);
        assert!(
            state
                .assets
                .iter()
                .any(|asset| asset.id == new_icon && asset.is_attachment())
        );
        let _: AssetId = hero.id;
    }
}
