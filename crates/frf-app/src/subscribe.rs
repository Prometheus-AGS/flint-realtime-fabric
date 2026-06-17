use std::sync::Arc;

use frf_domain::{ChannelId, Offset, TenantId};
use frf_ports::{AuthzProvider, EventStream, IdentityVerifier, LogBroker, RelationTuple};
use futures_util::StreamExt;
use tracing::instrument;

use crate::error::AppError;

pub struct SubscribeRequest {
    pub channel_id: ChannelId,
    pub bearer_token: String,
    pub from: Offset,
}

pub struct SubscribePipeline<L, A, I> {
    broker: Arc<L>,
    authz: Arc<A>,
    identity: Arc<I>,
}

impl<L, A, I> SubscribePipeline<L, A, I>
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

    /// Execute the subscribe pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Identity`] if the bearer token is invalid.
    /// Returns [`AppError::Forbidden`] if the subject is not permitted to subscribe.
    /// Returns [`AppError::Broker`] if the broker subscription fails.
    #[instrument(name = "app::subscribe", skip(self, req), fields(channel_id = %req.channel_id))]
    pub async fn execute(&self, req: SubscribeRequest) -> Result<EventStream, AppError> {
        let claims = self
            .identity
            .verify(&req.bearer_token)
            .await
            .map_err(AppError::Identity)?;

        let subscribe_tuple = RelationTuple {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            relation: "subscribe".to_owned(),
            object: req.channel_id.to_string(),
        };

        let allowed = self
            .authz
            .check(&subscribe_tuple)
            .await
            .map_err(AppError::Broker)?;

        if !allowed {
            return Err(AppError::Forbidden(format!(
                "subject {} may not subscribe to channel {}",
                claims.subject, req.channel_id
            )));
        }

        let consumer_id = claims.session_id.to_string();
        let raw_stream = self
            .broker
            .subscribe(req.channel_id, consumer_id, req.from)
            .await?;

        let authz = Arc::clone(&self.authz);
        let tenant_id: TenantId = claims.tenant_id;
        let subject = claims.subject;

        let filtered = raw_stream.filter_map(move |item| {
            let authz = Arc::clone(&authz);
            let subject = subject.clone();
            async move {
                match item {
                    Err(e) => Some(Err(e)),
                    Ok(envelope) => {
                        let view_tuple = RelationTuple {
                            tenant_id,
                            subject,
                            relation: "view".to_owned(),
                            object: envelope.id.to_string(),
                        };
                        match authz.check(&view_tuple).await {
                            Ok(true) => Some(Ok(envelope)),
                            Ok(false) => None,
                            Err(e) => Some(Err(e)),
                        }
                    }
                }
            }
        });

        Ok(Box::pin(filtered))
    }
}

impl<L, A, I> SubscribePipeline<L, A, I> {
    #[must_use]
    pub fn into_parts(self) -> (Arc<L>, Arc<A>, Arc<I>) {
        (self.broker, self.authz, self.identity)
    }
}
