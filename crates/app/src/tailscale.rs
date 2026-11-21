//! Tailscale client detection and Serve lifecycle for Remote access.
//!
//! The app never bundles or installs Tailscale. It looks for the user's own
//! CLI and reads only `tailscale status --json` and `tailscale version` for
//! detection. Serve owns two paths: `/fgc` proxies overlay pages and assets,
//! and `/` proxies the WebSocket endpoint that five frozen overlays dial at
//! the root. Stopping removes only those two paths. `tailscale serve reset`
//! is never used because it would discard other services' configuration.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

#[cfg(target_os = "windows")]
const CLI_NAME: &str = "tailscale.exe";
#[cfg(not(target_os = "windows"))]
const CLI_NAME: &str = "tailscale";

/// Path prefix this app owns on Tailscale Serve.
pub const SERVE_PATH: &str = "/fgc";

/// Root path this app owns for the overlays' WebSocket connection.
pub const WS_SERVE_PATH: &str = "/";

/// Both paths installed while Tailscale serving is active.
pub const SERVE_PATHS: [&str; 2] = [SERVE_PATH, WS_SERVE_PATH];

const SERVE_HTTPS_PORT: &str = "443";

/// Result of the last Tailscale Serve operation for this app's paths.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ServeState {
    /// No handlers installed by this app.
    #[default]
    Off,
    /// `/fgc` serves pages and `/` serves the WebSocket endpoint.
    Active,
    /// The last start or stop attempt failed.
    Failed(String),
}

/// What the installed Tailscale client is doing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TailscaleState {
    /// No Tailscale CLI was found.
    NotInstalled,
    /// The client is installed but its backend is not running.
    Stopped,
    /// The client needs an interactive sign-in.
    NeedsLogin,
    /// The client is connected to a tailnet.
    Ready {
        /// MagicDNS name of this device, without the trailing dot.
        dns_name: String,
        /// Whether this device is currently reachable from the tailnet.
        online: bool,
    },
    /// The CLI was found but its output could not be read.
    Unavailable {
        /// Short, non-sensitive reason.
        reason: String,
    },
}

/// Result of looking for and reading the user's Tailscale client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TailscaleStatus {
    /// Path to the CLI, when one was found.
    pub executable: Option<PathBuf>,
    /// Short version string from `tailscale version`.
    pub version: Option<String>,
    /// Current client state.
    pub state: TailscaleState,
}

/// Looks for the CLI on PATH first, then in the standard install folders.
#[must_use]
pub fn find_executable() -> Option<PathBuf> {
    if let Some(found) = find_on_path() {
        return Some(found);
    }
    program_files_roots()
        .into_iter()
        .map(|root| root.join("Tailscale").join(CLI_NAME))
        .find(|candidate| candidate.is_file())
}

fn find_on_path() -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(CLI_NAME))
        .find(|candidate| candidate.is_file())
}

#[cfg(target_os = "windows")]
fn program_files_roots() -> Vec<PathBuf> {
    ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn program_files_roots() -> Vec<PathBuf> {
    Vec::new()
}

/// Builds a CLI command that does not open a console window on Windows.
fn new_cli_command(executable: &Path) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let mut command = Command::new(executable);
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(executable)
    }
}

/// Reads the installed client once. Callers run this on a blocking task.
#[must_use]
pub fn detect() -> TailscaleStatus {
    match find_executable() {
        Some(executable) => detect_from(&executable),
        None => TailscaleStatus {
            executable: None,
            version: None,
            state: TailscaleState::NotInstalled,
        },
    }
}

fn detect_from(executable: &Path) -> TailscaleStatus {
    TailscaleStatus {
        executable: Some(executable.to_path_buf()),
        version: read_version(executable),
        state: match read_status(executable) {
            Ok(value) => classify(&value),
            Err(reason) => TailscaleState::Unavailable { reason },
        },
    }
}

