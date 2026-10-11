#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Owned live-Iggy acceptance scenarios for durable broker replay.
//!
//! The shell runner invokes each ignored test by exact name. They are ignored in
//! ordinary local suites because they require an isolated broker and data volume.

use std::time::{Duration, Instant};

use frf_broker_iggy::IggyBroker;
use frf_domain::ids::EventId;
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
use frf_ports::{LogBroker, PortError};
use futures_util::StreamExt;
use iggy::client::{Client, TopicClient};
use iggy::clients::client::IggyClient;
use iggy::utils::duration::IggyDuration;
use iggy::utils::expiry::IggyExpiry;
use tokio::time::timeout;
use uuid::Uuid;

const RECEIVE_TIMEOUT: Duration = Duration::from_secs(10);
const QUIET_TIMEOUT: Duration = Duration::from_secs(2);
const RESTART_CHANNEL: u128 = 0xc007_0000_0000_0000_0000_0000_0000_0001;
const RESTART_EVENT_A: u128 = 0xc007_0000_0000_0000_0000_0000_0000_00a1;
const RESTART_EVENT_B: u128 = 0xc007_0000_0000_0000_0000_0000_0000_00b2;
const RESTART_EVENT_C: u128 = 0xc007_0000_0000_0000_0000_0000_0000_00c3;
const RESTART_EVENT_D: u128 = 0xc007_0000_0000_0000_0000_0000_0000_00d4;

fn connection_string() -> String {
    std::env::var("FRF_BROKER_REPLAY_IGGY_URL")
        .expect("FRF_BROKER_REPLAY_IGGY_URL must name the owned Iggy fixture")
}

fn test_channel(id: ChannelId, path: &str) -> Channel {
    Channel {
        id,
        tenant_id: TenantId::from_uuid(Uuid::from_u128(7)),
        path: path.to_owned(),
    }
}

fn envelope(channel: Channel, event_id: u128, source_offset: u64, label: &str) -> EventEnvelope {
    let mut envelope = EventEnvelope::new(
        channel,
        Offset(source_offset),
        EventKind::EntityChange,
        serde_json::json!({"label": label}),
    );
    envelope.id = EventId::from_uuid(Uuid::from_u128(event_id));
    envelope
}

async fn receive(stream: &mut frf_ports::EventStream) -> EventEnvelope {
    timeout(RECEIVE_TIMEOUT, stream.next())
        .await
        .expect("timed out waiting for broker event")
        .expect("broker stream ended")
        .expect("broker stream returned an error")
}

async fn assert_quiet(stream: &mut frf_ports::EventStream) {
    assert!(
        timeout(QUIET_TIMEOUT, stream.next()).await.is_err(),
        "duplicate event unexpectedly occupied a new broker position"
    );
}

async fn settle_cancellation() {
    tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn assert_topic_contract(connection: &str, channel_id: ChannelId) {
    let client = IggyClient::from_connection_string(connection).expect("valid Iggy URL");
    client.connect().await.expect("raw Iggy client connect");
    let stream_name = format!("channel-{channel_id}");
    let stream_id = stream_name.as_str().try_into().expect("valid stream name");
    let topic_id = "events".try_into().expect("valid topic name");
    let topic = client
        .get_topic(&stream_id, &topic_id)
        .await
        .expect("get replay topic")
        .expect("replay topic exists");

    assert_eq!(topic.partitions_count, 1);
    assert_eq!(topic.partitions.len(), 1);
    assert_eq!(topic.partitions[0].id, 1);
    assert_eq!(
        topic.message_expiry,
        IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(86_400))
    );
    client.disconnect().await.expect("raw Iggy disconnect");
}

async fn expire_topic_history(connection: &str, channel_id: ChannelId) {
    let client = IggyClient::from_connection_string(connection).expect("valid Iggy URL");
    client.connect().await.expect("raw Iggy client connect");
    let stream_name = format!("channel-{channel_id}");
    let stream_id = stream_name.as_str().try_into().expect("valid stream name");
    let topic_id = "events".try_into().expect("valid topic name");
    let topic = client
        .get_topic(&stream_id, &topic_id)
        .await
        .expect("get expiry topic")
        .expect("expiry topic exists");
    client
        .update_topic(
            &stream_id,
            &topic_id,
            &topic.name,
            topic.compression_algorithm,
            Some(topic.replication_factor),
            IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(1)),
            topic.max_topic_size,
        )
        .await
        .expect("set accelerated test expiry");

    let deadline = Instant::now() + RECEIVE_TIMEOUT;
    loop {
        let current = client
            .get_topic(&stream_id, &topic_id)
            .await
            .expect("poll expiry topic")
            .expect("expiry topic remains available");
        if current.partitions[0].messages_count == 0 {
            break;
        }
        assert!(Instant::now() < deadline, "topic history did not expire");
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    client.disconnect().await.expect("raw Iggy disconnect");
}

