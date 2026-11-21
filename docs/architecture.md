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
│  │  └─ assets/               # Archivo/Saira, Inter, icons, maps, manifest.json (embedded)
│  └─ app/                     # fgc-app  (bin: fruiterra-gc)
│     ├─ src/{main,app,state,server,status,theme}.rs
│     ├─ src/views/            # seven destination views
│     ├─ src/components/       # hand-rolled + vendored primitives
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

- `app_data_dir()` uses `directories::ProjectDirs::from("com", "Fruiterra", "Fruiterra Graphics Controller")`. On Windows `ProjectDirs::data_dir()` appends a `data` component, so the helper returns its parent; otherwise the state file would live under `<base>\data\data\app.json`.
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
pub struct Projector<'a> { /* state, catalogue, resolver */ }

impl Projector<'_> {
    pub fn new(
        state: &AppState,
        catalog: &AssetCatalog,
        resolver: &dyn AssetUrlResolver,
    ) -> Self;
    pub fn scoreboard(&self, match_state: &Match) -> ScoreboardDto;
    pub fn scoreboard_messages(&self, match_state: &Match) -> Vec<Message<ScoreboardDto>>;
    pub fn casters(&self, match_state: &Match) -> CastersDto;
    pub fn casters_message(&self, match_state: &Match) -> Message<CastersDto>;
}

pub fn timer_tick(settings: &TimerSettings, hours: u32, minutes: u32, seconds: u32)
    -> Message<TimerPayload>;
pub fn timer_action(action: TimerActionDto) -> Message<TimerPayload>;
pub fn show(visible: bool) -> ShowMessage;

pub struct SectionCache { /* BTreeMap<String, Value> */ }
impl SectionCache {
    pub fn set<T: Serialize>(&mut self, message_type: &str, payload: &T)
        -> Result<(), ProtocolError>;
    pub fn set_value(&mut self, message_type: &str, payload: Value);
    pub fn init(&self) -> Message<BTreeMap<String, Value>>;
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)>;
}

pub trait AssetUrlResolver: Send + Sync { fn url(&self, asset: &Asset) -> Option<String>; }
pub struct ServedAssetResolver { /* base_url */ }
pub struct DataUrlResolver<F> { /* byte loader */ }
```

- `ServedAssetResolver` is constructed with `base_url` (`http://127.0.0.1:3001` or the LAN URL). `DataUrlResolver` takes a caller-supplied byte loader.
- Projection never performs I/O in `served` mode. In `data_url` mode the loader reads and base64-encodes, matching the original fallback.
- `scoreboard_messages` returns the same payload under all five scoreboard type names.
- Golden fixtures live in `crates/protocol/tests/fixtures/` and are compared as parsed JSON.

### `fgc-server` — HTTP + WebSocket

| Dependency | Why |
|---|---|
| `axum` 0.8 (`ws`) | HTTP + WebSocket |
| `tokio` 1 (`rt-multi-thread`, `net`, `sync`, `time`, `macros`, `signal`, `fs`) | runtime |
| `tokio-util` (`rt`) | `CancellationToken` |
| `tower-http` (`cors`) | CORS `*` |
| `rust-embed` 8 | overlays + bundled assets |
| `sha2` | `build.rs` bundled-manifest hashes and test fixtures (build/dev only) |
| `mime_guess` | content types |
| `futures-util` | split WebSocket streams |
| `tracing` | logs |
| `fgc-core`, `fgc-protocol` | state + DTOs |

Public surface:

```rust
pub struct ServerConfig {
    pub port: u16,
    pub allow_lan: bool,
    pub asset_mode: AssetMode,
    pub overlay_override_dir: Option<PathBuf>,
    pub blob_dir: Option<PathBuf>,      // asset-store root containing blobs/
    pub broadcast_capacity: usize,      // ring buffer, default 256
}

pub enum Outbound {
    Message { kind: String, data: serde_json::Value },
    Show { visible: bool },
}

impl Outbound {
    pub fn section<T: Serialize>(kind: impl Into<String>, payload: &T)
        -> Result<Self, ProtocolError>;
    pub fn from_message<T: Serialize>(message: &Message<T>) -> Result<Self, ProtocolError>;
    pub fn show(visible: bool) -> Self;
}

pub struct ServerHandle { /* cloneable */ }

impl ServerHandle {
    pub fn start(cfg: ServerConfig) -> Result<Self, ServerError>;
    pub fn broadcast(&self, msg: Outbound);          // non-blocking
    pub fn rebroadcast_all(&self);                   // resend every cached section
    pub fn status(&self) -> ServerStatus;            // Stopped|Starting|Running{port,clients}|Stopping|Failed
    pub fn port(&self) -> Option<u16>;
    pub fn watcher(&self) -> StatusWatcher;          // status after stop(self)
    pub async fn stop(self) -> Result<(), ServerError>;
    pub async fn restart(self, cfg: ServerConfig) -> Result<Self, ServerError>;
}

pub fn bundled_assets() -> &'static [BundledAsset];
pub fn bundled_catalog() -> Vec<Asset>;
```

