# Decisions log

## Phase 0 — Audit facts (locked)

- Source audited: `ThePeachZA/ow-graphics-control@master` (MIT, 8 commits, no releases).
- 7 UI destinations; 7 overlay HTML files (no timer overlay).
- WebSocket: keyed `init` map + full per-section snapshots; `show` supported by overlays but unused by the app.
- HTTP: Express + `ws`; `/overlays`, `/assets`, `/user-assets`, `/health`; CORS `*`; all-interface bind; ports 3000/3001 inconsistent.
- Persistence: single unversioned `app.json`, 500 ms debounce, absolute asset paths.
- Tournament: WIP, not overlay-connected, bracket propagation bugs.

## Phase 1 — Scope and parity

### Q1 — Parity vs improvements
**Decision: Option 1 — parity-first + correctness fixes.**
- Preserve every user-facing workflow and the existing overlay protocol.
- Fix defects that currently make features fail or risk data loss.
- Defer new capabilities to a post-v1 backlog (option 3), maintained in `docs/backlog.md`.
- Reduce external runtime dependencies where possible; specifically, serve fonts locally instead of PenPot/Google Fonts.

**Consequences**
- Existing overlay HTML must remain drop-in compatible (same message types and payload shapes).
- No protocol redesign in v1; a versioned protocol is a backlog item.
- No new features in v1 unless they are required to make an existing feature work (e.g. a timer overlay, if the timer is considered shipped).

### Q2 — Tournament View
**Decision: Option 2 — complete it properly in v1.**
- Fix single- and double-elimination generation, losers-bracket routing, winner/loser propagation, grand-final reset, and automatic champion detection.
- Make reset semantics recursive (clearing a result also clears downstream results).
- Complete round-robin groups, standings, tiebreakers and group-to-playoff advancement.
- Persist and restore full bracket state; remove reliance on the unused `rounds` field.
- Add unit tests for bracket generation/progression.
- Scope note: this covers the **control view**. A tournament/bracket **overlay** is a separate feature unless explicitly pulled into v1.

### Fonts / external dependencies
**Finding:** Apotek is a commercial typeface by Kerns & Cairns (Adobe Fonts / Type Network). The OTFs in the repo cannot be redistributed or self-hosted without an appropriate licence.
**Decision: Saira confirmed** (OFL 1.1, Omnibus-Type) — 9 weights, 7 widths, variable, 885 glyphs, free for commercial use, self-hosting and embedding. *(Superseded by the R1 Archivo trial at the end of this file.)*
- Map Light/Regular/Medium/SemiBold/Black to Saira 300/400/500/600/900.
- Bundle WOFF2 files + `OFL.txt`; serve from `/assets/fonts/`.
- Bundle Inter (OFL) locally for the control UI; remove the Google Fonts import.
- Use neutral `FGC Display*` aliases with a fallback stack (`'FGC Display', 'Apotek', sans-serif`) and run a metrics/visual QA pass on all overlays.
- **Review checkpoint:** once the rebuilt overlay HTML is rendering, compare Saira against **Archivo** and **Barlow** in situ. If either fits better, it is a five-file swap; no architectural change.

### Q3 — New features wanted
**Decision: all five are backlog-only; none enter v1.**
The requested ideas are recorded in `docs/backlog.md` §C with enough detail to schedule later:
1. Undo / redo.
2. Presets / show templates.
3. Multiple OBS scenes / overlay sets.
4. Tournament / bracket overlay.
5. Stream Deck / HTTP control API.
Rationale: they all add new surface area (UI, schema or protocol) and would delay the parity milestone. v1 remains parity + correctness + completed Tournament View.

**Consequences**
- The earlier v1 candidates (show/hide toggle, all overlay URLs, health indicator, hotkeys, timer overlay) are **not committed** either. They stay uncommitted until v1 is running; each is a small, isolated addition if pulled forward later.
- C13 (timer overlay) therefore resolves as **control-only in v1**: the timer broadcasts `matchTimer` exactly as today, and is documented as having no overlay consumer. Shipping a timer overlay is post-v1.

### Q4 — Platform support
**Decision: Windows-only for v1.**
- Windows 10/11, WebView2 runtime (preinstalled on current Windows; bootstrapped by the installer if absent).
- Single bundle target (`dx bundle --platform windows`), single CI runner (`windows-latest`).
- The overlay server and overlay HTML remain portable by construction; macOS/Linux are a post-v1 packaging/QA exercise, not a rewrite.
- No platform-conditional code paths in v1 beyond OS app-data directory resolution.

## Phase 1 — Decisions locked

| # | Decision |
|---|---|
| 1 | **Parity-first + correctness fixes.** Every existing workflow and the overlay protocol are preserved; no protocol redesign. |
| 2 | **Tournament View completed properly in v1** (control view only), meeting TR1–TR10. Bracket overlay is post-v1. |
| 3 | **Fonts:** Saira (OFL 1.1) replaces Apotek at weights 300/400/500/600/900; Inter bundled for the UI; all fonts served locally from `/assets/fonts/`; no PenPot/Google Fonts at runtime. Review Archivo/Barlow once overlays render. |
| 4 | **No new features in v1.** Undo/redo, presets, multiple scenes, bracket overlay and HTTP/Stream Deck API are backlogged. Timer overlay is post-v1; timer is control-only in v1. |
| 5 | **Windows-only v1.** WebView2, single bundle target, single CI runner. |
| 6 | **Licence:** MIT, carried over from the original project. |

**Out of scope for v1:** new features, protocol v2, macOS/Linux, auto-update/code signing, bracket overlay, drag/drop reseeding and forfeits. Multi-stage playoffs were pulled forward into v1 (see the final pre-release decisions).

## Phase 2 — Data model and protocol

### Q1 — Model fidelity
**Decision: clean Rust domain model + an explicit conversion layer.**
- The internal model is idiomatic Rust: strong newtypes (`TeamId`, `PlayerId`, `MapId`), enums for statuses, `Option` instead of sentinel values, no dead fields.
- The overlay wire format is produced by dedicated DTO structs, not by serialising the domain model directly. This is the *only* place camelCase/legacy field names exist.
- Persistence serialises the domain model (not the wire DTOs) and carries `schemaVersion`.
- **Drift risk is the main cost.** Mitigation: golden-file tests. Capture real payloads from the original app as fixtures; assert the new server emits byte-identical JSON for the same input. Any intentional difference must be recorded as a test expectation, not an accident.

**Consequences**
- C12 (dead/incorrect state) resolves as *remove*: `matches[]`, `substitutions[]`, NDI fields, `rounds` and `Match.scoreA/scoreB` do not exist in the domain model.
- Wire DTOs are allowed to contain fields the domain does not (e.g. legacy duplicates), but only where an existing overlay reads them.
- The conversion layer is unit-testable in isolation from Axum and Dioxus.

### Q2 — Legacy Electron import
**Decision: no legacy import.**
- v1 starts with a clean data directory. The old Electron `app.json` is not read automatically and there is no import UI.
- Consequence: C10 (portable exports) applies only to v1's own export format; no legacy path/asset relinking work.
- Users migrating from the Electron app re-enter teams and re-add assets. This must be stated plainly in the README.
- Backlog note: an importer can be added later without touching the domain model, by mapping the legacy JSON into the domain.

### Q3 — Wire-format parity
**Decision: shape parity, with transport cleanups allowed — bounded by the subscription contract.**
- Field names, nesting, casing and value semantics for every existing message stay identical, so existing overlay HTML renders unchanged.
- **Message type names are the subscription contract and are all preserved**: `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`, `casters`, `matchTimer`. Collapsing them to one type would break drop-in overlays, so that is explicitly *not* allowed.
- Allowed cleanups:
  - base64 data-URL images may be replaced by served asset URLs, provided all seven overlays are verified and a **data-URL fallback toggle** exists for third-party overlays that expect inline images. (Final URL scheme in Phase 5: `/assets/<hash>.<ext>`; `/user-assets` is not used in v1.)
  - unchanged sections are not re-broadcast; a section is only sent when its content actually changes.
  - `init` keeps the keyed-map shape (C6) but is built from current state rather than replayed from a cache.
- Not allowed: renaming fields, changing nesting, changing the meaning of `visible`, or adding required fields.
- `show` remains implemented server-side but unsent in v1 (no UI toggle); it is available for the B1 backlog item.

### Q4 — Persistence
**Decision: versioned JSON behind a small `Store` trait.**
- One `app.json` in the OS app-data dir (`%APPDATA%/<app-id>/data/app.json`), with `schemaVersion` at the root.
- Atomic writes: temp file + rename; one rotating backup (`app.json.bak`); failed write never truncates the live file (C7).
- Migration chain keyed by `schemaVersion`; unknown-but-newer versions are refused with a clear message rather than silently downgraded.
- `Store` trait with a JSON-file implementation and an in-memory implementation for tests. SQLite is a backlog option only.
- Exports are a separate, portable bundle (relative asset paths), not a copy of `app.json`.

### Q5 — Network exposure
**Decision: localhost by default, LAN opt-in.**
- Default bind `127.0.0.1`; OBS on the same PC works with no configuration.
- Settings toggle “Allow LAN access” rebinds to `0.0.0.0`, shows the LAN URL(s) and an explicit warning. Changing it restarts the listener (C4 semantics).
- Port: single source of truth, default **3001**, configurable; restart on change; bind failures surface as a clear error (C4).
- CORS stays `*` for compatibility with external tooling and is documented as such. No authentication in v1; the LAN toggle is the mitigation, and auth is a post-v1 item.

## Phase 2 — Decisions locked

| # | Decision |
|---|---|
| 1 | Clean Rust domain model + explicit wire DTO conversion layer; golden-file tests guard drift. |
| 2 | No legacy Electron import; v1 starts clean, README says so. |
| 3 | Shape parity with transport cleanups bounded by the subscription contract; all message type names preserved; `init` stays a keyed map. |
| 4 | Versioned JSON (`schemaVersion`) behind a `Store` trait, atomic writes + one backup; no SQLite. |
| 5 | Localhost by default, LAN opt-in, single port (3001), CORS `*`, no auth in v1. |
| 6 | Served asset URLs by default with a `data_url` fallback toggle. |
| 7 | `currentMapIndex` is raw 0-based everywhere (C2), documented as a third-party compatibility risk. |

**Deliverables:** `docs/data-model.md`, `docs/protocol.md`.
**Known breaking changes:** `currentMapIndex` off-by-one fix; unchanged sections are no longer re-sent; `init` is built from live state.

## Phase 3 — Architecture

### Q1 — Component strategy
**Decision: official `dioxus-primitives` + selective `dx components add`.**
- Pin the upstream git revision; vendored styled components keep their MIT/Apache-2.0 headers and are listed in `THIRD_PARTY_NOTICES.md`.
- Third-party libraries rejected: `dioxus_components` 0.1.2 is web-first with 11 components; `dioxus-tw-components` 0.2.12 self-describes as not production-ready and its maintainers ended support in July 2025.

### Q2 — Workspace
**Decision: 4 crates**, one-way dependencies: `fgc-core` → `fgc-protocol` → `fgc-server` → `fgc-app`.

### Q3 — Styling
**Decision: plain CSS with custom properties**, one stylesheet per view, no Node/Tailwind in dev or CI.

