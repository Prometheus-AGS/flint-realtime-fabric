use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use frf_domain::{SessionId, SignalEnvelope, SignalKind, TenantId};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
};
use futures_util::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{AppStateArc, SfuMode};

/// Query parameters for `/ws/v1/signal`.
///
/// A browser `WebSocket` cannot set an `Authorization` header, so the JWT is
/// passed as the `token` query parameter and verified here. `room` scopes the
/// signaling session; `tenant` is only honored in dev-endpoints builds with
/// `DEV_NO_AUTH=true` (it is otherwise taken from the verified token).
#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    pub room: String,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

/// JSON frame streamed to the browser signaling client. Mirrors the admin-UI
/// `SignalFrame` shape (camelCase).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignalFrame {
    from_session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_session: Option<String>,
    tenant_id: String,
    room_id: String,
    kind: &'static str,
    sfu_mode: &'static str,
    /// SDP / ICE payload (present on `offer`/`answer`/`ice-candidate`) — carries the media
    /// negotiation the browser needs to reach the SFU (p23-c003).
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    payload: serde_json::Value,
    timestamp: String,
}

/// Inbound frame the browser sends up the WebSocket (camelCase). Only the fields the sovereign
/// bridge needs are parsed; unknown fields are ignored.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InboundSignalFrame {
    kind: String,
    #[serde(default)]
    payload: serde_json::Value,
}

fn kind_to_str(kind: &SignalKind) -> &'static str {
    match kind {
        SignalKind::Offer => "offer",
        SignalKind::Answer => "answer",
        SignalKind::IceCandidate => "ice-candidate",
        SignalKind::IceRestart => "ice-restart",
        SignalKind::Hangup => "hangup",
        SignalKind::RoomJoin => "room-join",
        SignalKind::RoomLeave => "room-leave",
        _ => "unspecified",
    }
}

/// Map an inbound browser `kind` string to a domain [`SignalKind`]. Returns `None` for kinds the
/// browser cannot originate (e.g. `answer`, which only the SFU produces).
fn str_to_kind(kind: &str) -> Option<SignalKind> {
    match kind {
        "offer" => Some(SignalKind::Offer),
        "ice-candidate" | "ice_candidate" => Some(SignalKind::IceCandidate),
        "room-join" | "room_join" => Some(SignalKind::RoomJoin),
        "room-leave" | "room_leave" => Some(SignalKind::RoomLeave),
        "hangup" => Some(SignalKind::Hangup),
        _ => None,
    }
}

fn to_frame(env: &SignalEnvelope, sfu_mode: &'static str) -> SignalFrame {
    SignalFrame {
        from_session: env.from_session.to_string(),
        to_session: env.to_session.map(|s| s.to_string()),
        tenant_id: env.tenant_id.to_string(),
        room_id: env.room_id.clone(),
        kind: kind_to_str(&env.kind),
        sfu_mode,
        payload: env.payload.clone(),
        timestamp: env.timestamp.to_rfc3339(),
    }
}

/// Browser-facing WebSocket signaling endpoint (`/ws/v1/signal`).
///
/// Subscribes to inbound signals for a freshly-minted session in the resolved
/// tenant and streams them to the client as JSON [`SignalFrame`]s. The tenant is
/// derived from the verified JWT (`token` query param); dev-endpoints builds with
/// `DEV_NO_AUTH=true` fall back to the `tenant` query param.
#[instrument(name = "ws::signal", skip(state, ws))]
pub async fn ws_signal<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
    ws: WebSocketUpgrade,
    Query(params): Query<SignalQuery>,
) -> impl IntoResponse
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    let Some((tenant_id, subject)) = resolve_identity(&state, &params).await else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    let signaler = Arc::clone(&state.media_signaler);
    let bridge = state.media_bridge.clone();
    let sfu_mode = if state.config.sfu_mode == SfuMode::Sovereign {
        "sovereign"
    } else {
        "hosted"
    };
    let room = params.room;
    ws.on_upgrade(move |socket| {
        handle_signal_socket(socket, signaler, bridge, tenant_id, subject, room, sfu_mode)
    })
}

