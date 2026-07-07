use std::pin::Pin;
use std::sync::Arc;

use frf_app::{AppError, EntityRequest as AppEntityRequest, EntityUseCase};
use frf_domain::{ChangeOp, EntityChange, EntityId, TenantId};
use frf_ports::{AuthzProvider, EntityStore, IdentityVerifier, PortError};
use frf_proto::fv1::{
    self,
    entity_service_server::{EntityService, EntityServiceServer},
};
use tokio_stream::{Stream, StreamExt as _};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

/// gRPC service implementing `flint.v1.EntityService` (read side of the entity plane).
///
/// The concrete adapter types (`S`, `A`, `I`) are wired in `main.rs`; this file is
/// adapter-agnostic beyond the port-trait bounds. Conversion between the domain
/// `EntityChange` and the proto message lives here — no `frf-domain`/`frf-app` crate
/// imports `frf-proto`.
pub struct EntityGrpcService<S, A, I> {
    use_case: Arc<EntityUseCase<S, A, I>>,
}

impl<S, A, I> EntityGrpcService<S, A, I>
where
    S: EntityStore,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    #[must_use]
    pub fn new(use_case: Arc<EntityUseCase<S, A, I>>) -> Self {
        Self { use_case }
    }

    #[must_use]
    pub fn into_server(self) -> EntityServiceServer<Self> {
        EntityServiceServer::new(self)
    }
}

fn app_error_to_status(err: AppError) -> Status {
    match err {
        AppError::Forbidden(msg) => Status::permission_denied(msg),
        AppError::Identity(e) => Status::unauthenticated(e.to_string()),
        AppError::Broker(e) => Status::internal(e.to_string()),
        _ => Status::internal("unexpected error"),
    }
}

fn port_error_to_status(err: &PortError) -> Status {
    Status::internal(err.to_string())
}

fn parse_entity_id(s: &str) -> Result<EntityId, Status> {
    Uuid::parse_str(s)
        .map(EntityId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid entity_id UUID: {s}")))
}

fn parse_tenant_id(s: &str) -> Result<TenantId, Status> {
    Uuid::parse_str(s)
        .map(TenantId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid tenant_id UUID: {s}")))
}

fn extract_bearer<T>(request: &Request<T>) -> Result<String, Status> {
    request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
        .ok_or_else(|| Status::unauthenticated("missing Bearer token"))
}

fn change_op_to_proto(op: &ChangeOp) -> i32 {
    // Domain `ChangeOp` has no `Unspecified`; map each variant to its proto number.
    let proto = match op {
        ChangeOp::Insert => fv1::ChangeOp::Insert,
        ChangeOp::Update => fv1::ChangeOp::Update,
        ChangeOp::Delete => fv1::ChangeOp::Delete,
        ChangeOp::Upsert => fv1::ChangeOp::Upsert,
        // `#[non_exhaustive]` guard: any future variant maps to Unspecified rather than
        // silently miscoding as an existing op.
        _ => fv1::ChangeOp::Unspecified,
    };
    proto as i32
}

/// Convert a `serde_json::Value` to a `prost_types::Struct`. Only JSON objects map to a
/// `Struct`; a non-object value is wrapped under a single `"_value"` key so the caller
/// receives the data rather than an empty struct.
fn json_to_prost_struct(value: &serde_json::Value) -> prost_types::Struct {
    match value {
        serde_json::Value::Object(map) => prost_types::Struct {
            fields: map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                .collect(),
        },
        other => prost_types::Struct {
            fields: [("_value".to_owned(), json_to_prost_value(other))]
                .into_iter()
                .collect(),
        },
    }
}

fn json_to_prost_value(value: &serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind;
    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        // A non-representable number (NaN/inf can't appear in serde_json) falls back to 0.
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(items) => Kind::ListValue(prost_types::ListValue {
            values: items.iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(_) => Kind::StructValue(json_to_prost_struct(value)),
    };
    prost_types::Value { kind: Some(kind) }
}

fn prost_timestamp(ts: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: ts.timestamp(),
        // Subsecond nanos are always 0..1_000_000_000, well within i32, but convert
        // fallibly so the cast can never wrap. 0 is a harmless fallback.
        nanos: i32::try_from(ts.timestamp_subsec_nanos()).unwrap_or(0),
    }
}

fn domain_change_to_proto(change: &EntityChange) -> fv1::EntityChange {
    fv1::EntityChange {
        entity_id: change.entity_id.to_string(),
        tenant_id: change.tenant_id.to_string(),
        entity_type: change.entity_type.clone(),
        op: change_op_to_proto(&change.op),
        data: Some(json_to_prost_struct(&change.data)),
        previous: change.previous.as_ref().map(json_to_prost_struct),
        session_id: change.session_id.map(|s| s.to_string()).unwrap_or_default(),
        timestamp: Some(prost_timestamp(change.timestamp)),
        version: change.version,
    }
}

#[tonic::async_trait]
impl<S, A, I> EntityService for EntityGrpcService<S, A, I>
where
    S: EntityStore,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    #[instrument(name = "grpc::get_entity", skip(self, request))]
    async fn get_entity(
        &self,
        request: Request<fv1::GetEntityRequest>,
    ) -> Result<Response<fv1::EntityResponse>, Status> {
        let bearer_token = extract_bearer(&request)?;
        let req = request.into_inner();
        let app_req = AppEntityRequest {
            entity_id: parse_entity_id(&req.entity_id)?,
            tenant_id: parse_tenant_id(&req.tenant_id)?,
            bearer_token,
        };

        let change = self
            .use_case
            .get(app_req)
            .await
            .map_err(app_error_to_status)?;

        Ok(Response::new(fv1::EntityResponse {
            entity: change.as_ref().map(domain_change_to_proto),
        }))
    }

    type WatchEntityStream = Pin<Box<dyn Stream<Item = Result<fv1::EntityChange, Status>> + Send>>;

    #[instrument(name = "grpc::watch_entity", skip(self, request))]
    async fn watch_entity(
        &self,
        request: Request<fv1::WatchEntityRequest>,
    ) -> Result<Response<Self::WatchEntityStream>, Status> {
        let bearer_token = extract_bearer(&request)?;
        let req = request.into_inner();
        let app_req = AppEntityRequest {
            entity_id: parse_entity_id(&req.entity_id)?,
            tenant_id: parse_tenant_id(&req.tenant_id)?,
            bearer_token,
        };

        let stream = self
            .use_case
            .watch(app_req)
            .await
            .map_err(app_error_to_status)?;

        let mapped = stream.map(|res| {
            res.map(|change| domain_change_to_proto(&change))
                .map_err(|e| port_error_to_status(&e))
        });
        Ok(Response::new(Box::pin(mapped)))
    }
}