### Q4 — State ownership
**Decision: UI-authoritative.** Dioxus owns `Signal<AppState>`; the server caches the last projected payload per type for `init` and `/state`. No shared domain lock.

### Q5 — Server runtime
**Decision: dedicated OS thread with its own multi-thread tokio runtime**, driven by a command channel; overlays and bundled assets embedded with `rust-embed`; user override folder checked first.

### Naming and identity (confirmed)
- Display name: **Fruiterra Graphics Controller**
- Binary: `fruiterra-gc`
- Crates: `fgc-core`, `fgc-protocol`, `fgc-server`, `fgc-app`
- Data: `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\`
- `ProjectDirs::from("com", "Fruiterra", "Fruiterra Graphics Controller")`

## Phase 3 — Decisions locked

| # | Decision |
|---|---|
| 1 | 4-crate workspace with one-way dependencies; no async/Axum/Dioxus in `core`. |
| 2 | `dioxus-primitives` pinned git rev + selectively vendored styled components. |
| 3 | Plain CSS + variables; no Node toolchain. |
| 4 | UI owns state; server caches projected payloads; `tokio::broadcast<Arc<str>>` fan-out. |
| 5 | Server on a dedicated runtime thread; `rust-embed` overlays/assets with user overrides. |
| 6 | Naming: Fruiterra Graphics Controller / `fruiterra-gc` / `fgc-*`. |
| 7 | Dioxus pinned to `0.7.10` (0.8 is alpha). |

**Deliverable:** `docs/architecture.md`.

## Phase 4 — Tournament design

### Q1 — Grand final
**Decision: bracket reset, controlled by `grandFinalsReset` (default on).**
- If the losers-bracket team wins the first grand final, a reset match is created and played; otherwise the winner takes the tournament.
- Both the reset match and the champion can be overridden by hand.

### Q2 — Byes and seeding
**Decision: next power of two + top-seed byes + standard seeding, as a *default only*.**
- Generation is a starting point. The board is fully manual afterwards; external tournament sites remain the usual source of truth.

### Q3 — Automation after generation
**Decision: auto-advance with manual override.**
- Completing a match advances the winner into the linked next slot (and the loser into `loser_next`), unless that slot was placed by hand.
- Clearing or editing a result recursively clears the auto-filled downstream slots it produced.
- Champion is auto-set from the final unless `champion_manual`.

### Q4 — Editing scope
**Decision: full board editing.**
- Drag teams between any slots (swap on occupied slots); add/remove matches and rounds; rename rounds; reorder matches; edit `next` / `loser_next` links.
- Editing remains available after play starts.

### Q5 — Round robin
**Decision: groups + auto standings + optional playoff.**
- Groups are editable after play starts; played results are retained when teams change.
- Standings recompute from fixtures; optional playoff generated from top N per group (default 2) with cross-group seeding, then fully editable.

### Q6 — Match modal
**Decision: scores + explicit winner override.**
- Winner derived from scores; an explicit override is available and visibly flagged.
- Reset clears the result and the auto-filled downstream slots.

### Q7 — Tiebreakers
**Decision: configurable ordered criteria, persisted per tournament.**
- Criteria: wins, head-to-head, map differential, maps won, points for, points against, seed.
- Default: wins → head-to-head → map differential → points for → seed.
- Drag to reorder; reset-to-default button.

### Q8 — Team deletion
**Decision: block deletion while referenced.**
- Show which tournaments use the team and offer “remove from tournaments and delete”.

## Phase 4 — Decisions locked

| # | Decision |
|---|---|
| 1 | Generated default bracket (next power of two, top-seed byes, standard seeding); fully manual afterwards. |
| 2 | Auto-advance with per-slot manual override; recursive clear on result changes. |
| 3 | Grand-final reset flag-controlled; champion auto-set with manual override. |
| 4 | Full board editing, including rounds, matches, reorder and link editing, at any time. |
| 5 | Round-robin groups editable mid-event; standings derived; optional playoff with configurable qualifiers. |
| 6 | Match modal: scores, status, first-to, winner override, link editing, reset. |
| 7 | Configurable ordered tiebreakers, persisted per tournament. |
| 8 | Team deletion blocked while referenced, with an explicit cleanup action. |

**Deliverable:** `docs/tournament.md`; acceptance criteria T1–T14 replace TR1–TR10 in `docs/backlog.md`.

## Phase 5 — Assets, fonts and overlays

### Q1 — Overlay rebuild
**Decision: minimal-diff port.**
- Keep each overlay's markup, IDs, class names, CSS rules, animations and WebSocket handling intact.
- Change only the font faces and family references.
- No shared CSS/JS modules in v1; duplication is accepted to keep each overlay independently replaceable.

### Q2 — Font packaging
**Decision: five static Saira WOFF2 files (300/400/500/600/900) plus `OFL.txt`.** *(R1 switched the active files to Archivo; the packaging decision is unchanged.)*
- Served from stable `/assets/fonts/` paths.
- No subsetting, so accented names render; scripts Saira does not cover fall back to the system sans.

### Q3 — Asset organisation
**Decision: new content-addressed blob store.**
- Blobs live at `assets/blobs/<first-two-hex>/<sha256>.<ext>`.
- Identical bytes are stored once; bundled assets are embedded with the same hash-addressed identity.
- **This supersedes the Phase 2 `Asset.relative_path` / `source` fields.** `Asset` becomes `hash`, `ext` and `origin`.
- Category folders remain only as the source layout for bundled assets and the bundled `manifest.json`; they are not part of the served URL.

### Q4 — Overlay overrides
**Decision: per-file override.**
- `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays\<file>.html` is served instead of the embedded file when present.
- A missing, empty or unreadable override falls back to the embedded file; a readable but malformed file affects only that file.

### Q5 — Hashed scope
**Decision: hash everything except fonts.**
- Every binary asset is served as `/assets/<sha256>.<ext>` with immutable cache headers.
- Fonts stay at `/assets/fonts/<name>.woff2` because `@font-face` needs a stable URL.
- A `build.rs` hashes bundled files into an embedded manifest and fails the build on a missing or unsupported file.

### Q6 — Metadata and cleanup
**Decision: metadata in `app.json`; blob GC.**
- Asset metadata is part of `schemaVersion`; there is no second asset database.
- GC runs after a successful save and on startup, removing only unreferenced **user** blobs; embedded blobs are never removed.
- Replacing an image changes only the reference. The old asset and blob stay valid, which removes the original `_temp` bug structurally (C3).

## Phase 5 — Decisions locked

| # | Decision |
|---|---|
| 1 | Minimal-diff overlay port: only font faces and family names change. |
| 2 | Five static Saira WOFF2 files behind neutral `FGC Display*` aliases; fonts at stable `/assets/fonts/` paths. |
| 3 | Content-addressed blob store for every asset except fonts; `Asset` uses `hash`/`ext`/`origin`. |
| 4 | Per-file overlay overrides; embedded fallback per file. |
| 5 | Bundled assets hashed at build time into an embedded manifest; build fails on missing/unsupported files. |
| 6 | Asset metadata in `app.json`; GC after save and on startup; embedded blobs never deleted. |
| 7 | Served URLs `/assets/<hash>.<ext>` replace the Phase 2 category-path and `/user-assets` scheme. |

**Deliverable:** `docs/assets.md`; acceptance criteria AS1–AS10.
**Supersedes:** the Phase 2 asset path scheme (`relative_path`, `source`, `/assets/<category-dir>/…`, `/user-assets/…`). The wire payload shapes are unchanged apart from the URL strings.
**C3 resolution:** stale asset caches are fixed structurally — a changed image gets a new hash and therefore a new URL.
**C11 resolution:** asset requests accept only a 64-character lowercase hex hash and an allow-listed extension; no filesystem path is built from user input.

## Phase 6 — Build, packaging and release

### Q1 — Release artifacts
**Decision: NSIS installer + portable ZIP.** *(Corrected during M0; the original MSI decision could not satisfy Q5.)*
- The installer is the `nsis` package type from `dx bundle --release --desktop --package-types nsis`.
- The portable ZIP contains the `dx` app output (`target\dx\fruiterra-gc\release\windows\app\fruiterra-gc.exe` and its `assets\` folder) plus `README.md`, `LICENSE` and `THIRD_PARTY_NOTICES.md`.
- `SHA256SUMS.txt` is published with both artifacts.
- **Bundler reality:** in Dioxus CLI 0.7.10 the Windows installer package type is `nsis`, and it emits an `.exe` (it is not a portable executable). The WebView2 bootstrapper is only supported by the NSIS path. The WiX MSI has no WebView2 handling, so shipping only the MSI would have broken Q5.
- **Display name:** the CLI derives the installer product name from the binary target (`fruiterra-gc` → `FruiterraGc`). A custom NSIS template (`crates/app/installer/installer.nsi`, selected with `[bundle.windows.nsis] template`) keeps the approved **Fruiterra Graphics Controller** display name while the installed executable remains `fruiterra-gc.exe`.
- **Config table:** in Dioxus CLI 0.7.10 the bundler reads `[bundle.windows]`; the newer top-level `[windows]` table is parsed but ignored. A `webview_install_mode` under `[windows]` therefore silently falls back to the default `OfflineInstaller` (a ~150 MB embedded runtime). Settings must live under `[bundle.windows]`, and the value uses the externally tagged form `{ EmbedBootstrapper = { silent = true } }`.
- **UI assets are not embedded:** manganis embeds only asset metadata; the files are read from `<exe_dir>\assets\` at runtime. A plain `cargo build --release` therefore does not produce a runnable portable package. The portable ZIP is built from the `dx` app output (`dx build --release --desktop --features bundle`), which hashes and copies the assets. Server overlays remain embedded via `rust-embed`; only UI assets are loose files.

### Q2 — Code signing
**Decision: unsigned in v1.**
- SmartScreen warnings are documented in the README and release notes.
- A commented `signtool` step with named secrets is kept for later.
- Users can verify the SHA-256 checksum.

### Q3 — Release trigger
**Decision: tag releases + manual dispatch.**
- Every push and PR runs fmt, clippy and tests.
- A `v*` tag builds and publishes a GitHub release.
- Manual dispatch builds and uploads workflow artifacts only; it does not create a release.

### Q4 — Versioning
**Decision: first release `1.0.0`, semantic versioning.**
- The tag must match the workspace `Cargo.toml` version.
- `schemaVersion` is independent of the app version.

### Q5 — WebView2 installer
**Decision: embed the bootstrapper in the NSIS installer.**
- It downloads WebView2 only when missing.
- The portable ZIP requires WebView2 to be present; the README states this.
- The MSI is not shipped in v1 because Dioxus CLI 0.7.10 cannot embed a WebView2 bootstrapper in the WiX package.

## Phase 6 — Decisions locked

| # | Decision |
|---|---|
| 1 | Windows x64 NSIS installer + portable ZIP + SHA-256 checksums. |
| 2 | Unsigned release, documented SmartScreen warning, signing step ready but disabled. |
| 3 | CI: fmt/clippy/test on push and PR; `v*` tags publish; manual dispatch builds artifacts only. |
| 4 | First version `1.0.0`; tag must equal the workspace version. |
| 5 | NSIS installer embeds the WebView2 bootstrapper; portable ZIP requires WebView2. |
| 5a | No MSI in v1; the WiX path cannot embed the WebView2 bootstrapper. |
| 6 | `cargo-about` generates the dependency-licence inventory and CI verifies it. |
| 7 | Hidden `--smoke-test` validates the packaged server, embedded overlays and assets. |
| 8 | README must state: no Electron import, timer control-only, LAN has no auth, Windows-only, unsigned. |

**Deliverable:** `docs/build.md`.
**QA gate:** `docs/overlay-qa.md` must pass for all seven overlays before a release is published.

## Phase 7 — Implementation plan

**Status: approved.** Implementation proceeds in milestone order M0–M10 as written in `docs/implementation-plan.md`.

- M0 workspace scaffold is the first authorised work item.
- M1 (core domain, validation, atomic store, migrations) starts only after M0 is green.
- No milestone may skip its tests, and no backlog feature may be added.
- Every intentional wire difference must be recorded in `docs/protocol.md` and covered by a test.

**Deliverable:** `docs/implementation-plan.md`.

## M0 — Workspace and guardrails (verified)

**Status: green locally.** The workspace is not yet a git repository, so the GitHub Actions workflow has not run on a remote; every command in it was run locally instead.

- Toolchain: `rustc 1.99.1` / `cargo 1.99.1`, `stable-x86_64-pc-windows-msvc`, pinned in `rust-toolchain.toml`.
- Dioxus CLI: `0.7.10 (57d6794)`; Dioxus crates pinned to `=0.7.10`.
- `dioxus-primitives`: revision `9a758255ea26e2b20c8cecf4c1c946feb9e71da7` (MIT OR Apache-2.0), resolved in `Cargo.lock`.
- Local gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (5 tests) and `cargo doc --workspace --no-deps` all pass.
- Installer: `dx bundle --release --desktop --package-types nsis --features bundle` produces `target\dx\fruiterra-gc\bundle\windows\nsis\FruiterraGc_1.0.0_x64-setup.exe` (3,120,769 bytes). The generated script embeds the WebView2 bootstrapper, uses the display name **Fruiterra Graphics Controller**, installs per-user under `%LOCALAPPDATA%\Programs\Fruiterra Graphics Controller`, and carries `fruiterra-gc.exe` plus the hashed UI assets.
- Installer smoke test: silent install, launch and silent uninstall all succeeded; Start Menu and Desktop shortcuts and the Add/Remove Programs entry were created and removed cleanly.
- Portable payload: the `dx` app output (`fruiterra-gc.exe` + `assets\`) zipped with the three documents, then extracted and launched successfully.
- Tests guard the custom NSIS template against losing the approved display name or the CLI placeholders.
- M0 findings are recorded in the Phase 6 section above: the bundler reads `[bundle.windows]`, not `[windows]`, and UI assets are filesystem assets that must ship beside the executable.

## M1 — Core domain, validation and store (verified)

**Status: green locally.** `fgc-core` owns the persisted model, repair rules, versioned JSON store, migrations and asset/path helpers; it has no async runtime, HTTP or UI dependency.

- **Enum spellings are fixed by tests.** Lowercase for `Role`, `Position`, `MatchStatus`, `AssetOrigin`, `BracketMatchStatus`, `TimerMode`, `TimerAlignment` and `SocialPlatform`; kebab-case for `AssetCategory` (`game-mode`), `BracketFormat` (`single-elim`, `double-elim`, `round-robin`), `TournamentStatus` and `Tiebreaker`. `TeamSide` is uppercase `"A"`/`"B"` and `Side` is numeric `0`/`1`/`2`. `Scoring` is internally tagged with `mode`.
- **Unknown-field policy.** Unknown top-level `AppState` keys are preserved verbatim in a `#[serde(flatten)] extra` map and survive a save/load round trip. Unknown nested fields are ignored. This keeps forward compatibility without freezing every nested struct.
- **Scores are clamped at parse time.** Scores are `u16` in the domain model, so negative values are clamped to 0 by a deserializer rather than by `validate_and_repair`; `validate_and_repair` never sees a negative score.
- **Error paths use `serde_path_to_error`.** A bad document reports the JSON field path, for example `teams[0].players[0].role`, instead of a bare serde message.
- **Migrations are version-indexed.** A missing `schemaVersion` is treated as version 0. `MIGRATIONS[n]` upgrades from version `n`; a document newer than `SCHEMA_VERSION` is refused with `UnsupportedSchema`. There is no Electron import.
- **Store behaviour.** `JsonStore::save` writes `app.json.tmp`, fsyncs, rotates the current file to `app.json.bak`, then renames the temp file into place. `load` returns defaults when the primary is missing, and falls back to the backup when the primary is corrupt. `MemoryStore` backs pure logic and tests.
- **Validation is shared.** `validate_and_repair` is called by `migrate`, so files, imports and tests all use the same repair table. It logs each repair once via `tracing::warn!`.
- **Tests:** 36 `fgc-core` tests (1 unit + 7 assets + 9 migrate + 2 paths + 6 store + 11 validation) and 40 workspace tests pass; `fmt`, `clippy -D warnings` and `doc` are clean.

