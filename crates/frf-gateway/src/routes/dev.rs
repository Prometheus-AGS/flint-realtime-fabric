//! Dev-only HTTP endpoints gated by the `dev-endpoints` Cargo feature.
//!
//! These routes are compiled out unless `--features dev-endpoints` is passed.
//! Enable in compose builds via `CARGO_FEATURES=dev-endpoints` build arg.
//! Never enable in production images.

#[cfg(feature = "dev-endpoints")]
pub mod inject {
    use axum::Json;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use frf_ports::{
        ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker,
    };
    use serde::Deserialize;

    use crate::AppStateArc;

    #[derive(Debug, Deserialize)]
    pub struct InjectFederationEventRequest {
        pub tenant_id: String,
        pub protocol: String,
        pub source: String,
        pub content: serde_json::Value,
    }

    /// `POST /dev/inject-federation-event`
    ///
    /// Publishes a synthetic federation event directly to the LogBroker spine.
    /// Enables Layer 3 smoke tests to exercise the full subscribe fan-out path
    /// without a live Matrix/ATProto bridge.
    ///
    /// Returns `202 Accepted` on success, `400` on bad UUID, `503` on broker error.
    ///
    /// **Only compiled when `--features dev-endpoints` is passed — absent in default release builds.**
    pub async fn inject_federation_event<L, A, I, M, B, P>(
        State(state): State<AppStateArc<L, A, I, M, B, P>>,
        Json(body): Json<InjectFederationEventRequest>,
    ) -> impl IntoResponse
    where
        L: LogBroker + Send + Sync + 'static,
        A: AuthzProvider + Send + Sync + 'static,
        I: IdentityVerifier + Send + Sync + 'static,
        M: frf_ports::MediaSignaler,
        B: AgentEventBus + 'static,
        P: ActionPolicyProvider + 'static,
    {
        use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
        use uuid::Uuid;

        let tenant_id = match body.tenant_id.parse::<Uuid>() {
            Ok(u) => TenantId::from_uuid(u),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "invalid tenant_id — must be a valid UUID",
                )
                    .into_response();
            }
        };

        let channel_id = ChannelId::new();
        let channel = Channel {
            id: channel_id,
            tenant_id,
            path: format!("federation/{}/{}", body.protocol, body.source),
        };
        let envelope = EventEnvelope::new(
            channel,
            Offset::BEGINNING,
            EventKind::Custom(format!("federation.{}", body.protocol)),
            body.content,
        );

        tracing::debug!(
            protocol = %body.protocol,
            source = %body.source,
            "dev: publishing federation event to spine",
        );

        match state.log_broker.publish(envelope).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => {
                tracing::warn!(error = %e, "dev: federation event publish failed");
                (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response()
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct InjectSignalRequest {
        pub room_id: String,
        pub from_session: String,
        pub to_session: Option<String>,
        pub tenant_id: String,
        pub kind: String,
        pub payload: serde_json::Value,
    }

    /// `POST /dev/inject-signal`
    ///
    /// Injects a fake WebRTC signaling envelope directly into the SFU adapter
    /// for local smoke testing.  Only available in debug builds.
    ///
    /// Returns `202 Accepted` on success, `503 Service Unavailable` when the
    /// `media_signaler` rejects the send (e.g., session not registered).
    pub async fn inject_signal<L, A, I, M, B, P>(
        State(state): State<AppStateArc<L, A, I, M, B, P>>,
        Json(body): Json<InjectSignalRequest>,
    ) -> impl IntoResponse
    where
        L: LogBroker + Send + Sync + 'static,
        A: AuthzProvider + Send + Sync + 'static,
        I: IdentityVerifier + Send + Sync + 'static,
        M: frf_ports::MediaSignaler,
        B: AgentEventBus + 'static,
        P: ActionPolicyProvider + 'static,
    {
        use frf_domain::{SessionId, SfuMode, SignalEnvelope, SignalKind, TenantId};
        use uuid::Uuid;

        let tenant_id = match body.tenant_id.parse::<Uuid>() {
            Ok(u) => TenantId::from_uuid(u),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "invalid tenant_id — must be a valid UUID",
                )
                    .into_response();
            }
        };

        let from_session = match body.from_session.parse::<Uuid>() {
            Ok(u) => SessionId::from_uuid(u),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "invalid from_session — must be a valid UUID",
                )
                    .into_response();
            }
        };

        let to_session = body
            .to_session
            .as_deref()
            .and_then(|s| s.parse::<Uuid>().ok())
            .map(SessionId::from_uuid);

        let kind = match body.kind.as_str() {
            "offer" => SignalKind::Offer,
            "answer" => SignalKind::Answer,
            "ice_candidate" => SignalKind::IceCandidate,
            "ice_restart" => SignalKind::IceRestart,
            "hangup" => SignalKind::Hangup,
            "room_join" => SignalKind::RoomJoin,
            _ => SignalKind::RoomLeave,
        };

        let envelope = SignalEnvelope {
            from_session,
            to_session,
            tenant_id,
            room_id: body.room_id.clone(),
            kind,
            sfu_mode: SfuMode::Sovereign,
            payload: body.payload,
            timestamp: chrono::Utc::now(),
        };

        match state.media_signaler.send_signal(envelope).await {
            Ok(()) => {
                tracing::debug!(
                    room_id = %body.room_id,
                    from_session = %body.from_session,
                    "dev: signal injection accepted",
                );
                StatusCode::ACCEPTED.into_response()
            }
            Err(e) => {
                tracing::warn!(error = %e, "dev: signal injection failed");
                (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response()
            }
        }
    }
}
