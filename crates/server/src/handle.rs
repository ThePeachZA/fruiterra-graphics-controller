//! Public server handle, status and outbound payloads.
//!
//! The UI never touches the runtime directly. It sends [`Outbound`] values to
//! the server thread and observes lifecycle changes through [`ServerStatus`].

use std::fmt;
use std::sync::Arc;

use fgc_protocol::{Message, ProtocolError};
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use crate::config::ServerConfig;
use crate::control::ControlAction;
use crate::error::ServerError;
use crate::runtime::{self, Command, SharedState};

/// One payload to cache and fan out to overlay clients.
#[derive(Clone, Debug, PartialEq)]
pub enum Outbound {
    /// A `{ "type": kind, "data": ... }` section frame.
    Message {
        /// Message type read by the overlay subscriber.
        kind: String,
        /// Section payload.
        data: serde_json::Value,
    },
    /// The `show` frame, whose `visible` field is top-level.
    Show {
        /// Whether overlays should be visible.
        visible: bool,
    },
}

impl Outbound {
    /// Builds a section frame, serialising the payload once.
    pub fn section<T: Serialize>(
        kind: impl Into<String>,
        payload: &T,
    ) -> Result<Self, ProtocolError> {
        let kind = kind.into();
        let data = serde_json::to_value(payload).map_err(|source| ProtocolError::Serialize {
            message_type: kind.clone(),
            source,
        })?;
        Ok(Self::Message { kind, data })
    }

    /// Builds a section frame from a projected [`Message`].
    pub fn from_message<T: Serialize>(message: &Message<T>) -> Result<Self, ProtocolError> {
        Self::section(message.kind.clone(), &message.data)
    }

    /// Builds the `show` frame.
    #[must_use]
    pub fn show(visible: bool) -> Self {
        Self::Show { visible }
    }
}

/// Lifecycle state of one overlay server instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerStatus {
    /// No listener is running.
    Stopped,
    /// The thread is starting and has not bound yet.
    Starting,
    /// The listener is accepting connections.
    Running {
        /// Actual bound port (useful when port `0` was requested).
        port: u16,
        /// Connected WebSocket clients.
        clients: usize,
    },
    /// The listener is shutting down.
    Stopping,
    /// The server failed; the message is suitable for the UI.
    Failed(String),
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped => formatter.write_str("stopped"),
            Self::Starting => formatter.write_str("starting"),
            Self::Running { port, clients } => {
                write!(formatter, "running on port {port} with {clients} client(s)")
            }
            Self::Stopping => formatter.write_str("stopping"),
            Self::Failed(message) => write!(formatter, "failed: {message}"),
        }
    }
}

/// Cloneable handle to a running overlay server.
///
/// Dropping every clone closes the command channel, which stops the server
/// thread. Call [`ServerHandle::stop`] when the caller needs to await shutdown.
#[derive(Clone)]
pub struct ServerHandle {
    commands: mpsc::UnboundedSender<Command>,
    shared: Arc<SharedState>,
}

impl ServerHandle {
    /// Starts the server thread and waits for the listener to bind.
    pub fn start(config: ServerConfig) -> Result<Self, ServerError> {
        let (commands, shared) = runtime::start(config)?;
        Ok(Self { commands, shared })
    }

    /// Caches and broadcasts one payload without blocking the caller.
    pub fn broadcast(&self, message: Outbound) {
        if self.commands.send(Command::Broadcast(message)).is_err() {
            warn!("cannot broadcast: the overlay server is not running");
        }
    }

    /// Resends every cached section, for example after a port change.
    pub fn rebroadcast_all(&self) {
        if self.commands.send(Command::RebroadcastAll).is_err() {
            warn!("cannot rebroadcast: the overlay server is not running");
        }
    }

    /// Installs the authenticated control bridge and its action receiver.
    ///
    /// The caller keeps the receiver and must apply actions on its own UI
    /// task. Installing a bridge clears replay and rate-limit state.
    pub fn set_control(&self, token: &str, actions: mpsc::UnboundedSender<ControlAction>) {
        self.shared.set_control(token, actions);
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub fn status(&self) -> ServerStatus {
        self.shared.status()
    }

    /// Returns the bound port when the server is running.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        match self.status() {
            ServerStatus::Running { port, .. } => Some(port),
            _ => None,
        }
    }

    /// Returns a handle that can observe status after this handle is consumed.
    #[must_use]
    pub fn watcher(&self) -> StatusWatcher {
        StatusWatcher {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Stops the listener and waits for the server thread to finish.
    ///
    /// Stopping an already stopped server is not an error.
    pub async fn stop(self) -> Result<(), ServerError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        if self.commands.send(Command::Stop(ack_tx)).is_err() {
            return Ok(());
        }
        let _ = ack_rx.await;
        Ok(())
    }

    /// Stops this instance and starts a new one with `config`.
    pub async fn restart(self, config: ServerConfig) -> Result<Self, ServerError> {
        self.stop().await?;
        Self::start(config)
    }
}

/// Status observer that remains usable after [`ServerHandle::stop`].
#[derive(Clone)]
pub struct StatusWatcher {
    shared: Arc<SharedState>,
}

impl StatusWatcher {
    /// Returns the current lifecycle state.
    #[must_use]
    pub fn status(&self) -> ServerStatus {
        self.shared.status()
    }

    /// Returns the bound port when the server is running.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        match self.status() {
            ServerStatus::Running { port, .. } => Some(port),
            _ => None,
        }
    }
}
