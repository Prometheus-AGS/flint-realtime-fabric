use async_trait::async_trait;
use frf_domain::EventEnvelope;
use futures_core::Stream;

use crate::error::PortError;

/// The federation protocol family.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FederationProtocol {
    /// Tuwunel Matrix bridge.
    Matrix,
    /// Tranquil `ATProto` / Bluesky firehose bridge.
    AtProto,
}

/// An inbound event received from a federation peer.
#[derive(Debug, Clone)]
pub struct FederatedEvent {
    pub protocol: FederationProtocol,
    /// Source server identifier (Matrix homeserver or `ATProto` PDS).
    pub source: String,
    pub envelope: EventEnvelope,
}

/// A stream of inbound federated events.
pub type FederationStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<FederatedEvent, PortError>> + Send>>;

/// Send and receive events across federation bridges.
///
/// Implemented by `frf-bridge-matrix` (Tuwunel) or `frf-bridge-atproto` (Tranquil).
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait FederationBridge: Send + Sync + 'static {
    /// Emit an outbound event to a federation peer.
    async fn send(
        &self,
        protocol: FederationProtocol,
        destination: &str,
        envelope: EventEnvelope,
    ) -> Result<(), PortError>;

    /// Subscribe to inbound events from federation peers.
    async fn subscribe(&self, protocol: FederationProtocol) -> Result<FederationStream, PortError>;
}
