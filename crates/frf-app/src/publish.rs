use std::sync::Arc;

use frf_domain::{EventEnvelope, Offset};
use frf_ports::{AuthzProvider, IdentityVerifier, LogBroker, RelationTuple};
use tracing::instrument;

use crate::error::AppError;

pub struct PublishRequest {
    pub envelope: EventEnvelope,
    pub bearer_token: String,
}

pub struct PublishUseCase<L, A, I> {
    broker: Arc<L>,
    authz: Arc<A>,
    identity: Arc<I>,
}

impl<L, A, I> PublishUseCase<L, A, I>
where
    L: LogBroker,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    pub fn new(broker: Arc<L>, authz: Arc<A>, identity: Arc<I>) -> Self {
        Self {
            broker,
            authz,
            identity,
        }
    }

    /// Publish an event to the spine.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Identity`] if the bearer token is invalid.
    /// Returns [`AppError::Forbidden`] if the subject is not permitted to publish.
    /// Returns [`AppError::Broker`] if the broker publish fails.
    #[instrument(name = "app::publish", skip(self, req))]
    pub async fn execute(&self, req: PublishRequest) -> Result<Offset, AppError> {
        let claims = self
            .identity
            .verify(&req.bearer_token)
            .await
            .map_err(AppError::Identity)?;

        let publish_tuple = RelationTuple {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            relation: "publish".to_owned(),
            object: req.envelope.channel.id.to_string(),
        };

        let allowed = self
            .authz
            .check(&publish_tuple)
            .await
            .map_err(AppError::Broker)?;

        if !allowed {
            return Err(AppError::Forbidden(format!(
                "subject {} may not publish to channel {}",
                claims.subject, req.envelope.channel.id
            )));
        }

        let offset = self.broker.publish(req.envelope).await?;

        Ok(offset)
    }
}
