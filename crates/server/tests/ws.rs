//! WebSocket contract tests: init, requestInit, fan-out and lag recovery.

mod common;

use std::time::Duration;

use common::{start, wait_for_overlays};
use fgc_server::{Outbound, ServerConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::{TcpSocket, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, client_async};

type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn connect(port: u16) -> Client {
    let (socket, _response) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/"))
        .await
        .expect("websocket connect");
    socket
}

/// Connects with a deliberately tiny receive buffer so a client that stops
/// reading forces the server to block quickly.
async fn connect_with_small_receive_buffer(port: u16) -> Client {
    let socket = TcpSocket::new_v4().expect("tcp socket");
    socket
        .set_recv_buffer_size(4096)
        .expect("receive buffer size");
    let stream = socket
        .connect(format!("127.0.0.1:{port}").parse().expect("address"))
        .await
        .expect("tcp connect");
    let request = format!("ws://127.0.0.1:{port}/")
        .into_client_request()
        .expect("websocket request");
    let (client, _response) = client_async(request, MaybeTlsStream::Plain(stream))
        .await
        .expect("handshake");
    client
}

async fn next_json(socket: &mut Client) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(20), socket.next())
            .await
            .expect("timed out waiting for a frame")
            .expect("websocket stream ended")
            .expect("websocket frame");
        match message {
            Message::Text(text) => {
                return serde_json::from_str(text.as_str()).expect("frame is JSON");
            }
            Message::Ping(_) | Message::Pong(_) => {}
            other => panic!("unexpected frame: {other:?}"),
        }
    }
}

fn section(kind: &str, payload: Value) -> Outbound {
    Outbound::section(kind, &payload).expect("serialisable payload")
}

#[tokio::test]
async fn new_client_receives_empty_init_and_can_request_it_again() {
    let (handle, base_url) = start(ServerConfig::new(0));
    let port = handle.port().expect("running port");
    let client = reqwest::Client::new();
    let mut socket = connect(port).await;

    let init = next_json(&mut socket).await;
    assert_eq!(init["type"], "init");
    assert_eq!(init["data"], json!({}));

    handle.broadcast(section("scoreboard", json!({ "scoreA": 1 })));

    let scoreboard = next_json(&mut socket).await;
    assert_eq!(scoreboard["type"], "scoreboard");
    assert_eq!(scoreboard["data"]["scoreA"], 1);

    socket
        .send(Message::text(r#"{"type":"requestInit"}"#))
        .await
        .expect("send requestInit");
    let init = next_json(&mut socket).await;
    assert_eq!(init["type"], "init");
    assert_eq!(init["data"]["scoreboard"]["scoreA"], 1);

    drop(socket);
    wait_for_overlays(&client, &base_url, 0).await;

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn init_contains_five_scoreboard_keys_and_casters() {
    let (handle, _base_url) = start(ServerConfig::new(0));
    let port = handle.port().expect("running port");

    let scoreboard = json!({ "title": "Grand Final", "scoreA": 2, "scoreB": 1 });
    for kind in [
        "scoreboard",
        "gameOverlay",
        "mapSelect",
        "heroBans",
        "teamRoster",
    ] {
        handle.broadcast(section(kind, scoreboard.clone()));
    }
    handle.broadcast(section("casters", json!({ "castersData": [] })));

    let mut socket = connect(port).await;
    let init = next_json(&mut socket).await;
    assert_eq!(init["type"], "init");

    let data = init["data"].as_object().expect("init data object");
    for kind in [
        "scoreboard",
        "gameOverlay",
        "mapSelect",
        "heroBans",
        "teamRoster",
    ] {
        assert_eq!(data[kind], scoreboard, "{kind} payload mismatch");
    }
    assert_eq!(data["casters"], json!({ "castersData": [] }));
    assert!(data["casters"].get("scoreA").is_none());

    drop(socket);
    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn broadcasts_reach_every_connected_client() {
    let (handle, _base_url) = start(ServerConfig::new(0));
    let port = handle.port().expect("running port");

    let mut first = connect(port).await;
    let mut second = connect(port).await;
    assert_eq!(next_json(&mut first).await["type"], "init");
    assert_eq!(next_json(&mut second).await["type"], "init");

    handle.broadcast(section("scoreboard", json!({ "scoreA": 7 })));

    for socket in [&mut first, &mut second] {
        let frame = next_json(socket).await;
        assert_eq!(frame["type"], "scoreboard");
        assert_eq!(frame["data"]["scoreA"], 7);
    }

    drop(first);
    drop(second);
    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn a_lagging_client_receives_a_fresh_init() {
    let config = ServerConfig::new(0).with_broadcast_capacity(1);
    let (handle, _base_url) = start(config);
    let port = handle.port().expect("running port");

    let mut socket = connect_with_small_receive_buffer(port).await;
    assert_eq!(next_json(&mut socket).await["type"], "init");

    // The client is deliberately not reading. Large frames fill the tiny
    // socket buffer, the WS task blocks, and the one-slot broadcast ring
    // overflows.
    let padding = "x".repeat(512 * 1024);
    let payload = json!({ "padding": padding });
    for _ in 0..20 {
        handle.broadcast(section("scoreboard", payload.clone()));
    }

    // Give the server thread time to drain the command queue while this
    // client is not reading, so the broadcast ring actually overflows.
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut saw_init = false;
    for _ in 0..20 {
        let frame = next_json(&mut socket).await;
        if frame["type"] == "init" {
            assert_eq!(
                frame["data"]["scoreboard"]["padding"], payload["padding"],
                "init should reflect the last cached section"
            );
            saw_init = true;
            break;
        }
    }
    assert!(saw_init, "lagging client never received a fresh init");

    drop(socket);
    handle.stop().await.expect("stop");
}
