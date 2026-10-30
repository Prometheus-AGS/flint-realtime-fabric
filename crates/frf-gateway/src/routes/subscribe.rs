use std::sync::Arc;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use frf_app::{
    CredentialKind, OpenedStream, PresenceTracker, Resync, SubscribePipeline, SubscriptionGrant,
    is_resnapshot_required,
};
use frf_domain::{ChannelId, Offset, PresenceStatus};
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, EventStream, IdentityVerifier, LogBroker,
    MediaSignaler, PortError,
};
use futures_util::StreamExt;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::AppStateArc;

/// Subprotocol the gateway echoes when a client sends its ticket as a
/// `Sec-WebSocket-Protocol` value. The client must offer it alongside
/// `ticket.<jwt>`.
pub const SUBSCRIBE_SUBPROTOCOL: &str = "frf.v1";
/// Prefix of the `Sec-WebSocket-Protocol` value that carries a ticket.
pub const TICKET_SUBPROTOCOL_PREFIX: &str = "ticket.";

/// Close code: the credential's lifetime ended or the stream finished.
/// Reconnect with a fresh credential from the last applied offset + 1.
pub const CLOSE_ENDED: u16 = 4000;
/// Close code: the subscription's authority was withdrawn mid-stream.
pub const CLOSE_FORBIDDEN: u16 = 4403;
/// Close code: the broker failed.
pub const CLOSE_INTERNAL: u16 = 1011;

#[derive(Deserialize)]
pub struct SubscribeQuery {
    pub channel: String,
    /// Inclusive offset to start from. Absent means the beginning of the channel.
    pub from: Option<u64>,
    /// Single-use browser ticket. Never logged.
    pub ticket: Option<String>,
}

impl std::fmt::Debug for SubscribeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscribeQuery")
            .field("channel", &self.channel)
            .field("from", &self.from)
            .field("ticket", &self.ticket.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
}