#[tokio::test]
#[ignore = "requires the owned broker replay fixture"]
#[allow(clippy::too_many_lines)]
async fn broker_replay_contract() {
    let connection = connection_string();
    let broker = IggyBroker::new(&connection).await.expect("connect broker");
    let channel = test_channel(ChannelId::new(), "acceptance/c007/core");
    let first = envelope(channel.clone(), 0x00c0_0711, 7_001, "first");
    let second = envelope(channel.clone(), 0x00c0_0722, 7_002, "second");

    let first_published = broker.publish(first.clone()).await.expect("publish first");
    let second_published = broker
        .publish(second.clone())
        .await
        .expect("publish second");
    assert_eq!(first_published, Offset(0));
    assert!(second_published > first_published);
    assert_topic_contract(&connection, channel.id).await;

    let consumer = "crash-window";
    let mut before_crash = broker
        .subscribe(channel.id, consumer.to_owned(), Offset::BEGINNING)
        .await
        .expect("subscribe before crash");
    let first_delivery = receive(&mut before_crash).await;
    assert_eq!(first_delivery.id, first.id);
    assert_eq!(first_delivery.offset, first_published);
    assert_ne!(first_delivery.offset, first.offset);
    assert_eq!(
        broker
            .get_consumer_offset(channel.id, consumer)
            .await
            .unwrap(),
        None
    );
    drop(before_crash);
    settle_cancellation().await;

    let after_crash_broker = IggyBroker::new(&connection)
        .await
        .expect("reconnect broker after receiver crash");
    let mut after_crash = after_crash_broker
        .subscribe(channel.id, consumer.to_owned(), first_delivery.offset)
        .await
        .expect("reconnect at unacknowledged inclusive position");
    let replayed = receive(&mut after_crash).await;
    assert_eq!(replayed.id, first.id);
    assert_eq!(replayed.offset, first_delivery.offset);
    drop(after_crash);
    settle_cancellation().await;

    let inclusive_broker = IggyBroker::new(&connection)
        .await
        .expect("connect inclusive probe");
    let mut inclusive = inclusive_broker
        .subscribe(channel.id, "inclusive-probe".to_owned(), replayed.offset)
        .await
        .expect("inclusive replay");
    assert_eq!(receive(&mut inclusive).await.id, first.id);
    drop(inclusive);
    settle_cancellation().await;

    let strict_broker = IggyBroker::new(&connection)
        .await
        .expect("connect strict resume probe");
    let mut after_first = strict_broker
        .subscribe(
            channel.id,
            "strict-after-first".to_owned(),
            replayed.offset.next(),
        )
        .await
        .expect("resume strictly after first event");
    let second_delivery = receive(&mut after_first).await;
    assert_eq!(second_delivery.id, second.id);
    assert_eq!(second_delivery.offset, second_published);
    assert!(second_delivery.offset > replayed.offset);
    drop(after_first);
    settle_cancellation().await;
    let checkpoint_broker = IggyBroker::new(&connection)
        .await
        .expect("connect checkpoint client");
    checkpoint_broker
        .ack(channel.id, consumer, second_delivery.offset)
        .await
        .expect("ack second event");
    assert_eq!(
        checkpoint_broker
            .get_consumer_offset(channel.id, consumer)
            .await
            .unwrap(),
        Some(second_delivery.offset)
    );

    let independent = "independent-consumer";
    let mut independent_stream = broker
        .subscribe(channel.id, independent.to_owned(), Offset::BEGINNING)
        .await
        .expect("independent subscribe");
    assert_eq!(receive(&mut independent_stream).await.id, first.id);
    assert_eq!(
        broker
            .get_consumer_offset(channel.id, independent)
            .await
            .unwrap(),
        None
    );
    drop(independent_stream);

    let retried = broker
        .publish(first.clone())
        .await
        .expect("retry stable event");
    assert_eq!(retried, first_published);
    let mut dedupe_probe = broker
        .subscribe(
            channel.id,
            "dedupe-probe".to_owned(),
            second_delivery.offset.next(),
        )
        .await
        .expect("subscribe after final unique position");
    assert_quiet(&mut dedupe_probe).await;
    drop(dedupe_probe);

    let expiry_channel = test_channel(ChannelId::new(), "acceptance/c007/expiry");
    let mut expiry_first = envelope(expiry_channel.clone(), 0x00c0_07e1, 9_001, "expiry-first");
    expiry_first.payload = serde_json::json!({"label": "expiry-first", "pad": "x".repeat(2_048)});
    let mut expiry_second = envelope(expiry_channel.clone(), 0x00c0_07e2, 9_002, "expiry-second");
    expiry_second.payload = serde_json::json!({"label": "expiry-second", "pad": "x".repeat(2_048)});
    broker
        .publish(expiry_first)
        .await
        .expect("publish expiry first");
    broker
        .publish(expiry_second.clone())
        .await
        .expect("publish expiry second");
    let mut expiry_stream = broker
        .subscribe(
            expiry_channel.id,
            "expiry-position".to_owned(),
            Offset::BEGINNING,
        )
        .await
        .expect("read expiry positions");
    let _ = receive(&mut expiry_stream).await;
    let expired_position = receive(&mut expiry_stream).await;
    assert_eq!(expired_position.id, expiry_second.id);
    drop(expiry_stream);
    settle_cancellation().await;
    expire_topic_history(&connection, expiry_channel.id).await;
    let expired_error = broker
        .subscribe(
            expiry_channel.id,
            "expiry-probe".to_owned(),
            expired_position.offset,
        )
        .await
        .err()
        .expect("expired explicit cursor must fail");
    match expired_error {
        PortError::NotFound(message) => assert!(message.starts_with("resnapshot_required:")),
        other => panic!("unexpected expired cursor error: {other}"),
    }

    let concurrent_channel = test_channel(ChannelId::new(), "acceptance/c007/concurrent");
    let concurrent_a = envelope(
        concurrent_channel.clone(),
        0x00c0_07a1,
        11_001,
        "concurrent-a",
    );
    let concurrent_b = envelope(
        concurrent_channel.clone(),
        0x00c0_07b2,
        11_001,
        "concurrent-b",
    );
    let publisher_a = IggyBroker::new(&connection).await.expect("publisher A");
    let publisher_b = IggyBroker::new(&connection).await.expect("publisher B");
    let (offset_a, offset_b) = tokio::join!(
        publisher_a.publish(concurrent_a.clone()),
        publisher_b.publish(concurrent_b.clone())
    );
    let offset_a = offset_a.expect("concurrent publish A");
    let offset_b = offset_b.expect("concurrent publish B");
    assert_ne!(offset_a, offset_b);
    let mut concurrent_stream = broker
        .subscribe(
            concurrent_channel.id,
            "concurrent-position-probe".to_owned(),
            std::cmp::min(offset_a, offset_b),
        )
        .await
        .expect("read concurrent broker positions");
    for _ in 0..2 {
        let delivered = receive(&mut concurrent_stream).await;
        if delivered.id == concurrent_a.id {
            assert_eq!(delivered.offset, offset_a);
        } else if delivered.id == concurrent_b.id {
            assert_eq!(delivered.offset, offset_b);
        } else {
            panic!("unexpected concurrent event {}", delivered.id);
        }
    }
    drop(concurrent_stream);
    settle_cancellation().await;

    let restarted_producer = IggyBroker::new(&connection)
        .await
        .expect("restart producer client");
    let third = envelope(channel.clone(), 0x00c0_0733, 7_001, "third");
    let third_published = restarted_producer
        .publish(third.clone())
        .await
        .expect("publish after producer restart");
    let mut after_producer_restart = broker
        .subscribe(
            channel.id,
            "producer-restart".to_owned(),
            second_delivery.offset.next(),
        )
        .await
        .expect("subscribe after producer restart");
    let third_delivery = receive(&mut after_producer_restart).await;
    assert_eq!(third_delivery.id, third.id);
    assert_eq!(third_delivery.offset, third_published);
    assert!(third_delivery.offset > second_delivery.offset);
    assert_ne!(third_delivery.offset, third.offset);

    drop(after_producer_restart);
    broker
        .publish(envelope(channel, 0x00c0_0744, 7_001, "post-cancel"))
        .await
        .expect("receiver cancellation releases broker polling task");
    println!("BROKER_REPLAY_CORE_PASS");
}

