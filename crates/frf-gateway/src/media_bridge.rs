//! Bridges the signaling path to the sovereign media engine (`StrOmTransport`).
//!
//! ADR-005 puts port composition in the gateway: for `SFU_MODE=sovereign` the signal service
//! drives the `MediaTransport` here. This bridge maps a [`SignalEnvelope`] to the right media
//! action and returns any envelope to relay back (the SDP answer for an offer). Local ICE
//! candidates + connection-state changes flow out of `local_signals` (relayed separately).
//!
//! **The gate is not flipped by this bridge** — composing the media plane does not make
//! `SFU_MODE=sovereign` a production path; the end-to-end browser proof is deferred.

use std::sync::Arc;

use chrono::Utc;
use frf_domain::{SfuMode, SignalEnvelope, SignalKind};
use frf_media_str0m::StrOmTransport;
use frf_ports::{AuthzProvider, MediaTransport, RelationTuple};

/// The Zanzibar relation authorizing a participant to receive a room's media (ADR-007).
const MEDIA_VIEW_RELATION: &str = "view";

/// Maps signaling envelopes onto the sovereign `StrOmTransport` media engine.
///
/// Per [ADR-007](../../docs/decisions/adr-007-media-path-authz.md), a participant is authorized
/// with a Keto `check(subject, "view", room)` at `RoomJoin` before entering fan-out. Authz is
/// composed here in the gateway — the str0m adapter carries no authz dependency
/// (one-port-per-adapter + the absolute dependency rule).
pub struct MediaTransportBridge {
    transport: Arc<StrOmTransport>,
    authz: Option<Arc<dyn AuthzProvider>>,
}

impl MediaTransportBridge {
    #[must_use]
    pub fn new(transport: Arc<StrOmTransport>) -> Self {
        Self {
            transport,
            authz: None,
        }
    }

    /// Attach the media-path authorizer (ADR-007). Without it, `RoomJoin` is admitted on the
    /// JWT + `(tenant, room)` boundary alone; with it, an additional per-participant Keto
    /// `view` check gates entry to fan-out.
    #[must_use]
    pub fn with_authz(mut self, authz: Arc<dyn AuthzProvider>) -> Self {
        self.authz = Some(authz);
        self
    }

    /// Outbound trickle-ICE + connection-state envelopes for `session_id` — the sovereign engine's
    /// local candidates the browser needs to complete ICE (p27-c002). The WS route subscribes to
    /// this and relays each envelope as an `ice-candidate` frame.
    ///
    /// # Errors
    /// [`PortError`] if the session is unknown.
    pub async fn local_signals(
        &self,
        session_id: frf_domain::SessionId,
    ) -> Result<frf_ports::SignalStream, frf_ports::PortError> {
        self.transport.local_signals(session_id).await
    }

    /// Handle one inbound signal for the sovereign media plane. Returns an envelope to relay
    /// outbound (the SDP `Answer` for an `Offer`), or `None` when nothing needs relaying.
    ///
    /// Errors from the media engine are logged and swallowed — a bad frame from one client
    /// must not take the signaling task down.
    pub async fn handle(&self, env: &SignalEnvelope) -> Option<SignalEnvelope> {
        match env.kind {
            SignalKind::Offer => self.handle_offer(env).await,
            SignalKind::RoomJoin => {
                if self.authorized_to_view(env).await {
                    self.transport
                        .join_room(env.from_session, env.tenant_id, &env.room_id);
                }
                None
            }
            SignalKind::IceCandidate => {
                if let Err(e) = self
                    .transport
                    .add_remote_candidate(env.from_session, env.clone())
                    .await
                {
                    tracing::warn!(error = %e, "sovereign: add_remote_candidate failed");
                }
                None
            }
            SignalKind::RoomLeave | SignalKind::Hangup => {
                if let Err(e) = self
                    .transport
                    .remove_session(env.from_session, env.tenant_id)
                    .await
                {
                    tracing::warn!(error = %e, "sovereign: remove_session failed");
                }
                None
            }
            // Answer/IceRestart/unknown: nothing for the SFU-as-answerer to do here.
            _ => None,
        }
    }

    /// ADR-007: is the joining participant authorized to view this room's media?
    ///
    /// The subject is the authenticated, server-assigned `from_session` id — never a
    /// caller-supplied field. With no authorizer attached, entry falls back to the JWT +
    /// `(tenant, room)` boundary (returns `true`). A negative check **or a check error** denies
    /// the join (fail-closed): a session that cannot be authorized must not receive media.
    async fn authorized_to_view(&self, env: &SignalEnvelope) -> bool {
        let Some(authz) = &self.authz else {
            return true;
        };
        // Prefer the authenticated identity (JWT subject, p24-c003) — it is stable and
        // grantable in Keto. Fall back to the session id when no authenticated subject is
        // present (in-process/legacy paths), preserving prior behaviour.
        let subject = env
            .subject
            .clone()
            .unwrap_or_else(|| env.from_session.to_string());
        let tuple = RelationTuple {
            tenant_id: env.tenant_id,
            subject,
            relation: MEDIA_VIEW_RELATION.to_owned(),
            object: env.room_id.clone(),
        };
        match authz.check(&tuple).await {
            Ok(allowed) => {
                if !allowed {
                    tracing::debug!(room = %env.room_id, "sovereign: room-join denied (no view)");
                }
                allowed
            }
            Err(e) => {
                tracing::warn!(error = %e, room = %env.room_id, "sovereign: view check errored — denying join");
                false
            }
        }
    }

