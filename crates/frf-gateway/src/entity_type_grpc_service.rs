use std::pin::Pin;
use std::sync::Arc;

use frf_app::{
    AppError, EntityTypeWatchFrame, EntityTypeWatchRequest, EntityTypeWatchUseCase,
    ResnapshotReason as AppResnapshotReason, TypeWatchStart, WatchCheckpoint,
};
use frf_domain::{
    CanonicalValue, ChangeOp, EntityField, EntityKey, EntityTypeSelector, SourcePosition,
};
use frf_ports::{AuthzProvider, EntityStore, EntityTypeWatchSource, IdentityVerifier, PortError};
use frf_proto::fv2::{
    self,
    entity_service_server::{EntityService, EntityServiceServer},
    watch_entity_type_request, watch_entity_type_response,
};
use futures_util::{Stream, StreamExt as _};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

pub struct EntityTypeGrpcService<S, P: ?Sized, A, I> {
    use_case: Arc<EntityTypeWatchUseCase<S, P, A, I>>,
}

impl<S, P, A, I> EntityTypeGrpcService<S, P, A, I>
where
    S: EntityTypeWatchSource,
    P: EntityStore + ?Sized,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    #[must_use]
    pub fn new(use_case: Arc<EntityTypeWatchUseCase<S, P, A, I>>) -> Self {
        Self { use_case }
    }

    #[must_use]
    pub fn into_server(self) -> EntityServiceServer<Self> {
        EntityServiceServer::new(self)
    }
}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<fv2::WatchEntityTypeResponse, Status>> + Send>>;

#[tonic::async_trait]
impl<S, P, A, I> EntityService for EntityTypeGrpcService<S, P, A, I>
where
    S: EntityTypeWatchSource,
    P: EntityStore + ?Sized,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    type WatchEntityTypeStream = ResponseStream;

    #[instrument(name = "grpc::watch_entity_type", skip(self, request))]
    async fn watch_entity_type(
        &self,
        request: Request<fv2::WatchEntityTypeRequest>,
    ) -> Result<Response<Self::WatchEntityTypeStream>, Status> {
        let bearer_token = extract_bearer(&request)?;
        let input = request.into_inner();
        let entity_type = input
            .entity_type
            .ok_or_else(|| Status::invalid_argument("entity_type is required"))?;
        let entity_type = EntityTypeSelector {
            schema: entity_type.schema,
            name: entity_type.name,
            projection: entity_type.projection,
        };
        let tenant_id = Uuid::parse_str(&input.tenant_id)
            .map(frf_domain::TenantId::from_uuid)
            .map_err(|_| Status::invalid_argument("tenant_id must be a UUID"))?;
        let start = match input.start {
            Some(watch_entity_type_request::Start::Snapshot(_)) => TypeWatchStart::Snapshot,
            Some(watch_entity_type_request::Start::Live(_)) => TypeWatchStart::Live,
            Some(watch_entity_type_request::Start::Resume(checkpoint)) => {
                TypeWatchStart::Resume(WatchCheckpoint {
                    version: checkpoint.version,
                    token: checkpoint.token,
                    generation: checkpoint.generation,
                })
            }
            None => return Err(Status::invalid_argument("one start mode is required")),
        };
        let stream = self
            .use_case
            .watch(EntityTypeWatchRequest {
                entity_type,
                tenant_id,
                start,
                bearer_token,
            })
            .await
            .map_err(app_error_to_status)?;
        Ok(Response::new(Box::pin(stream.map(|item| {
            item.map_err(port_error_to_status).and_then(frame_to_proto)
        }))))
    }
}

fn extract_bearer<T>(request: &Request<T>) -> Result<String, Status> {
    request
        .metadata()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned)
        .ok_or_else(|| Status::unauthenticated("missing Bearer token"))
}

