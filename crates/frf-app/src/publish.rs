use std::sync::Arc;

use frf_domain::{EventEnvelope, Offset};
use frf_ports::{IdentityVerifier, LogBroker};
use tracing::instrument;

use crate::error::AppError;

pub struct PublishRequest {
    pub envelope: EventEnvelope,
    pub bearer_token: String,
}

pub struct PublishUseCase<L, I> {
    broker: Arc<L>,
    identity: Arc<I>,
}

impl<L, I> PublishUseCase<L, I>
where
    L: LogBroker,
    I: IdentityVerifier,
{
    pub fn new(broker: Arc<L>, identity: Arc<I>) -> Self {
        Self { broker, identity }
    }

    /// Publish an event to the spine.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Identity`] if the bearer token is invalid.
    /// Returns [`AppError::Broker`] if the broker publish fails.
    #[instrument(name = "app::publish", skip(self, req))]
    pub async fn execute(&self, req: PublishRequest) -> Result<Offset, AppError> {
        self.identity
            .verify(&req.bearer_token)
            .await
            .map_err(AppError::Identity)?;

        let offset = self.broker.publish(req.envelope).await?;

        Ok(offset)
    }
}
