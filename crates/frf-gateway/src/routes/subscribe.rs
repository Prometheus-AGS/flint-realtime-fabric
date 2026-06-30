use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use frf_app::SubscribeRequest;
use frf_domain::{ChannelId, Offset};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, EventStream, IdentityVerifier, LogBroker,
    MediaSignaler,
};
use futures_util::StreamExt;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::AppStateArc;

#[derive(Debug, Deserialize)]
pub struct SubscribeQuery {
    pub channel: String,
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
}

/// WebSocket subscribe endpoint.
///
/// Authenticates and authorises the caller before upgrading to a WebSocket.
/// Forwards each permitted [`EventEnvelope`] as a JSON text frame.
///
/// # Errors
///
/// Returns 401 if the `Authorization: Bearer <token>` header is missing.
#[instrument(name = "ws::subscribe", skip(state, ws, headers))]
pub async fn ws_subscribe<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Query(params): Query<SubscribeQuery>,
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
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(channel_uuid) = Uuid::parse_str(&params.channel) else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "channel must be a UUID",
        )
            .into_response();
    };

    let channel_id = ChannelId::from_uuid(channel_uuid);

    let req = SubscribeRequest {
        channel_id,
        bearer_token: token,
        from: Offset::BEGINNING,
    };

    match state.subscribe_pipeline.execute(req).await {
        Ok(stream) => ws.on_upgrade(move |socket| handle_socket(socket, stream)),
        Err(e) => {
            use frf_app::AppError;
            let status = match &e {
                AppError::Unauthorized(_) | AppError::Identity(_) => {
                    axum::http::StatusCode::UNAUTHORIZED
                }
                AppError::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string()).into_response()
        }
    }
}

async fn handle_socket(mut socket: WebSocket, mut stream: EventStream) {
    while let Some(item) = stream.next().await {
        match item {
            Ok(envelope) => {
                let Ok(json) = serde_json::to_string(&envelope) else {
                    break;
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
