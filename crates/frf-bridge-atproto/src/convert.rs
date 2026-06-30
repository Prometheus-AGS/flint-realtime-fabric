use frf_domain::{
    envelope::{Channel, EventEnvelope, EventKind, Offset},
    ids::{ChannelId, TenantId},
};
use frf_ports::federation::FederatedEvent;

use crate::error::AtProtoBridgeError;

/// Project a raw Jetstream commit event (as JSON) into a [`FederatedEvent`].
///
/// Jetstream event shape (simplified):
/// ```json
/// {
///   "did": "did:plc:abc123",
///   "time_us": 1234567890,
///   "type": "com",
///   "commit": {
///     "rev": "...",
///     "type": "c",
///     "collection": "app.bsky.feed.post",
///     "rkey": "3jwdwj2",
///     "record": { ... }
///   }
/// }
/// ```
///
/// # Errors
///
/// Returns [`AtProtoBridgeError::Projection`] if required fields are absent.
pub fn jetstream_event_to_federated(
    raw: &serde_json::Value,
    tenant_id: TenantId,
    channel_id: ChannelId,
) -> Result<FederatedEvent, AtProtoBridgeError> {
    let did = raw["did"]
        .as_str()
        .ok_or_else(|| AtProtoBridgeError::Projection("missing 'did' field".to_owned()))?
        .to_owned();

    let rkey = raw["commit"]["rkey"]
        .as_str()
        .unwrap_or("unknown")
        .to_owned();

    let source = format!("atproto:{did}/{rkey}");

    let collection = raw["commit"]["collection"]
        .as_str()
        .unwrap_or("unknown")
        .to_owned();

    let channel = Channel {
        id: channel_id,
        tenant_id,
        path: format!("federation/atproto/{collection}"),
    };

    let payload = raw["commit"]["record"].clone();

    let envelope = EventEnvelope::new(
        channel,
        Offset::BEGINNING,
        EventKind::Custom(format!("atproto_{}", collection.replace('.', "_"))),
        payload,
    );

    Ok(FederatedEvent {
        protocol: frf_ports::federation::FederationProtocol::AtProto,
        source,
        envelope,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_jetstream_commit_event() {
        let raw = serde_json::json!({
            "did": "did:plc:abc123",
            "time_us": 1_234_567_890u64,
            "type": "com",
            "commit": {
                "rev": "abc",
                "type": "c",
                "collection": "app.bsky.feed.post",
                "rkey": "3jwdwj2",
                "record": { "text": "hello world" }
            }
        });

        let result = jetstream_event_to_federated(&raw, TenantId::new(), ChannelId::new()).unwrap();

        assert_eq!(result.source, "atproto:did:plc:abc123/3jwdwj2");
        assert_eq!(
            result.protocol,
            frf_ports::federation::FederationProtocol::AtProto
        );
        assert_eq!(result.envelope.payload["text"], "hello world");
    }

    #[test]
    fn rejects_event_missing_did() {
        let raw = serde_json::json!({ "time_us": 123u64 });
        let result = jetstream_event_to_federated(&raw, TenantId::new(), ChannelId::new());
        assert!(result.is_err());
    }
}
