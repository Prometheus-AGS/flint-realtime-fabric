#![allow(clippy::expect_used, clippy::unwrap_used)]
//! No log line at any level carries a ticket value (F2). Its own test binary
//! because it installs the process-global TRACE subscriber.

#[path = "support/realtime.rs"]
mod realtime;

use std::io::Write;
use std::sync::{Arc, Mutex};

use frf_ports::LogBroker as _;
use futures_util::StreamExt as _;
use realtime::{channel_for, envelope, spawn, ticket_claims};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("capture").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ticket_value_never_reaches_the_logs() {
    let capture = Capture::default();
    let writer = capture.clone();
    tracing_subscriber::fmt()
        // Everything at TRACE except the test's own WebSocket *client*, which
        // logs the request line it sends.
        .with_env_filter(EnvFilter::new("trace,tungstenite::handshake::client=off"))
        .with_span_events(FmtSpan::FULL)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .init();

    let fixture = spawn().await;
    let topic = "t/acme/u/user-1/roster";
    let channel = channel_for(topic);
    let token = "ticket-SECRET-6f1d0c";
    fixture
        .tokens
        .insert(token, ticket_claims("user-1", 60, &[topic]));
    let url = format!(
        "ws://{}/ws/v1/subscribe?channel={}&ticket={token}&from=0",
        fixture.address, channel.id
    );

    let (mut socket, _) = tokio_tungstenite::connect_async(&url).await.expect("open");
    fixture.broker.publish(envelope(&channel, 1)).await.unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next()).await;
    // Replay (401) and a channel outside the claim (403) log on their error paths too.
    assert!(tokio_tungstenite::connect_async(&url).await.is_err());
    let other = channel_for("t/acme/u/user-2/roster");
    fixture.tokens.insert(
        "ticket-SECRET-second",
        ticket_claims("user-1", 60, &[topic]),
    );
    let forbidden = url
        .replace(&channel.id.to_string(), &other.id.to_string())
        .replace(token, "ticket-SECRET-second");
    assert!(tokio_tungstenite::connect_async(&forbidden).await.is_err());

    let logs = String::from_utf8(capture.0.lock().expect("capture").clone()).unwrap();
    assert!(
        logs.contains("ws::subscribe"),
        "subscribe span was captured"
    );
    assert!(!logs.contains("SECRET"), "a ticket value reached the logs");
}
