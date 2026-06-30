use chrono::{DateTime, TimeZone, Utc};
use frf_domain::{
    AgentEvent, AgentEventKind as DomainKind, AgentProtocol as DomainProtocol,
    ids::{AgentId, SessionId, TenantId},
};
use frf_proto::fv1::{
    AgentEvent as ProtoEvent, AgentEventKind as ProtoKind, AgentProtocol as ProtoProtocol,
};
use uuid::Uuid;

use crate::{ContentBlock, error::AgentProtoError};

/// Convert a proto `AgentEvent` into its domain counterpart.
///
/// The `content` `Struct` field is serialized to `serde_json::Value`; an
/// unrecognized `type` tag is preserved as [`ContentBlock::Unknown`].
///
/// # Errors
///
/// Returns [`AgentProtoError`] if required UUID fields cannot be parsed or
/// the timestamp is out of range.
pub fn domain_from_proto(proto: ProtoEvent) -> Result<AgentEvent, AgentProtoError> {
    let agent_id = parse_uuid_id(&proto.agent_id, "agent_id").map(AgentId::from_uuid)?;
    let tenant_id = parse_uuid_id(&proto.tenant_id, "tenant_id").map(TenantId::from_uuid)?;
    let session_id = parse_uuid_id(&proto.session_id, "session_id").map(SessionId::from_uuid)?;

    let protocol = proto_protocol_to_domain(proto.protocol);
    let kind = proto_kind_to_domain(proto.kind);

    let content_value: serde_json::Value = match proto.content {
        Some(s) => prost_struct_to_value(s),
        None => serde_json::Value::Null,
    };

    let content_block: ContentBlock = if content_value.is_null() {
        ContentBlock::Unknown
    } else {
        serde_json::from_value(content_value).unwrap_or(ContentBlock::Unknown)
    };

    let timestamp: DateTime<Utc> = match proto.timestamp {
        Some(ts) => Utc
            .timestamp_opt(ts.seconds, ts.nanos.unsigned_abs())
            .single()
            .ok_or_else(|| {
                AgentProtoError::InvalidTimestamp(format!("{}s {}ns", ts.seconds, ts.nanos))
            })?,
        None => Utc::now(),
    };

    Ok(AgentEvent {
        agent_id,
        tenant_id,
        session_id,
        protocol,
        kind,
        run_id: proto.run_id,
        content: serde_json::to_value(&content_block)?,
        timestamp,
    })
}

/// Convert a domain [`AgentEvent`] into its proto counterpart.
///
/// The `content` field is serialized from `serde_json::Value` into a
/// `prost_types::Struct`. Non-object JSON values are wrapped under a `"value"` key.
#[must_use]
pub fn domain_to_proto(event: AgentEvent) -> ProtoEvent {
    let protocol = domain_protocol_to_proto(&event.protocol);
    let kind = domain_kind_to_proto(&event.kind);

    let content_struct = json_value_to_prost_struct(event.content);

    let subsec_nanos = i32::try_from(event.timestamp.timestamp_subsec_nanos()).unwrap_or(i32::MAX);
    let timestamp = prost_types::Timestamp {
        seconds: event.timestamp.timestamp(),
        nanos: subsec_nanos,
    };

    ProtoEvent {
        agent_id: event.agent_id.to_string(),
        tenant_id: event.tenant_id.to_string(),
        session_id: event.session_id.to_string(),
        protocol: protocol as i32,
        kind: kind as i32,
        run_id: event.run_id,
        content: Some(content_struct),
        timestamp: Some(timestamp),
    }
}

fn domain_protocol_to_proto(p: &DomainProtocol) -> ProtoProtocol {
    match p {
        DomainProtocol::AgUi => ProtoProtocol::AgUi,
        DomainProtocol::A2a => ProtoProtocol::A2a,
        DomainProtocol::A2ui => ProtoProtocol::A2ui,
        DomainProtocol::Custom(_) | &_ => ProtoProtocol::Unspecified,
    }
}

fn domain_kind_to_proto(k: &DomainKind) -> ProtoKind {
    match k {
        DomainKind::RunStart => ProtoKind::RunStart,
        DomainKind::RunEnd => ProtoKind::RunEnd,
        DomainKind::TextDelta => ProtoKind::TextDelta,
        DomainKind::ToolCall => ProtoKind::ToolCall,
        DomainKind::ToolResult => ProtoKind::ToolResult,
        DomainKind::StateSnapshot => ProtoKind::StateSnapshot,
        DomainKind::Error => ProtoKind::Error,
        DomainKind::Custom(_) | &_ => ProtoKind::Unspecified,
    }
}

fn json_value_to_prost_struct(value: serde_json::Value) -> prost_types::Struct {
    if let serde_json::Value::Object(map) = value {
        let fields: std::collections::BTreeMap<_, _> = map
            .into_iter()
            .map(|(k, v)| (k, json_value_to_prost_value(v)))
            .collect();
        prost_types::Struct { fields }
    } else {
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("value".to_owned(), json_value_to_prost_value(value));
        prost_types::Struct { fields }
    }
}

fn json_value_to_prost_value(value: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.into_iter().map(json_value_to_prost_value).collect(),
        }),
        serde_json::Value::Object(map) => Kind::StructValue(prost_types::Struct {
            fields: map
                .into_iter()
                .map(|(k, v)| (k, json_value_to_prost_value(v)))
                .collect::<std::collections::BTreeMap<_, _>>(),
        }),
    };
    prost_types::Value { kind: Some(kind) }
}

fn parse_uuid_id(s: &str, field: &'static str) -> Result<Uuid, AgentProtoError> {
    Uuid::parse_str(s).map_err(|_| AgentProtoError::MissingField(field))
}

fn proto_protocol_to_domain(raw: i32) -> DomainProtocol {
    match ProtoProtocol::try_from(raw).unwrap_or(ProtoProtocol::Unspecified) {
        ProtoProtocol::AgUi => DomainProtocol::AgUi,
        ProtoProtocol::A2a => DomainProtocol::A2a,
        ProtoProtocol::A2ui => DomainProtocol::A2ui,
        ProtoProtocol::Unspecified => DomainProtocol::Custom("unspecified".to_owned()),
    }
}

fn proto_kind_to_domain(raw: i32) -> DomainKind {
    match ProtoKind::try_from(raw).unwrap_or(ProtoKind::Unspecified) {
        ProtoKind::RunStart => DomainKind::RunStart,
        ProtoKind::RunEnd => DomainKind::RunEnd,
        ProtoKind::TextDelta => DomainKind::TextDelta,
        ProtoKind::ToolCall => DomainKind::ToolCall,
        ProtoKind::ToolResult => DomainKind::ToolResult,
        ProtoKind::StateSnapshot => DomainKind::StateSnapshot,
        ProtoKind::Error => DomainKind::Error,
        ProtoKind::Unspecified => DomainKind::Custom("unspecified".to_owned()),
    }
}

fn prost_struct_to_value(s: prost_types::Struct) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = s
        .fields
        .into_iter()
        .map(|(k, v)| (k, prost_value_to_json(v)))
        .collect();
    serde_json::Value::Object(map)
}

fn prost_value_to_json(v: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match v.kind {
        Some(Kind::NullValue(_)) | None => serde_json::Value::Null,
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(Kind::NumberValue(n)) => serde_json::Number::from_f64(n)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.into_iter().map(prost_value_to_json).collect())
        }
        Some(Kind::StructValue(s)) => {
            let map = s
                .fields
                .into_iter()
                .map(|(k, v2)| (k, prost_value_to_json(v2)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}
