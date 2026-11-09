//! Lifecycle tests: start, stop, restart, bind failure and status observers.

mod common;

use common::assert_running;
use fgc_server::{ServerConfig, ServerError, ServerHandle, ServerStatus};

#[tokio::test]
async fn start_stop_and_restart_report_status() {
    let handle = ServerHandle::start(ServerConfig::new(0)).expect("server should start");
    assert_running(&handle.status());
    assert!(handle.port().expect("bound port") > 0);

    let watcher = handle.watcher();
    assert_running(&watcher.status());

    handle.stop().await.expect("stop");
    assert_eq!(watcher.status(), ServerStatus::Stopped);
    assert_eq!(watcher.port(), None);

    let handle = ServerHandle::start(ServerConfig::new(0)).expect("server should restart");
    assert_running(&handle.status());

    let handle = handle
        .restart(ServerConfig::new(0))
        .await
        .expect("restart should bind");
    assert_running(&handle.status());

    handle.stop().await.expect("stop");
}

#[tokio::test]
async fn bind_failure_is_typed_and_mentions_the_address() {
    let blocker = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("blocking listener");
    let port = blocker.local_addr().expect("blocker address").port();

    let error = match ServerHandle::start(ServerConfig::new(port)) {
        Ok(_) => panic!("the second bind should fail"),
        Err(error) => error,
    };

    match error {
        ServerError::Bind { address, .. } => {
            assert!(
                address.contains(&port.to_string()),
                "bind error should mention {port}: {address}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn dropping_every_handle_stops_the_server() {
    let handle = ServerHandle::start(ServerConfig::new(0)).expect("server should start");
    let port = handle.port().expect("bound port");
    let watcher = handle.watcher();
    drop(handle);

    for _ in 0..200 {
        if watcher.status() == ServerStatus::Stopped {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("server did not stop after its handle was dropped (port {port})");
}
