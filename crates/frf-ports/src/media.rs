use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{SessionId, SignalEnvelope, TenantId};
use futures_core::Stream;

use crate::error::PortError;

/// A stream of inbound WebRTC signals for a session.
pub type SignalStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<SignalEnvelope, PortError>> + Send>>;

/// WebRTC signaling relay — forwards offer/answer/ICE between peers via SFU.
///
/// Implemented by `frf-media-str0m` (sovereign) or `frf-media-livekit` (hosted).
/// Never stores media — signaling only.
/// Adapter crates MUST instrument methods with `#[tracing::instrument]`.
#[async_trait]
pub trait MediaSignaler: Send + Sync + 'static {
    /// Send a signaling message to one or all sessions in a room.
    async fn send_signal(&self, signal: SignalEnvelope) -> Result<(), PortError>;

    /// Subscribe to inbound signals for a session.
    async fn subscribe_signals(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<SignalStream, PortError>;

    /// Remove a session from all rooms (on disconnect / hangup).
    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError>;
}

/// Type-erased `MediaSignaler` that forwards all calls to an inner
/// `Arc<dyn MediaSignaler>`.  Enables the gateway to select between adapters
/// at runtime via `SFU_MODE` without changing generic type parameters.
pub struct DynMediaSignaler(Arc<dyn MediaSignaler>);

impl DynMediaSignaler {
    #[must_use]
    pub fn new(inner: Arc<dyn MediaSignaler>) -> Self {
        Self(inner)
    }
}

#[async_trait]
impl MediaSignaler for DynMediaSignaler {
    async fn send_signal(&self, signal: SignalEnvelope) -> Result<(), PortError> {
        self.0.send_signal(signal).await
    }

    async fn subscribe_signals(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        self.0.subscribe_signals(session_id, tenant_id).await
    }

    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError> {
        self.0.remove_session(session_id, tenant_id).await
    }
}
