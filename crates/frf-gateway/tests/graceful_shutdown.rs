#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

//! Integration test for graceful shutdown (p16-c019).
//!
//! Proves that `axum::serve(...).with_graceful_shutdown(future)` DRAINS an
//! in-flight request when shutdown is triggered, rather than aborting it. The
//! shutdown future is a controllable oneshot (a real SIGTERM would risk killing
//! the test runner); this exercises the same drain path `main` uses.

use std::time::Duration;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn in_flight_request_drains_on_shutdown() {
    // A slow handler: sleeps 300ms before responding. If shutdown ABORTED
    // in-flight requests, a request started just before shutdown would fail.
    let app = Router::new().route(
        "/slow",
        get(|| async {
            tokio::time::sleep(Duration::from_millis(300)).await;
            "done"
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    // Start an in-flight request.
    let url = format!("http://{addr}/slow");
    let client = reqwest::Client::new();
    let request = tokio::spawn(async move { client.get(&url).send().await });

    // Give the request time to be accepted, THEN trigger shutdown while it is
    // still in flight (the handler is mid-sleep).
    tokio::time::sleep(Duration::from_millis(50)).await;
    shutdown_tx.send(()).unwrap();

    // The in-flight request must complete successfully (drained, not dropped).
    let response = request
        .await
        .unwrap()
        .expect("request should not be dropped");
    assert!(response.status().is_success());
    assert_eq!(response.text().await.unwrap(), "done");

    // The server should have exited cleanly after draining.
    server.await.unwrap();
}

#[tokio::test]
async fn server_stops_accepting_after_shutdown() {
    let app = Router::new().route("/ping", get(|| async { "pong" }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    // One request works before shutdown.
    let client = reqwest::Client::new();
    let ok = client
        .get(format!("http://{addr}/ping"))
        .send()
        .await
        .unwrap();
    assert!(ok.status().is_success());

    // After shutdown the server drains and exits.
    shutdown_tx.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server should exit within 5s of shutdown")
        .unwrap();
}
