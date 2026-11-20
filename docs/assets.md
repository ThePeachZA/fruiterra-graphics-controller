# Assets, fonts and overlays — Phase 5

Status: **approved**. Implements Q1 (minimal-diff overlay port), Q2 (five static WOFF2 files — Archivo active, Saira retained for comparison), Q3 (new hashed asset store), Q4 (per-file overlay override), Q5 (hash everything except fonts), Q6 (metadata in `app.json` + GC).

This supersedes the Phase 2 asset path scheme: `Asset.relative_path` and `Asset.source` are replaced by `hash`, `ext` and `origin`, and served URLs become `/assets/<sha256>.<ext>`. Wire payload shapes are unchanged apart from the URL strings.

---

## 1. Content-addressed asset store

### 1.1 Layout

```
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\
├─ data\
│  ├─ app.json                 # domain state + asset metadata
│  └─ app.json.bak
├─ assets\
│  └─ blobs\
│     ├─ 3f\
│     │  └─ 3fa1c9…e2.png
│     └─ 9b\
│        └─ 9b77d0…41.webp
├─ overlays\                   # optional per-file overrides
│  └─ game-overlay.html
└─ logs\
```

- Blob path: `assets/blobs/<first two hex>/<sha256>.<ext>`.
- The hash is of the **file bytes**, not the filename. Identical files are stored once.
- Bundled assets are embedded in the binary with the same hash-addressed identity; they are never copied to disk.
- Fonts are the one exception: stable paths under `/assets/fonts/` (§4).

### 1.2 Metadata

`Asset` changes from the Phase 2 sketch: `relative_path` and `source` are replaced by `hash`, `ext` and `origin`.

```rust
pub struct Asset {
    pub id: AssetId,              // logical identity, referenced by teams/players/etc.
    pub name: String,             // display name
    pub category: AssetCategory,  // Logo | Portrait | Map | Hero | GameMode | Role | Side
    pub hash: String,             // 64 lowercase hex chars
    pub ext: String,              // png | jpg | jpeg | webp | gif | svg
    pub origin: AssetOrigin,      // Bundled | User
    pub original_filename: Option<String>, // for "from URL" and diagnostics
    pub added_at: Option<String>,
    // category-specific links (hero art, map mode)
    pub icon: Option<AssetId>,
    pub portrait: Option<AssetId>,
    pub game_mode: Option<AssetId>,
    // hidden media owned by another asset
    pub attachment: Option<AssetAttachment>,
}
```

- Metadata lives in `app.json` under `assets`; it is part of `schemaVersion`.
- Two logical assets may share a hash (same bytes, different names). Deleting one does not delete the blob while the other references it.
- Bundled `AssetId`s are deterministic: `b:<sha256>`. User assets get UUID v4.
- Display names for bundled assets come from the checked-in `bundled/manifest.json`; user asset names come from the filename or the user.

### 1.3 Bundled assets and `build.rs`

```
crates/server/assets/bundled/
├─ manifest.json          # [{ name, category, file, icon?, portrait?, gameMode? }]
├─ logos/…
├─ portraits/…
├─ maps/…
├─ heroes/icons/…
├─ heroes/portraits/…
├─ game-modes/…
├─ roles/…
└─ sides/…
```

`build.rs`:

1. Reads `bundled/manifest.json`.
2. Hashes every referenced file with SHA-256.
3. Emits `$OUT_DIR/bundled_assets.rs` containing `&[(category, name, hash, ext, id, links…)]`.
4. Fails the build if a referenced file is missing or has an unsupported extension.

The build is deterministic: same files → same hashes → same IDs. `rust-embed` embeds the folder for serving.

### 1.4 URL scheme and caching

| Path | Source | Cache |
|---|---|---|
| `/assets/<hash>.<ext>` | embedded blob if known, else `assets/blobs/…` | `public, max-age=31536000, immutable` |
| `/assets/fonts/<name>.woff2` | embedded | `public, max-age=31536000, immutable` |
| `/overlays/<file>.html` | override file if present, else embedded | `no-cache` |
| `/health`, `/state` | computed | `no-store` |

