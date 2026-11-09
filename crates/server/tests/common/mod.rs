//! Shared helpers for the server integration tests.
#![allow(dead_code)]

use std::time::Duration;

use fgc_server::{ServerConfig, ServerHandle, ServerStatus};

/// Starts a server and returns it with its loopback base URL.
pub fn start(config: ServerConfig) -> (ServerHandle, String) {
    let handle = ServerHandle::start(config).expect("server should start");
    let port = wait_for_running(&handle);
    (handle, format!("http://127.0.0.1:{port}"))
}

/// Waits until the handle reports a bound port.
pub fn wait_for_running(handle: &ServerHandle) -> u16 {
    for _ in 0..200 {
        if let Some(port) = handle.port() {
            return port;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("server never reported a port: {:?}", handle.status());
}

/// Polls `/health` until the connected-overlay count reaches `expected`.
pub async fn wait_for_overlays(client: &reqwest::Client, base_url: &str, expected: u64) {
    for _ in 0..200 {
        let health: serde_json::Value = client
            .get(format!("{base_url}/health"))
            .send()
            .await
            .expect("health request")
            .json()
            .await
            .expect("health json");
        if health["overlays"].as_u64() == Some(expected) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("overlay count never reached {expected}");
}

/// Returns a status snapshot suitable for assertions.
pub fn assert_running(status: &ServerStatus) {
    assert!(
        matches!(status, ServerStatus::Running { .. }),
        "expected running, got {status:?}"
    );
}
