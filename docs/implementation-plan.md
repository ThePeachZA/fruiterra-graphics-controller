# Implementation plan — Phase 7

Status: **draft for final approval**. No implementation code starts until this document is approved.

This plan turns the locked Phase 0–6 decisions into an ordered build. Each milestone has a deliverable, tests and an exit criterion. A milestone is not complete because code compiles; it is complete when its tests pass and its exit criterion is met.

---

## 1. Ground rules

1. **Parity first.** Every existing workflow and every overlay message shape is preserved unless a locked decision says otherwise.
2. **One-way dependencies.** `fgc-core` → `fgc-protocol` → `fgc-server` → `fgc-app`. No async runtime, Axum or Dioxus in `core`; no Dioxus in `server`.
3. **No user-data panics.** No `unwrap`/`expect` outside tests and startup. Malformed JSON is repaired or rejected with a readable error.
4. **Tests before wiring.** Domain and projection logic land with unit/golden tests before the UI consumes them.
5. **No Node toolchain.** Plain CSS, local fonts, Rust-only build.
6. **Small, reviewable commits.** One milestone per branch/PR-sized unit; each ends with fmt, clippy and tests green.
7. **Every intentional wire difference is a test expectation**, recorded in `docs/protocol.md`.
8. **No backlog features.** Undo/redo, presets, multiple scenes, bracket overlay and HTTP control API stay out.

---

## 2. Target layout

```text
fgc/
├─ Cargo.toml                     # workspace, shared versions
├─ Cargo.lock                     # committed
├─ rust-toolchain.toml            # pinned stable at M0
├─ Dioxus.toml                    # app + bundle config
├─ LICENSE                        # MIT
├─ README.md
├─ THIRD_PARTY_NOTICES.md
├─ about.toml                     # cargo-about config
├─ .github/workflows/
│  ├─ ci.yml
│  └─ release.yml
├─ crates/
│  ├─ core/
│  │  ├─ Cargo.toml
│  │  ├─ src/{lib,model,store,migrate,validation,assets,error,paths}.rs
│  │  └─ tests/{store,validation,migrate}.rs
│  ├─ protocol/
│  │  ├─ Cargo.toml
│  │  ├─ src/{lib,dto,projection,asset_url,init}.rs
│  │  └─ tests/{fixtures,golden}.rs
│  ├─ server/
│  │  ├─ Cargo.toml
│  │  ├─ build.rs                 # hashes bundled assets into a manifest
│  │  ├─ src/{lib,routes,ws,state,embed,overrides,error,lifecycle}.rs
│  │  ├─ overlays/                # seven HTML files
│  │  └─ assets/
│  │     ├─ bundled/              # maps, heroes, icons, modes, roles, sides, logos
│  │     ├─ fonts/                # Saira 300/400/500/600/900 + OFL.txt
│  │     └─ manifest.json
│  └─ app/
│     ├─ Cargo.toml
│     ├─ src/{main,app,state,theme,logging}.rs
│     ├─ src/views/{overlay,settings,talent,teams,match_controller,tournament,assets}.rs
│     ├─ src/components/          # hand-rolled + vendored primitives
│     ├─ assets/                  # UI CSS, Inter, icons
│     └─ tests/                   # pure logic extracted from views
├─ docs/                          # existing design documents
└─ scripts/                       # local bundle/QA helpers, no Node
```

The exact module split can change during implementation, but the crate boundaries and one-way dependencies do not.

---

## 3. Milestones

### M0 — Workspace and guardrails

**Deliverables**

- Cargo workspace with four crates, shared dependency versions and a committed `Cargo.lock`.
- `rust-toolchain.toml` pinned to the current stable; record the exact version.
- `Dioxus.toml` with app identity, icon and Windows bundle settings.
- `.github/workflows/ci.yml` running fmt, clippy, test and doc build.
- `LICENSE` (MIT), README skeleton, `THIRD_PARTY_NOTICES.md` skeleton, `about.toml`.
- `dioxus-primitives` git revision resolved and pinned; record the rev and licence in the notices.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all pass on an empty workspace.