fn read_version(executable: &Path) -> Option<String> {
    let output = new_cli_command(executable).arg("version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

fn read_status(executable: &Path) -> Result<Value, String> {
    let output = new_cli_command(executable)
        .args(["status", "--json"])
        .output()
        .map_err(|error| format!("could not start Tailscale: {error}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        return Ok(value);
    }
    if output.status.success() {
        Err("status output was not valid JSON".to_owned())
    } else {
        Err(format!("Tailscale exited with status {}", output.status))
    }
}

fn classify(value: &Value) -> TailscaleState {
    let backend = value
        .get("BackendState")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let dns_name = value
        .get("Self")
        .and_then(|self_node| self_node.get("DNSName"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .trim_end_matches('.')
        .to_owned();
    let online = value
        .get("Self")
        .and_then(|self_node| self_node.get("Online"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    match backend {
        "Running" => TailscaleState::Ready { dns_name, online },
        "NeedsLogin" | "NeedsLoginURL" | "NeedsMachineAuth" => TailscaleState::NeedsLogin,
        "Stopped" | "NoState" | "BadBackend" => TailscaleState::Stopped,
        "Starting" => TailscaleState::Unavailable {
            reason: "Tailscale is still starting".to_owned(),
        },
        "" => TailscaleState::Unavailable {
            reason: "status did not include a backend state".to_owned(),
        },
        other => TailscaleState::Unavailable {
            reason: format!("unsupported backend state ({other})"),
        },
    }
}

/// Installs both handlers this app owns, forwarding to the local overlay server.
///
/// `/fgc` carries the overlay pages and assets; `/` carries the WebSocket
/// endpoint that five frozen overlays dial at the root. `--bg` lets the CLI
/// return while tailscaled keeps the config, and `--yes` prevents a prompt.
/// Callers must run this on a blocking task. A partial install is rolled back.
pub fn start_serve(executable: &Path, port: u16) -> Result<(), String> {
    for (index, path) in SERVE_PATHS.iter().enumerate() {
        if let Err(error) = set_serve_path(executable, path, Some(port)) {
            for installed in &SERVE_PATHS[..index] {
                let _ = set_serve_path(executable, installed, None);
            }
            return Err(error);
        }
    }
    Ok(())
}

/// Removes only this app's two handlers.
///
/// This deliberately targets individual paths instead of using
/// `tailscale serve reset` or a port-wide `off`, so handlers installed by
/// other software are preserved. A missing handler is treated as success,
/// making teardown safe to call more than once.
pub fn stop_serve(executable: &Path) -> Result<(), String> {
    let mut failure = None;
    for path in SERVE_PATHS {
        if let Err(error) = set_serve_path(executable, path, None)
            && failure.is_none()
            && !missing_handler(&error)
        {
            failure = Some(error);
        }
    }
    failure.map_or(Ok(()), Err)
}

fn set_serve_path(executable: &Path, path: &str, target: Option<u16>) -> Result<(), String> {
    let mut arguments = vec!["serve".to_owned(), "--yes".to_owned()];
    if target.is_some() {
        arguments.push("--bg".to_owned());
    }
    arguments.push(format!("--https={SERVE_HTTPS_PORT}"));
    arguments.push(format!("--set-path={path}"));
    match target {
        Some(port) => arguments.push(format!("http://127.0.0.1:{port}/")),
        None => arguments.push("off".to_owned()),
    }

    let output = new_cli_command(executable)
        .args(&arguments)
        .output()
        .map_err(|error| format!("could not configure Tailscale Serve at {path}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("{path}: {}", serve_error(&output)))
    }
}

fn serve_error(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("Tailscale Serve exited with status {}", output.status)
    } else {
        stderr.to_owned()
    }
}

fn missing_handler(error: &str) -> bool {
    let lowered = error.to_ascii_lowercase();
    lowered.contains("handler does not exist") || lowered.contains("no serve config")
}

/// Short status line for the settings panel.
#[must_use]
pub fn summary(status: &TailscaleStatus) -> String {
    let version = status
        .version
        .as_deref()
        .map_or(String::new(), |value| format!("v{value} — "));
    match &status.state {
        TailscaleState::NotInstalled => "Tailscale is not installed.".to_owned(),
        TailscaleState::Stopped => format!("{version}Tailscale is installed but stopped."),
        TailscaleState::NeedsLogin => format!("{version}Tailscale needs a sign-in."),
        TailscaleState::Ready { dns_name, online } => format!(
            "{version}Connected as {dns_name} ({})",
            if *online { "online" } else { "offline" }
        ),
        TailscaleState::Unavailable { reason } => {
            format!("Could not read Tailscale status: {reason}.")
        }
    }
}

/// Plain-language next step, when there is one.
#[must_use]
pub fn remedy(status: &TailscaleStatus) -> Option<&'static str> {
    match status.state {
        TailscaleState::Ready { .. } => None,
        TailscaleState::NotInstalled => {
            Some("Install Tailscale from tailscale.com/download, then restart the app.")
        }
        TailscaleState::Stopped => Some("Start Tailscale from the tray icon, then use Refresh."),
        TailscaleState::NeedsLogin => {
            Some("Sign in from the Tailscale tray icon, then use Refresh.")
        }
        TailscaleState::Unavailable { .. } => {
            Some("Check the Tailscale client, then use Refresh to try again.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn status(state: TailscaleState) -> TailscaleStatus {
        TailscaleStatus {
            executable: Some(PathBuf::from("tailscale.exe")),
            version: Some("1.104.7".to_owned()),
            state,
        }
    }

    #[test]
    fn running_state_uses_the_magic_dns_name_without_the_trailing_dot() {
        let value = json!({
            "BackendState": "Running",
            "Self": { "DNSName": "sekhmet.example.ts.net.", "Online": true },
        });
        assert_eq!(
            classify(&value),
            TailscaleState::Ready {
                dns_name: "sekhmet.example.ts.net".to_owned(),
                online: true,
            }
        );
    }

    #[test]
    fn login_and_stopped_states_are_reported_separately() {
        assert_eq!(
            classify(&json!({ "BackendState": "NeedsLogin" })),
            TailscaleState::NeedsLogin
        );
        assert_eq!(
            classify(&json!({ "BackendState": "Stopped" })),
            TailscaleState::Stopped
        );
    }

    #[test]
    fn malformed_or_missing_status_never_looks_ready() {
        assert!(matches!(
            classify(&json!({})),
            TailscaleState::Unavailable { .. }
        ));
        assert!(matches!(
            classify(&json!({ "BackendState": "Starting" })),
            TailscaleState::Unavailable { .. }
        ));
    }

    #[test]
    fn missing_handler_errors_are_treated_as_a_successful_teardown() {
        assert!(missing_handler(
            "error: failed to remove web serve: handler does not exist"
        ));
        assert!(missing_handler("No serve config"));
        assert!(!missing_handler("permission denied"));
    }

    #[test]
    fn serve_state_starts_off() {
        assert_eq!(ServeState::default(), ServeState::Off);
    }

    #[test]
    fn summary_and_remedy_describe_every_state() {
        let ready = status(TailscaleState::Ready {
            dns_name: "host.example.ts.net".to_owned(),
            online: true,
        });
        assert!(summary(&ready).contains("v1.104.7"));
        assert!(summary(&ready).contains("host.example.ts.net"));
        assert!(remedy(&ready).is_none());

        let missing = TailscaleStatus {
            executable: None,
            version: None,
            state: TailscaleState::NotInstalled,
        };
        assert_eq!(summary(&missing), "Tailscale is not installed.");
        assert!(remedy(&missing).unwrap().contains("Install"));

        assert!(summary(&status(TailscaleState::Stopped)).contains("stopped"));
        assert!(
            remedy(&status(TailscaleState::Stopped))
                .unwrap()
                .contains("Start")
        );
        assert!(summary(&status(TailscaleState::NeedsLogin)).contains("sign-in"));
        assert!(
            remedy(&status(TailscaleState::NeedsLogin))
                .unwrap()
                .contains("Sign in")
        );
    }
}
