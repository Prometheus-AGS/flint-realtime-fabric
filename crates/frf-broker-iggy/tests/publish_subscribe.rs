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
    let broker = IggyBroker::new("iggy://guest:guest@localhost:8090")
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
