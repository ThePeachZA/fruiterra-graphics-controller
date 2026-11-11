# Fruiterra Graphics Controller <version>

Windows 10/11 x64. Unsigned build — SmartScreen may warn; verify SHA-256 below.

## What's new
- First Rust/Dioxus release. The Electron application is replaced by a native
  Windows desktop app with the same OBS-compatible overlays.
- Bundled Saira overlay fonts and Inter UI fonts; no runtime font downloads.
- Tournament boards: single and double elimination, round robin groups and
  playoffs, with editable slots and automatic advancement.
- Local, versioned JSON storage with a backup file and repair on load.

## Fixes
- Overlay text is escaped before rendering, so operator-entered names cannot
  inject markup.
- Asset URLs are content-addressed, so replaced images update immediately.

## Known issues
- No import from the old Electron app; data must be re-entered.
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
