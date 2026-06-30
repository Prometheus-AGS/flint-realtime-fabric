use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;

use crate::error::PortError;
use frf_domain::AgentEvent;

/// Stream of agent events from the bus.
pub type AgentEventStream = Pin<Box<dyn Stream<Item = AgentEvent> + Send>>;

/// Port seam for the actor-based agent event bus (`BossFang` / `LibreFang`).
///
/// Implementations must be `Send + Sync + 'static` so they can be stored in
/// `Arc` inside `AppState` and shared across Tokio tasks.
#[async_trait]
pub trait AgentEventBus: Send + Sync + 'static {
    /// Publish an agent event to the bus.
    ///
    /// # Errors
    ///
    /// Returns `PortError::Unavailable` if the underlying actor is unreachable.
    async fn publish(&self, event: AgentEvent) -> Result<(), PortError>;

    /// Subscribe to agent events for the given tenant.
    ///
    /// The returned stream yields events published to this tenant's channel
    /// until the stream is dropped.
    ///
    /// # Errors
    ///
    /// Returns `PortError::Unavailable` if the subscription actor cannot be reached.
    async fn subscribe(&self, tenant_id: &str) -> Result<AgentEventStream, PortError>;
}
