# Architecture — Phase 3 design

Status: **approved**. Implements Q1 (official primitives + selective copy-in), Q2 (4-crate workspace), Q3 (plain CSS + variables), Q4 (UI-authoritative state), Q5 (dedicated server runtime thread).

---

## 1. Workspace layout

```
fgc/
├─ Cargo.toml                  # [workspace] + shared deps
├─ rust-toolchain.toml         # pinned stable
├─ crates/
│  ├─ core/                    # fgc-core
│  │  └─ src/{model,store,migrate,assets,validation,error}.rs
│  ├─ protocol/                # fgc-protocol
│  │  ├─ src/{dto,projection,asset_url}.rs
│  │  └─ tests/{fixtures,golden}.rs
│  ├─ server/                  # fgc-server
│  │  ├─ src/{lib,routes,ws,state,embed,overrides,error}.rs
│  │  ├─ overlays/             # 7 HTML files (embedded)
│  │  └─ assets/               # Saira, Inter, icons, maps, manifest.json (embedded)
│  └─ app/                     # fgc-app  (bin: fruiterra-gc)
│     ├─ src/{main,app,state,views,components,theme}.rs
│     ├─ components/           # vendored via `dx components add`
│     └─ assets/               # UI CSS, Inter, app icons
├─ docs/
└─ .github/workflows/release.yml
```

Dependency direction is strictly one-way:

```
core  ←  protocol  ←  server
  ↑          ↑          ↑
  └──────────┴──────────┴── app
```

`core` has no async runtime, no Axum, no Dioxus. `protocol` has no I/O. `server` has no Dioxus. Only `app` knows about the webview.

---

## 2. Crates

### `fgc-core` — domain, validation, persistence

| Dependency | Why |
|---|---|
| `serde`, `serde_json` | persistence only |
| `thiserror` | typed errors |
| `uuid` (`v4`, `serde`) | IDs |
| `chrono` (`serde`) | `addedAt`, `createdAt` timestamps |
| `tracing` | structured logging |
| `tempfile` (dev/atomic writes) | temp-file + rename |

Public surface:

```rust
pub struct AppState { /* docs/data-model.md */ }

pub trait Store: Send + Sync {
    fn load(&self) -> Result<AppState, CoreError>;
    fn save(&self, state: &AppState) -> Result<(), CoreError>;
}

pub struct JsonStore { /* path, backup path */ }
pub struct MemoryStore { /* tests */ }

pub fn validate_and_repair(state: &mut AppState) -> Vec<Repair>;
pub fn migrate(raw: serde_json::Value) -> Result<AppState, CoreError>;
pub fn app_data_dir() -> Result<PathBuf, CoreError>;   // %APPDATA%\Fruiterra\Fruiterra Graphics Controller
pub fn assets_root() -> Result<PathBuf, CoreError>;    // …/assets
pub fn overlays_override_dir() -> Result<PathBuf, CoreError>;
```

- `app_data_dir()` uses `directories::ProjectDirs::from("com", "Fruiterra", "Fruiterra Graphics Controller")`.
- Atomic save: write `app.json.tmp`, fsync, rename over `app.json`, copy previous to `app.json.bak`.
- Migrations are a `Vec<fn(Value) -> Result<Value>>` indexed by `schemaVersion`.

### `fgc-protocol` — wire DTOs and projection

| Dependency | Why |
|---|---|
| `serde`, `serde_json` | DTO serialisation |
| `fgc-core` | reads the domain |
| `thiserror` | projection errors |

Public surface:

```rust
pub enum Outbound {
    Scoreboard(Box<ScoreboardDto>),   // sent under 5 type names
    Casters(CastersDto),
    Timer(TimerDto),
    Show { visible: bool },
}

pub struct InitMap(BTreeMap<String, serde_json::Value>);

pub fn project_scoreboard(state: &AppState, assets: &AssetResolver) -> ScoreboardDto;
pub fn project_casters(state: &AppState, assets: &AssetResolver) -> CastersDto;
pub fn project_timer(timer: &TimerSnapshot) -> TimerDto;
pub fn init_map(cache: &SectionCache) -> InitMap;

pub struct AssetResolver { /* base_url, data_url_mode */ }
```

