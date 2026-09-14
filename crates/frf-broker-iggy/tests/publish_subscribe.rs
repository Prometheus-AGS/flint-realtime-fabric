#![allow(clippy::unwrap_used, clippy::expect_used)] // test/bench crate — see clippy.toml + rules/rust/testing.md

// Run with: cargo test -p frf-broker-iggy -- --ignored

use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
use frf_ports::LogBroker;
use futures_util::StreamExt;
use uuid::Uuid;

fn test_channel() -> Channel {
    Channel {
        id: ChannelId::new(),
        tenant_id: TenantId::from_uuid(Uuid::nil()),
        path: "test/publish_subscribe".to_owned(),
    }
}

fn test_envelope(channel: Channel) -> EventEnvelope {
    EventEnvelope::new(
        channel,
        Offset(0),
        EventKind::EntityChange,
        serde_json::json!({"key": "value"}),
    )
}

#[tokio::test]
#[ignore = "requires a live Iggy server — run with: cargo test -p frf-broker-iggy -- --ignored"]
async fn publish_then_subscribe_receives_message() {
    let connection = std::env::var("IGGY_TEST_CONNECTION_STRING")
        .unwrap_or_else(|_| "iggy://iggy:iggy@127.0.0.1:8090".to_owned());
    let broker = IggyBroker::new(&connection)
        .await
        .expect("failed to connect to Iggy — ensure local Iggy is running");

    let channel = test_channel();
    let envelope = test_envelope(channel.clone());
    let channel_id = channel.id;

    broker
        .ensure_channel(channel.clone())
        .await
        .expect("ensure_channel failed");

    let published_offset = broker
        .publish(envelope.clone())
        .await
        .expect("publish failed");

    let mut stream = broker
        .subscribe(channel_id, "test-consumer".to_owned(), Offset::BEGINNING)
        .await
        .expect("subscribe failed");

    let received = stream
        .next()
        .await
        .expect("stream ended before receiving message")
        .expect("stream returned an error");

    assert_eq!(
        received.kind, envelope.kind,
        "received message kind mismatch"
    );
    assert_eq!(
        received.payload, envelope.payload,
        "received message payload mismatch"
    );
    let _ = published_offset;
}

/// A subscriber that knows only the well-known channel id receives what a publisher
/// sent to that channel — with **no channel id passed between the two halves**.
///
/// This is the property issue #2's symptom violated. The test above threads one
/// `channel.id` from publish into subscribe, so it passes even when the publisher mints
/// a fresh random id — which is exactly how the CDC consumer shipped broken. Here the
/// subscriber derives the id independently from `ChannelId::WELL_KNOWN_ENTITIES`, so
/// reverting a publisher to `ChannelId::new()` makes this fail.
///
/// Note it deliberately does *not* call `ensure_channel`: `publish` is now self-healing,
/// and that is the behaviour under test.
#[tokio::test]
#[ignore = "requires a live Iggy server — run with: cargo test -p frf-broker-iggy -- --ignored"]
async fn a_subscriber_knowing_only_the_well_known_id_receives_published_events() {
    let connection = std::env::var("IGGY_TEST_CONNECTION_STRING")
        .unwrap_or_else(|_| "iggy://iggy:iggy@127.0.0.1:8090".to_owned());
    let broker = IggyBroker::new(&connection)
        .await
        .expect("failed to connect to Iggy — ensure local Iggy is running");

    // Publisher side: builds its channel the way the CDC consumer does.
    let published = EventEnvelope::new(
        Channel {
            id: ChannelId::WELL_KNOWN_ENTITIES,
            tenant_id: TenantId::from_uuid(Uuid::from_u128(1)),
            path: "entities".to_owned(),
        },
        Offset(0),
        EventKind::EntityChange,
        serde_json::json!({"issue": "2", "probe": "well-known-channel"}),
    );
    broker
        .publish(published.clone())
        .await
        .expect("publish failed");

    // Subscriber side: knows ONLY the well-known constant. Nothing is carried over
    // from the publisher — that independence is the whole point of this test.
    let mut stream = broker
        .subscribe(
            ChannelId::WELL_KNOWN_ENTITIES,
            "issue-2-consumer".to_owned(),
            Offset::BEGINNING,
        )
        .await
        .expect("subscribe failed");

    let received = stream
        .next()
        .await
        .expect("stream ended before receiving a message — publisher and subscriber did not meet")
        .expect("stream returned an error");

    assert_eq!(
        received.channel.id,
        ChannelId::WELL_KNOWN_ENTITIES,
        "received an event from a different channel than the one subscribed to"
    );
    assert_eq!(
        received.payload, published.payload,
        "received message payload mismatch"
    );
}
