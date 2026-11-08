//! Overlay HTTP and WebSocket server.
//!
//! Routes, broadcast fan-out, embedded overlays and lifecycle management
//! arrive in milestone M3. This crate never depends on the UI framework.

pub use fgc_core::APP_NAME;
pub use fgc_protocol::{DEFAULT_PORT, default_base_url};

/// Returns the default overlay server port.
#[must_use]
pub fn default_port() -> u16 {
    DEFAULT_PORT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_defaults_match_protocol() {
        assert_eq!(default_port(), 3001);
        assert_eq!(default_base_url(), "http://127.0.0.1:3001");
    }
}