- `AssetResolver` is constructed with `base_url` (`http://127.0.0.1:3001` or the LAN URL) and mode (`served` | `data_url`).
- Projection never performs I/O in `served` mode. In `data_url` mode it reads and base64-encodes, matching the original fallback.
- Golden fixtures live in `crates/protocol/tests/fixtures/` and are compared as parsed JSON.

### `fgc-server` — HTTP + WebSocket

| Dependency | Why |
|---|---|
| `axum` 0.8 | HTTP + WS |
| `tokio` 1 (`rt-multi-thread`, `net`, `sync`, `time`, `macros`) | runtime |
| `tokio-util` | `CancellationToken` |
| `tower-http` (`cors`) | CORS `*` |
| `rust-embed` 8 | overlays + bundled assets |
| `sha2` | content hashes (runtime validation + `build.rs` bundled manifest) |
| `mime_guess` | content types |
| `tracing` | logs |
| `fgc-core`, `fgc-protocol` | state + DTOs |
| `base64` (only for `data_url` mode) | fallback |

Public surface:

```rust
pub struct ServerConfig {
    pub port: u16,
    pub allow_lan: bool,
    pub asset_mode: AssetMode,
    pub overlay_override_dir: Option<PathBuf>,
}

pub struct ServerHandle { /* cloneable */ }

impl ServerHandle {
    pub fn start(cfg: ServerConfig) -> Result<Self, ServerError>;
    pub fn broadcast(&self, msg: Outbound);          // non-blocking, coalescing
    pub fn rebroadcast_all(&self);                   // after port change
    pub fn status(&self) -> ServerStatus;            // Stopped|Starting|Running{port,clients}|Stopping|Failed
    pub async fn stop(self) -> Result<(), ServerError>;
    pub async fn restart(self, cfg: ServerConfig) -> Result<Self, ServerError>;
}
```

- Broadcast is serialised **once** into `Arc<str>` and fanned out with `tokio::sync::broadcast::Sender<Arc<str>>` (the tokio broadcast requirement).
- The server keeps a `SectionCache` (keyed map) so `init` and `/state` are always available without touching the UI thread.
- If a WS client lags behind the broadcast buffer, it receives a fresh `init` instead of silently missing frames.
- Static serving: overlays are served from a per-file user override (if present) or embedded HTML; assets are served as `/assets/<hash>.<ext>` from the embedded blob set or the user blob store; fonts use stable `/assets/fonts/<name>.woff2` paths.
- Asset requests accept only a 64-character lowercase hex hash and an allow-listed extension; overlay requests reject `..`, separators and non-`.html` names before any filesystem access (C11).
- `/health` → `{ status, port, overlays, version }`; `/state` → the keyed map.

### `fgc-app` — Dioxus desktop UI

| Dependency | Why |
|---|---|
| `dioxus` 0.7.10 (`desktop`) | UI |
| `dioxus-primitives` (git, pinned rev) | accessible widgets |
| vendored components | styled Dialog/Select/Tabs/etc. |
| `fgc-core`, `fgc-protocol`, `fgc-server` | domain, DTOs, server |
| `tracing`, `tracing-subscriber`, `tracing-appender` | logs |
| `anyhow` | binary-level error context |
| `arboard` | clipboard for overlay URLs |
| `rfd` | native file pickers for asset import/export |
| `reqwest` | import an asset from a URL (10 s timeout, 20 MB cap) |
| `directories` | paths (via core) |

---

## 3. State flow

