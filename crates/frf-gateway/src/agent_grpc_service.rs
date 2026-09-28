use std::pin::Pin;
use std::sync::Arc;

use frf_agentproto::domain_to_proto;
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
    RelationTuple,
};
use frf_proto::fv1::agent_service_server::{AgentService, AgentServiceServer};
use frf_proto::fv1::{AgentEvent as ProtoEvent, AgentRunRequest, agent_run_request::Payload};
use tokio_stream::{Stream, StreamExt as _};
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;

use crate::AppState;

pub struct AgentGrpcService<L, A, I, M, B, P> {
    state: Arc<AppState<L, A, I, M, B, P>>,
}

impl<L, A, I, M, B, P> AgentGrpcService<L, A, I, M, B, P> {
    #[must_use]
    pub fn new(state: Arc<AppState<L, A, I, M, B, P>>) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn into_server(self) -> AgentServiceServer<Self>
    where
        L: LogBroker + Send + Sync + 'static,
        A: AuthzProvider + Send + Sync + 'static,
        I: IdentityVerifier + Send + Sync + 'static,
        M: MediaSignaler + 'static,
        B: AgentEventBus + 'static,
        P: ActionPolicyProvider + 'static,
    {
        AgentServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl<L, A, I, M, B, P> AgentService for AgentGrpcService<L, A, I, M, B, P>
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    type RunAgentStream = Pin<Box<dyn Stream<Item = Result<ProtoEvent, Status>> + Send>>;

    // Bidirectional streaming RPC: the first client frame must be
    // `AgentRunStart`; subsequent frames may be `AgentRunControl`. The Fabric
    // agent bus is observational: it cannot cancel the runtime that owns a run.
    // JWT verification and Keto subscribe-time check happen
    // before the first frame is processed.
    #[instrument(name = "grpc::agent::run_agent", skip(self, request))]
    async fn run_agent(
        &self,
        request: Request<Streaming<AgentRunRequest>>,
    ) -> Result<Response<Self::RunAgentStream>, Status> {
        // JWT verification at the gRPC boundary — never trust claims downstream.
        let bearer_token = extract_bearer(&request)?;

        let claims = self
            .state
            .identity
            .verify(&bearer_token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        // Subscribe-time Keto check per ADR-002. Per-event Keto deferred until
        // a Redis-backed Keto cache is available to amortize latency at scale.
        let tenant_id = claims.tenant_id;
        let tuple = RelationTuple {
            tenant_id,
            subject: claims.subject.clone(),
            relation: "stream".to_owned(),
            object: "agent_bus".to_owned(),
        };

        let permitted = self
            .state
            .authz
            .check(&tuple)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !permitted {
            return Err(Status::permission_denied(
                "agent_bus:stream permission denied",
            ));
        }

        // Consume the inbound stream — first message must be AgentRunStart.
        let mut inbound = request.into_inner();

        let first = inbound
            .message()
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::invalid_argument("stream closed before AgentRunStart"))?;

        let start = match first.payload {
            Some(Payload::Start(s)) => s,
            Some(Payload::Control(_)) => {
                return Err(Status::invalid_argument(
                    "first frame must be AgentRunStart, not AgentRunControl",
                ));
            }
            None => {
                return Err(Status::invalid_argument(
                    "AgentRunRequest has empty payload",
                ));
            }
        };

        let agent_id = start.agent_id.clone();
        let session_id = start.session_id.clone();

        // Drain inbound frames so tonic does not buffer them unboundedly. A
        // cancel frame is rejected on the response stream as unsupported run
        // control. Closing this subscription only detaches the observer.
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        let cancel_guard = cancel_tx.clone();
        tokio::spawn(async move {
            while let Ok(Some(frame)) = inbound.message().await {
                if let Some(Payload::Control(ctrl)) = frame.payload
                    && ctrl.cancel
                {
                    let _ = cancel_tx.send(true);
                    break;
                }
            }
        });

        let domain_stream = self
            .state
            .agent_bus
            .subscribe(&tenant_id.to_string())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let filtered_stream = domain_stream
            .filter(move |ev| {
                // Filter to events for this specific agent + session.
                ev.agent_id.to_string() == agent_id && ev.session_id.to_string() == session_id
            })
            .map(|ev| Ok(domain_to_proto(ev)));

        // Retain a sender with the response stream so a normal inbound
        // half-close does not make `changed` spin or terminate observation.
        let proto_stream = futures_util::stream::unfold(
            (Some(filtered_stream), cancel_rx, cancel_guard),
            |(stream, mut cancel_rx, cancel_tx)| async move {
                let mut stream = stream?;
                tokio::select! {
                    biased;
                    changed = cancel_rx.changed() => {
                        if changed.is_ok() && *cancel_rx.borrow() {
                            Some((
                                Err(Status::unimplemented(
                                    "Fabric cannot cancel the agent runtime; observer stream detached",
                                )),
                                (None, cancel_rx, cancel_tx),
                            ))
                        } else {
                            None
                        }
                    }
                    event = stream.next() => event.map(|event| {
                        (event, (Some(stream), cancel_rx, cancel_tx))
                    }),
                }
            },
        );

        Ok(Response::new(Box::pin(proto_stream)))
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
