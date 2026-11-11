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
**Decision: Saira confirmed** (OFL 1.1, Omnibus-Type) — 9 weights, 7 widths, variable, 885 glyphs, free for commercial use, self-hosting and embedding.
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

**Out of scope for v1:** new features, protocol v2, macOS/Linux, auto-update/code signing, bracket overlay, drag/drop reseeding, forfeits, multi-stage playoffs.

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
**Decision: five static Saira WOFF2 files (300/400/500/600/900) plus `OFL.txt`.**
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
- A missing or broken override affects only that file.

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