```
 user action
     │
     ▼
 AppStore (Signal<AppState>)  ──►  validate_and_repair
     │                                   │
     │                                   ├─► mark dirty
     │                                   │
     ▼                                   ▼
 project DTOs  ──► ServerHandle::broadcast     saver loop (500 ms)
     │                                            │
     ▼                                            ▼
 SectionCache (server thread)                 JsonStore::save (atomic)
     │
     ▼
 tokio::broadcast<Arc<str>>  ──►  all WS clients
```

- **UI-authoritative** (Q4): the server never reads the domain. It caches what the UI last projected.
- Mutations go through `AppStore` methods, not ad-hoc signal writes, so validation and projection happen in one place.
- The saver loop checks a `dirty` flag every 500 ms. This replaces the original debounce and survives rapid edits without a timer per keystroke.
- Port/LAN/asset-mode changes rebuild the `AssetResolver` (base URL changes) and trigger `rebroadcast_all()`.

---

## 4. Server lifecycle

```
Stopped ──start──► Starting ──bind ok──► Running ──stop──► Stopping ──► Stopped
                       │                    │
                       └──bind error──► Failed ──dismiss/retry──► Stopped
```

- One dedicated OS thread named `fgc-server`, owning a multi-thread tokio runtime (Q5).
- The UI talks to it through an unbounded `tokio::sync::mpsc` command channel: `Start(cfg)`, `Stop`, `Broadcast(Outbound)`, `RebroadcastAll`, `Status(oneshot)`.
- `Stop` cancels a `CancellationToken`, axum performs a graceful shutdown, the thread joins, and the result is reported through a oneshot.
- The UI never blocks on the join; `stop()` is awaited inside `dioxus::prelude::spawn`.
- A panic in the server thread is caught at the thread boundary, logged, and surfaced as `Failed`.

---

## 5. UI structure

### Navigation (7 destinations, parity)

`Overlay` · `Settings` · `Talent` · `Teams` · `Match Controller` · `Tournament` · `Assets`

Match Controller keeps its 7 tabs: `Match`, `Map Selection`, `Hero Bans`, `Talent`, `Map Pool`, `Timer`, `Config`.

### Component inventory

| Need | Source |
|---|---|
| Dialog / AlertDialog | `dioxus-primitives` + copied styled version |
| Select / Combobox | `dioxus-primitives` + copied styled version |
| Tabs | `dioxus-primitives` + copied styled version |
| Dropdown / Context menu | `dioxus-primitives` + copied styled version |
| Tooltip / HoverCard | `dioxus-primitives` |
| Toast | `dioxus-primitives` + copied styled version |
| Switch / Slider | `dioxus-primitives` |
| ScrollArea / Separator / Badge | `dioxus-primitives` |
| Button, IconButton | hand-rolled (small, themed) |
| Panel / Card, Input, TextArea, NumberInput | hand-rolled |
| ColorPicker | hand-rolled (`<input type="color">` + hex field) |
| ImagePicker, FilePicker | hand-rolled over `rfd` + asset library |
| Bracket board, standings table | hand-rolled |
| Sidebar, status bar, page header | hand-rolled |

Vendored component files keep their upstream MIT/Apache-2.0 headers, and `THIRD_PARTY_NOTICES.md` records the pinned upstream revision.

### Styling (Q3)

- `crates/app/assets/theme.css` defines tokens: colours, spacing, radii, typography, z-index.
- One CSS file per view (`teams.css`, `match-controller.css`, …), imported once at startup.
- Class naming: `fgc-<block>__<element>--<modifier>`; no CSS modules, no Tailwind, no Node.
- Dark theme only in v1; light theme is a backlog item.
- UI font: Inter (OFL) bundled locally with `@font-face`. Overlay font: Saira (OFL), served by the server.

---

## 6. Error handling and logging

| Layer | Strategy |
|---|---|
| `core` | `CoreError` via `thiserror`; no panics on user data |
| `protocol` | `ProjectionError`; a missing asset yields `None`, not an error |
| `server` | `ServerError`; bind failures are typed and shown in the UI |
| `app` | `anyhow` at command boundaries; user-facing failures become a toast + `last_error` |

