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
    if std::env::args().any(|argument| argument == "--smoke-test") {
        run_smoke_test();
    }

    let _log_guard = init_logging();
    install_panic_hook();
    if !claim_single_instance() {
        tracing::info!("another instance is already running; exiting");
        return;
    }
    tracing::info!(version = fgc_core::VERSION, "starting application");
    launch();
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
