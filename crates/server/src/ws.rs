//! WebSocket endpoint.
//!
//! Each client receives a fresh `init` immediately after connecting, can ask
//! for another with `{ "type": "requestInit" }`, and receives every broadcast
//! frame. A client that lags behind the broadcast buffer receives a fresh
//! `init` instead of silently missing sections.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::extract::State;
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::runtime::SharedState;

/// Upgrades an HTTP request to a WebSocket connection.
pub(crate) async fn upgrade(
    websocket: WebSocketUpgrade,
    State(shared): State<Arc<SharedState>>,
) -> Response {
    websocket.on_upgrade(move |socket| handle(socket, shared))
}

async fn handle(socket: WebSocket, shared: Arc<SharedState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcasts = shared.tx.subscribe();

    shared.clients.fetch_add(1, Ordering::SeqCst);
    let _guard = ClientGuard(Arc::clone(&shared));

    match shared.init_frame() {
        Ok(frame) => {
            if sender.send(WsMessage::Text(frame.into())).await.is_err() {
                return;
            }
        }
        Err(error) => {
            error!(%error, "failed to serialise init for a new overlay client");
            return;
        }
    }

    loop {
        tokio::select! {
            _ = shared.shutdown.cancelled() => break,
            incoming = receiver.next() => match incoming {
                Some(Ok(WsMessage::Text(text))) => {
                    if is_request_init(text.as_str()) {
                        match shared.init_frame() {
                            Ok(frame) => {
                                if sender.send(WsMessage::Text(frame.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                error!(%error, "failed to reserialise init");
                                break;
                            }
                        }
                    }
                }
                Some(Ok(WsMessage::Close(_))) | None => break,
                Some(Ok(_)) => {}
                Some(Err(error)) => {
                    warn!(%error, "overlay WebSocket receive failed");
                    break;
                }
            },
            broadcast = broadcasts.recv() => match broadcast {
                Ok(frame) => {
                    if sender.send(WsMessage::Text(frame.to_string().into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(skipped, "overlay client lagged; resending init");
                    match shared.init_frame() {
                        Ok(frame) => {
                            if sender.send(WsMessage::Text(frame.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            error!(%error, "failed to reserialise init after lag");
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
}

fn is_request_init(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(serde_json::Value::as_str)
                .map(|kind| kind == "requestInit")
        })
        .unwrap_or(false)
}

/// Decrements the connected-client count when a socket task ends.
struct ClientGuard(Arc<SharedState>);

impl Drop for ClientGuard {
    fn drop(&mut self) {
        self.0.clients.fetch_sub(1, Ordering::SeqCst);
    }
}
