//! Conversions between `frf-domain` types and the generated proto types.
//!
//! Kept in the SDK (rather than shared from the gateway) because the gateway's
//! copies are private to its binary crate. The mapping mirrors
//! `flint.v1` message numbering.

use frf_domain::{Channel, EventEnvelope, EventKind, Offset};
use frf_proto::fv1;

use crate::error::SdkError;

fn kind_to_proto(kind: &EventKind) -> i32 {
    match kind {
        EventKind::EntityChange => 1,
        EventKind::AgentEvent => 2,
        EventKind::SyncOp => 3,
        EventKind::Presence => 4,
        EventKind::Signal => 5,
        // Custom and any future variant map to 6 (custom/unspecified).
        _ => 6,
    }
}

fn kind_from_proto(n: i32) -> EventKind {
    match n {
        1 => EventKind::EntityChange,
        2 => EventKind::AgentEvent,
        3 => EventKind::SyncOp,
        4 => EventKind::Presence,
        5 => EventKind::Signal,
        other => EventKind::Custom(other.to_string()),
    }
}

/// Convert a domain [`EventEnvelope`] into its proto representation.
pub(crate) fn envelope_to_proto(env: &EventEnvelope) -> fv1::EventEnvelope {
    fv1::EventEnvelope {
        id: env.id.to_string(),
        channel: Some(fv1::Channel {
            id: env.channel.id.to_string(),
            tenant_id: env.channel.tenant_id.to_string(),
            path: env.channel.path.clone(),
        }),
        offset: Some(fv1::Offset {
            value: env.offset.0,
        }),
        kind: kind_to_proto(&env.kind),
        payload: serde_json::to_vec(&env.payload).unwrap_or_default(),
        timestamp: None,
        correlation_id: env.correlation_id.clone().unwrap_or_default(),
    }
}

/// Convert a proto `EventEnvelope` into the domain type.
pub(crate) fn envelope_from_proto(proto: fv1::EventEnvelope) -> Result<EventEnvelope, SdkError> {
    let ch = proto
        .channel
        .ok_or_else(|| SdkError::InvalidResponse("missing channel".to_owned()))?;
    let offset = proto
        .offset
        .ok_or_else(|| SdkError::InvalidResponse("missing offset".to_owned()))?;

    let channel = Channel {
        id: parse_uuid_id(&ch.id, "channel id").map(frf_domain::ChannelId::from_uuid)?,
        tenant_id: parse_uuid_id(&ch.tenant_id, "tenant id")
            .map(frf_domain::TenantId::from_uuid)?,
        path: ch.path,
    };

    let payload = if proto.payload.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&proto.payload)
            .map_err(|e| SdkError::InvalidResponse(format!("invalid payload JSON: {e}")))?
    };

    let id = parse_uuid_id(&proto.id, "event id").map(frf_domain::ids::EventId::from_uuid)?;

    Ok(EventEnvelope {
        id,
        channel,
        offset: Offset(offset.value),
        kind: kind_from_proto(proto.kind),
        payload,
        timestamp: chrono::Utc::now(),
        correlation_id: if proto.correlation_id.is_empty() {
            None
        } else {
            Some(proto.correlation_id)
        },
    })
}

fn parse_uuid_id(s: &str, what: &str) -> Result<uuid::Uuid, SdkError> {
    uuid::Uuid::parse_str(s).map_err(|_| SdkError::InvalidResponse(format!("invalid {what}: {s}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, TenantId};

    fn sample_envelope() -> EventEnvelope {
        EventEnvelope::new(
            Channel {
                id: ChannelId::new(),
                tenant_id: TenantId::new(),
                path: "entity/user/updates".to_owned(),
            },
            Offset(7),
            EventKind::EntityChange,
            serde_json::json!({ "name": "alice" }),
        )
    }

    #[test]
    fn envelope_round_trips_through_proto() {
        let original = sample_envelope();
        let proto = envelope_to_proto(&original);
        let back = envelope_from_proto(proto).expect("round-trip should succeed");

        assert_eq!(back.id, original.id);
        assert_eq!(back.channel.id, original.channel.id);
        assert_eq!(back.channel.tenant_id, original.channel.tenant_id);
        assert_eq!(back.channel.path, original.channel.path);
        assert_eq!(back.kind, original.kind);
        assert_eq!(back.payload, original.payload);
    }

    #[test]
    fn from_proto_rejects_missing_channel() {
        let proto = fv1::EventEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            channel: None,
            offset: Some(fv1::Offset { value: 0 }),
            kind: 1,
            payload: vec![],
            timestamp: None,
            correlation_id: String::new(),
        };
        assert!(matches!(
            envelope_from_proto(proto),
            Err(SdkError::InvalidResponse(_))
        ));
    }

    #[test]
    fn from_proto_rejects_bad_uuid() {
        let proto = fv1::EventEnvelope {
            id: "not-a-uuid".to_owned(),
            channel: Some(fv1::Channel {
                id: uuid::Uuid::new_v4().to_string(),
                tenant_id: uuid::Uuid::new_v4().to_string(),
                path: "p".to_owned(),
            }),
            offset: Some(fv1::Offset { value: 0 }),
            kind: 1,
            payload: vec![],
            timestamp: None,
            correlation_id: String::new(),
        };
        assert!(matches!(
            envelope_from_proto(proto),
            Err(SdkError::InvalidResponse(_))
        ));
    }
}
