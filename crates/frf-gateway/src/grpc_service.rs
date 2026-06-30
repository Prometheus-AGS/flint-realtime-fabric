use std::pin::Pin;
use std::sync::Arc;

use frf_app::{AppError, PublishRequest, SubscribeRequest};
use frf_domain::ids::{ChannelId, EventId, TenantId};
use frf_domain::{Channel, EventEnvelope, EventKind, Offset};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
    PortError,
};
use frf_proto::fv1::spine_service_server::{SpineService, SpineServiceServer};
use frf_proto::fv1::{
    self, AckRequest, AckResponse, PublishResponse, SubscribeRequest as ProtoSubscribeRequest,
};
use tokio_stream::{Stream, StreamExt as _};
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::AppState;

pub struct SpineGrpcService<L, A, I, M, B, P> {
    state: Arc<AppState<L, A, I, M, B, P>>,
}

impl<L, A, I, M, B, P> SpineGrpcService<L, A, I, M, B, P> {
    #[must_use]
    pub fn new(state: Arc<AppState<L, A, I, M, B, P>>) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn into_server(self) -> SpineServiceServer<Self>
    where
        L: LogBroker + Send + Sync + 'static,
        A: AuthzProvider + Send + Sync + 'static,
        I: IdentityVerifier + Send + Sync + 'static,
        M: MediaSignaler + 'static,
        B: AgentEventBus + 'static,
        P: ActionPolicyProvider + 'static,
    {
        SpineServiceServer::new(self)
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

fn parse_channel_id(s: &str) -> Result<ChannelId, Status> {
    uuid::Uuid::parse_str(s)
        .map(ChannelId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid channel_id UUID: {s}")))
}

fn parse_tenant_id(s: &str) -> Result<TenantId, Status> {
    uuid::Uuid::parse_str(s)
        .map(TenantId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid tenant_id UUID: {s}")))
}

fn parse_event_id(s: &str) -> Result<EventId, Status> {
    uuid::Uuid::parse_str(s)
        .map(EventId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid event id UUID: {s}")))
}

fn proto_envelope_to_domain(proto: fv1::EventEnvelope) -> Result<EventEnvelope, Status> {
    let ch = proto
        .channel
        .ok_or_else(|| Status::invalid_argument("missing channel"))?;
    let offset = proto
        .offset
        .ok_or_else(|| Status::invalid_argument("missing offset"))?;

    let channel = Channel {
        id: parse_channel_id(&ch.id)?,
        tenant_id: parse_tenant_id(&ch.tenant_id)?,
        path: ch.path,
    };

    let kind = match proto.kind {
        1 => EventKind::EntityChange,
        2 => EventKind::AgentEvent,
        3 => EventKind::SyncOp,
        4 => EventKind::Presence,
        5 => EventKind::Signal,
        n => EventKind::Custom(n.to_string()),
    };

    let payload = if proto.payload.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&proto.payload)
            .map_err(|e| Status::invalid_argument(format!("invalid payload JSON: {e}")))?
    };

    let id = parse_event_id(&proto.id)?;

    Ok(EventEnvelope {
        id,
        channel,
        offset: Offset(offset.value),
        kind,
        payload,
        timestamp: chrono::Utc::now(),
        correlation_id: if proto.correlation_id.is_empty() {
            None
        } else {
            Some(proto.correlation_id)
        },
    })
}

fn domain_envelope_to_proto(env: &EventEnvelope) -> fv1::EventEnvelope {
    let kind_num = match &env.kind {
        EventKind::EntityChange => 1,
        EventKind::AgentEvent => 2,
        EventKind::SyncOp => 3,
        EventKind::Presence => 4,
        EventKind::Signal => 5,
        EventKind::Custom(_) | _ => 6,
    };

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
        kind: kind_num,
        payload: serde_json::to_vec(&env.payload).unwrap_or_default(),
        timestamp: None,
        correlation_id: env.correlation_id.clone().unwrap_or_default(),
    }
}

#[tonic::async_trait]
impl<L, A, I, M, B, P> SpineService for SpineGrpcService<L, A, I, M, B, P>
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    #[instrument(name = "grpc::publish", skip(self, request))]
    async fn publish(
        &self,
        request: Request<fv1::PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let bearer_token = extract_bearer(&request)?;
        let proto_req = request.into_inner();
        let proto_env = proto_req
            .envelope
            .ok_or_else(|| Status::invalid_argument("missing envelope"))?;

        let envelope = proto_envelope_to_domain(proto_env)?;
        let app_req = PublishRequest {
            envelope,
            bearer_token,
        };

        let offset = self
            .state
            .publish_usecase
            .execute(app_req)
            .await
            .map_err(app_error_to_status)?;

        Ok(Response::new(PublishResponse {
            offset: Some(fv1::Offset { value: offset.0 }),
        }))
    }

    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<fv1::EventEnvelope, Status>> + Send>>;

    #[instrument(name = "grpc::subscribe", skip(self, request))]
    async fn subscribe(
        &self,
        request: Request<ProtoSubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let bearer_token = extract_bearer(&request)?;
        let req = request.into_inner();

        let from_offset = req.from.map_or(Offset::BEGINNING, |o| Offset(o.value));

        let subscribe_req = SubscribeRequest {
            channel_id: parse_channel_id(&req.channel_id)?,
            bearer_token,
            from: from_offset,
        };

        let stream = self
            .state
            .subscribe_pipeline
            .execute(subscribe_req)
            .await
            .map_err(app_error_to_status)?;

        let mapped = stream.map(|res| {
            res.map(|env| domain_envelope_to_proto(&env))
                .map_err(|e| port_error_to_status(&e))
        });
        Ok(Response::new(Box::pin(mapped)))
    }

    #[instrument(name = "grpc::ack", skip(self, _request))]
    async fn ack(&self, _request: Request<AckRequest>) -> Result<Response<AckResponse>, Status> {
        Ok(Response::new(AckResponse {}))
    }
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