- No `unwrap`/`expect` outside tests and `main` startup.
- `tracing` spans around: load, save, project, broadcast, server start/stop, WS connect/disconnect.
- Log file: `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\fgc.log`, daily rotation, 7 files kept. `RUST_LOG` overrides the filter.
- Clippy: `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`.

---

## 7. Testing strategy

| Layer | Tests |
|---|---|
| `core` | validation/repair table, store round-trip, atomic-save failure, migrations, asset path safety, bracket algorithms (Phase 4) |
| `protocol` | golden fixtures for every message and `init` variant; field-presence assertions |
| `server` | route tests on an ephemeral port; WS `init` + `requestInit`; broadcast fan-out; lagging client gets `init`; traversal attempts rejected; start/stop/restart |
| `app` | pure logic extracted into testable modules (score calculation, map pool grouping, caster ordering); UI itself is manual |
| Overlays | manual OBS checklist per overlay at 1920×1080; a static test asserts each HTML file subscribes to the expected message types |
| End-to-end | one Windows CI job: `dx bundle`, launch the exe, hit `/health`, connect a WS client, assert `init` |

No automated pixel rendering in CI (risk E3). The manual checklist is versioned in `docs/overlay-qa.md`.

---

## 8. Build and packaging

- `dx bundle --release --desktop --package-types nsis` produces the NSIS Windows installer; the portable ZIP is built from the `dx` app output (`fruiterra-gc.exe` + `assets\`). Details in `docs/build.md`.
- Server overlays and overlay assets are embedded via `rust-embed`, so the server has no loose `public/` folder. UI assets (CSS, fonts, icons) are Dioxus/manganis files read from `<exe_dir>\assets\`; they are not embedded and ship beside the executable in both artifacts.
- WebView2: the NSIS installer embeds the bootstrapper and downloads the evergreen runtime only when missing; the portable ZIP requires it to be present.
- App icon and version metadata in `crates/app/assets/` and `Cargo.toml`.
- GitHub Actions `windows-latest`: fmt → clippy → test → bundle → checksums → upload artifacts → attach to release on tag. Manual dispatch builds artifacts only.
- First release `1.0.0`; unsigned with documented SmartScreen warning.

---

## 9. Risks

| # | Risk | Mitigation |
|---|---|---|
| A1 | `dioxus-primitives` is a git-only dependency and can change | Pin an exact `rev`; vendor copied components; upgrade deliberately |
| A2 | Dioxus 0.7.x API churn before 0.8 stabilises | Pin `=0.7.10`; no alpha features |
| A3 | `rust-embed` debug vs release path behaviour | One `embed` module with `#[cfg(debug_assertions)]` folder fallback; tests cover both |
| A4 | WebView2 missing on locked-down machines | Detect at startup; show a clear message with the bootstrapper link |
| A5 | Server thread panic takes the listener down | Catch at thread boundary, surface `Failed`, allow restart |
| A6 | Clipboard access from WebView2 | Use `arboard` in Rust, not `navigator.clipboard` |
| A7 | Long build times on Windows | Workspace split + `sccache` in CI; `opt-level` tuned for dev deps |
| A8 | Font licence files omitted from the bundle | `OFL.txt` shipped next to the fonts and listed in the README |

---

## 10. Decisions locked

1. Workspace paths and crate names: `fgc-core`, `fgc-protocol`, `fgc-server`, `fgc-app`; binary `fruiterra-gc`.
2. App identity: display name **Fruiterra Graphics Controller**, `ProjectDirs::from("com", "Fruiterra", "Fruiterra Graphics Controller")`, data under `%APPDATA%\Fruiterra\Fruiterra Graphics Controller`.
3. `ServerHandle` command-channel design (unbounded mpsc + oneshot acks).
4. Vendoring copied components under `crates/app/components/` with upstream licence headers.
5. No Node toolchain anywhere in dev or CI.
