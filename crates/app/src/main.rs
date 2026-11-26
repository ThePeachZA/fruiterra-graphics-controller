#![cfg_attr(
    all(feature = "bundle", target_os = "windows"),
    windows_subsystem = "windows"
)]

//! Fruiterra Graphics Controller — desktop entry point.
//!
//! Sets up logging and the panic hook, configures the native window, then
//! launches the Dioxus desktop application.

mod app;
mod assets_io;
mod components;
mod control;
mod hero_sync;
mod logo_colors;
mod remote_access;
mod server;
mod smoke;
mod state;
mod status;
mod tailscale;
mod theme;
mod timer;
mod views;

use std::backtrace::Backtrace;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder, icon_from_memory, tao::window::Icon};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

/// Window icon, embedded so the packaged executable has no external assets.
const ICON_PNG: &[u8] = include_bytes!("../assets/icons/icon.png");

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--smoke-test") {
        run_smoke_test();
    }

    apply_data_root_override(&arguments);

    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--import-bundle")
    {
        match arguments.get(index + 1) {
            Some(path) => run_import_bundle(path),
            None => {
                eprintln!("--import-bundle needs the bundle folder");
                std::process::exit(2);
            }
        }
    }

    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--import-pack")
    {
        match arguments.get(index + 1) {
            Some(path) => run_import_pack(path),
            None => {
                eprintln!("--import-pack needs the pack folder");
                std::process::exit(2);
            }
        }
    }

    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--sync-heroes")
    {
        match arguments.get(index + 1) {
            Some(path) => run_sync_heroes(path),
            None => {
                eprintln!("--sync-heroes needs the artwork folder");
                std::process::exit(2);
            }
        }
    }

    let _log_guard = init_logging();
    install_panic_hook();
    if !claim_single_instance() {
        tracing::info!("another instance is already running; exiting");
        return;
    }
    tracing::info!(
        version = fgc_core::VERSION,
        data_root = %fgc_core::data_root_description(),
        "starting application"
    );
    launch();
}

/// Points the application at the project folder given by `--data-dir <path>`.
///
/// Without the flag a debug build uses its own `Fruiterra Graphics Controller
/// (dev)` folder, so a development build never overwrites the shipped build's
/// project.
fn apply_data_root_override(arguments: &[String]) {
    let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--data-dir")
    else {
        return;
    };
    let Some(path) = arguments.get(index + 1) else {
        eprintln!("--data-dir needs a folder path");
        return;
    };
    let path = std::path::PathBuf::from(path);
    if let Err(error) = std::fs::create_dir_all(&path) {
        eprintln!("could not create {}: {error}", path.display());
        return;
    }
    fgc_core::set_data_root(path);
}

/// Imports a content pack into the active project folder and exits.
///
/// A content pack holds artwork only. The pack's assets are added to the
/// open project and the teams, talent, tournaments, settings and the current
/// match are left exactly as they are. Every blob is hash-verified before
/// anything is written.
fn run_import_pack(path: &str) -> ! {
    use fgc_core::{AssetStore, JsonStore, Store};

    let directory = std::path::PathBuf::from(path);
    let outcome = (|| -> Result<String, String> {
        let assets_root = fgc_core::assets_root().map_err(|error| error.to_string())?;
        let asset_store = AssetStore::new(assets_root);
        let state_file = fgc_core::state_file().map_err(|error| error.to_string())?;
        let store = JsonStore::new(state_file.clone());
        let mut state = store.load().unwrap_or_default();
        let outcome = asset_store
            .import_content_pack(&state.assets, &directory)
            .map_err(|error| error.to_string())?;
        for asset in &outcome.repaired {
            if let Some(current) = state.assets.iter_mut().find(|a| a.id == asset.id) {
                *current = asset.clone();
            }
        }
        state.assets.extend(outcome.added);
        store.save(&state).map_err(|error| error.to_string())?;
        Ok(format!(
            "Installed {} asset(s) and repaired {} link(s) from the content pack ({} file(s), {} bytes) into {}.",
            outcome.report.added,
            outcome.report.repaired,
            outcome.report.files,
            outcome.report.bytes,
            fgc_core::data_root_description()
        ))
    })();

    match outcome {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("could not import the content pack: {error}");
            std::process::exit(1);
        }
    }
}

/// Syncs hero artwork, hero roles and map game modes from a local artwork
/// folder and the owner's reference sheet, then exits.
///
/// The artwork folder is expected to contain `Icons/Square`,
/// `Icons/Hexagon` and `Transparent` subfolders named after each hero.
fn run_sync_heroes(path: &str) -> ! {
    use fgc_core::{AssetStore, JsonStore, Store};

    let art_dir = std::path::PathBuf::from(path);
    let outcome = (|| -> Result<hero_sync::HeroSyncReport, String> {
        if !art_dir.is_dir() {
            return Err(format!("{} is not a folder", art_dir.display()));
        }
        let csv = hero_sync::fetch_sheet_csv()?;
        let rows = hero_sync::parse_sheet_csv(&csv);
        let assets_root = fgc_core::assets_root().map_err(|error| error.to_string())?;
        let asset_store = AssetStore::new(assets_root);
        let state_file = fgc_core::state_file().map_err(|error| error.to_string())?;
        let store = JsonStore::new(state_file);
        let mut state = store.load().unwrap_or_default();
        let report = hero_sync::sync(&mut state, &asset_store, &art_dir, &rows);
        store.save(&state).map_err(|error| error.to_string())?;
        Ok(report)
    })();

    match outcome {
        Ok(report) => {
            println!("heroes updated: {}", report.heroes_updated);
            println!("icons set: {}", report.icons_set);
            println!("badges set: {}", report.badges_set);
            println!("transparents set: {}", report.transparents_set);
            println!("roles set: {}", report.roles_set);
            println!("map modes set: {}", report.map_modes_set);
            println!("skipped: {}", report.skipped.len());
            for line in &report.skipped {
                println!("  - {line}");
            }
            println!(
                "Synced hero artwork and metadata into {}.",
                fgc_core::data_root_description()
            );
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("Could not sync hero artwork: {error}");
            std::process::exit(1);
        }
    }
}

