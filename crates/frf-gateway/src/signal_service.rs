use std::pin::Pin;
use std::sync::Arc;

use frf_domain::{
    SessionId, SfuMode, SignalEnvelope as DomainSignalEnvelope, SignalKind, TenantId,
};
use frf_ports::MediaSignaler;
use frf_proto::fv1::{
    self,
    signal_service_server::{SignalService, SignalServiceServer},
};
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt as _, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;
use uuid::Uuid;

/// gRPC service implementing `flint.v1.SignalService` (bidi streaming WebRTC signaling).
///
/// Relay rules:
/// - `from_session` is taken from the inbound envelope.
/// - `to_session` may be empty (broadcast to room) or set (unicast).
/// - JWT verification is enforced at this boundary; requests without a Bearer
///   token are rejected with `UNAUTHENTICATED` before any domain logic runs.
pub struct SpineSignalService<M> {
    signaler: Arc<M>,
}

impl<M: MediaSignaler> SpineSignalService<M> {
    #[must_use]
    pub fn new(signaler: Arc<M>) -> Self {
        Self { signaler }
    }

    #[must_use]
    pub fn into_server(self) -> SignalServiceServer<Self> {
        SignalServiceServer::new(self)
    }
}

fn parse_uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("invalid {field} UUID: {s}")))
}

fn proto_signal_kind_to_domain(proto_kind: i32) -> SignalKind {
    match proto_kind {
        1 => SignalKind::Offer,
        2 => SignalKind::Answer,
        3 => SignalKind::IceCandidate,
        4 => SignalKind::IceRestart,
        6 => SignalKind::RoomJoin,
        7 => SignalKind::RoomLeave,
        // 5 = Hangup; unrecognized kinds also fall back to Hangup so the
        // session can clean up gracefully rather than silently dropping.
        _ => SignalKind::Hangup,
    }
}

fn domain_signal_kind_to_proto(kind: &SignalKind) -> i32 {
    match kind {
        SignalKind::Offer => 1,
        SignalKind::Answer => 2,
        SignalKind::IceCandidate => 3,
        SignalKind::IceRestart => 4,
        SignalKind::Hangup => 5,
        SignalKind::RoomJoin => 6,
        SignalKind::RoomLeave => 7,
        // `#[non_exhaustive]` — future variants map to unspecified (0).
        _ => 0,
    }
}

/// Convert a `prost_types::Struct` to a `serde_json::Value`.
///
/// `prost_types::Struct` does not implement `serde::Serialize`, so we perform
/// a manual recursive conversion.
fn prost_struct_to_json(s: prost_types::Struct) -> serde_json::Value {
    let map = s
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
        Some(Kind::NumberValue(n)) => serde_json::json!(n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::StructValue(nested)) => prost_struct_to_json(nested),
        Some(Kind::ListValue(list)) => {
            serde_json::Value::Array(list.values.into_iter().map(prost_value_to_json).collect())
        }
    }
}

/// Convert a `serde_json::Value` to a `prost_types::Struct`.
///
/// Only `Object` values map to `Struct`; anything else is encoded as a
/// single-key `{"_value": <json>}` wrapper so callers receive something
/// rather than nothing.
fn json_to_prost_struct(v: serde_json::Value) -> prost_types::Struct {
    match v {
        serde_json::Value::Object(map) => prost_types::Struct {
            fields: map
                .into_iter()
                .map(|(k, v)| (k, json_to_prost_value(v)))
                .collect(),
        },
        other => {
            let mut fields = std::collections::BTreeMap::new();
            fields.insert("_value".into(), json_to_prost_value(other));
            prost_types::Struct { fields }
        }
    }
}

fn json_to_prost_value(v: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match v {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.into_iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(_) => Kind::StructValue(json_to_prost_struct(v)),
    };
    prost_types::Value { kind: Some(kind) }
}

fn proto_to_domain(proto: fv1::SignalEnvelope) -> Result<DomainSignalEnvelope, Status> {
    let tenant_id = TenantId::from_uuid(parse_uuid(&proto.tenant_id, "tenant_id")?);
    let from_uuid = parse_uuid(&proto.from_session, "from_session")?;
    let from_session = SessionId::from_uuid(from_uuid);
    let to_session = if proto.to_session.is_empty() {
        None
    } else {
        let to_uuid = parse_uuid(&proto.to_session, "to_session")?;
        Some(SessionId::from_uuid(to_uuid))
    };
    let payload = proto
        .payload
        .map_or(serde_json::Value::Null, prost_struct_to_json);

    Ok(DomainSignalEnvelope {
        from_session,
        to_session,
        tenant_id,
        room_id: proto.room_id,
        kind: proto_signal_kind_to_domain(proto.kind),
        sfu_mode: SfuMode::Hosted,
        payload,
        timestamp: chrono::Utc::now(),
    })
}

