//! OBS-compatible wire protocol: DTOs, projection and the keyed `init` map.
//!
//! Message shapes are locked in `docs/protocol.md`. The full DTO layer and
//! golden fixtures arrive in milestone M2.

pub use fgc_core::APP_NAME;

/// Default overlay server port.
pub const DEFAULT_PORT: u16 = 3001;

/// Default loopback base URL for served assets and overlays.
#[must_use]
pub fn default_base_url() -> String {
    format!("http://127.0.0.1:{DEFAULT_PORT}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_uses_default_port() {
        assert_eq!(default_base_url(), "http://127.0.0.1:3001");
    }
}
