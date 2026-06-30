use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
};
use futures_util::StreamExt;
use serde_json;
use tracing::instrument;

use crate::AppStateArc;

/// WebSocket agent-event stream endpoint (`/ws/v1/agents`).
///
/// Upgrades to a WebSocket and streams [`AgentEvent`] JSON frames for the
/// tenant extracted from the `Authorization: Bearer <jwt>` header.
///
/// JWT verification is performed by the [`IdentityVerifier`] at this boundary.
/// The `tenant_id` is extracted from the verified claims — never from URL params.
///
/// # Errors
///
/// Returns 401 if the `Authorization: Bearer <token>` header is missing or invalid.
#[instrument(name = "ws::agents", skip(state, ws, headers))]
pub async fn ws_agent_stream<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> impl IntoResponse
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(claims) = state.identity.verify(&token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let tenant_id = claims.tenant_id.as_uuid().to_string();
    let bus = Arc::clone(&state.agent_bus);

    ws.on_upgrade(move |socket| handle_agent_socket(socket, bus, tenant_id))
}

async fn handle_agent_socket<B: AgentEventBus>(
    mut socket: WebSocket,
    bus: Arc<B>,
    tenant_id: String,
) {
    let stream = match bus.subscribe(&tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to subscribe to agent bus");
            return;
        }
    };

    tokio::pin!(stream);

    while let Some(event) = stream.next().await {
        let Ok(json) = serde_json::to_string(&event) else {
            break;
        };
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
}
