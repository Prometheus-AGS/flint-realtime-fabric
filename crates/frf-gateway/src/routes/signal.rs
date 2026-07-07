use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use frf_domain::{SessionId, SignalEnvelope, SignalKind, TenantId};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
};
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::AppStateArc;

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
    timestamp: String,
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

fn to_frame(env: &SignalEnvelope) -> SignalFrame {
    SignalFrame {
        from_session: env.from_session.to_string(),
        to_session: env.to_session.map(|s| s.to_string()),
        tenant_id: env.tenant_id.to_string(),
        room_id: env.room_id.clone(),
        kind: kind_to_str(&env.kind),
        sfu_mode: "hosted",
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
    let Some(tenant_id) = resolve_tenant(&state, &params).await else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    let signaler = Arc::clone(&state.media_signaler);
    let room = params.room;
    ws.on_upgrade(move |socket| handle_signal_socket(socket, signaler, tenant_id, room))
}

/// Resolve the tenant for this signaling session.
///
/// Production path: verify the `token` query param and take the tenant from the
/// claims. Dev-endpoints + `DEV_NO_AUTH=true`: accept the `tenant` query param.
async fn resolve_tenant<L, A, I, M, B, P>(
    state: &AppStateArc<L, A, I, M, B, P>,
    params: &SignalQuery,
) -> Option<TenantId>
where
    I: IdentityVerifier + Send + Sync + 'static,
{
    #[cfg(feature = "dev-endpoints")]
    if crate::config::dev_no_auth() {
        return params
            .tenant
            .as_deref()
            .and_then(|t| uuid::Uuid::parse_str(t).ok())
            .map(TenantId::from_uuid);
    }

    let token = params.token.as_deref()?;
    let claims = state.identity.verify(token).await.ok()?;
    Some(claims.tenant_id)
}

async fn handle_signal_socket<M: MediaSignaler>(
    mut socket: WebSocket,
    signaler: Arc<M>,
    tenant_id: TenantId,
    room_id: String,
) {
    let session_id = SessionId::new();

    let stream = match signaler.subscribe_signals(session_id, tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "signal subscribe failed");
            // Dropping the socket closes the connection.
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    tracing::info!(%session_id, room_id, "signal WS stream opened");
    tokio::pin!(stream);

    while let Some(item) = stream.next().await {
        match item {
            Ok(env) => {
                let frame = to_frame(&env);
                let Ok(json) = serde_json::to_string(&frame) else {
                    continue;
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "signal stream error");
                break;
            }
        }
    }

    let _ = signaler.remove_session(session_id, tenant_id).await;
}