/// Imports a project bundle into the active project folder and exits.
///
/// The bundle is the folder an export produced: `app.json`, `export.json` and
/// `blobs/`. Every blob is hash-verified before anything is written, and the
/// import replaces the open project.
fn run_import_bundle(path: &str) -> ! {
    use fgc_core::{AssetStore, JsonStore, Store};

    let directory = std::path::PathBuf::from(path);
    let outcome = (|| -> Result<String, String> {
        let assets_root = fgc_core::assets_root().map_err(|error| error.to_string())?;
        let asset_store = AssetStore::new(assets_root);
        let (state, report) = asset_store
            .import_bundle(&directory)
            .map_err(|error| error.to_string())?;
        let state_file = fgc_core::state_file().map_err(|error| error.to_string())?;
        JsonStore::new(state_file)
            .save(&state)
            .map_err(|error| error.to_string())?;
        Ok(format!(
            "Imported {} team(s), {} asset(s) and {} blob file(s) ({} bytes) into {}.",
            state.teams.len(),
            state.assets.len(),
            report.files,
            report.bytes,
            fgc_core::data_root_description()
        ))
    })();

    match outcome {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("Could not import the bundle: {error}");
            std::process::exit(1);
        }
    }
}

/// Claims the single-instance mutex.
///
/// Returns `false` when another process already owns the named mutex, so the
/// second launch exits instead of opening a second window and fighting over the
/// overlay port. The mutex handle is intentionally kept until the process exits.
#[cfg(target_os = "windows")]
fn claim_single_instance() -> bool {
    use std::ptr;

    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    // A local namespace mutex is per user session, which is what we want for a
    // desktop app. The name is stable across launches.
    let name: Vec<u16> = "Local\\FruiterraGraphicsController\0"
        .encode_utf16()
        .collect();

    // SAFETY: `name` is a valid, null-terminated UTF-16 string and the security
    // attributes pointer is null, which `CreateMutexW` accepts.
    let handle = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        // The mutex could not be created. Do not block startup for this.
        tracing::warn!("could not create the single-instance mutex");
        return true;
    }

    // SAFETY: `GetLastError` reads the calling thread's last error code.
    unsafe { GetLastError() != ERROR_ALREADY_EXISTS }
}

/// Claims the single-instance mutex on platforms without a Windows mutex.
#[cfg(not(target_os = "windows"))]
const fn claim_single_instance() -> bool {
    true
}

/// Runs the hidden packaged smoke test and exits with a CI-friendly status.
///
/// This path deliberately avoids the normal file logger and app data so a
/// packaged binary can be checked on a clean machine without creating state.
fn run_smoke_test() -> ! {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    match smoke::run() {
        Ok(()) => {
            tracing::info!("packaged smoke test passed");
            std::process::exit(0);
        }
        Err(error) => {
            tracing::error!(%error, "packaged smoke test failed");
            std::process::exit(1);
        }
    }
}

/// Initialises tracing, writing to a daily file when the log directory is
/// available and falling back to stderr otherwise.
fn init_logging() -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,fgc_app=debug,fgc_core=debug,fgc_protocol=debug,fgc_server=debug")
    });

    let Ok(directory) = fgc_core::logs_dir() else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .init();
        return None;
    };

    if let Err(error) = std::fs::create_dir_all(&directory) {
        eprintln!(
            "could not create the log directory {}: {error}",
            directory.display()
        );
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .init();
        return None;
    }

    let appender = match tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("fruiterra-gc.log")
        .max_log_files(7)
        .build(&directory)
    {
        Ok(appender) => appender,
        Err(error) => {
            eprintln!(
                "could not create the log file in {}: {error}",
                directory.display()
            );
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .init();
            return None;
        }
    };
    let (writer, guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .init();
    tracing::info!(directory = %directory.display(), "logging to file");
    Some(guard)
}

/// Records panics to the log file before the default behaviour terminates the
/// process.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let backtrace = Backtrace::force_capture();
        tracing::error!(panic = %info, %backtrace, "unhandled panic");
        eprintln!("Fruiterra Graphics Controller crashed: {info}");
    }));
}

/// Builds the native window and starts the Dioxus runtime.
fn launch() {
    let window = WindowBuilder::new()
        .with_title(fgc_core::APP_NAME)
        .with_inner_size(LogicalSize::new(1360.0, 880.0))
        .with_min_inner_size(LogicalSize::new(1040.0, 680.0));

    let mut config = Config::new()
        .with_window(window)
        .with_background_color((11, 13, 18, 255))
        .with_disable_context_menu(true);

    if let Ok(icon) = icon_from_memory::<Icon>(ICON_PNG) {
        config = config.with_icon(icon);
    }

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}