- URLs in the wire protocol are absolute: `http://<host>:<port>/assets/<hash>.<ext>`.
- Because the hash is content identity, a changed image gets a new URL. OBS never shows a stale logo after a replace — this fixes C3 (stale caches) structurally.
- Validation before any lookup: hash is exactly 64 lowercase hex characters; extension is on the allow-list. No filesystem path is ever built from user input beyond that.
- Range requests are supported for blobs and fonts. Asset `ETag`s are the quoted hash and extension (`"<hash>.<ext>"`); font `ETag`s are the quoted file name (`"<name>.woff2"`).

### 1.5 Import

| Path | Behaviour |
|---|---|
| File picker (`rfd`) | Read bytes → validate type/size → SHA-256 → reuse existing blob if present → create metadata → select |
| From URL | `reqwest` GET with a 10 s timeout, 20 MB cap, `http`/`https` only; then the same hash path |
| Replace an image | Only the *reference* changes. The previous asset stays in the library and its blob stays valid. |

**Deliberate fix:** the original moves the replaced asset's file to `_temp` while leaving its metadata in the library, producing broken thumbnails and a growing temp folder. v1 never mutates the old asset on replace. There is no `_temp` directory; unreferenced blobs are handled by GC.

Allowed types: `png`, `jpg`/`jpeg`, `webp`, `gif`, `svg`. Max 20 MB per file. SVG is sanitised to reject `<script>` and external references before storing.

Bundled assets are read-only in the library: they cannot be renamed or deleted because they live inside the binary. Imported assets are read-only once stored as well; “replace” is expressed by pointing the team, player or setting at a different asset, never by mutating bytes in place.

**Properties.** Map cards expose **Game mode** inline. Hero cards expose
**Properties…**, which opens a dialog for that one image:

- **Hero → Ban icon** is uploaded directly from the hero's properties dialog.
  The uploaded image is stored as a Hero-category asset with an
  `Asset.attachment` of kind `heroBanIcon` owned by that hero. It is hidden from
  the library, hero pickers and ban lists, so it cannot be reused for another
  hero. Replacing or removing the ban icon deletes the old attached asset;
  deleting the hero deletes all of its attachments. Without one, the ban strip
  falls back to the hero's own art (or its `portrait` link).
- **Map → Game mode** points a Map asset at a bundled or imported game-mode
  image. Unlinked maps group under **Other** on the map-pool overlay. The same
  field derives **Game mode** on map slots that use the map; a mode typed by
  hand is kept until the slot's map name changes.

The attachment marker is additive (`#[serde(default)]`), so older state files
load unchanged. `AssetAttachmentKind` leaves room for future attached media
such as animations or videos without adding another library category.

### 1.6 Deletion and garbage collection

- Deleting an asset removes its metadata. If no other asset shares its hash and it is not bundled, the blob becomes unreferenced.
- GC runs **after a successful save** (never mid-edit) and on startup. Startup GC only runs when the state file loaded successfully, so a corrupt or missing `app.json` cannot cause every blob to be treated as unreferenced.
- The keep set is the union of hashes referenced by the saved snapshot and the current in-memory state:
  1. Collect every `Asset.hash` in the saved state.
  2. Collect every `Asset.hash` in the in-memory state.
  3. Delete user blobs whose hash is in neither set.
  4. Never delete embedded blobs.
- Settings shows “Unused asset data: N files, X MB” with a **Clean up now** button (replaces the original “temp assets” counter, which was a symptom of the broken replace flow).
- A blob is never deleted while either the saved or the in-memory state references it, even if a save has not completed yet.

### 1.7 Export and import

- Export asks for a parent folder and creates a fresh timestamped subfolder, `fruiterra-project-<unix>/`, so a previous export can never leave stale blobs behind. The bundle contains:
  - `app.json` — the full domain state, including asset metadata;
  - `export.json` — a small manifest with `schemaVersion` and the app version;
  - `blobs/` — only the user blobs referenced by that state, at the normal `<prefix>/<hash>.<ext>` paths.