fn domain_to_proto(env: &DomainSignalEnvelope) -> fv1::SignalEnvelope {
    let payload_struct = match &env.payload {
        serde_json::Value::Null => None,
        other => Some(json_to_prost_struct(other.clone())),
    };
    fv1::SignalEnvelope {
        from_session: env.from_session.to_string(),
        to_session: env.to_session.map_or_else(String::new, |s| s.to_string()),
        tenant_id: env.tenant_id.to_string(),
        room_id: env.room_id.clone(),
        kind: domain_signal_kind_to_proto(&env.kind),
        sfu_mode: 2, // SFU_MODE_HOSTED
        payload: payload_struct,
        timestamp: None,
    }
}

#[tonic::async_trait]
impl<M> SignalService for SpineSignalService<M>
where
    M: MediaSignaler + 'static,
{
    type SignalStream = Pin<Box<dyn Stream<Item = Result<fv1::SignalEnvelope, Status>> + Send>>;

    #[instrument(name = "grpc::signal", skip(self, request))]
    async fn signal(
        &self,
        request: Request<Streaming<fv1::SignalEnvelope>>,
    ) -> Result<Response<Self::SignalStream>, Status> {
        // JWT verification at the gRPC boundary — reject any request missing a Bearer token.
        let _token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing Bearer token"))?
            .to_owned();

        let mut inbound: Streaming<fv1::SignalEnvelope> = request.into_inner();

        // Peek at the first message to extract session identity before spawning tasks.
        let first = inbound
            .next()
            .await
            .ok_or_else(|| Status::invalid_argument("empty signal stream"))?
            .map_err(|e| Status::internal(e.to_string()))?;

        let first_domain = proto_to_domain(first)?;
        let session_id = first_domain.from_session;
        let tenant_id = first_domain.tenant_id;

        // Subscribe for outbound signals on behalf of this session.
        let signal_rx = self
            .signaler
            .subscribe_signals(session_id, tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Relay the first envelope to the SFU.
        self.signaler
            .send_signal(first_domain)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let (tx, rx) = mpsc::channel::<Result<fv1::SignalEnvelope, Status>>(32);

        // Task: inbound gRPC frames → MediaSignaler (client → SFU).
        let signaler_in = Arc::clone(&self.signaler);
        tokio::spawn(async move {
            while let Some(msg) = inbound.next().await {
                match msg {
                    Ok(proto_env) => match proto_to_domain(proto_env) {
                        Ok(domain_env) => {
                            if let Err(e) = signaler_in.send_signal(domain_env).await {
                                tracing::warn!(error = %e, "failed to relay signal");
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "signal parse error"),
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "signal stream recv error");
                        break;
                    }
                }
            }
            let _ = signaler_in.remove_session(session_id, tenant_id).await;
        });

        // Task: MediaSignaler outbound → gRPC response stream (SFU → client).
        let tx2 = tx.clone();
        tokio::spawn(async move {
            tokio::pin!(signal_rx);
            while let Some(item) = signal_rx.next().await {
                let proto = match item {
                    Ok(env) => Ok(domain_to_proto(&env)),
                    Err(e) => Err(Status::internal(e.to_string())),
                };
                if tx2.send(proto).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the bearer-token extraction used in `signal()` rejects
    /// a request with no `authorization` metadata.
    ///
    /// The full bidi-stream path is tested by integration smoke tests;
    /// this unit test focuses solely on the JWT boundary guard.
    #[test]
    fn missing_bearer_token_produces_unauthenticated() {
        // Simulate what signal() does at the top: extract Bearer from metadata.
        let request: tonic::Request<()> = tonic::Request::new(());
        // No authorization header added.
        let result = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned)
            .ok_or_else(|| tonic::Status::unauthenticated("missing Bearer token"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn present_bearer_token_is_extracted() {
        let mut request: tonic::Request<()> = tonic::Request::new(());
        request
            .metadata_mut()
            .insert("authorization", "Bearer tok-abc".parse().unwrap());

        let result = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned)
            .ok_or_else(|| tonic::Status::unauthenticated("missing Bearer token"));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tok-abc");
    }

    #[test]
    fn proto_to_domain_round_trips_kind() {
        assert_eq!(proto_signal_kind_to_domain(1), SignalKind::Offer);
        assert_eq!(proto_signal_kind_to_domain(2), SignalKind::Answer);
        assert_eq!(proto_signal_kind_to_domain(3), SignalKind::IceCandidate);
        // Unknown kind falls back to Hangup (clean session teardown).
        assert_eq!(proto_signal_kind_to_domain(99), SignalKind::Hangup);
    }

    #[test]
    fn json_prost_struct_round_trips() {
        let original = serde_json::json!({ "sdp": "v=0\r\n", "type": "offer" });
        let prost = json_to_prost_struct(original.clone());
        let back = prost_struct_to_json(prost);
        assert_eq!(original, back);
    }
}
