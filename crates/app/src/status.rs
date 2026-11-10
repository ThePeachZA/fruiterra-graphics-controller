//! Display helpers for the overlay server lifecycle state.

use fgc_server::ServerStatus;

/// Short tone name used in CSS modifier classes.
#[must_use]
pub fn status_tone(status: &ServerStatus) -> &'static str {
    match status {
        ServerStatus::Running { .. } => "running",
        ServerStatus::Starting | ServerStatus::Stopping => "busy",
        ServerStatus::Failed(_) => "failed",
        ServerStatus::Stopped => "stopped",
    }
}

/// Human-readable status text for the status bar and Overlay view.
#[must_use]
pub fn status_text(status: &ServerStatus) -> String {
    match status {
        ServerStatus::Stopped => "Server stopped".to_owned(),
        ServerStatus::Starting => "Starting server…".to_owned(),
        ServerStatus::Running { clients, .. } => match clients {
            1 => "Running · 1 client".to_owned(),
            count => format!("Running · {count} clients"),
        },
        ServerStatus::Stopping => "Stopping server…".to_owned(),
        ServerStatus::Failed(message) => format!("Server failed · {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_status_reports_client_count() {
        let status = ServerStatus::Running {
            port: 3001,
            clients: 2,
        };
        assert_eq!(status_tone(&status), "running");
        assert_eq!(status_text(&status), "Running · 2 clients");
    }

    #[test]
    fn single_client_uses_singular_text() {
        let status = ServerStatus::Running {
            port: 3001,
            clients: 1,
        };
        assert_eq!(status_text(&status), "Running · 1 client");
    }

    #[test]
    fn transitions_share_the_busy_tone() {
        assert_eq!(status_tone(&ServerStatus::Starting), "busy");
        assert_eq!(status_tone(&ServerStatus::Stopping), "busy");
    }

    #[test]
    fn failures_include_the_message() {
        let status = ServerStatus::Failed("port in use".to_owned());
        assert_eq!(status_tone(&status), "failed");
        assert_eq!(status_text(&status), "Server failed · port in use");
    }
}