fn app_error_to_status(error: AppError) -> Status {
    match error {
        AppError::Forbidden(message) => Status::permission_denied(message),
        AppError::Unauthorized(message) => Status::unauthenticated(message),
        AppError::Identity(error) => Status::unauthenticated(error.to_string()),
        AppError::Broker(error) => port_error_to_status(error),
        _ => Status::internal("unexpected entity watch error"),
    }
}

fn port_error_to_status(error: PortError) -> Status {
    match error {
        PortError::PermissionDenied(message) => Status::permission_denied(message),
        PortError::NotFound(message) => Status::failed_precondition(message),
        PortError::Timeout => Status::deadline_exceeded("entity watch timed out"),
        other => Status::internal(other.to_string()),
    }
}

fn frame_to_proto(frame: EntityTypeWatchFrame) -> Result<fv2::WatchEntityTypeResponse, Status> {
    let frame = match frame {
        EntityTypeWatchFrame::Accepted(accepted) => {
            watch_entity_type_response::Frame::Accepted(fv2::WatchAccepted {
                stream_id: accepted.stream_id,
                entity_type: Some(entity_type_to_proto(&accepted.entity_type)),
                tenant_id: accepted.tenant_id.to_string(),
                retention_seconds: accepted.retention_seconds,
                buffer_capacity: accepted.buffer_capacity,
                start_checkpoint: Some(checkpoint_to_proto(accepted.start_checkpoint)),
                snapshot_barrier: accepted.snapshot_barrier.map(source_to_proto),
            })
        }
        EntityTypeWatchFrame::SnapshotRow(row) => {
            watch_entity_type_response::Frame::SnapshotRow(fv2::SnapshotRow {
                snapshot_id: row.snapshot_id,
                key: Some(key_to_proto(row.key)?),
                record: Some(record_to_proto(row.record)?),
            })
        }
        EntityTypeWatchFrame::SnapshotComplete {
            barrier,
            checkpoint,
            row_count,
        } => watch_entity_type_response::Frame::SnapshotComplete(fv2::SnapshotComplete {
            barrier: Some(source_to_proto(barrier)),
            checkpoint: Some(checkpoint_to_proto(checkpoint)),
            row_count,
        }),
        EntityTypeWatchFrame::Mutation {
            delivery,
            checkpoint,
        } => {
            let mutation = delivery.mutation;
            watch_entity_type_response::Frame::Mutation(fv2::EntityMutation {
                event_id: mutation.event_id,
                key: Some(key_to_proto(mutation.key)?),
                op: change_op_to_proto(&mutation.op)?.into(),
                record: if mutation.op == ChangeOp::Delete {
                    None
                } else {
                    mutation.record.map(record_to_proto).transpose()?
                },
                source: Some(source_to_proto(mutation.source)),
                broker: Some(fv2::BrokerPosition {
                    partition: delivery.broker_partition,
                    offset: delivery.broker_offset,
                }),
                checkpoint: Some(checkpoint_to_proto(checkpoint)),
                committed_at: Some(prost_types::Timestamp {
                    seconds: mutation.committed_at.timestamp(),
                    nanos: i32::try_from(mutation.committed_at.timestamp_subsec_nanos())
                        .unwrap_or_default(),
                }),
                entity_type: Some(fv2::EntityType {
                    schema: mutation.schema,
                    name: mutation.table,
                    projection: mutation.projection,
                }),
                tenant_id: mutation.tenant_id.to_string(),
            })
        }
        EntityTypeWatchFrame::CheckpointAdvanced(checkpoint) => {
            watch_entity_type_response::Frame::CheckpointAdvanced(fv2::CheckpointAdvanced {
                checkpoint: Some(checkpoint_to_proto(checkpoint)),
            })
        }
        EntityTypeWatchFrame::ResnapshotRequired { reason, message } => {
            watch_entity_type_response::Frame::ResnapshotRequired(fv2::ResnapshotRequired {
                reason: resnapshot_reason_to_proto(reason).into(),
                message,
            })
        }
        EntityTypeWatchFrame::Lagged {
            checkpoint_resumable,
            last_safe_checkpoint,
        } => watch_entity_type_response::Frame::Lagged(fv2::Lagged {
            checkpoint_resumable,
            last_safe_checkpoint: last_safe_checkpoint.map(checkpoint_to_proto),
        }),
        _ => return Err(Status::data_loss("unsupported entity watch frame")),
    };
    Ok(fv2::WatchEntityTypeResponse { frame: Some(frame) })
}

