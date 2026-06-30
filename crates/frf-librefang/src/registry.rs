use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use ractor::{Actor, ActorRef};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    error::LibreFangError,
    publisher::{PublisherActor, PublisherMessage},
};

struct TenantEntry {
    actor: ActorRef<PublisherMessage>,
    /// Monotonic timestamp of the most recent activity (publish or subscribe).
    last_active: Instant,
}

/// Per-tenant actor registry — lazily spawns one `PublisherActor` per active
/// tenant and evicts idle actors after a configurable timeout.
///
/// `DashMap` provides concurrent, shard-based access so that tenant operations
/// do not block each other.
pub struct TenantActorRegistry {
    entries: Arc<DashMap<String, TenantEntry>>,
}

impl TenantActorRegistry {
    /// Create a new, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Return the actor for `tenant_id`, spawning a new `PublisherActor` if one
    /// does not yet exist.
    ///
    /// # Errors
    ///
    /// Returns [`LibreFangError::SpawnFailed`] if the ractor actor cannot be started.
    pub async fn get_or_create(
        &self,
        tenant_id: &str,
    ) -> Result<ActorRef<PublisherMessage>, LibreFangError> {
        // Fast path — existing entry.
        if let Some(mut entry) = self.entries.get_mut(tenant_id) {
            entry.last_active = Instant::now();
            return Ok(entry.actor.clone());
        }

        // Slow path — spawn a new actor then insert under the tenant key.
        let (actor, _handle) = Actor::spawn(None, PublisherActor, ())
            .await
            .map_err(|e| LibreFangError::SpawnFailed(e.to_string()))?;

        self.entries.insert(
            tenant_id.to_owned(),
            TenantEntry {
                actor: actor.clone(),
                last_active: Instant::now(),
            },
        );

        tracing::debug!(tenant_id, "spawned new PublisherActor for tenant");
        Ok(actor)
    }

    /// Receive a subscribed event stream from the actor for `tenant_id`.
    ///
    /// # Errors
    ///
    /// Propagates any error from [`get_or_create`] or from the actor RPC call.
    pub async fn subscribe(
        &self,
        tenant_id: &str,
    ) -> Result<ReceiverStream<frf_domain::AgentEvent>, LibreFangError> {
        let actor = self.get_or_create(tenant_id).await?;

        let call_result = actor
            .call(
                |reply| PublisherMessage::Subscribe {
                    tenant_id: tenant_id.to_owned(),
                    reply,
                },
                None,
            )
            .await
            .map_err(|e| LibreFangError::SubscriptionFailed(e.to_string()))?;

        let rx: mpsc::Receiver<frf_domain::AgentEvent> = match call_result {
            ractor::rpc::CallResult::Success(rx) => rx,
            ractor::rpc::CallResult::Timeout => {
                return Err(LibreFangError::SubscriptionFailed(
                    "subscribe call timed out".to_owned(),
                ));
            }
            ractor::rpc::CallResult::SenderError => {
                return Err(LibreFangError::SubscriptionFailed(
                    "subscribe sender error".to_owned(),
                ));
            }
        };

        Ok(ReceiverStream::new(rx))
    }

    /// Spawn a background task that evicts tenant actors idle for more than
    /// `idle_secs` seconds.  The sweep runs every `sweep_interval_secs` seconds.
    ///
    /// Returns a `JoinHandle` that can be dropped to stop the eviction task.
    #[must_use]
    pub fn spawn_eviction_task(
        &self,
        idle_secs: u64,
        sweep_interval_secs: u64,
    ) -> tokio::task::JoinHandle<()> {
        let entries = Arc::clone(&self.entries);
        let idle_duration = Duration::from_secs(idle_secs);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(sweep_interval_secs));
            loop {
                interval.tick().await;
                let now = Instant::now();
                entries.retain(|tenant_id, entry| {
                    let alive = now.duration_since(entry.last_active) < idle_duration;
                    if !alive {
                        tracing::debug!(
                            tenant_id = tenant_id.as_str(),
                            "evicting idle tenant actor"
                        );
                        // Stop the actor gracefully; ignore send errors.
                        entry.actor.stop(None);
                    }
                    alive
                });
            }
        })
    }
}

impl Default for TenantActorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_or_create_returns_same_actor_for_same_tenant() {
        let registry = TenantActorRegistry::new();
        let actor1 = registry.get_or_create("tenant-a").await.unwrap();
        let actor2 = registry.get_or_create("tenant-a").await.unwrap();
        // Same actor ID means the same ractor actor was reused.
        assert_eq!(actor1.get_id(), actor2.get_id());
    }

    #[tokio::test]
    async fn get_or_create_returns_different_actors_for_different_tenants() {
        let registry = TenantActorRegistry::new();
        let actor_a = registry.get_or_create("tenant-a").await.unwrap();
        let actor_b = registry.get_or_create("tenant-b").await.unwrap();
        assert_ne!(actor_a.get_id(), actor_b.get_id());
    }

    #[tokio::test]
    async fn subscribe_returns_stream() {
        let registry = TenantActorRegistry::new();
        let _stream = registry.subscribe("tenant-a").await.unwrap();
        // Subscribe succeeded — stream is ready to receive events.
    }
}
