#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;
pub mod grpc_service;
pub mod routes;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use frf_app::{PublishUseCase, SubscribePipeline};
use frf_ports::{AuthzProvider, IdentityVerifier, LogBroker};

pub use config::GatewayConfig;
pub use error::GatewayError;

pub struct AppState<L, A, I> {
    pub subscribe_pipeline: Arc<SubscribePipeline<L, A, I>>,
    pub publish_usecase: Arc<PublishUseCase<L, A, I>>,
    pub config: Arc<GatewayConfig>,
}

/// Build the production Axum router with all adapters wired.
///
/// Exported so integration tests can construct the app without spawning
/// a real TCP listener.
pub fn build_router<L, A, I>(state: Arc<AppState<L, A, I>>) -> Router
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route(
            "/ws/v1/subscribe",
            get(routes::subscribe::ws_subscribe::<L, A, I>),
        )
        .route(
            "/v1/publish",
            post(routes::publish::publish_event::<L, A, I>),
        )
        .with_state(state)
}

/// Minimal router exposing only `/healthz` — used in unit tests that do not
/// require real adapter instances.
pub fn build_healthz_router() -> Router {
    Router::new().route("/healthz", get(routes::health::healthz))
}
