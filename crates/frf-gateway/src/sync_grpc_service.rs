use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use frf_app::{SyncRequest as AppSyncRequest, SyncUseCase};
use frf_domain::{EntityId, TenantId};
use frf_ports::{ApplyDelta, CrdtStore, OpStore};
use frf_proto::fv1::{
    self,
    sync_service_server::{SyncService, SyncServiceServer},
};
use tokio_stream::{Stream, StreamExt as _, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;
use uuid::Uuid;

/// gRPC service implementing `flint.v1.SyncService` (bidi streaming CRDT sync).
///
/// The concrete adapter types (`S`, `O`, `A`) are wired in `main.rs`;
/// this file is adapter-agnostic beyond the port trait bounds.
pub struct SyncGrpcService<S, O, A> {
    use_case: Arc<SyncUseCase<S, O, A>>,
}

impl<S, O, A> SyncGrpcService<S, O, A>
where
    S: CrdtStore,
    O: OpStore,
    A: ApplyDelta,
{
    #[must_use]
    pub fn new(use_case: Arc<SyncUseCase<S, O, A>>) -> Self {
        Self { use_case }
    }

    #[must_use]
    pub fn into_server(self) -> SyncServiceServer<Self>
    where
        S: CrdtStore,
        O: OpStore,
        A: ApplyDelta,
    {
        SyncServiceServer::new(self)
    }
}

fn parse_uuid(s: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(s).map_err(|_| Status::invalid_argument(format!("invalid {field} UUID: {s}")))
}

fn parse_entity_id(s: &str) -> Result<EntityId, Status> {
    parse_uuid(s, "entity_id").map(EntityId::from_uuid)
}

fn parse_tenant_id(s: &str) -> Result<TenantId, Status> {
    parse_uuid(s, "tenant_id").map(TenantId::from_uuid)
}

#[tonic::async_trait]
impl<S, O, A> SyncService for SyncGrpcService<S, O, A>
where
    S: CrdtStore,
    O: OpStore,
    A: ApplyDelta,
{
    type SyncStream = Pin<Box<dyn Stream<Item = Result<fv1::SyncResponse, Status>> + Send>>;

    /// Bidi streaming: client sends `SyncRequest` frames containing ops;
    /// server replies with merged-op batches and updated checkpoints.
    #[instrument(name = "grpc::sync", skip(self, request))]
    async fn sync(
        &self,
        request: Request<Streaming<fv1::SyncRequest>>,
    ) -> Result<Response<Self::SyncStream>, Status> {
        let use_case = Arc::clone(&self.use_case);
        let mut inbound: Streaming<fv1::SyncRequest> = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<fv1::SyncResponse, Status>>(32);

        tokio::spawn(async move {
            while let Some(msg) = inbound.next().await {
                let frame = match msg {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(e.to_string()))).await;
                        break;
                    }
                };

                for op in frame.ops {
                    let entity_id = match parse_entity_id(&op.entity_id) {
                        Ok(id) => id,
                        Err(s) => {
                            let _ = tx.send(Err(s)).await;
                            continue;
                        }
                    };
                    let tenant_id = match parse_tenant_id(&op.tenant_id) {
                        Ok(id) => id,
                        Err(s) => {
                            let _ = tx.send(Err(s)).await;
                            continue;
                        }
                    };

                    let req = AppSyncRequest {
                        entity_id,
                        tenant_id,
                        delta: Bytes::from(op.payload.clone()),
                        // `lamport` doubles as both confirmed_seq and new_version
                        // for the v1 protocol — the server uses the client's lamport
                        // as the acknowledged seq watermark.
                        confirmed_seq: op.lamport.saturating_sub(1),
                        new_version: op.lamport,
                    };

                    let result = use_case.apply_server_delta(req).await;

                    let response = match result {
                        Ok(sr) => {
                            let checkpoint = fv1::SyncCheckpoint {
                                entity_id: sr.entity_id.to_string(),
                                tenant_id: sr.tenant_id.to_string(),
                                encoded: sr.snapshot.to_vec(),
                                version: sr.version,
                            };
                            let echo_op = fv1::SyncOp {
                                entity_id: sr.entity_id.to_string(),
                                tenant_id: sr.tenant_id.to_string(),
                                session_id: String::new(),
                                kind: op.kind,
                                payload: op.payload,
                                lamport: sr.version,
                                timestamp: None,
                            };
                            Ok(fv1::SyncResponse {
                                merged_ops: vec![echo_op],
                                checkpoint: Some(checkpoint),
                            })
                        }
                        Err(e) => Err(Status::internal(e.to_string())),
                    };

                    if tx.send(response).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    /// Unary: return the current checkpoint for an entity.
    #[instrument(name = "grpc::get_checkpoint", skip(self, request))]
    async fn get_checkpoint(
        &self,
        request: Request<fv1::SyncCheckpoint>,
    ) -> Result<Response<fv1::SyncCheckpoint>, Status> {
        let req = request.into_inner();
        let entity_id = parse_entity_id(&req.entity_id)?;
        let tenant_id = parse_tenant_id(&req.tenant_id)?;

        let snapshot = self
            .use_case
            .snapshot(entity_id, tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let (encoded, version) = snapshot
            .map(|s| (s.encoded.to_vec(), s.version))
            .unwrap_or_default();

        Ok(Response::new(fv1::SyncCheckpoint {
            entity_id: req.entity_id,
            tenant_id: req.tenant_id,
            encoded,
            version,
        }))
    }
}