/// Resolve the tenant for this signaling session.
///
/// Production path: verify the `token` query param and return the tenant **and the
/// authenticated subject** (JWT subject) — the latter is used for the ADR-007 media `view`
/// check (p24-c003) so the grant is stable/seedable. Dev-endpoints + `DEV_NO_AUTH=true`:
/// accept the `tenant` query param with no authenticated subject.
async fn resolve_identity<L, A, I, M, B, P>(
    state: &AppStateArc<L, A, I, M, B, P>,
    params: &SignalQuery,
) -> Option<(TenantId, Option<String>)>
where
    I: IdentityVerifier + Send + Sync + 'static,
{
    #[cfg(feature = "dev-endpoints")]
    if crate::config::dev_no_auth() {
        return params
            .tenant
            .as_deref()
            .and_then(|t| uuid::Uuid::parse_str(t).ok())
            .map(TenantId::from_uuid)
            .map(|tid| (tid, None));
    }

    let token = params.token.as_deref()?;
    let claims = state.identity.verify(token).await.ok()?;
    Some((claims.tenant_id, Some(claims.subject)))
}

async fn handle_signal_socket<M: MediaSignaler>(
    socket: WebSocket,
    signaler: Arc<M>,
    bridge: Option<Arc<crate::media_bridge::MediaTransportBridge>>,
    tenant_id: TenantId,
    subject: Option<String>,
    room_id: String,
    sfu_mode: &'static str,
) {
    let session_id = SessionId::new();

    let stream = match signaler.subscribe_signals(session_id, tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "signal subscribe failed");
            let (mut sink, _) = socket.split();
            let _ = sink.send(Message::Close(None)).await;
            return;
        }
    };

    tracing::info!(%session_id, room_id, "signal WS stream opened");

    let (mut ws_sink, mut ws_src) = socket.split();
    // One sink, two producers (subscription stream + inbound bridge answers) — serialize sends
    // through an mpsc so the split sink is owned by exactly one task.
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(32);

    // Sink task: drain queued messages to the browser.
    let sink_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if ws_sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Outbound task: subscription frames → browser.
    let sub_tx = out_tx.clone();
    let sub_task = tokio::spawn(async move {
        tokio::pin!(stream);
        while let Some(item) = stream.next().await {
            match item {
                Ok(env) => {
                    if let Ok(json) = serde_json::to_string(&to_frame(&env, sfu_mode))
                        && sub_tx.send(Message::Text(json.into())).await.is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "signal stream error");
                    break;
                }
            }
        }
    });

    // Inbound loop: browser frames → sovereign bridge → relay the answer back.
    while let Some(Ok(msg)) = ws_src.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(inbound) = serde_json::from_str::<InboundSignalFrame>(&text) else {
            continue;
        };
        if let Some(answer) = drive_inbound(
            bridge.as_ref(),
            &inbound,
            session_id,
            tenant_id,
            subject.as_deref(),
            &room_id,
            sfu_mode,
        )
        .await
        {
            // The offer created the session; relay the SFU's trickle candidates + connection-state
            // out to the browser so ICE can complete (p27-c002). Spawn once, after the answer.
            if answer.kind == "answer"
                && let Some(b) = &bridge
            {
                spawn_local_signals_relay(b, session_id, sfu_mode, out_tx.clone());
            }
            if let Ok(json) = serde_json::to_string(&answer)
                && out_tx.send(Message::Text(json.into())).await.is_err()
            {
                break;
            }
        }
    }

    sub_task.abort();
    sink_task.abort();
    let _ = signaler.remove_session(session_id, tenant_id).await;
}

/// Spawn a task that relays the sovereign engine's outbound trickle candidates + connection-state
/// for `session_id` to the browser as `ice-candidate` frames (p27-c002). Without this the gateway
/// (the answerer) never sends its candidates and ICE cannot complete.
fn spawn_local_signals_relay(
    bridge: &Arc<crate::media_bridge::MediaTransportBridge>,
    session_id: SessionId,
    sfu_mode: &'static str,
    out_tx: tokio::sync::mpsc::Sender<Message>,
) {
    let bridge = Arc::clone(bridge);
    tokio::spawn(async move {
        let stream = match bridge.local_signals(session_id).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "sovereign: local_signals subscribe failed");
                return;
            }
        };
        tokio::pin!(stream);
        while let Some(item) = stream.next().await {
            let Ok(env) = item else { break };
            if let Ok(json) = serde_json::to_string(&to_frame(&env, sfu_mode))
                && out_tx.send(Message::Text(json.into())).await.is_err()
            {
                break;
            }
        }
    });
}

