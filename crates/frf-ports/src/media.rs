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