#[tokio::test]
#[ignore = "requires the owned broker replay fixture before server restart"]
async fn seed_broker_restart_state() {
    let connection = connection_string();
    let broker = IggyBroker::new(&connection).await.expect("connect broker");
    let channel = test_channel(
        ChannelId::from_uuid(Uuid::from_u128(RESTART_CHANNEL)),
        "acceptance/c007/restart",
    );
    let first = envelope(channel.clone(), RESTART_EVENT_A, 1, "restart-a");
    let second = envelope(channel.clone(), RESTART_EVENT_B, 1, "restart-b");
    let third = envelope(channel.clone(), RESTART_EVENT_C, 1, "restart-c");
    broker
        .publish(first.clone())
        .await
        .expect("publish restart A");
    broker
        .publish(second.clone())
        .await
        .expect("publish restart B");
    broker.publish(third).await.expect("publish restart C");
    let mut stream = broker
        .subscribe(channel.id, "restart-consumer".to_owned(), Offset::BEGINNING)
        .await
        .expect("subscribe restart seed");
    let first_received = receive(&mut stream).await;
    assert_eq!(first_received.id, first.id);
    let checkpointed = receive(&mut stream).await;
    assert_eq!(checkpointed.id, second.id);
    broker
        .ack(channel.id, "restart-consumer", checkpointed.offset)
        .await
        .expect("ack restart checkpoint");
    assert_topic_contract(&connection, channel.id).await;
    println!("BROKER_RESTART_SEED_PASS offset={}", checkpointed.offset.0);
}