- `Outbound` carries an already-serialised JSON value so the UI can reuse the protocol DTOs without the server depending on UI types.
- Broadcast is serialised **once** into `Arc<str>` and fanned out with `tokio::sync::broadcast::Sender<Arc<str>>` (the tokio broadcast requirement).
- The server keeps a `SectionCache` (keyed map) so `init` and `/state` are always available without touching the UI thread.
- If a WS client lags behind the broadcast buffer, it receives a fresh `init` instead of silently missing frames.
- Static serving: overlays are served from a per-file user override (if present) or embedded HTML; assets are served as `/assets/<hash>.<ext>` from the embedded blob set or the user blob store; fonts use stable `/assets/fonts/<name>.woff2` paths.
- Asset requests accept only a 64-character lowercase hex hash and an allow-listed extension; overlay requests reject `..`, separators and non-`.html` names before any filesystem access (C11).
- `/health` → `{ status, port, overlays, version }`; `/state` → the keyed map (not the `init` envelope).
- Asset and font responses send `Cache-Control: public, max-age=31536000, immutable`, an `ETag`, `Accept-Ranges: bytes` and single-range `206` responses. Overlay responses send `Cache-Control: no-cache` so OBS picks up overrides.
- `stop` is idempotent and waits for graceful shutdown (with a bounded abort fallback); dropping every handle closes the command channel and stops the thread.

### `fgc-app` — Dioxus desktop UI

| Dependency | Why |
|---|---|
| `dioxus` 0.7.10 (`desktop`) | UI |
| `dioxus-primitives` (git, pinned rev) | accessible widgets |
| `dioxus-icons`, `palette` | icons and colour maths for the vendored widgets |
| vendored components | styled Button/Input/Select/Switch/Tabs/Dialog/Toast/etc. |
| `fgc-core`, `fgc-protocol`, `fgc-server` | domain, DTOs, server |
| `tokio` | server lifecycle tasks and the status poller |
| `tracing`, `tracing-subscriber`, `tracing-appender` | logs |
| `anyhow` | binary-level error context |
| `directories` | paths (via core) |

`rfd` and `reqwest` arrive with asset import in M5; overlay URL copying uses the WebView clipboard API through `document::eval` with an `execCommand` fallback.

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
- The UI talks to it through an unbounded `tokio::sync::mpsc` command channel: `Broadcast(Outbound)`, `RebroadcastAll`, `Stop(oneshot)`. Starting creates the thread and binds before returning; restart stops the previous listener first.
- `Stop` cancels a `CancellationToken`, axum performs a graceful shutdown, the thread joins, and the result is reported through a oneshot.
- The UI never blocks on the join; `stop()` is awaited inside `dioxus::prelude::spawn`.
- A panic in the server thread is caught at the thread boundary, logged, and surfaced as `Failed`.
- Overlay health display uses existing state: the connected-client count comes from the 500 ms `ServerStatus` poll, while `ServerController` keeps a session-only `last_broadcast: Signal<Option<Instant>>`. It is set only when a frame is actually sent (including the initial start/restart frames), cleared on start/restart/stop, and aged by a one-second clock in the Overlay view. Nothing is written to `AppState`, `app.json`, the protocol, the server runtime or an endpoint; `/health` is unchanged.

---

## 5. UI structure

### Navigation (7 destinations, parity)

`Overlay` · `Settings` · `Talent` · `Teams` · `Match Controller` · `Tournament` · `Assets`

Match Controller keeps its 7 tabs: `Match`, `Map Selection`, `Hero Bans`, `Talent`, `Map Pool`, `Timer`, `Config`.

### Component inventory

| Need | Source |
|---|---|
| Button, Input, Textarea, Label | vendored styled component |
| Select / Combobox | vendored styled component; enum settings use a native `<select>` |
| Switch | vendored styled component |
| Tabs, Dialog, ColorPicker, Card | vendored styled component |
| Toast | `dioxus-primitives` + vendored styled version |
| Panel / Field / ToggleRow / KeyValues | hand-rolled |
| Sidebar, status bar, page header | hand-rolled |
| ImagePicker, FilePicker | M5 over `rfd` + asset library |
| Bracket board, standings table | M8 |

Vendored component files keep their upstream MIT/Apache-2.0 headers, and `THIRD_PARTY_NOTICES.md` records the pinned upstream revision.

### Styling (Q3)

- `crates/app/assets/theme.css` defines tokens: colours, spacing, radii, typography, z-index.
- `app.css` owns the shell, `components.css` owns the hand-rolled panel/form primitives, and `dx-components-theme.css` carries the vendored widget tokens.
- Per-view CSS (`overlay.css`, `settings.css`, `placeholder.css`, and later views) is linked once at startup.
- Vendored component CSS modules are registered through `#[css_module]` and bundled by `dx`.
- Class naming: `fgc-<block>__<element>--<modifier>`; no CSS modules for app styles, no Tailwind, no Node.
- Compact navigation is session-only: the flag lives in a shell signal because `AppState::settings` holds persisted broadcast/overlay/timer configuration, so storing a view preference there would change `app.json`. Labels stay in the DOM as clipped text and every nav button keeps `aria-label`/`title`.
- Live overlay visibility is session-only for the same reason: `AppStore` keeps an ephemeral `overlay_visible` signal, broadcasts the existing `Outbound::Show { visible }` on change, and seeds every start/restart plan with the current value. Hidden state survives a server restart but never enters `AppState` or `app.json`; a fresh app launch starts visible.
- Dark theme only in v1; light theme is a backlog item.
- UI font: Inter (OFL) bundled locally with `@font-face`. Overlay font: Archivo (OFL), selected in R1 and served by the server; Saira stays bundled as a reversible fallback.

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
- Log file: `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\logs\fruiterra-gc.log`, daily rotation, 7 files kept. `RUST_LOG` overrides the filter.
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
