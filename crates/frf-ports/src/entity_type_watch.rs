use async_trait::async_trait;
use frf_domain::{EntityTypeDelivery, Offset};
use futures_core::Stream;

use crate::PortError;

pub type EntityTypeDeliveryStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<EntityTypeDelivery, PortError>> + Send>>;

/// Durable committed-mutation source for the versioned entity-type watch use case.
///
/// Implementations translate one broker channel into canonical typed deliveries.
/// Authorization, filtering, checkpoint binding and subscriber buffering remain
/// application-layer responsibilities.
#[async_trait]
pub trait EntityTypeWatchSource: Send + Sync + 'static {
    /// Subscribe inclusively from an authoritative broker offset.
    async fn subscribe(
        &self,
        consumer_id: String,
        from: Offset,
    ) -> Result<EntityTypeDeliveryStream, PortError>;

    /// Return the inclusive current high-water offset, or `None` when empty.
    async fn head_offset(&self) -> Result<Option<Offset>, PortError>;
}
