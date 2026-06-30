#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod agent_grpc_service;
pub mod config;
pub mod error;
pub mod grpc_service;
pub mod routes;
pub mod signal_service;
pub mod sync_grpc_service;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use frf_app::{PublishUseCase, SubscribePipeline};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, FederationBridge, IdentityVerifier,
    LogBroker, MediaSignaler,
};

pub use config::{GatewayConfig, PolicyEngineMode, SfuMode};
pub use error::GatewayError;

pub struct AppState<L, A, I, M, B, P> {
    pub subscribe_pipeline: Arc<SubscribePipeline<L, A, I>>,
    pub publish_usecase: Arc<PublishUseCase<L, A, I>>,
    pub media_signaler: Arc<M>,
    pub agent_bus: Arc<B>,
    /// Identity verifier — used at every gateway boundary to verify JWTs.
    pub identity: Arc<I>,
    /// `AuthZ` provider — used for subscribe-time Keto checks (ADR-002).
    pub authz: Arc<A>,
    /// Log broker — used by federation ingest tasks to publish to the spine.
    pub log_broker: Arc<L>,
    /// Action policy provider — Cedar or no-op, governs mutation action checks.
    pub action_policy: Arc<P>,
    /// Federation bridges paired with their protocol — each runs a background ingest task.
    pub federation_bridges: Vec<(
        frf_ports::FederationProtocol,
        Arc<dyn FederationBridge + Send + Sync>,
    )>,
    pub config: Arc<GatewayConfig>,
}

/// Type alias eliminating `Arc<AppState<...>>` verbosity from route handlers.
pub type AppStateArc<L, A, I, M, B, P> = Arc<AppState<L, A, I, M, B, P>>;

/// Build the production Axum router with all adapters wired.
///
/// Exported so integration tests can construct the app without spawning
/// a real TCP listener.
pub fn build_router<L, A, I, M, B, P>(state: Arc<AppState<L, A, I, M, B, P>>) -> Router
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    #[allow(unused_mut)]
    let mut router = Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route(
            "/ws/v1/subscribe",
            get(routes::subscribe::ws_subscribe::<L, A, I, M, B, P>),
        )
        .route(
            "/v1/publish",
            post(routes::publish::publish_event::<L, A, I, M, B, P>),
        )
        .route(
            "/ws/v1/agents",
            get(routes::agents::ws_agent_stream::<L, A, I, M, B, P>),
        );

    #[cfg(feature = "dev-endpoints")]
    {
        router = router
            .route(
                "/dev/inject-federation-event",
                post(routes::dev::inject::inject_federation_event::<L, A, I, M, B, P>),
            )
            .route(
                "/dev/inject-signal",
                post(routes::dev::inject::inject_signal::<L, A, I, M, B, P>),
            );
    }

    router.with_state(state)
}

/// Minimal router exposing only `/healthz` — used in unit tests that do not
/// require real adapter instances.
pub fn build_healthz_router() -> Router {
    Router::new().route("/healthz", get(routes::health::healthz))
}
