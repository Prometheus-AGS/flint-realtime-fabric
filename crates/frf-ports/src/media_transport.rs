//! The `MediaTransport` port — the sovereign SFU per-session media engine.
//!
//! Separate from [`crate::media::MediaSignaler`] (which is signaling-only) per ADR-005:
//! `MediaTransport` owns the per-session `Rtc` + UDP transport + connection lifecycle. The
//! `frf-media-str0m` adapter implements **both** ports as distinct concerns; the hosted
//! (`LiveKit`) path has no `MediaTransport` implementation. This crate holds no
//! implementation — the absolute dependency rule.
//!
//! ICE candidates cross this port as [`SignalEnvelope`]s (`SignalKind::IceCandidate`),
//! consistent with how signaling already represents them, so candidates ride the existing
//! signaling transport. **RTP forwarding is not on this port** — that is phase-21.

use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{SessionId, SignalEnvelope, TenantId};

use crate::error::PortError;
use crate::media::SignalStream;

/// The connection lifecycle of a per-session media transport.
///
/// Phase-20 drives a session as far as [`ConnectionState::Connected`] (DTLS complete); RTP
/// forwarding on top of a connected session is phase-21.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Session created; negotiation done but transport not yet started.
    New,
    /// ICE/DTLS in progress.
    Connecting,
    /// DTLS handshake complete — the transport is established (media may flow in phase-21).
    Connected,
    /// The peer went away or the session was torn down.
    Disconnected,
    /// The transport failed (ICE/DTLS error).
    Failed,
}

/// A per-session sovereign media engine: negotiate a session, exchange trickle-ICE
/// candidates, and drive it to a connected transport.
///
/// Implemented by `frf-media-str0m` (sovereign). Adapter methods MUST be instrumented with
/// `#[tracing::instrument]`. RTP forwarding is out of scope for this port (phase-21).
#[async_trait]
pub trait MediaTransport: Send + Sync + 'static {
    /// Negotiate a session from a remote SDP offer and start its transport, returning the
    /// SDP answer.
    async fn create_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
        offer_sdp: &str,
    ) -> Result<String, PortError>;

    /// Feed an inbound trickle-ICE candidate (carried as a `SignalKind::IceCandidate`
    /// envelope) into the session's transport.
    async fn add_remote_candidate(
        &self,
        session_id: SessionId,
        candidate: SignalEnvelope,
    ) -> Result<(), PortError>;

    /// Subscribe to the session's outbound signals — its own local ICE candidates and
    /// connection-state changes — as `SignalEnvelope`s to relay back over the signaling path.
    async fn local_signals(&self, session_id: SessionId) -> Result<SignalStream, PortError>;

    /// The current connection state of the session's transport.
    async fn connection_state(&self, session_id: SessionId) -> Result<ConnectionState, PortError>;

    /// Tear down a session's transport (on disconnect / hangup).
    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError>;
}

/// Type-erased [`MediaTransport`] forwarding to an inner `Arc<dyn MediaTransport>`, so the
/// gateway can select the sovereign engine at runtime under `SFU_MODE` without changing
/// generic type parameters (mirrors [`crate::media::DynMediaSignaler`]).
pub struct DynMediaTransport(Arc<dyn MediaTransport>);

impl DynMediaTransport {
    #[must_use]
    pub fn new(inner: Arc<dyn MediaTransport>) -> Self {
        Self(inner)
    }
}

#[async_trait]
impl MediaTransport for DynMediaTransport {
    async fn create_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
        offer_sdp: &str,
    ) -> Result<String, PortError> {
        self.0
            .create_session(session_id, tenant_id, offer_sdp)
            .await
    }

    async fn add_remote_candidate(
        &self,
        session_id: SessionId,
        candidate: SignalEnvelope,
    ) -> Result<(), PortError> {
        self.0.add_remote_candidate(session_id, candidate).await
    }

    async fn local_signals(&self, session_id: SessionId) -> Result<SignalStream, PortError> {
        self.0.local_signals(session_id).await
    }

    async fn connection_state(&self, session_id: SessionId) -> Result<ConnectionState, PortError> {
        self.0.connection_state(session_id).await
    }

    async fn remove_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
    ) -> Result<(), PortError> {
        self.0.remove_session(session_id, tenant_id).await
    }
}
