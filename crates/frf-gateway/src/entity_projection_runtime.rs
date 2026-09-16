use std::sync::Arc;

use anyhow::{Context as _, Result};
use frf_gateway::GatewayConfig;
use frf_gateway::entity_projector::EntityProjector;
use frf_gateway::entity_store_mem::InMemoryEntityStore;
use frf_ports::EntityStore;
use frf_projection_surreal::SurrealEntityProjection;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::configured_broker::ConfiguredLogBroker;

pub(crate) struct ProjectionRuntime {
    pub store: Arc<dyn EntityStore>,
    pub readiness: watch::Receiver<bool>,
    pub tasks: Vec<JoinHandle<()>>,
}

pub(crate) async fn start(
    config: &GatewayConfig,
    broker: Arc<ConfiguredLogBroker>,
    shutdown: watch::Receiver<bool>,
    cdc_readiness: watch::Receiver<bool>,
) -> Result<ProjectionRuntime> {
    if !config.cdc_enabled {
        return Ok(ProjectionRuntime {
            store: Arc::new(InMemoryEntityStore::new()),
            readiness: watch::channel(true).1,
            tasks: Vec::new(),
        });
    }

    let projection = Arc::new(
        SurrealEntityProjection::connect(
            required(
                config.entity_projection_url.as_deref(),
                "ENTITY_PROJECTION_URL",
            )?,
            required(
                config.entity_projection_username.as_deref(),
                "ENTITY_PROJECTION_USERNAME",
            )?,
            required(
                config.entity_projection_password.as_deref(),
                "ENTITY_PROJECTION_PASSWORD",
            )?,
            required(
                config.entity_projection_namespace.as_deref(),
                "ENTITY_PROJECTION_NAMESPACE",
            )?,
            required(
                config.entity_projection_database.as_deref(),
                "ENTITY_PROJECTION_DATABASE",
            )?,
        )
        .await
        .context("connect durable entity projection")?,
    );
    let store: Arc<dyn EntityStore> = projection;
    let projector = EntityProjector::new(broker, Arc::clone(&store));
    let (projector_tx, projector_rx) = watch::channel(false);
    let projector_task = tokio::spawn(async move {
        if let Err(error) = projector.run_until_shutdown(shutdown, projector_tx).await {
            tracing::error!(error = %error, "entity projector exited with error");
        }
    });
    let (combined_tx, combined_rx) = watch::channel(false);
    let readiness_task = tokio::spawn(combine_readiness(cdc_readiness, projector_rx, combined_tx));
    Ok(ProjectionRuntime {
        store,
        readiness: combined_rx,
        tasks: vec![projector_task, readiness_task],
    })
}

fn required<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str> {
    value.with_context(|| format!("{name} must be set when CDC_ENABLED=true"))
}

async fn combine_readiness(
    mut cdc: watch::Receiver<bool>,
    mut projection: watch::Receiver<bool>,
    combined: watch::Sender<bool>,
) {
    loop {
        combined.send_replace(*cdc.borrow() && *projection.borrow());
        tokio::select! {
            result = cdc.changed() => {
                if result.is_err() {
                    combined.send_replace(false);
                    return;
                }
            }
            result = projection.changed() => {
                if result.is_err() {
                    combined.send_replace(false);
                    return;
                }
            }
        }
    }
}
