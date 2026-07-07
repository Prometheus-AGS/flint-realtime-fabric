//! FFI transport client wrapping `frf-sdk-rust` for Swift / Kotlin / Dart.
//!
//! Async methods run on the ambient tokio runtime (`async_runtime = "tokio"`).
//! Domain types cross the FFI boundary as JSON strings so the wire encoding is
//! never part of the FFI contract and the surface stays language-neutral.

use std::sync::Arc;

use frf_domain::{ChannelId, EventEnvelope, Offset};
use frf_sdk_rust::{FrfClient, ReconnectPolicy, SubscribeTarget, resilient_subscribe};
use futures_util::StreamExt as _;
use tokio::sync::Mutex;

use crate::error::ClientFfiError;

/// Callback the host app implements to receive subscription events.
///
/// `on_event` is called once per delivered envelope (as a JSON string).
/// `on_error` is called if the stream fails; after it, no further events arrive.
#[uniffi::export(with_foreign)]
pub trait EventCallback: Send + Sync {
    fn on_event(&self, envelope_json: String);
    fn on_error(&self, message: String);
}

/// A connected Flint Realtime Fabric client.
///
/// Wraps the hand-written Rust SDK ([`FrfClient`]). Construct with
/// [`FrfFfiClient::connect`], then `publish` / `subscribe` / `ack`. `subscribe` is
/// resilient: it reconnects with backoff and resumes from the last offset, so mobile
/// clients survive gateway restarts without special handling.
#[derive(uniffi::Object)]
pub struct FrfFfiClient {
    inner: Arc<Mutex<FrfClient>>,
    /// Endpoint + token retained so `subscribe` can build a self-reconnecting stream.
    endpoint: String,
    token: Option<String>,
}

#[uniffi::export(async_runtime = "tokio")]
impl FrfFfiClient {
    /// Connect to the gateway's gRPC endpoint, optionally with a bearer token.
    ///
    /// # Errors
    ///
    /// Returns [`ClientFfiError::Connect`] if the endpoint is invalid or the
    /// transport connection fails.
    #[uniffi::constructor]
    pub async fn connect(
        endpoint: String,
        token: Option<String>,
    ) -> Result<Arc<Self>, ClientFfiError> {
        let client = FrfClient::connect(endpoint.clone(), token.clone()).await?;
        Ok(Arc::new(Self {
            inner: Arc::new(Mutex::new(client)),
            endpoint,
            token,
        }))
    }

    /// Publish an event (given as a JSON `EventEnvelope`), returning the offset.
    ///
    /// # Errors
    ///
    /// Returns [`ClientFfiError::InvalidArgument`] if the JSON is not a valid
    /// envelope, or [`ClientFfiError::Request`] if the gateway rejects it.
    pub async fn publish(&self, envelope_json: String) -> Result<u64, ClientFfiError> {
        let envelope: EventEnvelope = serde_json::from_str(&envelope_json)
            .map_err(|e| ClientFfiError::InvalidArgument(format!("invalid envelope JSON: {e}")))?;
        let mut client = self.inner.lock().await;
        let offset = client.publish(&envelope).await?;
        Ok(offset.0)
    }

    /// Subscribe to a channel. Events are delivered to `callback` until the
    /// stream ends or errors. Returns immediately after validating the channel id;
    /// delivery runs on a background task.
    ///
    /// The subscription is **resilient**: on a transport error or gateway restart it
    /// reconnects with exponential backoff and resumes from the last delivered offset,
    /// so no event is skipped. `callback.on_error` fires only if reconnection is
    /// permanently exhausted.
    ///
    /// # Errors
    ///
    /// Returns [`ClientFfiError::InvalidArgument`] if `channel_id` is not a UUID.
    // UniFFI-exported args must be owned (`&str` is not supported across the FFI
    // boundary), so `channel_id` is taken by value even though it is only parsed.
    #[allow(clippy::needless_pass_by_value)]
    pub fn subscribe(
        &self,
        channel_id: String,
        consumer_id: String,
        from_offset: u64,
        callback: Arc<dyn EventCallback>,
    ) -> Result<(), ClientFfiError> {
        let uuid = uuid::Uuid::parse_str(&channel_id).map_err(|_| {
            ClientFfiError::InvalidArgument(format!("invalid channel_id: {channel_id}"))
        })?;
        let target = SubscribeTarget {
            endpoint: self.endpoint.clone(),
            token: self.token.clone(),
            channel_id: ChannelId::from_uuid(uuid),
            consumer_id,
            from: Offset(from_offset),
        };

        // resilient_subscribe owns its own connection and reconnects transparently.
        let stream = resilient_subscribe(target, ReconnectPolicy::default());

        tokio::spawn(async move {
            tokio::pin!(stream);
            while let Some(item) = stream.next().await {
                match item {
                    Ok(envelope) => match serde_json::to_string(&envelope) {
                        Ok(json) => callback.on_event(json),
                        Err(e) => callback.on_error(format!("serialize event: {e}")),
                    },
                    Err(e) => {
                        callback.on_error(e.to_string());
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Acknowledge consumption up to `offset` for a channel/consumer.
    ///
    /// # Errors
    ///
    /// Returns [`ClientFfiError::InvalidArgument`] if `channel_id` is not a UUID, or
    /// [`ClientFfiError::Request`] if the ack RPC fails.
    pub async fn ack(
        &self,
        channel_id: String,
        consumer_id: String,
        offset: u64,
    ) -> Result<(), ClientFfiError> {
        let uuid = uuid::Uuid::parse_str(&channel_id).map_err(|_| {
            ClientFfiError::InvalidArgument(format!("invalid channel_id: {channel_id}"))
        })?;
        let channel = ChannelId::from_uuid(uuid);
        let mut client = self.inner.lock().await;
        client.ack(channel, consumer_id, Offset(offset)).await?;
        Ok(())
    }
}
