//! Overlay HTTP and WebSocket server.
//!
//! The server owns no domain state. The UI projects DTOs and sends them as
//! [`Outbound`] values; this crate caches the last payload per message type,
//! fans frames out to OBS Browser Sources, and serves the embedded overlays
//! and assets.
//!
//! ```no_run
//! use fgc_server::{Outbound, ServerConfig, ServerHandle};
//!
//! # fn main() -> Result<(), fgc_server::ServerError> {
//! let handle = ServerHandle::start(ServerConfig::new(0))?;
//! handle.broadcast(Outbound::show(true));
//! let _ = handle;
//! # Ok(())
//! # }
//! ```

mod config;
mod control;
mod embed;
mod error;
mod handle;
mod overrides;
mod range;
mod routes;
mod runtime;
mod ws;

pub use config::{AssetMode, ServerConfig};
pub use control::{
    CONTROL_PATH, ControlAction, ControlBridge, ControlRequest, generate_control_token,
};
pub use embed::{
    BundledAsset, bundled_asset_bytes, bundled_asset_bytes_by_hash, bundled_assets,
    bundled_catalog, embedded_font, embedded_overlay,
};
pub use error::ServerError;
pub use handle::{Outbound, ServerHandle, ServerStatus, StatusWatcher};

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
