# Build, packaging and release — Phase 6

Status: **approved**, with a bundler-reality correction recorded in `docs/decisions.md`. Implements Phase 6 Q1–Q5: NSIS installer + portable ZIP, unsigned release, tag-driven CI, version `1.0.0`, and an embedded WebView2 bootstrapper.

---

## 1. Release artifacts

Every tagged release publishes two Windows x64 artifacts plus checksums:

| Artifact | Produced by | Purpose |
|---|---|---|
| `fruiterra-gc-<version>-windows-x64-setup.exe` | `dx bundle` (`nsis` package type) | Normal install: Start Menu entry, uninstaller, embedded WebView2 bootstrapper |
| `fruiterra-gc-<version>-windows-x64-portable.zip` | ZIP of the `dx` release app output (`fruiterra-gc.exe` + `assets\`) + docs | USB/locked-down use; no install or registry writes |
| `SHA256SUMS.txt` | CI | Integrity check for both artifacts |

- Dioxus CLI 0.7.10 calls this package type `nsis` and emits an `.exe` installer; it is **not** a portable executable. The portable ZIP is built from the `dx` release app output, not from the installer.
- The installer is renamed to a predictable, lowercase artifact name; the internal product name stays **Fruiterra Graphics Controller**.
- The portable ZIP contains `fruiterra-gc.exe`, the generated `assets\` folder, `README.md`, `LICENSE` and `THIRD_PARTY_NOTICES.md`. The server's overlays and overlay assets are embedded via `rust-embed`, but the **UI** assets (CSS, and later fonts/icons) are Dioxus/manganis assets read from `<exe_dir>\assets\` at runtime. They are not embedded, so the folder must stay beside the executable.
- A plain `cargo build --release` is not sufficient for a portable package: manganis hashes and copies assets only when the build runs through `dx`. Use `dx build --release --desktop --features bundle`, or reuse the `app` folder from the `dx bundle` run.
- The portable build requires WebView2 to already be present; the README says so.
- No auto-updater, no background service, no registry configuration beyond what the installer writes.
- **MSI is not shipped in v1.** Dioxus 0.7.10 produces a WiX MSI, but that path has no WebView2 bootstrapper support. The NSIS installer is the only package that can honour the approved WebView2 decision. An MSI can be added later for managed deployment, with WebView2 documented as a prerequisite.

### Version

- First Rust release: **`1.0.0`**.
- Semantic versioning: patch for fixes, minor for backwards-compatible features, major for schema or protocol breaks.
- The version lives in the workspace `Cargo.toml`; the release workflow refuses a tag that does not match it.
- `schemaVersion` is independent of the app version and is bumped only when the persisted model changes.

### Signing

- **Unsigned in v1.** No certificate is stored in the repository or CI.
- The README and every release note state that Windows SmartScreen may warn on first run, and explain how to verify the SHA-256 checksum.
- The workflow contains a commented `signtool` step with the expected secret names (`WINDOWS_CERT_PFX_BASE64`, `WINDOWS_CERT_PASSWORD`, `WINDOWS_TIMESTAMP_URL`) so signing can be enabled later without restructuring.
- The installer is not signed, so it can also trigger SmartScreen.

---

## 2. `Dioxus.toml` and Cargo metadata

```toml
[application]
name = "Fruiterra Graphics Controller"
# Used only by `dx components add`; relative to the invocation directory.
asset_dir = "crates/app/assets"

[bundle]
identifier = "com.fruiterra.fgc"
publisher = "Fruiterra"
# Bundle paths are relative to the crate directory (crates/app).
icon = ["assets/icons/icon.ico"]

# Dioxus CLI 0.7.10 reads Windows bundle settings from [bundle.windows].
# The top-level [windows] table is parsed but not wired into the bundler.
[bundle.windows]
# Embed the small bootstrapper; download only if WebView2 is absent.
webview_install_mode = { EmbedBootstrapper = { silent = true } }

# The CLI derives the installer product name from the binary target
# (`fruiterra-gc` -> `FruiterraGc`). A custom template keeps the approved
# display name while the installed executable stays `fruiterra-gc.exe`.
[bundle.windows.nsis]
template = "installer/installer.nsi"
start_menu_folder = "Fruiterra Graphics Controller"
installer_icon = "assets/icons/icon.ico"
```

- `crates/app/installer/installer.nsi` is a copy of the CLI's built-in NSIS template with only the display name changed. It must be re-checked against `dioxus-cli`'s `NSIS_TEMPLATE` whenever the pinned CLI version changes.
- **Config table quirk:** in Dioxus CLI 0.7.10 the bundler reads `config.bundle.windows`, not the newer top-level `[windows]` table. Keys under `[windows]` are accepted but ignored, and `webview_install_mode` silently falls back to `OfflineInstaller` (a ~150 MB embedded runtime). Keep the settings under `[bundle.windows]`.
- The WebView2 mode is the externally tagged enum from `dioxus-cli`'s `bundle.rs`, so the TOML is `{ EmbedBootstrapper = { silent = true } }` (no `type =` key).
- NSIS `install_mode` is left at its default `CurrentUser`: per-user install under `%LOCALAPPDATA%\Programs\Fruiterra Graphics Controller`, no UAC prompt, uninstall entry in the current user's registry hive.
- `Dioxus.toml` lives at the workspace root. The CLI walks up from `crates/app` to find it, and `default-members = ["crates/app"]` selects the app package.
- The binary target is `fruiterra-gc`; the display name is set in the app and installer metadata.
- `#![cfg_attr(all(feature = "bundle", target_os = "windows"), windows_subsystem = "windows")]` hides the console in release builds. Logs still go to `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\fruiterra-gc.log`.
- `bundle` is an app-crate feature enabled by the release command, so `cargo run` keeps the console for development.
- The icon is a multi-resolution `.ico`; the PNG set is used by the UI and documentation.

Build commands:

```powershell
# Installer (NSIS, embeds the WebView2 bootstrapper)
dx bundle --release --desktop --package-types nsis --features bundle

# Portable app output (exe + hashed assets), without building an installer
dx build --release --desktop --features bundle

# Development build for UI work and app QA
dx build --desktop
# launch target\dx\fruiterra-gc\debug\windows\app\fruiterra-gc.exe
```

A plain `cargo build` binary is not runnable for UI work: `manganis` hashes,
processes and copies the asset bundle only when the build runs through `dx`. The
`cargo build` executable has no `assets\` folder beside it, so every `asset!` URL
returns 404: the UI renders unstyled and CSS-module dialogs (for example the
asset link dialogs) lay out in normal document flow instead of over the window.
Use `dx build --desktop` or `dx serve` for anything that inspects the app UI.

The installer output is discovered with `dx bundle --json-output` rather than a hard-coded path. The workflow parses `BundleOutput.bundles[]` and fails if the NSIS installer is missing. The portable payload is the `dx` app output directory `target\dx\fruiterra-gc\release\windows\app\` (`fruiterra-gc.exe` plus `assets\`); the same folder is available after `dx bundle`.

---

## 3. CI workflows

### 3.1 `ci.yml` — every push and pull request

| Step | Command / action |
|---|---|
| Checkout | `actions/checkout@v4` |
| Toolchain | `dtolnay/rust-toolchain@stable` with `rustfmt`, `clippy` |
| Cache | `Swatinem/rust-cache@v2` |
| Format | `cargo fmt --all --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` |
| Docs check | `cargo doc --workspace --no-deps` |
| Notices check (separate job) | Regenerate `THIRD_PARTY_NOTICES.md` with `cargo-about` 0.9.3 and fail if it differs |
| Bundle smoke (optional job) | `dx bundle --release` without publishing |

- Runner: `windows-latest` only, matching the Windows-only v1 decision.
- No Node, no Tailwind, no npm.
- The Dioxus CLI is installed at the pinned version: `cargo install dioxus-cli --version 0.7.10 --locked`.
- `RUSTFLAGS` are left at defaults; `sccache` is optional and must not be required for a successful build.

### 3.2 `release.yml` — tags and manual dispatch

**Triggers**

- Push of a tag matching `v*` → build, publish a GitHub release, upload artifacts.
- `workflow_dispatch` → build and upload workflow artifacts only; no release is created.

**Permissions**

```yaml
permissions:
  contents: write
```

**Steps**

1. Checkout the tagged commit.
2. Verify `v<version>` matches the workspace `Cargo.toml` version. Fail otherwise.
3. Install Rust stable and `dioxus-cli` 0.7.10.
4. Run `cargo fmt --check`, `cargo clippy`, `cargo test`.
5. Run `dx bundle --release --desktop --package-types nsis --features bundle`.
6. Locate the NSIS installer output via JSON mode.
7. Rename the installer to `fruiterra-gc-<version>-windows-x64-setup.exe`.
8. Build the portable ZIP from the `dx` app output (`target\dx\fruiterra-gc\release\windows\app\`: `fruiterra-gc.exe` + `assets\`) plus `README.md`, `LICENSE`, `THIRD_PARTY_NOTICES.md`.
9. Generate `SHA256SUMS.txt`.
10. Upload all three files as workflow artifacts.
11. On a tag only, create the GitHub release with generated notes plus the release-note template (§5).
12. On manual dispatch, stop after the artifact upload.

**Concurrency:** one release run per tag; cancel older runs for the same ref.

**Failure behaviour:** a failed format, lint, test or bundle step fails the job; no partial release is published.

### 3.3 Packaged smoke test

The app binary supports a hidden `--smoke-test` flag used only by CI:

1. Start with a temporary data directory.
2. Start the overlay server on an ephemeral port.
3. Request `/health` and assert `status == "ok"`.
4. Open a WebSocket, send `requestInit`, and assert an `init` message arrives.
5. Request one embedded overlay and one embedded asset; assert HTTP 200.
6. Exit 0.

This validates the packaged binary, embedded overlays, embedded assets and the server runtime without automating the webview. It is not shown in the UI or documented as a user feature.

---

## 4. README

`README.md` is the user-facing entry point. Required sections:

1. **What it is** — Windows broadcast control for Overwatch, with OBS-compatible HTML overlays.
2. **Requirements** — Windows 10/11 x64, WebView2 (installer bootstraps it), OBS Browser Source.
3. **Install** — installer or portable ZIP; SmartScreen note and checksum verification.
4. **Quick start** — start the server, add teams/assets, create a match, copy the overlay URL into OBS at 1920×1080.
5. **Overlay URLs** — the seven bundled files, the `?port=` parameter, and the default port 3001.
6. **Migrating from the Google Sheets workflow** - **no automatic import**: teams, talent, matches and assets must be re-entered. This is stated plainly, not buried.
7. **Timer** - control-only in v1; there is no bundled timer overlay. Third-party timer overlays still receive `matchTimer`.
8. **Network and security** - localhost by default; "Allow LAN access" binds `0.0.0.0`; CORS is `*`; there is no authentication in v1.
9. **Fonts and licences** - Archivo (overlay), Saira (retained fallback) and Inter (UI) are OFL 1.1; local only; no runtime font requests.
10. **Build from source** — Rust stable, `dioxus-cli` 0.7.10, `cargo test`, `dx bundle`.
11. **Troubleshooting** — SmartScreen, WebView2 missing, port already in use, firewall prompt on LAN mode.
12. **Known limitations** - no import from the earlier Google Sheets workflow, timer control-only, no code signing, Windows-only, bracket overlay post-v1.
13. **Licence** - MIT, stated in the font/licence section with a link to `LICENSE`; no separate top-level section.

The README must not claim features that are backlogged (undo/redo, presets, multiple scenes, HTTP control API, bracket overlay).

---

## 5. Release notes template

```markdown
# Fruiterra Graphics Controller <version>

Windows 10/11 x64. Unsigned build — SmartScreen may warn; verify SHA-256 below.

## What's new
- <user-facing changes>

## Fixes
- <fixes>

## Known issues
- No import from the earlier Google Sheets workflow; data must be re-entered.
- The timer is control-only; there is no bundled timer overlay.
- LAN mode has no authentication; use it only on a trusted network.
- Bracket overlay is not included in v1.

## Assets
- `fruiterra-gc-<version>-windows-x64-setup.exe`
- `fruiterra-gc-<version>-windows-x64-portable.zip`
- `SHA256SUMS.txt`
```

GitHub's generated notes are appended below this template, not used instead of it.

---

## 6. Licences and notices

| File | Contents |
|---|---|
| `LICENSE` | MIT licence, carried over from the original project |
| `THIRD_PARTY_NOTICES.md` | Archivo, Saira and Inter (OFL 1.1, with the full OFL text), vendored Dioxus components (upstream MIT/Apache-2.0 headers and pinned revision), and a generated inventory of Rust dependency licences |
| `crates/server/assets/fonts/OFL.txt` | OFL text shipped beside the overlay fonts |
| `crates/app/assets/fonts/OFL.txt` | OFL text shipped beside the UI font |

- The dependency inventory is generated with `cargo-about` and committed. CI regenerates it and fails if it differs, so a new dependency cannot ship without a notice.
- Generation is driven by `about.toml` (accepted licence identifiers) and `about.hbs` (the notices template). Regenerate with:

  ```powershell
  cargo about generate --workspace --target x86_64-pc-windows-msvc --all-features about.hbs -o THIRD_PARTY_NOTICES.md
  ```

  `cargo-about` must be installed with its CLI feature:
  `cargo install cargo-about --version 0.9.3 --locked --features cli`.
- The workflow uses the same command for the tag check, so the committed file must be regenerated whenever `Cargo.lock` changes.
- Apotek is **not** shipped. The only `Apotek` reference is the CSS fallback token.
- Vendored component files keep their original copyright headers.

---

## 7. Release checklist

| # | Check |
|---|---|
| 1 | `cargo fmt --all --check` passes. |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| 3 | `cargo test --workspace` passes. |
| 4 | `dx bundle` produces the NSIS installer `.exe`. |
| 5 | Packaged `--smoke-test` passes. |
| 6 | Installer runs, launches, and uninstalls cleanly on a clean Windows VM. |
| 7 | Portable ZIP runs from a folder with no install. |
| 8 | `docs/overlay-qa.md` is completed for all seven overlays at 1920×1080. |
| 9 | `THIRD_PARTY_NOTICES.md` is current. |
| 10 | `README.md` and release notes match the shipped feature set. |
| 11 | `SHA256SUMS.txt` matches both artifacts. |
| 12 | Tag version equals the workspace version. |

---

## 8. Risks

| # | Risk | Mitigation |
|---|---|---|
| B1 | Unsigned binaries trigger SmartScreen | Document the warning, publish checksums, keep the signing step ready |
| B2 | WebView2 bootstrapper needs network on old images | Embed the bootstrapper; document the offline limitation; full offline installer remains an option |
| B3 | `dioxus-cli` version drift | Pin `0.7.10` in CI and document it in the README |
| B4 | NSIS tooling missing or changed on the runner | The CLI downloads and hash-verifies NSIS; the bundle step is a required CI check and fails loudly |
| B5 | `dx bundle` output paths change | Discover outputs via `--json-output`; fail if the installer is missing |
| B6 | Portable ZIP misses a sidecar resource | The server's overlays are embedded via `rust-embed`, but the UI's manganis `assets\` folder is not. Zip the whole `dx` app output (exe + `assets\`), and re-check after any new runtime file is added |
| B7 | A new dependency ships without a licence notice | `cargo-about` check in CI |
| B8 | Release notes claim backlog features | Release checklist item 10 and a fixed template |

---

## 9. Decisions locked

1. Artifact names: `fruiterra-gc-<version>-windows-x64-setup.exe` and `...-portable.zip`, plus `SHA256SUMS.txt`.
2. Installer product name **Fruiterra Graphics Controller**, publisher **Fruiterra**, identifier `com.fruiterra.fgc`.
3. Release notes: fixed template plus GitHub generated notes.
4. `cargo-about` as the dependency-licence inventory tool, checked in CI.
5. Hidden `--smoke-test` flag for packaged verification.
6. Unsigned release with a commented signing step and documented SmartScreen warning.
7. First version `1.0.0`; tag must equal the workspace version.
8. Windows x64 only; NSIS installer and portable ZIP.
9. Embedded WebView2 bootstrapper in the NSIS installer; portable ZIP requires WebView2.
10. CI: fmt/clippy/test on every push and PR; tag `v*` publishes; manual dispatch builds artifacts only.
11. No MSI in v1: the WiX path cannot embed the WebView2 bootstrapper. Revisit only with an explicit decision.

---

## 10. Wiki maintenance

The end-user guide lives in [`docs/wiki/`](wiki/Home.md) so it is versioned with
the code and reviewed in the same pull requests as user-facing changes. When a
UI label, workflow or limitation changes, update the matching page in that
folder.

Publish the pages to the GitHub wiki with:

```powershell
pwsh ./scripts/publish-wiki.ps1
```

The script clones `https://github.com/<owner>/<repo>.wiki.git`, copies every
`docs/wiki/*.md` page, commits and pushes. Pass `-Repo owner/name` to target a
different repository.

> [!IMPORTANT]
> The GitHub wiki must be enabled first. Private repositories on the GitHub Free
> plan cannot enable wikis, so the wiki clone fails with `Repository not found`
> until the repository is public or the plan supports private wikis. Until then,
> `docs/wiki/Home.md` is the published end-user guide.
