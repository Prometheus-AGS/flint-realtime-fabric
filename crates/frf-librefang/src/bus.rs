use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::AgentEvent;
use frf_ports::{AgentEventBus, AgentEventStream, PortError};
use tracing::instrument;

use crate::{error::LibreFangError, publisher::PublisherMessage, registry::TenantActorRegistry};

const IDLE_EVICTION_SECS: u64 = 300;
const SWEEP_INTERVAL_SECS: u64 = 60;

/// `AgentEventBus` implementation backed by a `TenantActorRegistry`.
///
/// Each tenant gets its own `PublisherActor` mailbox.  Actors are lazily
/// spawned on first use and evicted after `IDLE_EVICTION_SECS` of inactivity.
pub struct LibreFangBus {
    registry: Arc<TenantActorRegistry>,
    /// Background eviction task handle — kept alive for the lifetime of the bus.
    _eviction_handle: tokio::task::JoinHandle<()>,
}

impl LibreFangBus {
    /// Start the actor registry and return a connected bus handle.
    ///
    /// # Errors
    ///
    /// Returns [`LibreFangError::SpawnFailed`] if the registry cannot be initialised.
    /// Start the actor registry with configurable eviction parameters.
    ///
    /// `idle_secs` — actors idle longer than this are evicted (default: `IDLE_EVICTION_SECS`).
    /// `sweep_interval_secs` — how often the eviction sweep runs (default: `SWEEP_INTERVAL_SECS`).
    pub fn start_with_config(
        idle_secs: u64,
        sweep_interval_secs: u64,
    ) -> Result<Self, LibreFangError> {
        let registry = Arc::new(TenantActorRegistry::new());
        let eviction_handle = registry.spawn_eviction_task(idle_secs, sweep_interval_secs);
        Ok(Self {
            registry,
            _eviction_handle: eviction_handle,
        })
    }

    pub fn start() -> Result<Self, LibreFangError> {
        let registry = Arc::new(TenantActorRegistry::new());
        let eviction_handle = registry.spawn_eviction_task(IDLE_EVICTION_SECS, SWEEP_INTERVAL_SECS);
        Ok(Self {
            registry,
            _eviction_handle: eviction_handle,
        })
    }
}

#[async_trait]
impl AgentEventBus for LibreFangBus {
    #[instrument(skip(self, event), fields(tenant = %event.tenant_id, kind = ?event.kind))]
    async fn publish(&self, event: AgentEvent) -> Result<(), PortError> {
        let tenant_key = event.tenant_id.to_string();
        let actor = self
            .registry
            .get_or_create(&tenant_key)
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;

        actor
            .cast(PublisherMessage::Publish(event))
            .map_err(|e| PortError::Transport(e.to_string()))
    }

    #[instrument(skip(self), fields(tenant = %tenant_id))]
    async fn subscribe(&self, tenant_id: &str) -> Result<AgentEventStream, PortError> {
        let stream = self
            .registry
            .subscribe(tenant_id)
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;

        Ok(Box::pin(stream))
    }
}