fn subprotocol_ticket(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(axum::http::header::SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .find_map(|protocol| protocol.strip_prefix(TICKET_SUBPROTOCOL_PREFIX))
        .filter(|ticket| !ticket.is_empty())
        .map(str::to_owned)
}

/// The credential presented on the upgrade: a ticket (query parameter first,
/// then subprotocol), else the bearer header. `None` when there is none.
fn credential(params: &SubscribeQuery, headers: &HeaderMap) -> Option<(String, CredentialKind)> {
    if let Some(ticket) = params.ticket.clone().filter(|t| !t.is_empty()) {
        return Some((ticket, CredentialKind::Ticket));
    }
    if let Some(ticket) = subprotocol_ticket(headers) {
        return Some((ticket, CredentialKind::Ticket));
    }

    #[cfg(feature = "dev-endpoints")]
    if crate::config::dev_no_auth() {
        // Auth bypassed — dev-endpoints build with DEV_NO_AUTH=true.
        return Some((String::new(), CredentialKind::Bearer));
    }

    bearer_token(headers).map(|token| (token, CredentialKind::Bearer))
}

/// WebSocket subscribe endpoint.
///
/// Authenticates and authorises the caller before upgrading to a WebSocket,
/// then forwards each permitted [`frf_domain::EventEnvelope`] as a JSON text
/// frame. When `from` is no longer retained, sends one
/// `{"type":"resync","channel":<uuid>,"head":<offset|null>}` frame and streams
/// from after the head. Accepts `{"type":"presence.heartbeat","status"?:...}`
/// text frames from the client.
///
/// Once open, and again after every resync frame, the subscriber alone is sent
/// one presence snapshot frame per live member of the channel (see
/// [`presence_snapshot_frame`]) before any broker event.
///
/// The credential is a ticket (`?ticket=<jwt>` or subprotocol
/// `ticket.<jwt>` offered with `frf.v1`) or `Authorization: Bearer <jwt>`. The
/// query string is never recorded on the span.
///
/// # Errors
///
/// Returns 400 for a malformed `channel` or `from`, 401 for a missing,
/// invalid, replayed or over-long credential, 403 when the credential does not
/// cover the channel.
#[instrument(name = "ws::subscribe", skip_all, fields(channel = tracing::field::Empty))]
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
    let Some((token, kind)) = credential(&params, &headers) else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(channel_uuid) = Uuid::parse_str(&params.channel) else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "channel must be a UUID",
        )
            .into_response();
    };
    tracing::Span::current().record("channel", tracing::field::display(channel_uuid));
    let channel_id = ChannelId::from_uuid(channel_uuid);
    let from = Offset(params.from.unwrap_or(Offset::BEGINNING.0));

    let pipeline = Arc::clone(&state.subscribe_pipeline);
    let opened = match pipeline.authorize(channel_id, &token, kind).await {
        Ok(grant) => pipeline
            .open(&grant, from)
            .await
            .map(|opened| (grant, opened)),
        Err(error) => Err(error),
    };

    match opened {
        Ok((grant, opened)) => {
            crate::routes::metrics::record_subscribe_opened();
            ws.protocols([SUBSCRIBE_SUBPROTOCOL])
                .on_upgrade(move |socket| handle_socket(socket, pipeline, grant, opened))
        }
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

/// A message a subscriber may send on its socket.
#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientFrame {
    #[serde(rename = "presence.heartbeat")]
    PresenceHeartbeat {
        #[serde(default)]
        status: Option<HeartbeatStatus>,
    },
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum HeartbeatStatus {
    Online,
    Away,
    Busy,
}

impl From<HeartbeatStatus> for PresenceStatus {
    fn from(status: HeartbeatStatus) -> Self {
        match status {
            HeartbeatStatus::Online => Self::Online,
            HeartbeatStatus::Away => Self::Away,
            HeartbeatStatus::Busy => Self::Busy,
        }
    }
}

fn resync_frame(channel_id: ChannelId, resync: Resync) -> String {
    serde_json::json!({
        "type": "resync",
        "channel": channel_id.to_string(),
        "head": resync.head.map(|offset| offset.0),
    })
    .to_string()
}

/// A presence snapshot frame: the JSON of a [`frf_app::PresenceTracker`]
/// `kind: "presence"` [`frf_domain::EventEnvelope`], identical in shape to a
/// published presence event except that `offset` is `null`.
///
/// The frame is sent only to the subscriber that opened, never published, and
/// has no broker position. `offset: null` is the sole marker: a client must
/// apply its payload as current presence and must not read, store or compare
/// it as a resume offset. `id` is fresh per frame and `timestamp` is the
/// snapshot time; `payload.last_seen` and `payload.expires_at` are the record's.
fn presence_snapshot_frame(envelope: &frf_domain::EventEnvelope) -> Option<String> {
    let mut frame = serde_json::to_value(envelope).ok()?;
    *frame.get_mut("offset")? = serde_json::Value::Null;
    Some(frame.to_string())
}

/// Send the grant holder the channel's live presence. Returns `false` when the
/// socket is gone. A snapshot that cannot be read is logged and skipped; the
/// stream still delivers every later change.
async fn send_presence_snapshot<L, A, I>(
    socket: &mut WebSocket,
    pipeline: &SubscribePipeline<L, A, I>,
    grant: &SubscriptionGrant,
) -> bool
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    let snapshot = match pipeline.presence_snapshot(grant).await {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(%error, "presence snapshot could not be read");
            return true;
        }
    };
    for envelope in &snapshot {
        let Some(frame) = presence_snapshot_frame(envelope) else {
            tracing::warn!("presence snapshot frame could not be serialized");
            continue;
        };
        if socket.send(Message::Text(frame.into())).await.is_err() {
            return false;
        }
    }
    true
}