- Bundled assets are not exported; they come from the binary.
- Import validates `export.json` and `app.json`, then reads, type-checks, sanitises and hash-verifies every referenced user blob before writing anything locally. A hash mismatch, missing blob or unsupported schema aborts the import with no partial store.
- Import replaces the current project (teams, talent, tournaments, settings and asset library). The UI confirms before doing so.

---

## 2. Overlay rebuild (minimal diff)

### 2.1 Scope

For each of the seven files (`game-overlay`, `map-select`, `hero-bans`, `map-pool`, `team-roster-home`, `team-roster-away`, `row-casters`):

1. Replace the 2–5 `@font-face` blocks with the five neutral display faces (§4).
2. Replace `font-family: 'Apotek…'` with the corresponding `'FGC Display…'` alias.
3. Change nothing else: markup, IDs, class names, CSS rules, animations, JS and WebSocket handling stay byte-identical wherever possible.

### 2.2 What stays exactly the same

- `?port=` query parameter and `ws://localhost:<port>` connection.
- Reconnect loop (2 s).
- `init` handling, including `d.data || data`.
- `show` handling (`display: block/none`).
- All DOM structure and class names used by CSS.
- All animation timings and selectors.
- File names and paths under `/overlays/`.

### 2.3 What changes

| Change | Reason |
|---|---|
| PenPot font URLs → local `/assets/fonts/` | C1, no runtime network dependency |
| `Apotek*` family names → `FGC Display*` | Neutral alias; a font swap is a five-file change |
| No other changes | Lowest risk for OBS |

If an overlay is later found to depend on a protocol quirk not covered by `docs/protocol.md`, the fix goes into the protocol tests first, then the overlay.

Post-v1 visual polish may change spacing and overflow handling, but not the
overlay's visual identity, DOM contracts or protocol. The approved scope and
guardrails are in `docs/backlog.md` §C1.

---

## 3. Per-file overlay overrides

- Directory: `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays\`.
- On request for `/overlays/<file>.html`:
  1. Reject names containing `/`, `\`, `..`, or not ending in `.html`.
  2. If `<override-dir>/<file>` exists and is a regular file, serve it.
  3. Otherwise serve the embedded file.
- A missing, empty or unreadable override falls back to the embedded file; a readable but malformed file is served as-is and affects only that file.
- Overrides are plain HTML: users can tweak colours or layout for OBS without rebuilding.
- A “Open overlays folder” button in Settings creates the directory and reveals it.
- No override is provided for assets or fonts in v1.

---

## 4. Fonts

### 4.1 Files

```
crates/server/assets/fonts/
├─ archivo-300.woff2          # active overlay font (R1 trial)
├─ archivo-400.woff2
├─ archivo-500.woff2
├─ archivo-600.woff2
├─ archivo-900.woff2
├─ OFL-Archivo.txt
├─ saira-300.woff2            # fallback, retained for the A/B comparison
├─ saira-400.woff2
├─ saira-500.woff2
├─ saira-600.woff2
├─ saira-900.woff2
├─ OFL.txt                    # Saira licence
└─ inter/                     # UI only, bundled by the app crate
   ├─ inter-400.woff2
   ├─ inter-500.woff2
   ├─ inter-600.woff2
   └─ OFL.txt
```

### 4.2 Neutral family aliases

Each overlay declares the same five faces:

```css
@font-face { font-family: 'FGC Display Light';    font-weight: 300; font-style: normal; font-display: swap;
             src: url('/assets/fonts/archivo-300.woff2') format('woff2'); }
@font-face { font-family: 'FGC Display';          font-weight: 400; font-style: normal; font-display: swap;
             src: url('/assets/fonts/archivo-400.woff2') format('woff2'); }
@font-face { font-family: 'FGC Display Medium';   font-weight: 500; font-style: normal; font-display: swap;
             src: url('/assets/fonts/archivo-500.woff2') format('woff2'); }
@font-face { font-family: 'FGC Display SemiBold'; font-weight: 600; font-style: normal; font-display: swap;
             src: url('/assets/fonts/archivo-600.woff2') format('woff2'); }
