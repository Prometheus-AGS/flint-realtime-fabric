use frf_domain::{
    envelope::{Channel, EventEnvelope, EventKind, Offset},
    ids::{ChannelId, TenantId},
};
use frf_ports::federation::FederatedEvent;

use crate::{client::RawMatrixEvent, error::MatrixBridgeError};

/// Project a raw Matrix room event into a [`FederatedEvent`].
///
/// # Errors
///
/// Returns [`MatrixBridgeError::Projection`] if required fields are absent.
pub fn matrix_event_to_federated(
    raw: RawMatrixEvent,
    room_id: &str,
    tenant_id: TenantId,
    channel_id: ChannelId,
) -> Result<FederatedEvent, MatrixBridgeError> {
    let event_id = raw.event_id.as_deref().unwrap_or("unknown").to_owned();

    let source = format!("matrix:{room_id}/{event_id}");

    let channel = Channel {
        id: channel_id,
        tenant_id,
        path: format!("federation/matrix/{room_id}"),
    };

    let envelope = EventEnvelope::new(
        channel,
        Offset::BEGINNING,
        EventKind::Custom("matrix_room_event".to_owned()),
        raw.content,
    );

    Ok(FederatedEvent {
        protocol: frf_ports::federation::FederationProtocol::Matrix,
        source,
        envelope,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::RawMatrixEvent;

    #[test]
    fn projects_matrix_event_to_federated() {
        let raw = RawMatrixEvent {
            event_id: Some("$event-abc".to_owned()),
            sender: Some("@user:example.com".to_owned()),
            content: serde_json::json!({"msgtype": "m.text", "body": "hello"}),
        };

        let tenant_id = TenantId::new();
        let channel_id = ChannelId::new();

        let result =
            matrix_event_to_federated(raw, "!room:example.com", tenant_id, channel_id).unwrap();

        assert_eq!(result.source, "matrix:!room:example.com/$event-abc");
        assert!(result.source.starts_with("matrix:"));
        assert_eq!(
            result.protocol,
            frf_ports::federation::FederationProtocol::Matrix
        );
    }

    #[test]
    fn handles_missing_event_id() {
        let raw = RawMatrixEvent {
            event_id: None,
            sender: None,
            content: serde_json::json!({}),
        };

        let result =
            matrix_event_to_federated(raw, "!room:example.com", TenantId::new(), ChannelId::new())
                .unwrap();

        assert!(result.source.contains("unknown"));
    }
}