fn close_for(error: &PortError) -> (u16, &'static str) {
    match error {
        PortError::PermissionDenied(reason) if reason == "token expired" => {
            (CLOSE_ENDED, "subscription_ended")
        }
        PortError::PermissionDenied(_) => (CLOSE_FORBIDDEN, "forbidden"),
        _ => (CLOSE_INTERNAL, "internal_error"),
    }
}

async fn close(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await;
}

async fn handle_socket<L, A, I>(
    mut socket: WebSocket,
    pipeline: Arc<SubscribePipeline<L, A, I>>,
    grant: SubscriptionGrant,
    opened: OpenedStream,
) where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    let OpenedStream { mut stream, resync } = opened;
    let mut resynced = resync.is_some();
    if let Some(resync) = resync
        && socket
            .send(Message::Text(resync_frame(grant.channel_id, resync).into()))
            .await
            .is_err()
    {
        return;
    }
    if !send_presence_snapshot(&mut socket, &pipeline, &grant).await {
        return;
    }
    let mut delivered = false;
    loop {
        tokio::select! {
            item = stream.next() => {
                match item {
                    Some(Ok(envelope)) => {
                        let Ok(json) = serde_json::to_string(&envelope) else {
                            close(&mut socket, CLOSE_INTERNAL, "internal_error").await;
                            return;
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            return;
                        }
                        delivered = true;
                        crate::routes::metrics::record_delivery();
                    }
                    Some(Err(error)) if !delivered && !resynced && is_resnapshot_required(&error) => {
                        // The broker reported the offset gone on first delivery.
                        let Some(next) = reopen_at_head(&mut socket, &pipeline, &grant).await else {
                            return;
                        };
                        stream = next;
                        resynced = true;
                    }
                    Some(Err(error)) => {
                        let (code, reason) = close_for(&error);
                        close(&mut socket, code, reason).await;
                        return;
                    }
                    None => {
                        close(&mut socket, CLOSE_ENDED, "subscription_ended").await;
                        return;
                    }
                }
            }
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        client_frame(pipeline.presence(), &grant, text.as_str()).await;
                    }
                    Some(Ok(Message::Close(_)) | Err(_)) | None => return,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}

async fn reopen_at_head<L, A, I>(
    socket: &mut WebSocket,
    pipeline: &SubscribePipeline<L, A, I>,
    grant: &SubscriptionGrant,
) -> Option<EventStream>
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    match pipeline.open_at_head(grant).await {
        Ok(OpenedStream { stream, resync }) => {
            let resync = resync.unwrap_or(Resync { head: None });
            let frame = resync_frame(grant.channel_id, resync);
            if socket.send(Message::Text(frame.into())).await.is_err() {
                return None;
            }
            send_presence_snapshot(socket, pipeline, grant)
                .await
                .then_some(stream)
        }
        Err(error) => {
            tracing::warn!(%error, "resync reopen failed");
            close(socket, CLOSE_INTERNAL, "internal_error").await;
            None
        }
    }
}

/// Apply one client text frame. Unknown or malformed frames are ignored.
async fn client_frame<L>(presence: &PresenceTracker<L>, grant: &SubscriptionGrant, text: &str)
where
    L: LogBroker,
{
    let Ok(ClientFrame::PresenceHeartbeat { status }) = serde_json::from_str(text) else {
        return;
    };
    let status = status.unwrap_or(HeartbeatStatus::Online).into();
    if let Err(error) = presence.heartbeat(grant, status).await {
        tracing::warn!(%error, "presence heartbeat could not be published");
    }
}

/// Run [`PresenceTracker::sweep_at`] every [`frf_app::PRESENCE_SWEEP_INTERVAL`].
pub fn spawn_presence_sweeper<L>(presence: Arc<PresenceTracker<L>>) -> tokio::task::JoinHandle<()>
where
    L: LogBroker + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(frf_app::PRESENCE_SWEEP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            if let Err(error) = presence.sweep_at(chrono::Utc::now()).await {
                tracing::warn!(%error, "presence expiry could not be published");
            }
        }
    })
}
