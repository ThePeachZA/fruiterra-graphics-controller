//! Display helpers for the overlay server lifecycle state.

use std::time::Duration;

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

/// Connected-client count for status rows and the Overlay health panel.
#[must_use]
pub fn client_count_text(clients: usize) -> String {
    match clients {
        1 => "1 client".to_owned(),
        count => format!("{count} clients"),
    }
}

/// Friendly age of the last broadcast frame, or `Never` before any frame.
#[must_use]
pub fn broadcast_age_text(age: Option<Duration>) -> String {
    match age {
        None => "Never".to_owned(),
        Some(age) if age < Duration::from_secs(1) => "Just now".to_owned(),
        Some(age) if age < Duration::from_secs(60) => format!("{}s ago", age.as_secs()),
        Some(age) if age < Duration::from_secs(3600) => format!("{}m ago", age.as_secs() / 60),
        Some(age) => format!("{}h ago", age.as_secs() / 3600),
    }
}

/// Human-readable status text for the status bar and Overlay view.
#[must_use]
pub fn status_text(status: &ServerStatus) -> String {
    match status {
        ServerStatus::Stopped => "Server stopped".to_owned(),
        ServerStatus::Starting => "Starting server…".to_owned(),
        ServerStatus::Running { clients, .. } => {
            format!("Running · {}", client_count_text(*clients))
        }
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

    #[test]
    fn client_count_text_handles_zero_one_and_many() {
        assert_eq!(client_count_text(0), "0 clients");
        assert_eq!(client_count_text(1), "1 client");
        assert_eq!(client_count_text(2), "2 clients");
    }

    #[test]
    fn broadcast_age_text_handles_age_boundaries() {
        assert_eq!(broadcast_age_text(None), "Never");
        assert_eq!(
            broadcast_age_text(Some(Duration::from_millis(999))),
            "Just now"
        );
        assert_eq!(broadcast_age_text(Some(Duration::from_secs(1))), "1s ago");
        assert_eq!(broadcast_age_text(Some(Duration::from_secs(59))), "59s ago");
        assert_eq!(broadcast_age_text(Some(Duration::from_secs(60))), "1m ago");
        assert_eq!(
            broadcast_age_text(Some(Duration::from_secs(3599))),
            "59m ago"
        );
        assert_eq!(
            broadcast_age_text(Some(Duration::from_secs(3600))),
            "1h ago"
        );
    }
}
