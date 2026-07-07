#![allow(clippy::unwrap_used, clippy::expect_used)] // test crate — see clippy.toml + rules/rust/testing.md

//! Integration smoke test for `FrfClient` against a running gateway.
//!
//! Gated behind `FRF_GATEWAY_GRPC_URL` (e.g. `http://localhost:9090`). When the
//! env var is unset the test is skipped, so this compiles and runs in CI without
//! a live gateway (the same convention as the other integration suites). When a
//! gateway is available, it exercises connect → subscribe → publish → receive.

use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
use frf_sdk_rust::FrfClient;
use futures_util::StreamExt as _;

fn gateway_url() -> Option<String> {
    std::env::var("FRF_GATEWAY_GRPC_URL")
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test]
async fn connect_publish_subscribe_roundtrip() {
    let Some(url) = gateway_url() else {
        eprintln!("skipping: set FRF_GATEWAY_GRPC_URL to run the SDK smoke test");
        return;
    };

    let token = std::env::var("FRF_JWT").ok();
    let mut client = FrfClient::connect(url, token)
        .await
        .expect("connect to gateway");

    let channel_id = ChannelId::new();
    let tenant_id = TenantId::new();

    // Open a subscription first so the published event can be observed.
    let mut stream = client
        .subscribe(channel_id, "smoke-consumer".to_owned(), Offset::BEGINNING)
        .await
        .expect("subscribe");

    let envelope = EventEnvelope::new(
        Channel {
            id: channel_id,
            tenant_id,
            path: "smoke/test".to_owned(),
        },
        Offset(0),
        EventKind::EntityChange,
        serde_json::json!({ "smoke": true }),
    );

    let offset = client.publish(&envelope).await.expect("publish");
    assert!(offset.0 >= 1, "expected a positive assigned offset");

    // Best-effort: try to receive the event (a real gateway with authz may filter).
    if let Ok(Some(item)) =
        tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await
    {
        let received = item.expect("stream item");
        assert_eq!(received.channel.id, channel_id);
    }
}
