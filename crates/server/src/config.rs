//! Server configuration.

use std::path::PathBuf;

use fgc_protocol::DEFAULT_PORT;

/// How the app resolves asset URLs in projected payloads.
///
/// The server always serves `/assets/<hash>.<ext>`; this value is carried for
/// the caller so the app can rebuild its projector when the mode changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetMode {
    /// Absolute `/assets/<hash>.<ext>` URLs.
    #[default]
    Served,
    /// Inline `data:` URLs.
    DataUrl,
}

/// Configuration for one overlay server instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// Port to bind. `0` asks the operating system for an ephemeral port.
    pub port: u16,
    /// Bind to all interfaces instead of loopback.
    pub allow_lan: bool,
    /// Asset URL mode carried for the caller.
    pub asset_mode: AssetMode,
    /// Optional per-file overlay override directory.
    pub overlay_override_dir: Option<PathBuf>,
    /// Optional root of the content-addressed user blob store.
    pub blob_dir: Option<PathBuf>,
    /// Broadcast ring-buffer capacity. Small values make lag recovery testable.
    pub broadcast_capacity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            allow_lan: false,
            asset_mode: AssetMode::Served,
            overlay_override_dir: None,
            blob_dir: None,
            broadcast_capacity: 256,
        }
    }
}

impl ServerConfig {
    /// Creates a loopback configuration for `port`.
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            port,
            ..Self::default()
        }
    }

    /// Sets whether the server binds to all interfaces.
    #[must_use]
    pub fn with_lan(mut self, allow_lan: bool) -> Self {
        self.allow_lan = allow_lan;
        self
    }

    /// Sets the asset URL mode carried for the caller.
    #[must_use]
    pub fn with_asset_mode(mut self, asset_mode: AssetMode) -> Self {
        self.asset_mode = asset_mode;
        self
    }

    /// Sets the per-file overlay override directory.
    #[must_use]
    pub fn with_overlay_override_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.overlay_override_dir = Some(dir.into());
        self
    }

    /// Sets the user blob store root.
    #[must_use]
    pub fn with_blob_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.blob_dir = Some(dir.into());
        self
    }

    /// Sets the broadcast ring-buffer capacity.
    #[must_use]
    pub fn with_broadcast_capacity(mut self, capacity: usize) -> Self {
        self.broadcast_capacity = capacity;
        self
    }
}
