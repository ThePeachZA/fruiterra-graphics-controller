//! Server thread, shared state and command loop.
//!
//! One dedicated OS thread named `fgc-server` owns a multi-thread Tokio
//! runtime. The UI communicates through an unbounded command channel; all
//! payloads are serialised once here and fanned out through
//! `tokio::sync::broadcast<Arc<str>>`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use fgc_protocol::{Message, ProtocolError, SectionCache, ShowMessage};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::config::ServerConfig;
use crate::control::{ControlAction, ControlBridge, ControlGuard};
use crate::error::ServerError;
use crate::handle::Outbound;

/// Commands accepted by the server thread.
pub(crate) enum Command {
    /// Cache and fan out one payload.
    Broadcast(Outbound),
    /// Resend every cached section.
    RebroadcastAll,
    /// Gracefully stop the listener and acknowledge when finished.
    Stop(oneshot::Sender<()>),
}

/// Lifecycle marker stored behind a mutex.
#[derive(Clone, Debug)]
enum Lifecycle {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

/// State shared by the command loop, HTTP routes and WebSocket handlers.
pub(crate) struct SharedState {
    /// Last payload per message type, used for `init`, `requestInit` and
    /// `/state`.
    pub(crate) cache: RwLock<SectionCache>,
    /// Fan-out channel. Each frame is an `Arc<str>` so cloning is cheap.
    pub(crate) tx: broadcast::Sender<Arc<str>>,
    /// Connected WebSocket clients.
    pub(crate) clients: AtomicUsize,
    /// Actual bound port, which differs from the request when port `0` is
    /// used.
    pub(crate) port: AtomicU16,
    /// Port requested in the configuration.
    requested_port: u16,
    /// Whether the listener binds all interfaces.
    allow_lan: bool,
    /// Optional per-file overlay overrides.
    pub(crate) overlay_override_dir: Option<PathBuf>,
    /// Optional content-addressed asset-store root (the directory containing
    /// `blobs/`).
    pub(crate) blob_dir: Option<PathBuf>,
    /// Cancelled to close WebSocket clients and stop the listener.
    pub(crate) shutdown: CancellationToken,
    /// Authenticated control bridge, installed by the application.
    control: RwLock<Option<Arc<ControlBridge>>>,
    /// Replay and rate-limit state for the control endpoint.
    control_guard: ControlGuard,
    /// Lifecycle marker.
    status: Mutex<Lifecycle>,
}

impl SharedState {
    fn new(config: ServerConfig) -> Self {
        let (tx, _) = broadcast::channel(config.broadcast_capacity.max(1));
        Self {
            cache: RwLock::new(SectionCache::new()),
            tx,
            clients: AtomicUsize::new(0),
            port: AtomicU16::new(0),
            requested_port: config.port,
            allow_lan: config.allow_lan,
            overlay_override_dir: config.overlay_override_dir,
            blob_dir: config.blob_dir,
            shutdown: CancellationToken::new(),
            control: RwLock::new(None),
            control_guard: ControlGuard::default(),
            status: Mutex::new(Lifecycle::Starting),
        }
    }

    fn status_lock(&self) -> std::sync::MutexGuard<'_, Lifecycle> {
        match self.status.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn set_running(&self) {
        *self.status_lock() = Lifecycle::Running;
    }

    fn set_stopping(&self) {
        *self.status_lock() = Lifecycle::Stopping;
    }

    fn set_stopped(&self) {
        *self.status_lock() = Lifecycle::Stopped;
    }

    fn set_failed(&self, message: impl Into<String>) {
        *self.status_lock() = Lifecycle::Failed(message.into());
    }

    /// Snapshot of the lifecycle for the public handle.
    pub(crate) fn status(&self) -> crate::handle::ServerStatus {
        let lifecycle = self.status_lock().clone();
        match lifecycle {
            Lifecycle::Starting => crate::handle::ServerStatus::Starting,
            Lifecycle::Running => crate::handle::ServerStatus::Running {
                port: self.port.load(Ordering::SeqCst),
                clients: self.clients.load(Ordering::SeqCst),
            },
            Lifecycle::Stopping => crate::handle::ServerStatus::Stopping,
            Lifecycle::Stopped => crate::handle::ServerStatus::Stopped,
            Lifecycle::Failed(message) => crate::handle::ServerStatus::Failed(message),
        }
    }

    /// Serialises the current keyed `init` frame.
    pub(crate) fn init_frame(&self) -> Result<String, ProtocolError> {
        let cache = match self.cache.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        serde_json::to_string(&cache.init()).map_err(|source| ProtocolError::Serialize {
            message_type: "init".to_owned(),
            source,
        })
    }

    /// Serialises the keyed state map returned by `GET /state`.
    pub(crate) fn state_frame(&self) -> Result<String, ProtocolError> {
        let cache = match self.cache.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        serde_json::to_string(&cache.init().data).map_err(|source| ProtocolError::Serialize {
            message_type: "state".to_owned(),
            source,
        })
    }

    /// Sends one already-serialised frame to every subscriber.
    pub(crate) fn send(&self, frame: String) {
        let _ = self.tx.send(Arc::from(frame));
    }

    /// Installs or replaces the authenticated control bridge.
    pub(crate) fn set_control(&self, token: &str, actions: mpsc::UnboundedSender<ControlAction>) {
        let bridge = Arc::new(ControlBridge::new(token, actions));
        match self.control.write() {
            Ok(mut guard) => *guard = Some(bridge),
            Err(poisoned) => *poisoned.into_inner() = Some(bridge),
        }
        self.control_guard.clear();
    }