#[tokio::test]
#[ignore = "requires the owned broker replay fixture after server restart"]
async fn verify_broker_restart_state() {
    let connection = connection_string();
    println!("BROKER_RESTART_VERIFY_CONNECTING");
    let broker = timeout(RECEIVE_TIMEOUT, IggyBroker::new(&connection))
        .await
        .expect("timed out reconnecting after server restart")
        .expect("reconnect broker");
    println!("BROKER_RESTART_VERIFY_CONNECTED");
    let channel = test_channel(
        ChannelId::from_uuid(Uuid::from_u128(RESTART_CHANNEL)),
        "acceptance/c007/restart",
    );
    let stored = timeout(
        RECEIVE_TIMEOUT,
        broker.get_consumer_offset(channel.id, "restart-consumer"),
    )
    .await
    .expect("timed out reading durable checkpoint")
    .expect("read durable checkpoint")
    .expect("checkpoint survives restart");
    println!("BROKER_RESTART_VERIFY_CHECKPOINT offset={}", stored.0);
    let mut stream = broker
        .subscribe(channel.id, "restart-consumer".to_owned(), stored.next())
        .await
        .expect("resume after durable checkpoint");
    let third = receive(&mut stream).await;
    assert_eq!(
        third.id,
        EventId::from_uuid(Uuid::from_u128(RESTART_EVENT_C))
    );
    drop(stream);

    let fourth = envelope(channel.clone(), RESTART_EVENT_D, 1, "restart-d");
    broker
        .publish(fourth.clone())
        .await
        .expect("publish after restart");
    let mut after_third = broker
        .subscribe(channel.id, "post-restart".to_owned(), third.offset.next())
        .await
        .expect("read post-restart position");
    let fourth_delivery = receive(&mut after_third).await;
    assert_eq!(fourth_delivery.id, fourth.id);
    assert!(fourth_delivery.offset > third.offset);
    assert_ne!(fourth_delivery.offset, fourth.offset);
    drop(after_third);

    let original = envelope(channel.clone(), RESTART_EVENT_A, 999, "restart-a-retry");
    broker
        .publish(original)
        .await
        .expect("retry pre-restart stable ID");
    let mut dedupe_probe = broker
        .subscribe(
            channel.id,
            "restart-dedupe".to_owned(),
            fourth_delivery.offset.next(),
        )
        .await
        .expect("probe persisted deduplication");
    assert_quiet(&mut dedupe_probe).await;
    assert_topic_contract(&connection, channel.id).await;
    println!(
        "BROKER_RESTART_VERIFY_PASS checkpoint={} replayed={} appended={}",
        stored.0, third.offset.0, fourth_delivery.offset.0
    );
}