**Exit criterion:** CI is green on an empty but complete workspace, and `dx bundle --package-types nsis` produces a hello-world NSIS installer. The portable ZIP is built from the `dx` app output (exe + `assets\`).

---

### M1 — Core domain, validation and store

**Deliverables**

- Domain types from `docs/data-model.md`: teams, players, match, maps, bans, talent, assets, tournament shapes, settings.
- Strong ID newtypes and enums; `serde` only for persistence.
- `Store` trait, `JsonStore` with temp-file + rename + one backup, `MemoryStore`.
- `schemaVersion`, migration chain, unknown-newer-version refusal.
- Validation and repair table from `docs/data-model.md` §4.
- Content-addressed asset metadata helpers: hash validation, extension allow-list, deterministic bundled IDs.
- Path helpers for `%APPDATA%\Fruiterra\Fruiterra Graphics Controller`.

**Tests**

- Store round-trip, atomic-save failure (temp file left behind, live file intact), backup rotation.
- Every validation/repair row has a test.
- Migration from an empty/old fixture and refusal of a newer schema.
- Hash/extension validation rejects traversal-like and malformed input.
- Tournament shape round-trips without standings.

**Exit criterion:** core has no async/Axum/Dioxus dependency and all tests pass.

---

### M2 — Protocol DTOs, projection and golden tests

**Deliverables**

- Wire DTOs for `scoreboard`, `gameOverlay`, `mapSelect`, `heroBans`, `teamRoster`, `casters`, `matchTimer`, `show`, `init`.
- Projection functions and `AssetResolver` (`served` and `data_url` modes).
- Keyed `init` map built from a section cache.
- Golden fixtures captured from the original app; parsed-JSON comparison.
- Quirk tests: five identical scoreboard payloads, `castersData` wrapper, `roleIcons` dual keys, lower-case mode keys, case-sensitive hero keys, `mapPoolData` ordering, raw 0-based `currentMapIndex`.

**Tests**

- `scoreboard_full`, `scoreboard_minimal`, `init_after_broadcast`, `init_empty`, `casters`, `timer_tick`, `timer_action`, `show`.
- Asset URL tests: `/assets/<hash>.<ext>`, stable font paths, absolute base URL.
- `data_url` fallback test with a tiny fixture.
- Projection performs no file I/O in `served` mode (verified by construction, not timing).

**Exit criterion:** every fixture passes, and any intentional difference is named and documented in `docs/protocol.md`.

---

### M3 — Server

**Deliverables**

- Axum app with `/overlays/<file>.html`, `/assets/<hash>.<ext>`, `/assets/fonts/<name>.woff2`, `/health`, `/state`.
- WebSocket at `/` with `init` on connect, `requestInit`, broadcast fan-out and lag recovery.
- `SectionCache`, `tokio::broadcast<Arc<str>>`, serialised-once payloads.
- Dedicated OS thread + multi-thread tokio runtime; command channel; `ServerHandle::{start,broadcast,rebroadcast_all,status,stop,restart}`.
- `rust-embed` overlays/assets; per-file override folder; path validation.
- `build.rs` hashing bundled assets into an embedded manifest; fail on missing/unsupported files.
- Bind errors typed and surfaced; graceful shutdown.

**Tests**

- Route tests on an ephemeral port.
- WS connect → `init`; `requestInit`; five scoreboard keys; `casters` key.
- Broadcast reaches multiple clients; lagging client receives a fresh `init`.
- Start/stop/restart and bind-failure status.
- Overlay override changes one file only; traversal attempts rejected.
- Asset hash/extension rejection; immutable cache headers; ETag.
- Embedded asset manifest is deterministic across two builds.

**Exit criterion:** the server crate has no Dioxus dependency and the packaged smoke test can run against it.

---

### M4 — App shell, state and server integration

**Deliverables**

- Dioxus desktop entry point, logging, panic boundary and single-instance guard (if needed).
- Seven-destination navigation matching the original: Overlay, Settings, Talent, Teams, Match Controller, Tournament, Assets.
- `AppStore` over `Signal<AppState>`; mutations go through methods that validate, mark dirty and project.
- Saver loop (500 ms) and shutdown save.
- `ServerHandle` integration: start/stop/restart, port/LAN/asset-mode changes, status display.
- Theme tokens, Inter, base components (Button, Panel, Input, Select, Toggle, Tabs, Modal, ColorPicker).
- Vendored primitives with licence headers.
- Error toasts and a `last_error` surface.

**Exit criterion:** app launches, loads/saves state, starts and stops the server, and displays live status.

---

### M5 — Assets view and asset pipeline

**Deliverables**

- Asset library UI by category with search, rename, delete and preview.
- Import from file (`rfd`) and URL (`reqwest`), size/type limits, SVG sanitisation.
- Content-addressed storage; duplicate bytes reused; replace only changes the reference.
- GC on startup and after save; “Unused asset data” count and cleanup button.
- Export/import bundle with referenced user blobs and hash validation.
- “Open overlays folder” action and per-file override documentation.

**Tests**

- Import dedupe, replace keeps old asset valid, GC never removes embedded or referenced blobs.
- Export/import round-trip on a second temporary directory.
- Invalid URL, oversize file, unsupported extension, unsafe SVG rejected.

**Exit criterion:** AS1–AS6 pass, and no `_temp` directory exists anywhere.

**Implementation notes**

- Bundled assets appear in the library but are read-only; rename/delete actions are shown only for user assets.
- Exports create a fresh `fruiterra-project-<unix>/` subfolder beneath the chosen parent and contain `app.json`, `export.json` and the referenced user blobs under `blobs/`.
- GC after save uses the union of the saved snapshot and the current in-memory state; startup GC runs only when the state loaded successfully.
- The asset library lives in `crates/app/src/views/assets.rs`; native dialogs and URL fetching live in `crates/app/src/assets_io.rs`; hashing, GC and bundle I/O live in `crates/core/src/asset_store.rs`.

---

### M6 — Teams and Talent

**Deliverables**

- Team CRUD: name, tag, record, colours, logo, roster.
- Player CRUD: role, position, portrait, featured hero, pronouns.
- Talent CRUD: display name, real name, pronouns, social, portrait.
- Team deletion guard with reference list and “remove from tournaments and delete”.
- Asset pickers using the hashed library.

**Tests**

- CRUD and validation; duplicate IDs regenerated.
- Delete blocked while referenced; cleanup path removes references.
- Roster ordering and role/position values survive round-trip.

**Exit criterion:** a full team, roster and talent set can be created, edited, saved and reloaded.

**Implementation notes**

- Team, player and talent edits go through `AppStore` mutations, so every change validates, marks the state dirty and reprojects the overlay payloads immediately.
- Cross-entity reference discovery lives in `fgc-core::references`; the Teams deletion dialog lists every tournament and current-match reference, and “remove references and delete” clears entrants, bracket slots/winners, groups, group fixtures, champions and the current match before removing the team.
- Deleting talent clears any current-match caster, panel or interviewer assignment that used them.
- The reusable `AssetPicker` merges the bundled and user catalogues, filters by category and name, and previews images through the same data-URL path as the asset library.
- Roster order is the stored `players` vector order; role and position are chosen from the shared core enums, so the saved JSON uses the documented lowercase spellings.

---

### M7 — Match Controller

**Deliverables**

- Seven tabs: Match, Map Selection, Hero Bans, Talent, Map Pool, Timer, Config.
- Match setup: teams, title, subtitle, first-to, status, side, swap sides, loser picks, portraits.
- Map selection: add/remove/reorder maps, mode text, scores, completed, chosen-by.
- Hero bans per map with first-ban team.
- Talent assignment: caster1/2, panel1–3, interviewer, interviewee.
- Map pool ordering and grouping.
- Timer: count-up/count-down/target, start/pause/reset, settings, broadcast; control-only.
- Live projection and broadcast; Force Update sends current state.
- Score calculation derived or manual; `currentMapIndex` raw 0-based.
- Overlay URL display and copy.

**Tests**

- Pure score calculation, map-pool grouping and caster ordering.
- Projection changes only when content changes.
- Timer tick/action payload shapes.
- `currentMapIndex` is 0-based in every payload.

**Exit criterion:** all existing Match Controller workflows are usable and the seven overlays update from live changes.

---

### M8 — Tournament

**Deliverables**

- Tournament CRUD and setup: name, format, entrants/seeds, first-to, grand-final reset.
- Single-elimination generation: next power of two, top-seed byes, standard seeding.
- Double-elimination generation: winners, losers, drop mapping, grand final and reset.
- Round-robin groups, fixtures, derived standings and optional playoff generation.
- Full board editing: drag/drop with swap, add/remove matches and rounds, rename, reorder, link editing.
- Auto-advance with per-slot manual override and recursive clear.
- Match modal: slots, scores, status, first-to, winner override, links, reset, delete.
- Configurable ordered tiebreakers with head-to-head mini-tables.
- Champion auto-set with manual override.
- Keyboard slot movement and board zoom.

**Tests**

- Generation for 2–32 entrants in all formats.
- Byes, seeding order, drop mapping and grand-final reset.
- Propagation, manual override, recursive clear and champion rules.
- Tiebreaker order and head-to-head.
- Repair table from `docs/tournament.md` §6.
- Board operations: swap, add/remove round/match, cycle rejection.

**Exit criterion:** T1–T14 pass; a 32-team double-elimination board is usable at 1080p.

---

### M9 — Overlays, fonts and compatibility

**Deliverables**

- Seven overlay HTML files ported with minimal diff: only `@font-face` and family names change.
- Five Saira WOFF2 files, `OFL.txt`, Inter for the UI.
- All assets served from `/assets/<hash>.<ext>`; fonts from stable paths.
- `show` supported server-side but not sent.
- Overlay QA checklist completed at 1920×1080.
- Any protocol quirk discovered here is fixed in `protocol` tests first, then the overlay.

**Tests**

- Static test that each overlay subscribes to the expected message types.
- No external `http://`/`https://` references in overlay HTML except the WebSocket endpoint.
- Font files exist, are valid WOFF2 and are referenced by the expected family aliases.
- Manual QA sign-off in `docs/overlay-qa.md`.

**Exit criterion:** AS7–AS10 pass and every overlay renders correctly from `init` and live messages.

---

### M10 — Packaging, release and documentation

**Deliverables**

- Final `Dioxus.toml` bundle config, icon set and `windows_subsystem` feature.
- `release.yml`: tag and manual dispatch, artifact discovery via JSON mode, NSIS installer rename, portable ZIP, checksums, release notes.
- Hidden `--smoke-test` flag and packaged smoke test in CI.
- `README.md` complete; `LICENSE`; `THIRD_PARTY_NOTICES.md` generated and checked.
- Release checklist completed on a clean Windows VM.
- First `v1.0.0` release candidate built locally and via CI.

**Exit criterion:** a tagged CI run publishes the NSIS installer, portable ZIP and checksums; a clean VM installs, launches, runs overlays and uninstalls.

---

## 4. Dependency order

```text
M0 ─► M1 ─► M2 ─► M3 ─► M4 ─► M5 ─► M6 ─► M7 ─► M8
                                      │
                                      ├─► M9 (can start once M3 and M7 payloads are stable)
                                      └─► M10 (after M8 and M9)
```

- M9 can overlap M8 only after the protocol payloads and Match Controller projection are stable.
- M10 cannot start until the overlay QA gate passes.
- CI is introduced at M0 and extended at M10; it is never deferred to the end.

---

## 5. Definition of done

A milestone is done when:

1. Its deliverables exist and are wired into the app or server.
2. Its tests pass locally and in CI.
3. `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.
4. No new `unwrap`/`expect` outside tests or startup.
5. User-facing strings and errors are clear and do not leak internal paths.
6. The relevant design document is updated if implementation revealed a gap.
7. No backlog feature was added accidentally.

The project is done when:

- C1–C13 are resolved or explicitly documented.
- T1–T14 pass.
- AS1–AS10 pass.
- The overlay QA checklist passes for all seven files.
- A tagged release publishes both artifacts and checksums.
- The README accurately describes the shipped feature set.

---

## 6. Test strategy

| Layer | What is tested | Where |
|---|---|---|
| Domain | validation, repair, store, migrations, tournament algorithms | `fgc-core` unit/integration tests |
| Protocol | DTO shapes, `init`, quirks, asset URLs, `data_url` fallback | `fgc-protocol` golden tests |
| Server | routes, WS, broadcast, lag recovery, overrides, lifecycle | `fgc-server` integration tests |
| App logic | score calculation, grouping, ordering, board operations | extracted pure modules in `fgc-app` |
| Overlays | message subscriptions, no external URLs, font presence | static tests + manual QA |
| Packaged app | health, WS `init`, embedded overlay/asset | hidden `--smoke-test` in CI |
| Release | Installer install/launch/uninstall, portable run | clean Windows VM checklist |

No automated pixel rendering in CI. The manual QA checklist is the compatibility gate.

---

## 7. Risks and mitigations

| # | Risk | Mitigation |
|---|---|---|
| I1 | Dioxus 0.7 API churn | Pin `=0.7.10`; no alpha features; upgrade only deliberately |
| I2 | `dioxus-primitives` git rev changes or disappears | Pin exact rev; vendor copied components; keep licence headers |
| I3 | Protocol drift from the original | Golden fixtures captured before UI work; parsed-JSON comparison |
| I4 | WebView2 HTML5 drag/drop is inconsistent | Pointer-based drag with keyboard alternative |
| I5 | Asset hashing blocks the UI on large files | Hash off the UI thread; size cap; progress state |
| I6 | Tournament propagation creates cycles or stale slots | Pure functions + repair tests; reject self/cycle links |
| I7 | Overlay QA is manual and easy to skip | Versioned checklist and release gate |
| I8 | Unsigned installer triggers SmartScreen | Documented, checksums, signing step ready |
| I9 | Windows-only CI hides portability issues | Keep core/protocol/server OS-agnostic; no Windows-only code outside paths/bundling |
| I10 | Build time grows with four crates | Workspace split, rust-cache, optional sccache; do not require sccache |

---

## 8. Scope guard

**In v1:** parity workflows, correctness fixes C1–C13, completed Tournament control view T1–T14, hashed assets AS1–AS10, seven overlays, Windows NSIS installer + portable ZIP.

**Not in v1:** undo/redo, presets, multiple OBS scenes, bracket overlay, HTTP/Stream Deck API, timer overlay, macOS/Linux, auto-update, code signing, legacy Electron import, multi-stage playoffs, custom points, match history.

---

## 9. Approval

Approving this plan authorises implementation in milestone order. It does not authorise skipping tests, adding backlog features, or changing locked protocol/schema decisions without a new decision entry.

**First implementation step after approval:** M0 workspace scaffold, then M1 core model and store. No UI or server code before M1 is green.
