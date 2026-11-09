//! Typed server errors.
//!
//! Bind failures carry the address that was attempted so the UI can show a
//! clear message without leaking internal paths.

/// Errors raised while starting or stopping the overlay server.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// The listener could not bind to the requested address.
    #[error("failed to bind the overlay server to {address}: {source}")]
    Bind {
        /// Address that was attempted, for example `127.0.0.1:3001`.
        address: String,
        /// Underlying operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// The server thread could not be spawned.
    #[error("failed to start the overlay server thread: {0}")]
    Spawn(#[source] std::io::Error),
    /// The server did not report readiness in time.
    #[error("the overlay server did not start in time")]
    StartTimeout,
    /// The server is not running.
    #[error("the overlay server is not running")]
    NotRunning,
}
