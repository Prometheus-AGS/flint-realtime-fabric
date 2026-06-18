#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::sync::Arc;

use anyhow::Result;
use frf_app::{PublishUseCase, SubscribePipeline};
use frf_authz_keto::KetoAuthzProvider;
use frf_broker_iggy::IggyBroker;
use frf_gateway::{AppState, GatewayConfig};
use frf_identity_ory::OryIdentityVerifier;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let config = GatewayConfig::from_env()?;

    let broker = Arc::new(IggyBroker::new(&config.iggy_connection_string).await?);
    let authz = Arc::new(KetoAuthzProvider::new(
        &config.keto_base_url,
        &config.keto_namespace,
    ));
    let identity = Arc::new(OryIdentityVerifier::new(
        &config.oathkeeper_jwks_url,
        &config.jwt_audience,
    ));

    let subscribe_pipeline = Arc::new(SubscribePipeline::new(
        Arc::clone(&broker),
        Arc::clone(&authz),
        Arc::clone(&identity),
    ));
    let publish_usecase = Arc::new(PublishUseCase::new(
        Arc::clone(&broker),
        Arc::clone(&authz),
        Arc::clone(&identity),
    ));

    let bind_addr = config.bind_addr;
    let state = Arc::new(AppState {
        subscribe_pipeline,
        publish_usecase,
        config: Arc::new(config),
    });

    let app = frf_gateway::build_router(state);

    tracing::info!("frf-gateway listening on {bind_addr}");
    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