@font-face { font-family: 'FGC Display Black';    font-weight: 900; font-style: normal; font-display: swap;
             src: url('/assets/fonts/archivo-900.woff2') format('woff2'); }
```

Mapping used when editing the CSS:

| Original | Alias |
|---|---|
| `'Apotek Light'` | `'FGC Display Light'` |
| `'Apotek'` | `'FGC Display'` |
| `'Apotek Medium'` | `'FGC Display Medium'` |
| `'Apotek Semibold'` | `'FGC Display SemiBold'` |
| `'Apotek Black'` | `'FGC Display Black'` |

- Fallback stack: `'FGC Display', 'Apotek', sans-serif`.
- Review checkpoint: the R1 pass rejected Saira as too unlike Apotek. All seven
  overlays now load Archivo; Saira stays beside it for an A/B comparison.
  Switching either way means replacing the five WOFF2 files and their `src:`
  URLs, plus the `FONT_PREFIX` constant in `crates/server/tests/overlays.rs`.
  No layout CSS changes.
- No subsetting: accented Latin renders from Archivo; scripts Archivo does not
  cover (CJK, Arabic, etc.) fall back to the system sans. This is verified with
  the Cyrillic and CJK fixture names.
- The overlays also apply a shared `fitText()` helper: long names shrink to a
  floor and then ellipsize on one line. Team names are capped at 32 characters
  and player/talent names at 24 in the app.

### 4.3 Licensing

- Archivo, Saira and Inter are OFL 1.1. `OFL-Archivo.txt` and `OFL.txt` ship
  beside the overlay fonts, `inter/OFL.txt` beside the UI font, and all three
  are listed in `THIRD_PARTY_NOTICES.md`.
- The original Apotek OTFs are **removed from the repository**; they are not redistributable.
- The CSS fallback name `'Apotek'` remains only as a font-family fallback token, not as a shipped font.

---

## 5. QA checklist (manual, per overlay)

Run at 1920×1080 in OBS with a loaded match:

| # | Check |
|---|---|
| 1 | Overlay renders on first load from `init` (no blank frame until the next change) |
| 2 | No network requests to `design.penpot.app` or Google Fonts |
| 3 | Text renders in the active overlay font (Archivo) at the correct weight |
| 4 | Long team names, tags and player names do not overflow |
| 5 | Accented names render correctly; a CJK name falls back cleanly |
| 6 | Logos, portraits and map images load from `/assets/<hash>` |
| 7 | Replacing an image updates the overlay immediately (new hash, no stale cache) |
| 8 | Server restart → overlay reconnects within ~2 s and repaints |
| 9 | `show` hides and shows the overlay (test page) |
| 10 | `map-pool` groups and ordering match the original |
| 11 | `game-overlay` map number, first-to and side indicators are correct |
| 12 | `hero-bans` uses the correct current map and first-ban order |
| 13 | `team-roster-home`/`away` show the correct side |
| 14 | `row-casters` order and social icons are correct |
| 15 | A third-party timer overlay receives `matchTimer` ticks and actions |

The checklist lives in `docs/overlay-qa.md` and is completed for every release. This section is the design source for that document.

---

## 6. Acceptance criteria

| ID | Requirement |
|---|---|
| AS1 | Assets are stored once per unique content; duplicate imports do not duplicate bytes. |
| AS2 | `/assets/<hash>.<ext>` serves immutable, correctly typed content; invalid hashes/extensions are rejected. |
| AS3 | Bundled assets are embedded and hash-stable across builds. |
| AS4 | Replacing a reference never corrupts the previous asset or its metadata. |
| AS5 | GC removes only unreferenced user blobs and never removes embedded blobs. |
| AS6 | Export/import round-trips a project with relative blobs on another machine. |
| AS7 | All seven overlays load with zero external network requests. |
| AS8 | A per-file override changes one overlay and leaves the other six untouched. |
| AS9 | Font swap (Archivo ↔ Saira, or Barlow) requires only replacing five WOFF2 files, five `src:` URLs and the test prefix. |
| AS10 | The QA checklist passes for all seven overlays at 1920×1080. |
