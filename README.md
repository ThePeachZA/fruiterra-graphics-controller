# Fruiterra Graphics Controller

Windows broadcast control for Overwatch, with OBS-compatible HTML overlays.

> **Status: pre-release scaffold (milestone M0).** The workspace, toolchain and
> packaging pipeline are in place. The domain model, server, overlays and UI are
> built in milestones M1–M10 as described in `docs/implementation-plan.md`.
> Nothing in this repository is a usable broadcast tool yet.

## What it is

Fruiterra Graphics Controller is a Rust/Dioxus desktop application that manages
teams, talent, matches, maps and tournaments, and serves seven plain-HTML
overlays for OBS Browser Sources at **1920×1080**. It replaces the earlier
Electron/React "OW Graphics Control" while keeping the existing overlay
WebSocket protocol.

## Requirements

- Windows 10 or Windows 11, x64.
- Microsoft Edge WebView2 runtime. The installer bootstraps it when missing.
- OBS Studio with a Browser Source for each overlay.

## Install

Releases ship two artifacts:

| Artifact | Use |
|---|---|
| `fruiterra-gc-<version>-windows-x64-setup.exe` | Normal installation: Start Menu entry, uninstaller and WebView2 bootstrapper. |
| `fruiterra-gc-<version>-windows-x64-portable.zip` | Run from a folder; no install or registry writes. Requires WebView2 to be present. |

Releases are **unsigned**. Windows SmartScreen may warn on first run. Verify the
published `SHA256SUMS.txt` before running either artifact.

## Quick start

1. Launch the application.
2. Add teams, players, talent and assets.
3. Create a match and set teams, maps, first-to and sides.
4. Start the overlay server (default port **3001**).
5. Add each overlay URL to OBS as a Browser Source at 1920×1080 with a
   transparent background.

## Overlay URLs

```
http://127.0.0.1:3001/overlays/game-overlay.html
http://127.0.0.1:3001/overlays/map-select.html
http://127.0.0.1:3001/overlays/hero-bans.html
http://127.0.0.1:3001/overlays/map-pool.html
http://127.0.0.1:3001/overlays/team-roster-home.html
http://127.0.0.1:3001/overlays/team-roster-away.html
http://127.0.0.1:3001/overlays/row-casters.html
```

The server port can be changed in Settings. A third-party overlay may pass
`?port=<port>` when it connects from a different origin.

## Migrating from the Electron app

There is **no automatic import**. Teams, talent, matches and assets must be
re-entered. The new data directory is:

```
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\
```

## Timer

The timer is **control-only** in v1. There is no bundled timer overlay.
Third-party overlays still receive `matchTimer` ticks and actions.

## Network and security

- The server binds to localhost by default.
- "Allow LAN access" binds `0.0.0.0` so other machines can load overlays.
- CORS is `*` to match the original overlays.
- There is **no authentication** in v1. Use LAN mode only on a trusted network.

## Fonts and licences

- Overlay font: **Saira** (OFL 1.1), served locally at stable `/assets/fonts/`
  paths.
- UI font: **Inter** (OFL 1.1), bundled locally.
- No PenPot or Google Fonts requests at runtime. Apotek is not redistributed.
- Application licence: **MIT** (see `LICENSE`).
- Full dependency inventory: `THIRD_PARTY_NOTICES.md`.

## Build from source

Requirements: Rust stable (pinned by `rust-toolchain.toml`), the MSVC C++
build tools, and Dioxus CLI 0.7.10.

```powershell
cargo install dioxus-cli --version 0.7.10 --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Installer
dx bundle --release --desktop --package-types nsis --features bundle

# Portable app output (exe + assets folder)
dx build --release --desktop --features bundle
```

The portable build is the `dx` app output, not a plain `cargo build`: the UI
assets are hashed and copied into `assets\` beside the executable. Keep that
folder with `fruiterra-gc.exe`.

The workspace layout is described in `docs/architecture.md`; the milestone
order and exit criteria are in `docs/implementation-plan.md`.

## Troubleshooting

| Symptom | Fix |
|---|---|
| SmartScreen warning | Expected for unsigned builds. Verify `SHA256SUMS.txt`, then choose "More info → Run anyway". |
| "WebView2 not found" with the portable ZIP | Install the Evergreen WebView2 runtime, or use the installer. |
| Port already in use | Change the port in Settings or stop the process using it. |
| Windows Firewall prompt in LAN mode | Allow access on private networks only. |

## Known limitations

- No import from the old Electron application; data must be re-entered.
- The timer is control-only; there is no bundled timer overlay.
- No code signing in v1.
- Windows x64 only.
- No bracket overlay in v1.
- No undo/redo, presets, multiple scenes, HTTP/Stream Deck API, auto-update,
  match history, custom points or multi-stage playoffs.

## Licence

MIT. See `LICENSE`.
