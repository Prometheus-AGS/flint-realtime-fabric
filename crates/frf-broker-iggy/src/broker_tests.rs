use super::*;
use crate::broker_config::DEFAULT_REPLAY_RETENTION_SECONDS;
use frf_domain::{EventKind, TenantId};
use iggy::clients::consumer::AutoCommit;
use iggy::messages::poll_messages::PollingKind;
use iggy::utils::duration::IggyDuration;
use iggy::utils::expiry::IggyExpiry;
use std::future;
use uuid::Uuid;

fn envelope(source_offset: u64) -> EventEnvelope {
    EventEnvelope::new(
        Channel {
            id: ChannelId::WELL_KNOWN_ENTITIES,
            tenant_id: TenantId::from_uuid(Uuid::nil()),
            path: "entities".to_owned(),
        },
        Offset(source_offset),
        EventKind::EntityChange,
        serde_json::json!({"id": "case-42"}),
    )
}

#[test]
fn producer_message_identity_is_the_stable_event_id() {
    let envelope = envelope(41);
    let expected = envelope.id.as_uuid().as_u128();
    let message = encode_message(&envelope).expect("test envelope must serialize");
    assert_eq!(message.id, expected);
}

#[test]
fn decoded_envelope_uses_the_broker_offset() {
    let envelope = envelope(41);
    let payload = serde_json::to_vec(&envelope).expect("test envelope must serialize");
    let decoded = decode_message(&payload, 9001).expect("test envelope must decode");
    assert_eq!(decoded.offset, Offset(9001));
}

#[test]
fn explicit_offset_is_inclusive_and_beginning_uses_first() {
    let beginning = polling_strategy(Offset::BEGINNING);
    assert_eq!(beginning.kind, PollingKind::First);

    let explicit = polling_strategy(Offset(17));
    assert_eq!(explicit.kind, PollingKind::Offset);
    assert_eq!(explicit.value, 17);
}

#[test]
fn polling_never_commits_before_application_ack() {
    assert_eq!(consumer_commit_policy(), AutoCommit::Disabled);
}

#[test]
fn replay_history_is_bounded_to_the_contract_minimum() {
    assert_eq!(
        replay_retention(DEFAULT_REPLAY_RETENTION_SECONDS),
        IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(86_400))
    );
    assert_eq!(
        replay_retention(172_800),
        IggyExpiry::ExpireDuration(IggyDuration::new_from_secs(172_800))
    );
}

#[tokio::test]
async fn configured_replay_retention_rejects_zero() {
    let Err(error) = IggyBroker::with_replay_retention("iggy://unused", 0).await else {
        panic!("zero retention must fail before connecting");
    };
    assert!(error.to_string().contains("retention"));
}

#[tokio::test]
async fn dropping_the_receiver_cancels_a_pending_poll() {
    let (tx, rx) = mpsc::channel(CHANNEL_BUF);
    drop(rx);
    let mut pending = Box::pin(future::pending::<()>());
    assert!(next_while_open(&tx, pending.as_mut()).await.is_none());
}
