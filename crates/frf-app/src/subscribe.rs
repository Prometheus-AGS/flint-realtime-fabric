use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use frf_domain::{ChannelId, EventEnvelope, Offset, TenantId};
use frf_ports::{
    AuthzProvider, EventStream, IdentityVerifier, LogBroker, PortError, RelationTuple,
};
use futures_util::{StreamExt, stream};
use tokio::time::Instant;
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
        let deadline = token_deadline(claims.expires_at)?;

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

        let state = AuthorizedStream {
            raw_stream,
            authz: Arc::clone(&self.authz),
            subscribe_tuple,
            tenant_id: claims.tenant_id,
            subject: claims.subject,
            deadline,
            terminated: false,
        };

        Ok(Box::pin(stream::unfold(state, next_authorized::<A>)))
    }
}

struct AuthorizedStream<A> {
    raw_stream: EventStream,
    authz: Arc<A>,
    subscribe_tuple: RelationTuple,
    tenant_id: TenantId,
    subject: String,
    deadline: Instant,
    terminated: bool,
}

fn token_deadline(expires_at: u64) -> Result<Instant, AppError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        AppError::Identity(PortError::PermissionDenied(
            "system clock is before the Unix epoch".to_owned(),
        ))
    })?;
    let lifetime = Duration::from_secs(expires_at)
        .checked_sub(now)
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| {
            AppError::Identity(PortError::PermissionDenied("token expired".to_owned()))
        })?;
    Instant::now().checked_add(lifetime).ok_or_else(|| {
        AppError::Identity(PortError::PermissionDenied(
            "token expiry is outside the supported clock range".to_owned(),
        ))
    })
}

async fn next_authorized<A>(
    mut state: AuthorizedStream<A>,
) -> Option<(Result<EventEnvelope, PortError>, AuthorizedStream<A>)>
where
    A: AuthzProvider,
{
    if state.terminated {
        return None;
    }
    loop {
        let item = tokio::select! {
            biased;
            () = tokio::time::sleep_until(state.deadline) => return None,
            item = state.raw_stream.next() => item?,
        };
        let envelope = match item {
            Ok(envelope) => envelope,
            Err(error) => return Some(terminal(error, state)),
        };
        if envelope.channel.tenant_id != state.tenant_id {
            continue;
        }

        match check_before_deadline(&*state.authz, &state.subscribe_tuple, state.deadline).await {
            Ok(true) => {}
            Ok(false) => {
                return Some(terminal(
                    PortError::PermissionDenied("subscription authority ended".to_owned()),
                    state,
                ));
            }
            Err(error) => return Some(terminal(error, state)),
        }

        let view_tuple = RelationTuple {
            tenant_id: state.tenant_id,
            subject: state.subject.clone(),
            relation: "view".to_owned(),
            object: envelope.id.to_string(),
        };
        match check_before_deadline(&*state.authz, &view_tuple, state.deadline).await {
            Ok(true) if Instant::now() < state.deadline => return Some((Ok(envelope), state)),
            Ok(true | false) => {}
            Err(error) => return Some(terminal(error, state)),
        }
    }
}

async fn check_before_deadline<A>(
    authz: &A,
    tuple: &RelationTuple,
    deadline: Instant,
) -> Result<bool, PortError>
where
    A: AuthzProvider,
{
    tokio::time::timeout_at(deadline, authz.check(tuple))
        .await
        .map_err(|_| PortError::PermissionDenied("token expired".to_owned()))?
}

fn terminal<A>(
    error: PortError,
    mut state: AuthorizedStream<A>,
) -> (Result<EventEnvelope, PortError>, AuthorizedStream<A>) {
    state.terminated = true;
    (Err(error), state)
}

impl<L, A, I> SubscribePipeline<L, A, I> {
    #[must_use]
    pub fn into_parts(self) -> (Arc<L>, Arc<A>, Arc<I>) {
        (self.broker, self.authz, self.identity)
    }
}
