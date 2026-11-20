# Fruiterra Graphics Controller <version>

Windows 10/11 x64. Unsigned build — SmartScreen may warn; verify SHA-256 below.

## What's new
- First Rust/Dioxus release. The earlier Google Sheets-based workflow is
  replaced by a native Windows desktop app with the same OBS-compatible
  overlays.
- Bundled Archivo overlay fonts and Inter UI fonts; no runtime font downloads.
- Match Controller with combined **Maps & Bans**, match scheduling, a saved
  match library with portable export/import, and an always-on timer widget on
  every page with icon controls and expandable settings.
- Tournament boards: single and double elimination, round-robin groups,
  multi-stage group-to-playoff events and **Push to match**. Participants are
  arranged in the generation preview; live results and match details stay
  editable, with automatic advancement.
- Talent roles are chosen per match assignment as Caster, Host, Analyst,
  Interviewer or Other.
- Asset library improvements: multi-file import, quick upload from image
  pickers, automatic team-colour sampling from logos, a hero **Properties…**
  dialog with attached ban-icon upload, map **Game mode** properties, and
  faster lazy-loaded thumbnails.
- Local, versioned JSON storage with a backup file and repair on load.

## Fixes
- Overlay text is escaped before rendering, so operator-entered names cannot
  inject markup.
- Asset URLs are content-addressed, so replaced images update immediately.
- Overlay Browser Source URLs are hidden by default on the Overlay page, with a
  show/hide toggle. The overlay server can start automatically when enabled in
  Settings.
- Project bundle export verifies every referenced image before writing anything,
  so a corrupt asset cannot leave a partial bundle. Bundles use
  `blobs/<prefix>/<hash>.<ext>` and contain no bundled art.

## Known issues
- No import from the earlier Google Sheets workflow; data must be re-entered.
- The timer is control-only; there is no bundled timer overlay.
- LAN mode has no authentication; use it only on a trusted network.
- Bracket overlay is not included in v1.
- The build is unsigned, so Windows SmartScreen may warn on first run.

## Assets
- `fruiterra-gc-<version>-windows-x64-setup.exe`
- `fruiterra-gc-<version>-windows-x64-portable.zip`
- `SHA256SUMS.txt`

Verify the download with:

```powershell
Get-FileHash .\fruiterra-gc-<version>-windows-x64-setup.exe -Algorithm SHA256
```