fn entity_type_to_proto(entity_type: &EntityTypeSelector) -> fv2::EntityType {
    fv2::EntityType {
        schema: entity_type.schema.clone(),
        name: entity_type.name.clone(),
        projection: entity_type.projection.clone(),
    }
}

fn checkpoint_to_proto(checkpoint: WatchCheckpoint) -> fv2::ClientCheckpoint {
    fv2::ClientCheckpoint {
        version: checkpoint.version,
        token: checkpoint.token,
        generation: checkpoint.generation,
    }
}

fn source_to_proto(source: SourcePosition) -> fv2::SourcePosition {
    fv2::SourcePosition {
        epoch: source.epoch,
        commit_lsn: source.commit_lsn,
        transaction_index: source.transaction_index,
    }
}

fn key_to_proto(key: EntityKey) -> Result<fv2::EntityKey, Status> {
    Ok(fv2::EntityKey {
        parts: key
            .parts
            .into_iter()
            .map(|part| {
                Ok(fv2::KeyPart {
                    column: part.column,
                    value: Some(canonical_to_proto(part.value)?),
                })
            })
            .collect::<Result<Vec<_>, Status>>()?,
        canonical_id: key.canonical_id,
    })
}

fn record_to_proto(fields: Vec<EntityField>) -> Result<fv2::EntityRecord, Status> {
    Ok(fv2::EntityRecord {
        fields: fields
            .into_iter()
            .map(|field| {
                Ok(fv2::EntityField {
                    column: field.column,
                    value: Some(canonical_to_proto(field.value)?),
                })
            })
            .collect::<Result<Vec<_>, Status>>()?,
    })
}

fn canonical_to_proto(value: CanonicalValue) -> Result<fv2::CanonicalValue, Status> {
    use fv2::canonical_value::Kind;
    let kind = match value {
        CanonicalValue::Null => Kind::NullValue(fv2::NullKind::Null.into()),
        CanonicalValue::Bool(value) => Kind::BoolValue(value),
        CanonicalValue::SignedInteger(value) => Kind::SignedInteger(value),
        CanonicalValue::UnsignedInteger(value) => Kind::UnsignedInteger(value),
        CanonicalValue::Float(value) => Kind::FloatValue(value),
        CanonicalValue::Text(value) => Kind::TextValue(value),
        CanonicalValue::Bytes(value) => Kind::BytesValue(decode_base64url(&value)?),
        CanonicalValue::Decimal(value) => Kind::DecimalValue(value),
        CanonicalValue::Uuid(value) => Kind::UuidValue(value),
        CanonicalValue::Timestamp { seconds, nanos } => {
            Kind::TimestampValue(prost_types::Timestamp { seconds, nanos })
        }
        CanonicalValue::Date(value) => Kind::DateValue(value),
        CanonicalValue::Time(value) => Kind::TimeValue(value),
        CanonicalValue::TimestampWithoutZone(_) => {
            return Err(Status::data_loss(
                "timestamp without time zone has no lossless v2 canonical encoding",
            ));
        }
        CanonicalValue::Json(value) => Kind::JsonValue(decode_base64url(&value)?),
        _ => return Err(Status::data_loss("unsupported canonical source value")),
    };
    Ok(fv2::CanonicalValue { kind: Some(kind) })
}

fn decode_base64url(value: &str) -> Result<Vec<u8>, Status> {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| Status::data_loss("invalid canonical base64url value"))
}