## M2 — Protocol DTOs, projection and golden tests (verified)

**Status: green locally.** `fgc-protocol` owns the wire DTOs, asset catalogue, URL resolvers, section cache and projector. It depends only on `fgc-core`, `serde`, `base64` and `thiserror`; it has no server, HTTP or UI dependency.

- **Projection is pure in served mode.** `Projector` reads `AppState` and an `AssetCatalog`, and formats `/assets/<hash>.<ext>` URLs. It performs no filesystem access. `DataUrlResolver` is the only I/O-capable path, and it reads bytes solely through a caller-supplied loader.
- **`show` shape corrected.** The overlays read `visible` at the top level of the frame (`{ "type": "show", "visible": true }`), not under `data`. `docs/protocol.md` §3.4 previously showed the nested form and is fixed. `ShowMessage` serialises the flat shape; `SectionCache` never stores `show`.
- **Hero art convention.** A `Hero` asset's own file is its large portrait art; `Asset.icon` links the small icon, and an optional `Asset.portrait` link overrides the large art. `heroIcons` reads the icon; `heroPortraits` and a player's `heroImageUrl` prefer the override and otherwise use the asset itself. This preserves the original's `hero.path || hero.portraitPath` preference and fixes the original's `heroImageUrl` always being `null` because hero entries lacked a `path`.
- **Watermark fallback.** No configured watermark, or a configured id missing from the catalogue, falls back to the bundled `Peach` logo. A configured asset whose bytes are unavailable in `data_url` mode resolves to `null` instead of falling back, matching the original.
- **Small parity fixes.** Empty `auxColor` serialises as `#090909`; `record` defaults to `"0-0"` when absent or empty; `pronouns` and `socialMedia` are omitted when unset; `sideIcons` always carries `attack` and `defend` keys; `currentMapIndex` is the raw zero-based value and is `null` when no map is selected.
- **Quirks locked by tests.** Five identical scoreboard payloads under five type names; `casters` carries `{ castersData }`; `roleIcons` has lower-case and original-case keys; `gameModeIcons` keys are lower-case; hero keys are case-sensitive; `mapPoolData` groups by mode, keeps first-occurrence map order, sorts modes alphabetically with `Flashpoint` and `Push` last, always emits `modeIcons` as an array and skips unknown maps; `maps[].type` is the game-mode display string; chosen-by names use team tags.
- **Golden fixtures.** `scoreboard_full`, `scoreboard_minimal`, `init_after_broadcast`, `init_empty`, `casters`, `timer_tick`, three timer actions and `show` are checked in under `crates/protocol/tests/fixtures/`. They were derived field-by-field from the original `MatchController.tsx`, `electron/main.ts` and overlay HTML rather than captured from a running Electron build; provenance is recorded in `docs/protocol.md` §6.
- **Tests:** 67 `fgc-protocol` tests (1 unit + 16 asset URL/catalogue + 11 golden + 8 init + 31 quirks) and 106 workspace tests pass; `fmt --check`, `clippy -D warnings` and `doc` are clean.





## M10 - Packaging and release (implemented, gates pending)

**Status: implementation green locally and in CI; tagging `v1.0.0` remains blocked on the manual gates in `docs/manual-qa.md`.**