    /// Returns the installed control bridge, if the app has configured one.
    pub(crate) fn control_bridge(&self) -> Option<Arc<ControlBridge>> {
        match self.control.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Returns the shared replay and rate-limit state.
    pub(crate) fn control_guard(&self) -> &ControlGuard {
        &self.control_guard
    }
}

/// Starts the server thread and waits until the listener is bound.
pub(crate) fn start(
    config: ServerConfig,
) -> Result<(mpsc::UnboundedSender<Command>, Arc<SharedState>), ServerError> {
    let shared = Arc::new(SharedState::new(config));
    let (commands, command_rx) = mpsc::unbounded_channel();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    let thread_shared = Arc::clone(&shared);
    std::thread::Builder::new()
        .name("fgc-server".to_owned())
        .spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => {
                        runtime.block_on(run(thread_shared.clone(), command_rx, ready_tx));
                    }
                    Err(source) => {
                        thread_shared
                            .set_failed(format!("failed to build the server runtime: {source}"));
                        let _ = ready_tx.send(Err(ServerError::Spawn(source)));
                    }
                }
            }));
            if outcome.is_err() {
                error!("the overlay server thread panicked");
                thread_shared.set_failed("the overlay server thread panicked");
            }
        })
        .map_err(ServerError::Spawn)?;

    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(_port)) => Ok((commands, shared)),
        Ok(Err(error)) => Err(error),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            drop(commands);
            Err(ServerError::StartTimeout)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(ServerError::StartTimeout),
    }
}

async fn run(
    shared: Arc<SharedState>,
    mut commands: mpsc::UnboundedReceiver<Command>,
    ready: std::sync::mpsc::Sender<Result<u16, ServerError>>,
) {
    let address = if shared.allow_lan {
        format!("0.0.0.0:{}", shared.requested_port)
    } else {
        format!("127.0.0.1:{}", shared.requested_port)
    };

    let listener = match TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(source) => {
            let error = ServerError::Bind { address, source };
            shared.set_failed(error.to_string());
            let _ = ready.send(Err(error));
            return;
        }
    };

    let port = match listener.local_addr() {
        Ok(address) => address.port(),
        Err(source) => {
            let error = ServerError::Bind { address, source };
            shared.set_failed(error.to_string());
            let _ = ready.send(Err(error));
            return;
        }
    };

    shared.port.store(port, Ordering::SeqCst);
    shared.set_running();
    if ready.send(Ok(port)).is_err() {
        warn!("overlay server bound after the caller stopped waiting");
    }

    let shutdown = shared.shutdown.clone();
    let app = crate::routes::router(Arc::clone(&shared));
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await
    });

    loop {
        match commands.recv().await {
            Some(Command::Broadcast(outbound)) => apply_outbound(&shared, outbound),
            Some(Command::RebroadcastAll) => rebroadcast_all(&shared),
            Some(Command::Stop(ack)) => {
                stop(shared, server).await;
                let _ = ack.send(());
                return;
            }
            None => {
                stop(shared, server).await;
                return;
            }
        }
    }
}

async fn stop(shared: Arc<SharedState>, mut server: JoinHandle<Result<(), std::io::Error>>) {
    shared.set_stopping();
    shared.shutdown.cancel();
    match tokio::time::timeout(Duration::from_secs(2), &mut server).await {
        Ok(result) => finish(&shared, result),
        Err(_) => {
            warn!("overlay server did not shut down in time; aborting the listener task");
            server.abort();
            let _ = server.await;
            shared.set_stopped();
        }
    }
}

fn finish(
    shared: &SharedState,
    result: Result<Result<(), std::io::Error>, tokio::task::JoinError>,
) {
    match result {
        Ok(Ok(())) => shared.set_stopped(),
        Ok(Err(source)) => {
            error!(%source, "overlay server stopped with an error");
            shared.set_failed(source.to_string());
        }
        Err(source) => {
            error!(%source, "overlay server task failed");
            shared.set_failed(source.to_string());
        }
    }
}

fn apply_outbound(shared: &SharedState, outbound: Outbound) {
    match outbound {
        Outbound::Message { kind, data } => {
            let frame = match serde_json::to_string(&Message::new(kind.clone(), data.clone())) {
                Ok(frame) => frame,
                Err(source) => {
                    error!(%source, %kind, "failed to serialise an overlay message");
                    return;
                }
            };
            {
                let mut cache = match shared.cache.write() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                cache.set_value(&kind, data);
            }
            shared.send(frame);
        }
        Outbound::Show { visible } => match serde_json::to_string(&ShowMessage::new(visible)) {
            Ok(frame) => shared.send(frame),
            Err(source) => error!(%source, "failed to serialise a show message"),
        },
    }
}

fn rebroadcast_all(shared: &SharedState) {
    let frames: Vec<String> = {
        let cache = match shared.cache.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cache
            .iter()
            .filter_map(|(kind, data)| {
                match serde_json::to_string(&Message::new(kind, data.clone())) {
                    Ok(frame) => Some(frame),
                    Err(source) => {
                        error!(%source, %kind, "failed to reserialise a cached section");
                        None
                    }
                }
            })
            .collect()
    };
    for frame in frames {
        shared.send(frame);
    }
}
