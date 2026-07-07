#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod agent_grpc_service;
pub mod authz_grpc_service;
pub mod config;
pub mod entity_grpc_service;
pub mod entity_store_mem;
pub mod error;
pub mod grpc_service;
pub mod routes;
pub mod signal_service;
pub mod sync_grpc_service;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, post};
use frf_app::{PublishUseCase, SubscribePipeline};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, FederationBridge, IdentityVerifier,
    LogBroker, MediaSignaler,
};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor,
};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

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
        .route("/readyz", get(routes::health::readyz::<L, A, I, M, B, P>))
        .route("/metrics", get(routes::metrics::metrics))
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
        )
        .route(
            "/ws/v1/signal",
            get(routes::signal::ws_signal::<L, A, I, M, B, P>),
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

    // Serve the embedded admin UI for any route not matched above (SPA fallback).
    // API routes take precedence; unmatched GETs return the UI / its assets.
    let router = router.fallback(routes::admin_ui::serve_admin_ui);

    apply_security_layers(router, &state.config).with_state(state)
}

/// Apply the security / resource-exhaustion middleware (rate limit, body-size
/// cap, CORS) to every route. Generic over the router's state type so it can be
/// applied before `with_state`. Kept separate so `build_router` stays small.
fn apply_security_layers<S>(mut router: Router<S>, config: &GatewayConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Global request rate limit (token bucket over ALL clients). A global key is
    // used deliberately: per-IP limiting requires trusted X-Forwarded-For / proxy
    // ConnectInfo, and an IP extractor fails closed (500) when it cannot determine
    // a client IP. A global cap is a reliable baseline DoS guard everywhere; per-IP
    // limiting is a future enhancement once proxy-trust config exists.
    // A degenerate config (rate or burst == 0) skips the layer with a warning
    // rather than panicking at startup.
    if let Some(governor_conf) = GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .per_second(config.rate_limit_per_sec)
        .burst_size(config.rate_limit_burst)
        .finish()
    {
        router = router.layer(GovernorLayer::new(Arc::new(governor_conf)));
    } else {
        tracing::warn!(
            rate_limit_per_sec = config.rate_limit_per_sec,
            rate_limit_burst = config.rate_limit_burst,
            "rate limit disabled — invalid RATE_LIMIT_PER_SEC / RATE_LIMIT_BURST (must be > 0)"
        );
    }

    // Cap request body size to bound memory per request.
    router = router.layer(RequestBodyLimitLayer::new(config.max_body_bytes));

    // Explicit CORS policy — empty origin list means no cross-origin browser
    // access (safe default); configured origins are allowed exactly.
    router.layer(build_cors_layer(&config.cors_allowed_origins))
}

/// Build a `CorsLayer` from an exact-match allowlist of origins.
///
/// An empty list yields a restrictive layer that permits no cross-origin
/// requests. Invalid origin strings are skipped with a warning.
fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| {
            o.parse::<HeaderValue>()
                .map_err(|_| tracing::warn!(origin = %o, "ignoring invalid CORS origin"))
                .ok()
        })
        .collect();

    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(origins)
}

/// Minimal router exposing only `/healthz` — used in unit tests that do not
/// require real adapter instances.
pub fn build_healthz_router() -> Router {
    Router::new().route("/healthz", get(routes::health::healthz))
}

/// Resolve when the process receives a shutdown signal: SIGTERM (orchestrator
/// stop) or SIGINT (ctrl-c). Await this as the graceful-shutdown future so axum
/// drains in-flight requests and WebSocket streams before exiting.
///
/// On non-Unix platforms, only ctrl-c is awaited.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to install SIGTERM handler");
                // Never resolve — fall back to ctrl-c only.
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received SIGINT (ctrl-c) — draining"),
        () = terminate => tracing::info!("received SIGTERM — draining"),
    }
}

/// A minimal router with the production security middleware
/// ([`apply_security_layers`]) applied, for integration tests that exercise the
/// rate-limit / body-size / CORS behavior without constructing full adapter state.
///
/// Exposes `GET /healthz` and `POST /echo` (echoes the request body).
pub fn build_security_test_router(config: &GatewayConfig) -> Router {
    let router = Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/echo", post(|body: String| async move { body }));
    apply_security_layers(router, config)
}