- **Process deviation.** M10 implementation started before the OBS sign-off at the maintainer's request. The plan says M10 cannot start until the overlay QA gate passes; the tag is still withheld until R1-R8 pass, so the release itself remains gated.
- **Release workflow.** `.github/workflows/release.yml` runs on `v*` tags and manual dispatch. It verifies the tag against the workspace version, runs fmt/clippy/tests, regenerates the notices, discovers the NSIS installer from `BundleOutput` JSON, renames it, builds the portable ZIP from the `dx` app output, runs the packaged `--smoke-test`, writes `SHA256SUMS.txt` and uploads the artifacts. Only a tag creates a GitHub release; manual dispatch stops after the artifact upload.
- **Smoke test.** The hidden `--smoke-test` path starts the real server on an ephemeral port with a temporary data directory, checks `/health`, an embedded overlay, a bundled asset, the `init` handshake and `requestInit`, then exits 0 without opening a window or touching app data.
- **Notices.** `THIRD_PARTY_NOTICES.md` is generated from `about.toml` and `about.hbs` with `cargo-about` 0.9.3 (CLI feature). CI regenerates it and fails if the committed file differs.
- **Local release candidate.** `dx bundle --release --desktop --package-types nsis --features bundle` produced the NSIS installer and the `dx` app output; the packaged binary passed `--smoke-test`. The portable ZIP was staged and checked locally (exe, `assets\`, README, LICENSE, notices) and the two artifact hashes were generated.
- **CI release candidate.** Manual dispatch run `43050860851` passed every step and uploaded `fruiterra-gc-1.0.0-windows-x64` (installer, portable ZIP, checksums). Tag-only steps were skipped as designed. CI run `43050843933` passed the main checks and the new notices job.

## R1 - Overlay fixes after the first OBS pass (implemented, re-run pending)

**Status: fixes implemented and measured in WebView2; the OBS re-run is the remaining gate.** The first OBS pass failed with eleven findings; root causes and dispositions are recorded in `docs/manual-qa.md` §2.1.

- **Hero art fallback.** `heroIcons` now uses the linked icon when present, falls back to the hero's own art or `portrait` link, and skips an asset that is already another hero's icon. Imported heroes without an icon link therefore render in the game overlay's ban strips. Locked by `crates/protocol/tests/quirks.rs`.
- **One fitting rule.** All seven overlays share an inline `fitText(el, maxSize, minSize)` helper: shrink to a floor, then ellipsize on one line. It measures in scale-normalised layout pixels so the 0.9 transform animation on some headers cannot fool it, and re-fits on `document.fonts.ready` after each render.
- **Name limits.** `MAX_TEAM_NAME_CHARS = 32`, `MAX_PLAYER_NAME_CHARS = 24`, `MAX_TALENT_NAME_CHARS = 24`. `validate_and_repair` truncates overlong names with `RepairKind::NameTooLong`; the Teams and Talent inputs enforce `maxlength`. Long names still shrink before truncating, so 32 characters can render in full where the box allows.
- **Layout fixes.** Game overlay name box is single-line with ellipsis and inset logo art; map-select names/types/chosen-by are centred; map-pool reveal is CSS-driven (no inline opacity writes or second fade); roster info blocks are a fixed 106px with fixed line heights; caster handles get one `@`; hero-ban and map-pool text use block widths and a lower fit floor.
- **Archivo trial.** The R1 finding that Saira does not read like Apotek is addressed by switching all seven overlays to five static Archivo WOFF2 files (300/400/500/600/900). Saira files and `OFL.txt` stay beside them for the A/B comparison; `OFL-Archivo.txt` is added. The overlay test pins `FONT_PREFIX = "archivo"`, so reverting is a one-line change plus the `@font-face` `src:` URLs.
- **Not defects.** The map-select canvas is exactly 1920×1080 and the maps row is centred (the enlarged current card shifts visual weight); the watermark has no transform or letter-spacing (the apparent stretch is font metrics); hero bans are current-map only by design.

## R1 re-run - second OBS pass (2026-11-14)

**Status: Pass with notes on every overlay; no disconnects or flickers during the 10-minute watch.** The full result and environment are recorded in `docs/manual-qa.md` §1.1 and §2.2.

- **Caster platform icons.** `getSocialIcon()` now maps every `SocialPlatform` variant (`twitter`, `twitch`, `youtube`, `instagram`, `tiktok`). An unmapped platform no longer renders an empty 20px icon slot.
- **Rounded-corner fringing.** Card containers add `clip-path: inset(0 round 6px)` and the image layer gets a matching top radius. Applied to both rosters, the casters row and the map pool; game overlay and map select already had the clip.
- **Asset links in the UI.** Hero assets gained a **Ban icon…** editor for `Asset.icon`; map assets gained a **Game mode…** editor for `Asset.game_mode`. This closed the gap behind R1-15 and backlog C14. Linked icons are excluded from `heroIcons`, `heroPortraits`, the hero-ban datalist and the icon picker, so an icon never appears as a hero. `AssetCatalog::is_linked_icon` backs the filter and is covered by tests. The hero editor was superseded by the A40 properties dialog; the map editor remains as the inline **Game mode** select.
- **OBS animation caveat.** With **Shutdown source when not visible** off, OBS keeps the page loaded and CSS entrance animations play only once. The runbook and wiki now tell the operator to enable **Refresh browser when scene becomes active** if the animation must replay on every scene entry.
- **Backlog, not fixes.** Map-pool per-mode row animation (D5), roster card autofill (D6) and a hero-ban strip polish pass (D7) were requested during the pass and stay post-v1.

## QA gates R2 and R3 (2026-11-15)

**Status: R2 passed; R3 is blocked and accepted as such.** Both are recorded in `docs/manual-qa.md` §2 and the sign-off log.

- **R2 offline font check.** The repository owner restarted OBS with the network disconnected and confirmed all seven overlays still rendered Archivo from `/assets/fonts/`. No external font request is needed, which is what the bundled-font design was for.
- **R3 accepted as blocked.** No original pre-rebuild overlay HTML exists, so drop-in compatibility with an old file cannot be demonstrated. The wire contract is still covered by the M2 projection tests, `crates/protocol/tests/quirks.rs` and the server's `init`/`requestInit` smoke test, and the rebuilt overlays were validated in the R1 OBS pass. The maintainer accepts this gap for v1; the gate stays listed as Blocked rather than Done so the record is accurate.

## R4 - Timer target fix (2026-11-15)

**Status: fixed and verified on the desktop build; the R4 re-check is the remaining step.** Findings are recorded in `docs/manual-qa.md` §2.3.

- **Root cause.** Count to target evaluated `Math.max(0, Math.floor((Date.parse(target) - Date.now()) / 1000))` through `document::eval`. Dioxus runs an eval script as an async function body, so a trailing expression evaluates to `undefined`, not a value. The `join::<i64>()` result therefore failed to deserialize, the error was discarded by `Err(_) => 0`, and the zero remaining time stopped the timer on the first tick. Count up and count down never used eval, which is why only target mode failed.
- **Fix.** `target_countdown_script` builds the script with an explicit `return`, and an eval failure is logged at `warn` instead of being swallowed. A unit test asserts the script starts with `return ` and quotes the target.
- **Guard.** **Start** is disabled while **Count to target** has no target, and the field carries a hint, so an empty target can no longer look like a broken timer.
- **Not changed.** The timer remains control-only; no bundled timer overlay is added. The target is still view state and is not persisted, matching the existing rule that timer run state is not saved.

## Remote overlay access backlogged (2026-11-15)

**Status: recorded as post-v1 backlog C3; nothing is implemented for v1.** Raised
after the A3 pass, where an overlay served from a Tailscale origin only worked
because the browser was on the host PC.

- **Detect, do not bundle.** The app will drive the user's installed Tailscale
  client instead of embedding it (tsnet is Go) or shipping a sidecar. The mode is
  off by default and a missing client is a normal state.
- **Origin-aware connection first.** The hardcoded `ws://localhost:<port>` is the
  real blocker for any remote viewer. Replacing it with a same-origin resolver
  while keeping `?port=`/`?host=` is a prerequisite and also improves LAN mode.
- **Funnel is gated.** Public exposure needs an explicit confirmation, a token
  and a documented teardown; it is not a default and does not persist across
  launches.
- **Not in v1.** No code, settings, protocol or packaging changes for this are
  made before the v1 tag. Design details and open questions are in
  `docs/backlog.md` C3.

## Remote-access UI ships ahead of Tailscale plumbing (2026-11-21)

**Status: D8-2 done 2026-11-21; D8-3 to D8-7 remain open.** The owner chose an
honest UI-first slice for the first C3 item instead of pulling client detection
and serve lifecycle forward.

- **Persisted mode.** `OverlaySettings.remoteAccess` stores `off`, `lan`,
  `tailscale` or `funnel` in the versioned state file. The existing `allowLan`
  flag stays the bind source of truth; validation maps an older `allowLan: true`
  save to LAN mode, and the LAN switch keeps the two fields in sync. The
  selector used by this first slice was removed in D8-8.
- **What is live.** Off (loopback) and LAN work exactly as before, including a
  listener restart when access settings change. Settings shows the active
  access state, server state and base URL (the URL stays masked while URLs are
  hidden), plus the shared Browser Source panel with copy and Open in browser
  actions.
- **What was deliberately not live at the time.** Funnel stayed disabled and
  Tailscale was disabled until detection landed in D8-3. Neither mode bound a
  different interface or generated a remote URL, and no `tailscale serve` or
  `funnel` command was run.
- **Next.** D8-3 detection, D8-4 serve lifecycle with teardown, D8-5 gated
  Funnel, D8-6 remote URL generation and D8-7 docs/QA finish the feature.

## Tailscale detection landed (2026-11-21)

**Status: D8-3 done 2026-11-21; D8-4 to D8-7 followed.**

- **Detection only.** Settings reads the user's installed CLI — PATH first,
  then `%ProgramFiles%\Tailscale` and `%ProgramFiles(x86)\Tailscale` — and runs
  only `tailscale status --json` and `tailscale version` on a blocking task.
  `BackendState` plus `Self.DNSName` and `Self.Online` become not installed,
  stopped, needs login, or connected. Each non-ready state has a plain remedy
  and a Refresh button; nothing is installed or bundled.
- **Switch unlock.** The Tailscale switch is enabled only when the client
  reports `Running`. Turning it on still left the local server in place at this
  point: remote serving arrived in D8-4. Funnel stays unavailable.
- **Privacy and safety.** Only the short version, this device's MagicDNS name
  and its online flag are displayed. Peer lists, auth URLs and keys are not.
  No serve, funnel or reset command was issued at this stage.
- **Next.** D8-4 serve lifecycle with teardown, D8-5 gated Funnel, D8-6 remote
  URL generation and D8-7 docs/QA.

## Tailscale Serve owns two paths (2026-11-21)

**Status: D8-4 done 2026-11-21; the `/` path was added with D8-6 the same day.**

- **Start.** With Tailscale on, starting or restarting the overlay server runs
  `tailscale serve --bg --yes --https=443 --set-path <path> http://127.0.0.1:<port>`
  for `/fgc` and `/` on a blocking task. `--bg` lets the CLI return while
  tailscaled keeps the config; `--yes` prevents an interactive prompt. The local
  server still binds loopback only. A failed second install rolls back the
  first, so a half-configured pair is not left behind.
- **Why `/` exists.** Five of the seven overlays are frozen by this backlog's
  file restriction, and all seven dial `wss://location.host/` with no path.
  With only `/fgc`, a page at `/fgc/overlays/<file>.html` loads but its
  WebSocket hits a 404. Tailscale matches the longest prefix and strips it, so
  a companion `/` handler forwards the root WebSocket to the same local server
  while `/fgc/overlays/...` still reaches `/overlays/...`. This was verified
  with HTTP and WebSocket probes before it was wired into the app.
- **Exposure tradeoff.** The root handler makes the whole local overlay server
  reachable unprefixed on the tailnet (`/`, `/health`, `/state`,
  `/overlays/...`, `/assets/...`). It is the same loopback service, not a new
  one, but any pre-existing root Serve handler on this node is taken over while
  the app serves and is not restored by teardown. The owner approved this
  tradeoff so all seven overlays work without editing the frozen five.
- **Teardown.** Stopping the server, switching Tailscale off, and dropping the
  app root all run
  `tailscale serve --yes --https=443 --set-path <path> off` for both paths. Exit
  teardown is synchronous because spawned tasks may not finish during shutdown.
- **Isolation.** Only these two paths are removed. The port-wide `off` and
  `tailscale serve reset` are never used, so handlers installed by other
  software survive. A "handler does not exist" result is treated as success,
  making teardown idempotent. Named services were evaluated and rejected: this
  tailnet node is not tagged, so `--service svc:fgc` is unavailable.
- **Honesty.** The app does not install or bundle Tailscale. Settings shows
  whether the handlers are active, preparing or failed, with the raw CLI error
  on failure.
- **Next.** D8-5 gated Funnel and D8-7 docs/QA.

## Scope-aware overlay URLs (2026-11-21)

**Status: D8-6 done 2026-11-21. D8-5 Funnel and D8-7 docs/QA remain open.**

- **Shapes.** Tailscale uses `https://<dnsname>/fgc/overlays/<file>.html` from
  this device's MagicDNS name, with no `?port=`. LAN keeps
  `http://<lan-ip>:<port>/overlays/<file>.html`; with both switches off,
  loopback keeps `http://127.0.0.1:<port>/overlays/<file>.html`. The same base
  URL drives served asset URLs, so an overlay loaded through `/fgc` asks for
  `/fgc/assets/...` instead of a localhost address on the OBS machine.
- **Timing.** Detection runs once when the store is created, not only when
  Settings mounts, so a launch straight into Overlay still gets remote URLs.
  A persisted Tailscale setting with auto-start waits for that first result;
  otherwise the server would begin with local asset URLs. Until the DNS name is
  known, Settings says "waiting for Tailscale address" instead of inventing a
  URL; when it arrives the base URL is refreshed and projected frames are
  resent. The last known detection is kept during a manual Refresh so URLs do
  not flicker out of the panel.
- **Funnel stays hidden.** Funnel is not offered in the two-switch UI, so a
  persisted value keeps its pending notice and local URLs. The token belongs
  only in future Funnel URLs (D8-5) and is never added to Tailscale URLs.
- **Reachability.** The `/` Serve handler described above is part of making
  these URLs usable: it is what lets the five frozen overlays keep their
  WebSocket after the page loads under `/fgc`.
- **Next.** D8-5 gated Funnel with an explicit public-exposure confirmation and
  unguessable token, then D8-7 docs/QA.

## Remote overlay docs and QA (2026-11-21)

**Status: D8-7 in progress. A44 now has owner evidence; A42 and A45 await owner
results. D8-8 completed the settings clean-up after this documentation pass.**

- **Wiki framing.** The public wiki is still the v1.0.0 guide, so the new
  [Remote overlays](wiki/Remote-Overlays) page opens with a clear post-v1 note:
  published v1.0.0 has loopback and LAN behaviour through **Allow LAN
  connections** but no Remote access panel or Tailscale serving; Funnel is not
  implemented; Tailscale applies to later builds. Settings, Overlays in OBS,
  Troubleshooting and FAQ keep their v1 wording and cross-link the new page
  instead of claiming that the released version has remote serving.
- **Content.** The page describes the two independent **Local network** and
  **Tailscale** switches, tailnet ACLs and HTTPS prerequisites, the Windows
  firewall rule for LAN, exact `/fgc` URL shapes, why the companion `/` handler
  exists, the exposure tradeoff where a prior root handler is taken over and
  not restored, targeted teardown, and the failure modes from the design notes.
  No public Funnel URL is promised.
- **Runbook.** §7 adds A42–A47. A42 repeats LAN access through the **Local
  network** switch on a second PC; A44 uses a second tailnet device; A45 uses a
  phone on cellular. A43 checks the host's URLs, Serve paths and WebSocket. A46
  checks that every teardown removes only `/fgc` and `/`. A47 checks the
  two-switch settings model and the absence of a CLI console. A5 remains the
  owner's pass for the v1.0.0 LAN UI and is not rewritten as a new result.
- **Evidence rule.** Same-host HTTPS and WebSocket probes prove the Serve path
  and the local server, not a remote device. A43 is marked host-side only. A44
  is Pass from the owner's OBS report on another computer: "I have tested the
  Tailscale overlays on another computer in OBS, and everything seems to be
  working fine for now." The device/OS, exact URL, screenshot and WebSocket log
  were not supplied, so those gaps are recorded rather than assumed. A42 and
  A45 stay Pending until the owner supplies results from the named environments.
  The register records the exact QA `app.json` hash before and after so post-v1
  checks do not disturb the release fixture.
- **Teardown wording.** "Empty" means no app handlers remain. Because the app
  deliberately preserves other Serve paths, `tailscale serve status` only
  reports `No serve config` on a node with no other handlers. The evidence
  shows an unrelated `/other` handler surviving every teardown and a clean node
  returning `No serve config` after both Tailscale switch-off and Stop server.
- **Next.** Owner results for A42 and A45; D8-5 gated Funnel afterwards.

## Remote-access switches and a quiet Tailscale CLI (2026-11-21)

**Status: D8-8 done 2026-11-21. No state schema or overlay change.**

- **Problem.** The owner reported brief CMD popups during Tailscale CLI calls
  and said the settings were unclear: an **Allow LAN connections** button sat
  alongside an **Access scope** selector, and Tailscale and local access should
  be available at the same time.
- **Settings decision.** One **Remote access** panel now has two independent
  switches. **Local network** keeps using `overlay.allow_lan` as the bind source
  of truth; **Tailscale** keeps using `overlay.remote_access == Tailscale` as
  the Serve gate. `set_local_network` and `set_tailscale` no longer clear each
  other, and the status line can read **Local network + Tailscale**. The
  duplicate LAN switch and the **Access scope** selector are gone. No new field
  was added, so the QA `app.json` hash stays
  `47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.
- **Console decision.** On Windows, every Tailscale CLI process is created with
  `CREATE_NO_WINDOW`. Detection and Serve calls remain direct `tailscale.exe`
  children, so no `cmd.exe` or `conhost.exe` helper window is created. A
  process watch during repeated Refresh calls saw only `status --json` and
  `version` invocations with the app as parent.
- **What stayed.** The root `/` Serve handler remains because the frozen
  overlays dial the WebSocket at the page root. Teardown still removes only
  `/fgc` and `/`, never uses `tailscale serve reset`, and leaves Local network
  untouched when Tailscale is switched off.
- **Evidence.** Runbook A47 records CDP checks of both switches and the absence
  of stale controls, the combined persisted state, the seven `/fgc` URLs, the
  process watch, and clean Serve teardown. Build, lint and tests pass, as does
  `dx build --desktop`. Funnel remains unimplemented.

## Asset cleanup guarded against a failed load (2026-11-15)

**Status: fixed 2026-11-21; v1 correctness item C15.** Found while recovering the test fixture.
A PowerShell `Set-Content -Encoding utf8` wrote a BOM into both `app.json` and
`app.json.bak`; serde rejected both, the app started on the empty default state,
and the 20 user blobs were removed.

- **Cause.** `clean_up_unused` and the post-save collector build the keep set from
  the in-memory state plus `store.load().ok()`. With an unreadable store the
  second half is `None` and the first half is empty, so the keep set is empty and
  every blob is garbage. The startup collector is already skipped when the load
  fails (`loaded == false`), but the **Clean up now** button and the saver path
  are not.
- **Guard.** Track whether the last load succeeded (the existing `Bootstrap::loaded`
  is enough) and refuse every cleanup collector while it is false, with a
  plain-language notice. Only a full project-bundle import replaces the state
  and re-enables cleanup; an explicit user save does not.
- **Fix.** `AppStore` now carries `cleanup_allowed`, seeded from whether the
  startup load succeeded. Manual, post-save and exit collection all go through
  `cleanup_keep_hashes()`, which returns no keep set while cleanup is paused or
  the saved state is unreadable. Settings disables **Clean up now** and shows a
  plain-language explanation; no blobs are deleted in that state. Importing a
  full project bundle replaces the state and re-enables cleanup; a single-image
  import does not, because it would otherwise orphan the unknown blobs.

## Origin-aware overlay connection pulled forward (2026-11-16)

**Status: fixed in v1.** D8-1 was a post-v1 backlog item; the A5 LAN run showed
it was a v1 blocker, so it was pulled forward before the tag.

- **Problem.** LAN mode correctly bound `0.0.0.0`, but all seven overlays dialled
  `ws://localhost:<port>`. A browser on a second PC therefore connected to its
  own localhost, not the host PC. A3 had passed only because that browser ran on
  the host machine.
- **Fix.** Each overlay now resolves the socket from the page origin:
  `ws(s)://location.host` when the app served the page, `?host=`/`?port=`
  overrides for third-party pages, and `wss` when the page is HTTPS. The old
  `ws://localhost` literal is gone.
- **Compatibility.** No message, filename or DOM change. Existing third-party
  pages that pass `?port=` behave as before; pages served from the app now follow
  whatever host and port the viewer used.
- **Tests.** `crates/server/tests/overlays.rs` asserts the resolver in all seven
  files, checks `location.host` and both query overrides, rejects a
  `ws://localhost` literal, and scans both `http://` and `https://` for external
  URLs.
- **Scope.** Only D8-1 moved. D8-2 to D8-7 (settings UI, Tailscale detection,
  serve lifecycle, Funnel, URL generation, docs) remain post-v1 backlog.

## A5 - LAN firewall hardening (2026-11-16)

**Status: applied and verified with a second PC.** The A5 run needed a firewall
rule that kept LAN mode usable without exposing the port beyond the show network.

- **Network profile.** The Ethernet network used for the show is set to
  **Private** in Windows. A **Public** profile is treated as untrusted and blocks
  LAN mode by design; the wiki tells users to change it.
- **One scoped rule.** Inbound **TCP** only, **Private** profile,
  `RemoteAddress = LocalSubnet`, `Program` = `fruiterra-gc.exe`, action Allow,
  enabled. No UDP, no `Any` remote address, no `Public` profile.
- **What was removed.** Windows had auto-generated a TCP rule for the executable
  on `Private, Public` with any remote address, plus a matching UDP rule. Both
  were deleted; the UDP rule had already been removed during the run.
- **Why not broader.** The overlay server has no authentication in v1, so the
  blast radius is the subnet, not the internet. CORS remains `*` and is unchanged.
- **Verification.** The second PC painted the live overlay and updated after a
  score edit both before and after the rule change; turning **Allow LAN
  connections** off returned the listener to `127.0.0.1:3001` and the LAN IP
  stopped answering.
- **Docs.** `docs/wiki/Overlays-in-OBS.md`, `docs/wiki/Troubleshooting.md` and
  `docs/build.md` now state the Private profile and the TCP/LocalSubnet rule.

## Overlay override guard (2026-11-17)

**Status: fixed during A7.** The A7 run showed that a readable but empty
override was served as a zero-byte page, leaving that OBS source blank. The
runbook also expected malformed HTML to fall back, which the server cannot detect
without parsing the document.

- **Guard.** `read_override` now treats an empty file and a non-UTF-8 file as
  missing: it logs a warning and serves the embedded overlay instead. A locked
  or unreadable file already fell back through the I/O error path.
- **Not changed.** A readable, valid-UTF-8 but malformed HTML file is still
  served as-is. It affects only that one overlay, which is the documented
  per-file rule; silently replacing hand-written HTML would be surprising.
- **Tests.** `crates/server/src/overrides.rs` covers empty, non-UTF-8 and valid
  override files.
- **Docs.** The runbook A7 step and the wiki/assets wording now say **missing,
  empty or unreadable**, not "broken".

## Dynamic select options must set `selected` (2026-11-17)

**Status: fixed during A22.** Native `<select>` controls whose options come from
a saved value showed the first option instead of the saved one on first render.
It affected the Match Controller Team A/Team B, first-to and talent pickers, the
Teams role/position pickers, the Talent social platform and most tournament
pickers. Re-selecting the same value looked correct only because the user's
change re-rendered the option that now existed.

- **Root cause.** Dioxus 0.7 declares `select.value` as a volatile attribute
  (`dioxus-html`), so the web interpreter always writes it, as a property
  (`node.value = …`). Dynamic `<option>` children are created in later
  mutations, so the write lands while the select has only its placeholder or no
  options at all, and the browser discards it. Static options are part of the
  same template as the select, so they exist when `value` is written and work.
- **Fix.** Every dynamically generated `<option>` carries
  `selected: <bool>`. The interpreter writes `option.selected` as a property
  (`node.selected = truthy(value)`) once the option exists, which the browser
  keeps. `initial_selected` is not a substitute: it writes `defaultSelected`,
  which only matters for a reset.
- **Scope.** Five views and roughly forty option sites: `assets.rs` (import
  category), `match_controller.rs` (first-to, Team A, Team B, six talent slots,
  interviewee), `teams.rs` (role, position), `talent.rs` (social platform) and
  `tournament.rs` (format, default first-to, status, entrant pickers, group
  count, qualifiers, add-team, fixture teams, tiebreaker, champion, bracket
  slots, fixture and dialog status, dialog first-to, winner override, winner and
  loser targets). Selects built from literal `<option>`s (Format, Status, Side
  indicator, timer mode, slot numbers) are unchanged.
- **Rule.** A `<select>` whose options are produced by a `for` loop or an
  `if let` branch must set `selected` on each option. Keeping `value` on the
  select as well is harmless and keeps the attribute in the markup.
- **Tests.** No automated test: the markup only runs inside wasm. The A22 re-run
  (first-render check of every affected picker) is the verification.

## A25 - Bracket slots accept only one incoming link (2026-11-17)

**Status: fixed during A25.** Setting a manual winner with valid `next` and
`loser_next` targets left the target slots empty even after an explicit
refresh. Links, scores, status and the manual winner persisted, so the data was
saved correctly; only the propagated slots were wrong.

- **Root cause.** `set_link` accepted a second link into an already-fed
  `(match, slot)`. `propagate_once` walks every match in board order, so the
  pending competing source wrote `None` into the slot after the completed
  source had written its team. The result depended on iteration order, and
  `fill_structural_byes` had the same ambiguity because it keys incoming links
  by `(match, slot)` in a map.
- **Fix.** `set_link` enforces one source per slot: when a link is set to an
  already-fed target slot, the link that fed it before is cleared first. This
  includes the same match's other link kind, so a winner and a loser can never
  target the same slot. `repair_links` keeps the first link in board order and
  clears later duplicates, so an older or hand-edited save self-heals on load.
- **UI.** The match modal states that choosing a target slot replaces whichever
  match fed it before.
- **Tests.** `linking_into_an_occupied_slot_takes_it_over_and_propagates` and
  `refresh_keeps_one_link_per_target_slot` in `crates/core/src/tournament.rs`.
  The temporary A25 reproduction probe was deleted after the fix.
- **No new repair kind.** The load-time dedupe runs inside `repair_links`
  without a `TournamentRepairKind`, because a duplicated slot source is an
  impossible state rather than a user-visible data problem; the board shows the
  deterministic result.

## A26-1 - Head-to-head is re-evaluated inside every tie (2026-11-17)

**Status: fixed during A26.** With **Head-to-head** first, a team that won the
head-to-head fixture could still rank below its rival when both teams were
level on league points.

- **Root cause.** `sort_rows_recursive` split the tied rows by the current
  criterion and then recursed with `criteria[index + 1..]`. For head-to-head
  that is wrong: the mini-table is relative to the set of teams still tied, so
  the next subgroup needs its own mini-table, not the previous one. The
  top-level result carried into every subgroup and the head-to-head winner
  could stay below its rival.
- **Fix.** When the criterion is `Tiebreaker::HeadToHead`, recursion keeps
  `&criteria[index..]` so head-to-head is recomputed for each subgroup. Every
  other criterion is an absolute key and keeps `criteria[index + 1..]`.
- **Semantics.** The tiebreaker list remains the full standings sort, not a
  points-first list with tiebreakers after it. `Wins` is one point per win, so
  the default order still breaks ties on points; moving another criterion
  above `Wins` makes it more important than points.
- **UI copy.** The tiebreaker panel and `docs/wiki/Tournament.md` now say the
  list sorts the standings and that head-to-head compares the teams still
  level, instead of implying that every criterion only runs after points.
- **Tests.** `head_to_head_first_recomputes_the_mini_table_per_tie` in
  `crates/core/src/tournament.rs`; the temporary A26 probe was deleted after
  the fix.

## A26-2 - Standings fall back to the team list for non-entrants (2026-11-17)

**Status: fixed during A26.** A group could contain a team that was not a
tournament entrant. Its standings row showed the raw GUID because the name
lookup only checked `entrant_names`.

- **Fix.** `crates/app/src/views/tournament.rs` falls back to the full `teams`
  list, the same lookup the group member chips already use.
- **No model change.** Group membership is not restricted to entrants, which
  keeps the board editable after play starts.

## A21-1 - The timer keeps running across destinations (2026-11-18)

**Status: fixed during A21.** Starting the clock and then visiting another
destination stopped the once-per-second ticks and reset the in-app clock to
`00:00:00`, so a third-party timer page froze mid-count.

- **Root cause.** `MatchControllerView` owned the `TimerState` signals and the
  tick loop. Navigating away unmounted the view, which dropped the loop and the
  signals with it.
- **Fix.** `crates/app/src/timer.rs` now owns `TimerState`, the count-to-target
  value and the tick loop as `TimerController`, provided once at the app root in
  `crates/app/src/app.rs`. The Match Controller reads the controller from
  context; the clock helpers and the two broadcast helpers moved with it.
- **Behaviour.** The clock keeps counting and keeps broadcasting while the
  operator visits any destination. Run state is still never persisted, and the
  count-to-target value still clears on restart.
- **Docs.** `docs/wiki/Match-Controller.md` now says the clock keeps running
  while you visit another page and that only a restart clears the target; the
  M7 note in `docs/implementation-plan.md` and the A21 runbook step were
  updated.

## A12-1 - Dialogs cap to the viewport and scroll (2026-11-18)

**Status: fixed during A12.** At the 1040×680 minimum window the tournament
**Edit match** dialog measured 684 px tall, so its top and bottom edges were
clipped by 2 px each and nothing could scroll them into view.

- **Root cause.** The vendored `.dx-dialog` is a fixed-size, vertically centred
  box with no `max-height` and `overflow-y: visible`, so a dialog taller than
  the window simply overflows.
- **Fix.** `crates/app/assets/theme.css` caps
  `[role="dialog"][class^="dx-dialog-"]` at `calc(100vh - 2rem)` and sets
  `overflow-y: auto`. The class is a CSS-module hash, so the rule matches the
  prefix; the backdrop shares the prefix but is excluded by `role`.
- **Not changed.** The vendored dialog CSS is untouched, matching A11-1, and
  the dialog keeps its centred position and width.
- **Verified.** On a fresh `dx build --desktop` binary at 1040×680 the dialog
  sits at 16 px margins, scrolls 36 px, and **Save result** stays reachable.
  Every destination and the four dialogs were re-audited at 100%, 125%, 150%,
  1280×720 and 1040×680 with no other overflow.

## Pre-release UI touch-ups (2026-11-18)

**Status: implemented in the app; documentation and manual QA follow-up in progress.** A 19-item pre-release sweep adjusted UI flows and a few persisted fields while keeping the overlay protocol and bundled overlay files drop-in compatible.

- **Persisted defaults and schema fields.** `OverlaySettings.auto_start` defaults to `false`; `DisplaySettings.show_hero_bans` defaults to `true`; `Team.logo_background` defaults to `false`; `Match.scheduled_at` defaults to `None`; `AppState.matches` defaults to an empty list. Schema v2 removes the per-person `Talent.talent_type`; the on-air role now lives on each match talent assignment (see the final pre-release decisions below).
- **Shell and navigation.** Sidebar order is **Match Controller, Overlay, Teams, Talent, Assets, Tournament, Settings**. Destinations use Lucide icons (`Gamepad2`, `MonitorPlay`, `UsersRound`, `Mic`, `Images`, `Trophy`, `Settings2`) instead of letter abbreviations.
- **Overlay URLs and server startup.** Browser Source URLs are hidden by default on the Overlay page and revealed with **Show URLs**. Settings adds **Start server on launch**; when enabled, the app starts the overlay server during bootstrap if it is not already running.
- **Teams and talent.** Choosing a team logo samples colours in the WebView2 canvas: the dominant non-neutral colour becomes the team background and a contrasting text colour is selected. **Separate logo background** is off by default; when off, the projected `logoBgColor` uses the team background colour. Teams show a player-card preview in the roster flow. Talent selection is edited per match assignment, where the on-air role is chosen from Caster, Host, Analyst, Interviewer or Other.
- **Match Controller.** The page has five tabs: **Match, Maps & Bans, Talent, Map Pool, Config**. Map selection and hero bans are merged; hero ban pickers appear only when `settings.display.show_hero_bans` is enabled. **Loser picks** and **Hero bans** toggles live in Config. The Match tab includes an optional **Scheduled start** (`datetime-local`) and a **Match library** panel that can save/load snapshots, export the whole library, or import matches. Timer settings moved out of this view into the floating timer widget.
- **Match library.** Export/import uses portable versioned JSON (`schema_version`, `matches`) written by `fgc_core::write_match_library` and read by `fgc_core::read_match_library`. Import clamps `first_to` to 1–4, drops duplicate IDs, refuses a newer schema (`CoreError::InvalidMatchLibrary`), and adds only matches whose IDs are not already present. Saved matches keep their map pools and scheduled times; importing does not replace teams, talent or assets.
- **Timer.** Settings no longer contains timer controls. The app shell shows one floating timer widget on every destination; it is now the only timer control. It shares `TimerController` run state and broadcasts `TimerActionDto` start/pause/stop actions.
- **Tournament.** Tabs are conditional by format: single/double elimination shows **Setup, Board, Settings**, round robin shows **Setup, Groups, Settings**, and multi-stage shows all four. Fixtures expose **Push to overlay**.
- **Assets and pickers.** Asset import supports multiple files with per-file error reporting. Image pickers can upload directly into the relevant category and auto-select the newly created asset. Asset thumbnails use a cached data URL keyed by `hash:ext`, blob reads and base64 encoding run in `tokio::task::spawn_blocking`, and gallery cards reuse `AssetThumb` with fixed name/badge layout.
- **Overlay compatibility.** Bundled overlays remain plain HTML at 1920×1080 and drop-in compatible. No overlay message shape or filename changed for these touch-ups.

## Final pre-release timer, talent, tournament and asset decisions (2026-11-18)

**Timer widget is the only control.** The Settings timer card and the Match
Controller **Timer** tab are removed. `FloatingTimer` in `crates/app/src/app.rs`
shows the clock plus icon Start, Pause and Stop actions, with an expandable
settings popup for mode, countdown duration, target, alignment, leading zeroes
and standalone mode. `TimerController` still owns run state at the app root, so
the clock keeps ticking and broadcasting across destinations. Stop returns the
display to the configured starting value rather than only pausing it.

**Talent roles are per assignment, not per person.** `TalentType` is now
`Caster`, `Host`, `Analyst`, `Interviewer` or `Other`. Each occupied slot in
`MatchTalent` stores a `role`; the same talent can have different roles in
different matches. `panelist` is accepted as an alias for `Analyst` and
`observer` as an alias for `Other`. Persisted `schemaVersion` is 2 with
`MIGRATIONS = [migrate_v0_to_v1, migrate_v1_to_v2]`; v2 removes `talents[].type`
and gives every occupied assignment a default role based on its slot when no
legacy person-level role exists. Wire DTO `castersData[].role` uses the same
five slugs, and the fixture files were updated.

**Multi-stage tournaments and Push to overlay.**
`BracketFormat::MultiStage` serialises as `"multi-stage"`.
`generate_multistage()` generates or keeps groups, seeds a playoff bracket from
the configured qualifiers, and restores `MultiStage` if playoff generation
fails. `generate_round_robin()` sets the format to round robin;
`generate_playoff()` leaves the format unchanged. The UI shows **Setup, Board,
Groups, Settings** for multi-stage events. **Push to overlay** loads the
selected group fixture or bracket match into `AppState.current_match`, mapping
`Pending/Live/Completed` to `Pre/Live/Post` and using manual scores.

**Asset thumbnails.** `AssetThumb` resolves URLs through a `use_future` plus
`Signal<Option<String>>`, caches by `hash:ext`, and renders images with
`loading="lazy"` and `decoding="async"`. Picker and card names use `title`
plus a two-line CSS clamp. Runtime verification in the built WebView2 app
showed 34 thumbnails resolved to `data:image/png;base64,…`, all complete with
non-zero natural width, and placeholders returned after navigating away and
back before the async resolution completed. The 2026-11-18 "refresh only when
the old derived text still matches" rule for map modes was replaced after the
A40 owner finding; see the A40 section below.

## Overlay appearance deferred (2026-11-19)

An appearance pass was prepared for the seven overlays, but the owner asked for
it to be reverted and handled in a new session. The shipped HTML files were
restored byte-for-byte from the pre-change backups, and no pass-specific files
or references remain in the pending v1 changes. The existing overlay protocol,
geometry, fonts and filenames ship unchanged.

## A40 — Asset properties, map modes and attached hero icons (2026-11-19)

The owner failed A40 with this exact answer:

> Fail at step 2: the maps don't respect the game mode in the selector. Note that it does update correctly in map pool. Further note, I'm still not happy with the current system of assigning map types, and by extension, hero ban icons, to a map / hero. In the old react application, we use to give a tag to a map to change it's mode, based on the existing mode icons that were imported. Basically like a properties or taxonomy approach.

The chosen scope was **Tags + bug fix (Recommended)**. The map half passed the
owner recheck with one wording change: remove “tag” from the visible label.

The owner then rejected the inline hero ban-icon tag with this exact answer:

> The game mode tag works fine (just maybe remove tag from the label. However, the hero icon doesn't necessarily work well, as it is showing two names for both images, as they are both uploaded as "hero" category. Maybe for heroes, we have the ability to open up an image properties, and upload a hero icon directly onto that page? Since the icon shouldn't get reused for any other hero. This also adds scalability in case we want to add other elements, like videos or animations down the line.

**Game mode is an inline property.** `Asset.game_mode` drives Map Pool and map
slots. Map cards show **Game mode**; Match Controller map slots show **Game
mode** with the placeholder **From map**. `MapSlot.game_mode_manual`
(`gameModeManual` on the wire, `#[serde(default)]`) is false for a derived mode
and true after the owner types a value. `set_map_name()` re-derives from the
selected map and clears the manual flag; `set_map_mode()` marks the value
manual. `set_asset_game_mode()` refreshes non-manual slots in the current and
saved matches. `AssetCatalog` remains the shared lookup.

**Hero ban icons are attached media, not a second library hero.** Hero cards
show **Properties…**. The dialog uploads a file directly and stores it as a
Hero asset with `Asset.attachment = { owner, kind: "heroBanIcon" }`, while
`Asset.icon` keeps pointing at that asset. Attached files are hidden from the
library, hero pickers, ban lists and the `heroIcons`/`heroPortraits` overlay
maps. Replacing or removing the icon deletes the previous attached asset;
deleting the hero deletes all its attachments. Legacy linked library icons are
unlinked but never deleted. `/state` and overlay filenames are unchanged.

`AssetAttachment` and `AssetAttachmentKind` are additive
(`#[serde(default)]`), so no schema version changed. The `kind` field is the
extension point for future attached media such as animations or videos.

Assistant verification passed for both the first tag implementation and the
final properties rework on fresh `dx build --desktop` binaries with CDP 9222.
The final pass confirmed that map slots follow **Game mode** while hand-typed
modes survive until the map name changes, that attached hero icons stay hidden
from the library, pickers and ban lists through upload, replace, remove, rename
and hero deletion, and that dedupe and lazy thumbnail loading still work.
`cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (53 tests) and `dx build --desktop` passed. QA state was
restored from `app-before-a40.json`; the hash is
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144` before and
after checks. The owner recheck found the two issues below; both were fixed and
the owner passed A40 on 2026-11-20.

The rework exposed two `AlreadyBorrowed` panics in `AppStore`: `delete_asset()`
and `set_asset_link()` each held a `state.peek()` borrow across `mutate()`. Both
now copy the values they need and release the read borrow before mutating. The
fix is internal-only; regression tests cover hero deletion and map game-mode
edits through the store. Test-only constructors keep those tests independent of
the running app's background tasks.

**The owner's first A40 recheck answer was:**

> Fail at step 5: Replacing ban icon did override the image, but the preview image did not update until closing and reopening the properties menu. Extra note: the properties button does not fit well on the card, and the text is being truncated, review the placement of the button. Everything else passed

`AssetThumb` now resolves its data URL with `use_resource` against a signal that
mirrors the current `asset` prop. The old `use_future` captured the first prop
value and never reran, which is why only closing and reopening the dialog
remounted it. The hero card action row wraps and gives **Properties…** its own
full-width line; the measured label is no longer truncated. The same static
checks, 53 app tests and `dx build --desktop` passed after the fixes. CDP
confirmed the preview changed from image A to image B without closing the
dialog, the card metrics at 1360×860, all thumbnails, and `/state`
`gameOverlay.heroIcons.Sojourn` using the new attached hash. QA state was
restored to hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`, with all
23 user blobs present and temporary QA media removed. The owner then passed the
recheck in `docs/manual-qa-runbook.md` §A40, so A40 is accepted.

## A17 — Transactional bundle export and flat blob layout (2026-11-20)

The owner failed A17 with this exact answer:

> Fail: Error message "Something needs attention  Could not export the project: asset blob hash mismatch: expected 348188027503e8f22cbfe1bd20604fb09d48e7be21c349c9d1017ce3fb6d4383, got f31394496e59eb16785024dd57985adc8a0253c4755eb30eab91a774fb9c7a66"; no app.json or export.json available; images are copied though in /blobs/blobs/<>; How do I install the app when we haven't made a release yet or committed changes?

Two code decisions followed. First, `export_bundle()` now validates all
referenced blobs before it creates the bundle directory. This mirrors
`import_bundle()` and prevents a partial export when one source blob is corrupt.
Second, export and import call `blob_path(directory, …)` instead of
`blob_path(directory.join("blobs"), …)`; `blob_path()` already adds `blobs`, so
the old call produced `blobs/blobs/...`. The bundle format is now the documented
`blobs/<prefix>/<hash>.<ext>`, with no compatibility shim because v1 has not
shipped.

The corrupt QA `Curro` blob was restored from the source with the expected hash;
the fixture stayed at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`. Tests cover
both the no-partial-export rule and the flat layout. The install question is
answered with the portable `dx build --desktop` folder copied to a second
Windows account/VM; a real installer and tag remain blocked on the release
gates. On recheck the owner answered **Blocked** because no second
account/VM/installer was available, so second-machine import remains unrun and
A17 is not a Pass. On 2026-11-21 the owner explicitly accepted that result for
release with **“Accept A17; authorize tag”**; A17 is blocked-accepted.

## A8 — Hero-bans icon scope and portrait cropping (2026-11-20)

The owner passed A8 with this exact note:

> Pass: note that hero ban images should only apply on game-overlay.html, not on hero-bans.html; hero-bans.html should also ensure to crop images to the correct dimensions to ensure there is no overflow

The note overrides the earlier overlay freeze for `hero-bans.html` only; no
other shipped overlay HTML file was touched. The change is intentionally narrow:
projection controls which URLs each overlay receives, and `hero-bans.html`
controls how its portraits are clipped.

Ban icons are excluded by ID *and* content identity. `icon_asset_ids()` includes
attached icons and legacy `hero.icon` links; `icon_identities()` adds their
`(hash, ext)` pairs. Both `heroIcons` and `heroPortraits` skip those IDs and
identities. This preserves the name-keyed wire contract while stopping a
duplicate Hero record with the icon's bytes from winning the portraits map.
`game-overlay.html` remains the only overlay that applies `heroIcons`;
`hero-bans.html` reads `heroPortraits`.

Cropping is handled in CSS so arbitrary user images cannot change the 1920×1080
layout. Columns and portrait wrappers get `min-width: 0` / `min-height: 0`; the
wrapper clips with `overflow: hidden`; the image is `display: block` and uses
`object-fit: cover` with `object-position: center`. CDP injected a tall and a
wide portrait, confirmed both filled only their slots with no document overflow,
and confirmed the game overlay still used the separate ban icons. Tests
`duplicate_icon_files_do_not_become_hero_portraits`,
`hero_ban_icons_are_confined_to_the_game_overlay` and
`hero_bans_crops_portraits_inside_their_slots` lock in the behaviour.

## A9 — Clearing stale team logos (2026-11-20)

The owner recheck found that `/state` was correct but the live OBS image was
not: a fresh app launch with two logo-less teams still painted two old logos,
and **Clear** left the old image until the OBS source was restarted. The cause
was purely client-side. `game-overlay.html` assigned
`homeLogo.style.backgroundImage` and `awayLogo.style.backgroundImage` only
inside truthy `logoUrl` checks, so `null` never removed a background already
present in the live DOM. A fresh OBS source had no stale background and worked,
which matched the report.

The owner approved a scoped override of the overlay freeze for
`game-overlay.html` only. The fix assigns the background unconditionally: the
URL when present, otherwise `none`. An explicit `null` team object also clears
its slot; an absent team object in a partial frame is left alone to preserve
compatibility with third-party senders. No other overlay HTML file was touched
by this fix.

A placeholder image containing the team's **TAG** or another abbreviation, for
slots with no logo, was noted for the later overlay refresh. It is deliberately
not bundled into this functional fix. The owner rechecked the fix without
refreshing or restarting the OBS source and answered **“Pass.”** A9 is accepted
and M3-6 is Done.

## A32 follow-ups — projection, scheduled timer and map defaults (2026-11-20)

A32 was accepted with five notes; none revoked the Pass. All five were
implemented before `v1.0.0` and verified with CDP on a fresh `dx build --desktop`
binary.

- **Ban icons are now a projection concern.** When
  `settings.display.show_hero_bans` is false, `Projector::scoreboard()` returns
  empty `heroIcons` and `heroBans` for all five identical scoreboard messages,
  while `heroPortraits` is unchanged. No overlay HTML was changed for this fix;
  `hero-bans.html` and `game-overlay.html` remain the only two changed overlays
  in the pending v1 changes, for their earlier A8/A9 fixes.
- **Scheduled timer mode is additive.** `TimerMode::Scheduled` serialises as
  `"scheduled"`. The floating widget reads the current match's `scheduled_at`
  reactively, including when the mode or date changes at runtime, and reuses the
  existing target-countdown tick loop. No second persisted copy of the date was
  introduced.
- **Map count follows the format.** `default_map_slots(first_to)` returns
  1/3/5/7 for Ft1–Ft4 and drives new matches, resets and format changes.
  `resize_maps_for_first_to()` keeps entered maps and user-added tiebreakers and
  removes only trailing empty slots, so the **+ Add map** escape hatch remains.
  A derived game mode on an unnamed slot is not user data and no longer leaves a
  phantom tiebreaker behind; a manually entered mode or a named tiebreaker is
  still preserved. Slots created with **+ Add map** carry the new
  `userAdded` flag so an empty tiebreaker also survives validation and reload.
  The owner's later report of phantom tiebreakers on startup was traced to
  saved slots carrying derived modes on unnamed maps. On load,
  `trim_trailing_empty_maps_for_first_to()` removes only those trailing empty
  slots and records `RepairKind::MapCountNormalized`; it never pads a saved
  project back up to the format count.
- **Map fields use native selects.** Datalists were removed from map name and
  game mode. Every option stays visible alongside **Custom…** and an imported
  value entry; custom text inputs appear only when that option is selected.
- **Field sizing is shared.** Map-slot selects and text inputs use the same
  `2.75rem` border-box height as **Chosen by**. CDP measured name, mode,
  chosen-by and score fields at 44 px.

CDP also confirmed the scheduled countdown decreases each second, hero bans off
projects zero icons/bans with portraits intact, New match gives five slots,
Ft1/Ft2/Ft4/Ft3 give 1/3/7/5, and **+ Add map** grows 5→6. QA `app.json` was
restored to hash
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

## Phantom tiebreakers on startup (2026-11-20)

Before starting A33 the owner reported:

> Before I get to this one, I see that First to 1 still generates a tiebreaker match. Also, when I started the app, there were 7 tiebreaker maps. Unknown if those were remnants after the update or if it was a bug. Please investigate. Once this is complete, reiterate the QA steps and we can continue

The saved QA state had `firstTo: 1` but eight trailing map slots. The first two
slots carried only derived game-mode names (`Push`, `Escort`) on unnamed maps.
The old emptiness check counted a derived mode as user data, so format changes
and earlier loads left those slots behind as phantom tiebreakers.

The fix treats a slot with no name, no manually entered mode, no score, no
completed flag and no chooser as empty even when a derived mode is present.
Format changes use `resize_maps_for_first_to()`; loading uses the trim-only
`trim_trailing_empty_maps_for_first_to()` so saved projects are not padded back
up to the format default. Named tiebreakers and manually entered modes remain
user data and are preserved. `RepairKind::MapCountNormalized` records the load
repair.

CDP verification on a fresh `dx build --desktop` binary:

| Scenario | Result |
|---|---|
| QA `app.json` with `firstTo: 1` and eight legacy slots | Startup showed **1** map slot, not eight. |
| Switch that match to Ft4 then Ft1 | **7** slots then **1** slot; no tiebreaker remained at Ft1. |
| Forced legacy `firstTo: 1` state with eight slots and derived modes on two unnamed maps | Startup normalised to **1** slot. |
| Forced `firstTo: 1` state with `Ilios`, a named `Tiebreaker`, and a manual-mode-only slot | All **3** slots survived with their names and modes. |

QA `app.json` was restored again to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

## A33 + Add map note (2026-11-20)

Owner answer: **“Pass. Note: new map button does not add a new map.”** The A33
Pass stands; the note became A33-1 and was closed before v1.0.0.

The **+ Add map** handler called `insert_map()` correctly, but
`AppStore::mutate()` runs `validate_and_repair()` after every edit. An empty
newly inserted slot had no user data, so the load-time map-count normalisation
removed it in the same mutation. `MapSlot.user_added` (`userAdded` on the wire,
`#[serde(default)]`) now records slots created by **+ Add map**.
`empty_map_slot()` defaults it to `false`, `insert_map()` sets it to `true`, and
`map_slot_is_empty()` treats marked slots as user data. Older files load with
the flag set to `false`.

CDP verification on a fresh `dx build --desktop` binary: restored QA state
started with 1 slot; **+ Add map** grew it to 2; saved `app.json` contained
`userAdded: true` on the new slot; both slots remained after an app restart.
QA `app.json` was restored to
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

## Owner acceptance A34 (2026-11-20)

Owner answer: **“Pass.”** A34 is accepted. The per-match role editor, schema 2
`panelist`/`observer` aliases and picker quick-upload flow all passed the
runbook without notes. Touch-ups 12, 18 and the final talent-role change are
accepted.

## Owner acceptance A35 (2026-11-20)

Owner answer: **“Pass.”** A35 is accepted. Multi-file import, per-file error
reporting, thumbnail rendering and the rename/delete/link regression checks all
passed the runbook without notes. Touch-ups 15, 16 and 17 are accepted.

## Owner acceptance A36 (2026-11-20)

Owner answer: **“Pass.”** A36 is accepted. Conditional tournament tabs, the
multi-stage Groups + Board layout with the locked Board, format changes and
reload persistence all passed the runbook without notes. Touch-up 14 and the
final multi-stage change are accepted.

## Owner acceptance A41 (2026-11-20)

Owner answer: **“Pass.”** A41 is accepted. All seven bundled overlays passed the
human OBS gate at 1920×1080, including data-driven states, colours, icon/mode
links, long names, reconnect/`show`, offline fonts and the one-minute watch.
M9-5 and M9-6 are complete; overlay appearance work remains deferred.

## Current-tree release evidence refresh (2026-11-20)

After the final A41 documentation edits, the full static/release gate set was
re-run on the uncommitted `1.0.0` tree: format, clippy, all workspace tests,
`dx build --desktop`, `git diff --check`, and the overlay diff restriction all
passed. `THIRD_PARTY_NOTICES.md` regeneration matched the committed file. A
fresh NSIS installer and release app output were built with `dx bundle`; the
packaged `--smoke-test` passed; a portable ZIP and `dist\SHA256SUMS.txt` were
generated locally. QA `app.json` remained at
`47CA19074C16CF12BDC3C7737D1A9B404229DC62DC8828D3B477E52CD563A144`.

This evidence closes `docs/build.md` §7 items 1–5 and 9 on the current tree and
marks M9-9 Done. R5–R8 were subsequently accepted with the deviations recorded
below; R9 remains after tagging. No tag was created.

## Owner acceptance R5 (2026-11-20)

Owner answer: **“Pass: note, I did not use a clean VM, as I didn't have one set
up. I recorded it on a separate computer from where the build is taking
place”.** R5 is accepted with that recorded deviation. The install, launch,
server/overlay, uninstall and data-retention checks passed on a separate
computer using the fresh installer in `dist\`. The clean-VM precondition was not
met; the owner explicitly accepted the substitution by answering Pass. M10-4 is
Done. M10-3 remains Partial because the desktop shortcut was not separately
recorded.

## Owner acceptance R6 (2026-11-20)

Owner answer: **“Pass.”** R6 is accepted. The current portable ZIP ran from
`C:\Tools\FGC-Portable\` on a machine with WebView2 and no installed copy: all
five required entries were present, the UI loaded with fonts and styles, all
seven overlays loaded in OBS, and no install entry was created. M10-5 is Done.

## Owner acceptance R7 (2026-11-20)

Owner answer: **“Pass.”** R7 is accepted. Both current artifacts were downloaded
through a browser to a fresh folder; SmartScreen warned as expected for the
unsigned build, identified the app and allowed **Run anyway**; and both
`Get-FileHash` values matched `dist\SHA256SUMS.txt` exactly. M10-7 is Done.

## Owner acceptance R8 (2026-11-20)

Owner answer: **“Pass.”** R8 is accepted. On a VM without WebView2 the installer
added the runtime and the app launched; on a second clean snapshot without
WebView2 the portable build failed with a clear install-WebView2-or-use-installer
message and no silent crash. R1–R8 are now Pass or explicitly accepted; R9 is
after tagging.

## Pre-tag sign-off closeout (2026-11-21)

`README.md` and `docs/release-notes-template.md` were reviewed against the
shipped source and wiki. The obsolete live-bracket "fully editable"/keyboard
movement wording and the release-note "Push to overlay"/editable-slots wording
were replaced with the shipped generation-preview, **Push to match**, live
result editing and documented limitations. M10-9 and `docs/build.md` §7 item 10
are Done; item 12 remains pending until the tag exists. Because README is
packaged in the portable ZIP, both release artifacts and `dist\SHA256SUMS.txt`
were rebuilt from the current tree; the packaged `--smoke-test` and new checksum
comparison passed. No application code or overlay changed after R5–R8.

A final pre-tag sign-off row was added to `docs/manual-qa.md`. R1–R8 are Pass or
explicitly accepted, A41 Passes all seven overlays, all 15 overlay rows are
ticked, and §7 items 1–11 are verified or accepted. R3 remains blocked-accepted,
R5 used an owner-accepted separate computer instead of a clean VM, and A17's
second-machine import remains unrun but was explicitly accepted as
blocked-accepted on 2026-11-21. The owner authorized commit, tag and push in the
same answer; R9 must run after `v1.0.0` is pushed.

## Owner acceptance A17 and release authorization (2026-11-21)

Owner answer: **“Accept A17; authorize tag”**. A17 is accepted for release as a
blocked gate, not as a Pass: export, bundle layout and hash validation are fixed
and assistant-verified, while the second-machine import remains unrun. The owner
authorized committing all uncommitted changes, creating and pushing the
`v1.0.0` tag, and then running R9.

## Release execution and R9 closeout (2026-11-21)

All uncommitted work was committed as `c244d29` (`feat: ship v1.0.0 broadcast
controller`) and pushed to `main`. The annotated tag `v1.0.0` was created with
message `Fruiterra Graphics Controller v1.0.0` and pushed; it points at
`c244d29`. The workspace version is `1.0.0`, so the tag check passed.

Tag push triggered [Release run
44826182532](https://github.com/ThePeachZA/fruiterra-graphics-controller/actions/runs/44826182532).
Every step passed, including format, clippy, tests, third-party notices, the
NSIS bundle, packaged smoke test, checksum generation and publication. The
[v1.0.0 release](https://github.com/ThePeachZA/fruiterra-graphics-controller/releases/tag/v1.0.0)
contains the installer, portable ZIP and `SHA256SUMS.txt`. All three assets were
downloaded; `Get-FileHash` matched the published checksums for installer
`04d97d7f6dacc98f6e93068e1cf5ae693e701e9a298d8b92ca7c72788469ecf3` and
portable ZIP
`17693cd63254633397fb02216c283d31842695e34ecf195aae9b03306eb0433a`. The
`SHA256SUMS.txt` asset itself hashes to
`6f350b4185ee700173508aed28bcd260621a44b822b9fd8487ce9864e9d4f3f5`.

The published notes were checked against the raw GitHub API body so console
encoding could not alter the comparison. They match
`docs/release-notes-template.md` with `<version>` replaced by `1.0.0`, plus the
generated changelog, describe only shipped features and repeat the known
limitations. R9 is **Pass**. The workflow rebuilt the published artifacts, so
their hashes differ from the locally refreshed pre-tag artifacts used for R7;
R9 verified the published set and no application code or overlay changed
between the two sets.

R1–R9 are now closed, and `docs/build.md` §7 items 1–12 are complete. R3 remains
blocked-accepted, R5 used the owner-accepted computer substitution, and A17's
second-machine import remains unrun but explicitly accepted for release. The tag
stays on the release commit; the R9 evidence is recorded in a follow-up
documentation-only commit.