/// Convert an inbound browser frame into a domain envelope, drive the sovereign media bridge,
/// and return the `Answer` frame to relay back (only for an `Offer` that produces one).
/// Returns `None` when there is no bridge, the kind is not browser-originable, or no answer.
async fn drive_inbound(
    bridge: Option<&Arc<crate::media_bridge::MediaTransportBridge>>,
    inbound: &InboundSignalFrame,
    session_id: SessionId,
    tenant_id: TenantId,
    subject: Option<&str>,
    room_id: &str,
    sfu_mode: &'static str,
) -> Option<SignalFrame> {
    let bridge = bridge?;
    let kind = str_to_kind(&inbound.kind)?;
    let env = SignalEnvelope {
        from_session: session_id,
        to_session: None,
        tenant_id,
        room_id: room_id.to_owned(),
        kind,
        subject: subject.map(str::to_owned),
        sfu_mode: frf_domain::SfuMode::Sovereign,
        payload: inbound.payload.clone(),
        timestamp: chrono::Utc::now(),
    };
    let answer = bridge.handle(&env).await?;
    Some(to_frame(&answer, sfu_mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_bridge::MediaTransportBridge;
    use frf_media_str0m::StrOmTransport;
    use str0m::media::{Direction, MediaKind};
    use str0m::{Candidate, Rtc};

    #[test]
    fn browser_originable_kinds_map_to_domain_kinds() {
        assert!(matches!(str_to_kind("offer"), Some(SignalKind::Offer)));
        assert!(matches!(
            str_to_kind("ice-candidate"),
            Some(SignalKind::IceCandidate)
        ));
        assert!(matches!(
            str_to_kind("room-join"),
            Some(SignalKind::RoomJoin)
        ));
        // A browser never *originates* an answer — the SFU produces it. Ignore it inbound.
        assert!(str_to_kind("answer").is_none());
        assert!(str_to_kind("garbage").is_none());
    }

    #[test]
    fn frame_carries_payload_and_dynamic_sfu_mode() {
        let env = SignalEnvelope {
            from_session: SessionId::new(),
            to_session: None,
            tenant_id: TenantId::new(),
            room_id: "r".to_owned(),
            kind: SignalKind::Answer,
            sfu_mode: frf_domain::SfuMode::Sovereign,
            payload: serde_json::json!({ "sdp": "v=0" }),
            timestamp: chrono::Utc::now(),
            subject: None,
        };
        let frame = to_frame(&env, "sovereign");
        assert_eq!(frame.sfu_mode, "sovereign");
        assert_eq!(
            frame.payload.get("sdp").and_then(|v| v.as_str()),
            Some("v=0")
        );
    }

    fn offer_sdp() -> String {
        let mut remote = Rtc::builder().build(std::time::Instant::now());
        let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0));
        remote.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
        let mut change = remote.sdp_api();
        change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
        let (offer, _pending) = change.apply().expect("offer");
        offer.to_sdp_string()
    }

    #[tokio::test]
    async fn inbound_offer_drives_bridge_and_yields_answer_frame() {
        let bridge = Some(Arc::new(MediaTransportBridge::new(Arc::new(
            StrOmTransport::new(),
        ))));
        let inbound = InboundSignalFrame {
            kind: "offer".to_owned(),
            payload: serde_json::json!({ "sdp": offer_sdp() }),
        };
        let answer = drive_inbound(
            bridge.as_ref(),
            &inbound,
            SessionId::new(),
            TenantId::new(),
            None,
            "room-1",
            "sovereign",
        )
        .await
        .expect("an answer frame");
        assert_eq!(answer.kind, "answer");
        assert_eq!(answer.sfu_mode, "sovereign");
        assert!(
            answer
                .payload
                .get("sdp")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.starts_with("v=0")),
            "answer frame carries the SFU's SDP"
        );
    }

    #[tokio::test]
    async fn inbound_without_bridge_is_ignored() {
        let inbound = InboundSignalFrame {
            kind: "offer".to_owned(),
            payload: serde_json::json!({ "sdp": offer_sdp() }),
        };
        assert!(
            drive_inbound(
                None,
                &inbound,
                SessionId::new(),
                TenantId::new(),
                None,
                "room-1",
                "hosted",
            )
            .await
            .is_none(),
            "no sovereign bridge → inbound media frames are not handled"
        );
    }
}
