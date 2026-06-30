use std::time::Duration;

use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{error::PortError, federation::FederatedEvent};
use futures_util::{SinkExt, StreamExt};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::convert::jetstream_event_to_federated;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 500;

/// Connect to a Bluesky Jetstream WebSocket endpoint and stream `FederatedEvent`s.
///
/// Sends a subscription filter frame on connect specifying the lexicon collections
/// to receive.  Reconnects with exponential back-off on disconnect.
pub fn jetstream_stream(
    url: String,
    collections: Vec<String>,
    tenant_id: TenantId,
    channel_id: ChannelId,
) -> impl futures_util::Stream<Item = Result<FederatedEvent, PortError>> + Send + 'static {
    async_stream::stream! {
        let mut attempts: u32 = 0;

        loop {
            match connect_async(&url).await {
                Ok((mut ws, _)) => {
                    attempts = 0;

                    // Send subscription filter.
                    let filter = serde_json::json!({ "wantedCollections": collections });
                    let filter_msg = Message::Text(filter.to_string());
                    if ws.send(filter_msg).await.is_err() {
                        tracing::warn!("jetstream: failed to send subscription filter");
                        continue;
                    }

                    while let Some(msg_result) = ws.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                match serde_json::from_str::<serde_json::Value>(&text) {
                                    Ok(raw) => {
                                        match jetstream_event_to_federated(&raw, tenant_id, channel_id) {
                                            Ok(event) => yield Ok(event),
                                            Err(e) => {
                                                tracing::debug!(error = %e, "jetstream: projection error, skipping");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::debug!(error = %e, "jetstream: JSON parse error, skipping");
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                tracing::info!("jetstream: server closed connection");
                                break;
                            }
                            Ok(_) => {} // ping/pong/binary — ignore
                            Err(e) => {
                                tracing::warn!(error = %e, "jetstream: WS error");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, attempt = attempts, "jetstream: connection failed");
                }
            }

            attempts += 1;
            if attempts >= MAX_RECONNECT_ATTEMPTS {
                tracing::error!("jetstream: max reconnect attempts reached — stopping stream");
                yield Err(PortError::Transport(format!(
                    "jetstream: failed to connect after {MAX_RECONNECT_ATTEMPTS} attempts"
                )));
                break;
            }

            let backoff = INITIAL_BACKOFF_MS * u64::from(2_u32.saturating_pow(attempts - 1));
            tracing::info!(backoff_ms = backoff, "jetstream: reconnecting");
            sleep(Duration::from_millis(backoff)).await;
        }
    }
}