    /// Negotiate an offer into a session and build the `Answer` envelope to relay back.
    async fn handle_offer(&self, env: &SignalEnvelope) -> Option<SignalEnvelope> {
        let offer_sdp = env.payload.get("sdp").and_then(serde_json::Value::as_str)?;
        match self
            .transport
            .create_session(env.from_session, env.tenant_id, offer_sdp)
            .await
        {
            Ok(answer_sdp) => Some(answer_envelope(env, &answer_sdp)),
            Err(e) => {
                tracing::warn!(error = %e, "sovereign: create_session failed");
                None
            }
        }
    }
}

/// Build the `Answer` envelope (back to the offerer) carrying the SDP answer.
fn answer_envelope(offer: &SignalEnvelope, answer_sdp: &str) -> SignalEnvelope {
    SignalEnvelope {
        from_session: offer.from_session,
        to_session: Some(offer.from_session),
        tenant_id: offer.tenant_id,
        room_id: offer.room_id.clone(),
        kind: SignalKind::Answer,
        sfu_mode: SfuMode::Sovereign,
        payload: serde_json::json!({ "sdp": answer_sdp }),
        timestamp: Utc::now(),
        // Server-generated answer — no authenticated subject to carry.
        subject: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frf_domain::{SessionId, TenantId};
    use frf_ports::PortError;
    use std::sync::Mutex;
    use str0m::media::{Direction, MediaKind};
    use str0m::{Candidate, Rtc};

    /// Records every tuple it is asked about and answers with a fixed verdict, so a test can
    /// assert both *that* the bridge consulted authz and *what* it asked (ADR-007).
    struct RecordingAuthz {
        verdict: Result<bool, ()>,
        seen: Mutex<Vec<RelationTuple>>,
    }

    impl RecordingAuthz {
        fn allow() -> Arc<Self> {
            Arc::new(Self {
                verdict: Ok(true),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn deny() -> Arc<Self> {
            Arc::new(Self {
                verdict: Ok(false),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn erroring() -> Arc<Self> {
            Arc::new(Self {
                verdict: Err(()),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    #[async_trait::async_trait]
    impl AuthzProvider for RecordingAuthz {
        async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
            self.seen.lock().expect("lock").push(tuple.clone());
            self.verdict
                .map_err(|()| PortError::Transport("stub check failure".to_owned()))
        }
        async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
            Ok(())
        }
        async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
            Ok(())
        }
    }

    /// A real SDP offer from a str0m `Rtc`, so the bridge negotiates real 0.21 SDP.
    fn offer_sdp() -> String {
        let mut remote = Rtc::builder().build(std::time::Instant::now());
        let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0));
        remote.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
        let mut change = remote.sdp_api();
        change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
        let (offer, _pending) = change.apply().expect("offer");
        offer.to_sdp_string()
    }

    fn envelope(
        kind: SignalKind,
        sid: SessionId,
        tid: TenantId,
        payload: serde_json::Value,
    ) -> SignalEnvelope {
        SignalEnvelope {
            from_session: sid,
            to_session: None,
            tenant_id: tid,
            room_id: "room-1".to_owned(),
            kind,
            subject: None,
            sfu_mode: SfuMode::Sovereign,
            payload,
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn offer_creates_a_session_and_returns_an_answer() {
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()));
        let sid = SessionId::new();
        let tid = TenantId::new();
        let env = envelope(
            SignalKind::Offer,
            sid,
            tid,
            serde_json::json!({ "sdp": offer_sdp() }),
        );

        let out = bridge.handle(&env).await.expect("an answer envelope");
        assert_eq!(out.kind, SignalKind::Answer);
        assert_eq!(out.to_session, Some(sid), "answer goes back to the offerer");
        let sdp = out.payload.get("sdp").and_then(serde_json::Value::as_str);
        assert!(
            sdp.is_some_and(|s| s.starts_with("v=0")),
            "answer carries valid SDP"
        );
    }

    #[tokio::test]
    async fn local_signals_relays_the_hosts_trickle_candidate() {
        // p27-c002: after an offer creates the session, the bridge's local_signals stream yields
        // the SFU's outbound trickle candidate — what the WS route relays so ICE can complete.
        use tokio_stream::StreamExt as _;
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()));
        let sid = SessionId::new();
        let tid = TenantId::new();
        bridge
            .handle(&envelope(
                SignalKind::Offer,
                sid,
                tid,
                serde_json::json!({ "sdp": offer_sdp() }),
            ))
            .await
            .expect("answer");

        let mut stream = bridge.local_signals(sid).await.expect("local_signals");
        let first = stream.next().await.expect("an envelope").expect("no err");
        assert_eq!(first.kind, SignalKind::IceCandidate);
        assert_eq!(first.from_session, sid);
    }

    #[tokio::test]
    async fn offer_without_sdp_returns_none() {
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()));
        let env = envelope(
            SignalKind::Offer,
            SessionId::new(),
            TenantId::new(),
            serde_json::json!({ "not_sdp": true }),
        );
        assert!(bridge.handle(&env).await.is_none());
    }

    #[tokio::test]
    async fn room_join_then_leave_is_handled_without_relay() {
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()));
        let sid = SessionId::new();
        let tid = TenantId::new();
        // Create the session first (join_room needs a registered session).
        let offer = envelope(
            SignalKind::Offer,
            sid,
            tid,
            serde_json::json!({ "sdp": offer_sdp() }),
        );
        bridge.handle(&offer).await.expect("answer");

        assert!(
            bridge
                .handle(&envelope(
                    SignalKind::RoomJoin,
                    sid,
                    tid,
                    serde_json::Value::Null
                ))
                .await
                .is_none()
        );
        assert!(
            bridge
                .handle(&envelope(
                    SignalKind::RoomLeave,
                    sid,
                    tid,
                    serde_json::Value::Null
                ))
                .await
                .is_none()
        );
    }

    // ADR-007: media fan-out is authorized per participant at room-join.

    #[tokio::test]
    async fn authorized_join_consults_authz_with_view_tuple_and_admits() {
        let authz = RecordingAuthz::allow();
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()))
            .with_authz(Arc::clone(&authz) as Arc<dyn AuthzProvider>);
        let sid = SessionId::new();
        let tid = TenantId::new();

        assert!(
            bridge
                .authorized_to_view(&envelope(
                    SignalKind::RoomJoin,
                    sid,
                    tid,
                    serde_json::Value::Null
                ))
                .await
        );

        let seen = authz.seen.lock().expect("lock");
        assert_eq!(seen.len(), 1, "authz was consulted exactly once");
        let tuple = &seen[0];
        assert_eq!(
            tuple.relation, "view",
            "ADR-007: the media relation is `view`"
        );
        assert_eq!(tuple.object, "room-1", "object is the room");
        assert_eq!(
            tuple.subject,
            sid.to_string(),
            "with no authenticated subject, the tuple falls back to the session id"
        );
        assert_eq!(tuple.tenant_id, tid);
    }

    #[tokio::test]
    async fn authenticated_subject_is_used_for_the_view_tuple() {
        // p24-c003: when the envelope carries a verified JWT subject, the `view` check must
        // authorize on *that* stable identity (which Keto can be seeded for) — not the
        // ephemeral session id.
        let authz = RecordingAuthz::allow();
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()))
            .with_authz(Arc::clone(&authz) as Arc<dyn AuthzProvider>);
        let mut env = envelope(
            SignalKind::RoomJoin,
            SessionId::new(),
            TenantId::new(),
            serde_json::Value::Null,
        );
        env.subject = Some("user:alice".to_owned());

        assert!(bridge.authorized_to_view(&env).await);
        let seen = authz.seen.lock().expect("lock");
        assert_eq!(
            seen[0].subject, "user:alice",
            "the authenticated subject, not the session id, is authorized"
        );
    }

    #[tokio::test]
    async fn unauthorized_join_is_denied() {
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()))
            .with_authz(RecordingAuthz::deny() as Arc<dyn AuthzProvider>);
        assert!(
            !bridge
                .authorized_to_view(&envelope(
                    SignalKind::RoomJoin,
                    SessionId::new(),
                    TenantId::new(),
                    serde_json::Value::Null
                ))
                .await,
            "a `view`=false subject must not enter fan-out"
        );
    }

    #[tokio::test]
    async fn errored_check_fails_closed() {
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()))
            .with_authz(RecordingAuthz::erroring() as Arc<dyn AuthzProvider>);
        assert!(
            !bridge
                .authorized_to_view(&envelope(
                    SignalKind::RoomJoin,
                    SessionId::new(),
                    TenantId::new(),
                    serde_json::Value::Null
                ))
                .await,
            "a check error denies the join (fail-closed) — never admit on uncertainty"
        );
    }

    #[tokio::test]
    async fn no_authz_falls_back_to_jwt_and_tenant_boundary() {
        // Without an authorizer the bridge admits on the JWT + (tenant, room) boundary alone.
        let bridge = MediaTransportBridge::new(Arc::new(StrOmTransport::new()));
        assert!(
            bridge
                .authorized_to_view(&envelope(
                    SignalKind::RoomJoin,
                    SessionId::new(),
                    TenantId::new(),
                    serde_json::Value::Null
                ))
                .await
        );
    }
}