fn change_op_to_proto(op: &ChangeOp) -> Result<fv2::ChangeOp, Status> {
    Ok(match op {
        ChangeOp::Insert => fv2::ChangeOp::Insert,
        ChangeOp::Update => fv2::ChangeOp::Update,
        ChangeOp::Delete => fv2::ChangeOp::Delete,
        ChangeOp::Upsert => fv2::ChangeOp::Upsert,
        _ => return Err(Status::data_loss("unsupported entity mutation operation")),
    })
}

fn resnapshot_reason_to_proto(reason: AppResnapshotReason) -> fv2::ResnapshotReason {
    match reason {
        AppResnapshotReason::HistoryExpired => fv2::ResnapshotReason::HistoryExpired,
        AppResnapshotReason::SourceEpochChanged => fv2::ResnapshotReason::SourceEpochChanged,
        AppResnapshotReason::CheckpointScopeMismatch => {
            fv2::ResnapshotReason::CheckpointScopeMismatch
        }
        _ => fv2::ResnapshotReason::CheckpointUnsupported,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use chrono::Utc;
    use frf_app::{EntityTypeWatchFrame, WatchCheckpoint};
    use frf_domain::{
        CanonicalValue, ChangeOp, CommittedEntityMutation, EntityField, EntityKey,
        EntityTypeDelivery, KeyPart, SourcePosition, TenantId,
    };

    use super::{canonical_to_proto, frame_to_proto};

    #[test]
    fn malformed_canonical_bytes_fail_closed() {
        let status = canonical_to_proto(CanonicalValue::Bytes("%%%".to_owned()))
            .expect_err("malformed base64url must fail");
        assert_eq!(status.code(), tonic::Code::DataLoss);
    }

    #[test]
    fn timestamp_without_zone_fails_instead_of_changing_type() {
        let status = canonical_to_proto(CanonicalValue::TimestampWithoutZone(
            "2026-09-16T12:34:56.123456".to_owned(),
        ))
        .expect_err("the frozen v2 proto has no lossless timestamp-without-zone field");
        assert_eq!(status.code(), tonic::Code::DataLoss);
    }

    #[test]
    fn delete_transport_never_exposes_a_record() {
        let tenant = TenantId::from_uuid(uuid::Uuid::from_u128(1));
        let key = EntityKey {
            parts: vec![KeyPart {
                column: "id".to_owned(),
                value: CanonicalValue::Text("one".to_owned()),
            }],
            canonical_id: "frfkey:v1:one".to_owned(),
        };
        let frame = EntityTypeWatchFrame::Mutation {
            delivery: EntityTypeDelivery {
                mutation: CommittedEntityMutation {
                    event_id: "frfevent:v1:one".to_owned(),
                    schema: "public".to_owned(),
                    table: "orders".to_owned(),
                    projection: "default".to_owned(),
                    entity_type: "public.orders@default".to_owned(),
                    tenant_id: tenant,
                    key,
                    op: ChangeOp::Delete,
                    record: Some(vec![EntityField {
                        column: "secret".to_owned(),
                        value: CanonicalValue::Text("must-not-cross".to_owned()),
                    }]),
                    unchanged_toast: Vec::new(),
                    source: SourcePosition {
                        epoch: "epoch".to_owned(),
                        commit_lsn: 1,
                        transaction_index: 0,
                    },
                    committed_at: Utc::now(),
                },
                broker_partition: 0,
                broker_offset: 1,
            },
            checkpoint: WatchCheckpoint {
                version: 1,
                token: vec![1],
                generation: 1,
            },
        };
        let response = frame_to_proto(frame).expect("delete frame must encode");
        let Some(frf_proto::fv2::watch_entity_type_response::Frame::Mutation(mutation)) =
            response.frame
        else {
            panic!("expected mutation frame");
        };
        assert!(mutation.record.is_none());
    }
}
