//! Hidden packaged smoke test used by CI (`--smoke-test`).
//!
//! The test starts the real overlay server on an ephemeral port with a
//! temporary data directory, then exercises the packaged HTTP and WebSocket
//! surface: `/health`, an embedded overlay, a bundled asset and the `init`
//! handshake. It never opens a window or touches the user's app data.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use fgc_server::{ServerConfig, ServerHandle};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Runs the smoke test on a private Tokio runtime.
pub fn run() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build the smoke-test runtime")?;
    runtime.block_on(run_async())
}

async fn run_async() -> Result<()> {
    let root = std::env::temp_dir().join(format!("fgc-smoke-{}", std::process::id()));
    let blob_dir = root.join("blobs");
    std::fs::create_dir_all(&blob_dir).context("create the temporary data directory")?;

    let handle = ServerHandle::start(ServerConfig::new(0).with_blob_dir(&blob_dir))
        .context("start the overlay server")?;
    let port = handle.port().context("the overlay server did not bind")?;
    let base = format!("http://127.0.0.1:{port}");

    let result = exercise(&base, port).await;
    let _ = handle.stop().await;
    let _ = std::fs::remove_dir_all(&root);
    result
}

async fn exercise(base: &str, port: u16) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("build the smoke-test HTTP client")?;

    let health: Value = client
        .get(format!("{base}/health"))
        .send()
        .await
        .context("request /health")?
        .error_for_status()
        .context("/health returned an error")?
        .json()
        .await
        .context("parse /health")?;
    if health.get("status").and_then(Value::as_str) != Some("ok") {
        bail!("/health did not report status ok: {health}");
    }
    if health.get("port").and_then(Value::as_u64) != Some(u64::from(port)) {
        bail!("/health reported the wrong port: {health}");
    }

    let overlay = client
        .get(format!("{base}/overlays/game-overlay.html"))
        .send()
        .await
        .context("request the game overlay")?
        .error_for_status()
        .context("the game overlay returned an error")?
        .text()
        .await
        .context("read the game overlay")?;
    if !overlay.contains("<title>Game Overlay</title>") {
        bail!("the embedded game overlay did not contain its title");
    }

    let asset = fgc_server::bundled_assets()
        .first()
        .context("no bundled assets are embedded")?;
    let bytes = client
        .get(format!("{base}/assets/{}.{}", asset.hash, asset.ext))
        .send()
        .await
        .context("request a bundled asset")?
        .error_for_status()
        .context("the bundled asset returned an error")?
        .bytes()
        .await
        .context("read the bundled asset")?;
    if bytes.is_empty() {
        bail!("the bundled asset response was empty");
    }

    let (mut socket, _) = connect_async(format!("ws://127.0.0.1:{port}/"))
        .await
        .context("connect the overlay WebSocket")?;
    validate_init(&next_init(&mut socket).await?)?;

    socket
        .send(Message::text(r#"{"type":"requestInit"}"#))
        .await
        .context("send requestInit")?;
    validate_init(&next_init(&mut socket).await?)?;

    let _ = socket.close(None).await;
    Ok(())
}

async fn next_init(socket: &mut Socket) -> Result<Value> {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(15), socket.next())
            .await
            .context("timed out waiting for an init frame")?
            .context("the overlay WebSocket closed")?
            .context("read an overlay WebSocket frame")?;
        match message {
            Message::Text(text) => {
                let value: Value = serde_json::from_str(text.as_str())
                    .context("parse an overlay WebSocket frame")?;
                if value.get("type").and_then(Value::as_str) == Some("init") {
                    return Ok(value);
                }
            }
            Message::Ping(_) | Message::Pong(_) => {}
            other => bail!("unexpected overlay WebSocket frame: {other:?}"),
        }
    }
}

fn validate_init(value: &Value) -> Result<()> {
    if value.get("type").and_then(Value::as_str) != Some("init") {
        bail!("expected an init frame, got {value}");
    }
    if !value.get("data").is_some_and(Value::is_object) {
        bail!("the init frame did not contain an object payload");
    }
    Ok(())
}
