use async_trait::async_trait;
use frf_domain::{Channel, ChannelId, Cursor, EventEnvelope, Offset};
use futures_core::Stream;

use crate::error::PortError;

/// A stream of events from a `LogBroker` subscription.
/// Boxed to keep the trait object-safe.
pub type EventStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<EventEnvelope, PortError>> + Send>>;

/// Durable event spine — publish, subscribe, seek, acknowledge.
///
/// Implemented by `frf-broker-iggy`. Wired in `frf-gateway`.
/// Adapter crates MUST instrument their implementations with
/// `#[tracing::instrument(name = "port::LogBroker::<method>")]`.
#[async_trait]
pub trait LogBroker: Send + Sync + 'static {
    /// Publish an event to a channel. Returns the assigned `Offset`.
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError>;

    /// Open a streaming subscription starting from `from`.
    ///
    /// Pass `Offset::BEGINNING` to replay from the start.
    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError>;

    /// Seek a named cursor to an explicit offset without consuming events.
    async fn seek(&self, cursor: Cursor) -> Result<(), PortError>;

    /// Acknowledge delivery up to and including `offset` for a consumer.
    async fn ack(
        &self,
        channel_id: ChannelId,
        consumer_id: &str,
        offset: Offset,
    ) -> Result<(), PortError>;

    /// Ensure the channel exists; create if absent.
    async fn ensure_channel(&self, channel: Channel) -> Result<(), PortError>;
}
