use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use frf_domain::{ChannelId, EventEnvelope, Offset, SessionId, TenantId};
use frf_ports::{
    AuthzProvider, EventStream, IdentityVerifier, LogBroker, PortError, RelationTuple,
};
use futures_util::{StreamExt, stream};
use tokio::time::Instant;
use tracing::instrument;
use uuid::Uuid;

use crate::error::AppError;
use crate::presence::PresenceTracker;
use crate::subscribe_ticket::{CredentialKind, TicketReplayGuard, admit_ticket, match_topic};

pub struct SubscribeRequest {
    pub channel_id: ChannelId,
    pub bearer_token: String,
    pub from: Offset,
}

/// What a verified subscriber may read on one channel, and until when.
///
/// Produced by [`SubscribePipeline::authorize`]; the credential is verified
/// (and a ticket consumed) once, so a resync can reopen the stream without it.
#[derive(Debug, Clone)]
pub struct SubscriptionGrant {
    pub channel_id: ChannelId,
    pub tenant_id: TenantId,
    /// Verified `sub` of the credential.
    pub subject: String,
    pub session_id: SessionId,
    /// The topic path matched from the `topics` claim. `None` when the
    /// subscription was authorized by relation tuples.
    pub topic: Option<String>,
    deadline: Instant,
    authority: StreamAuthority,
}

impl SubscriptionGrant {
    /// The channel path this grant publishes presence under: the matched topic,
    /// or the channel id when no topic was matched.
    #[must_use]
    pub fn path(&self) -> String {
        self.topic
            .clone()
            .unwrap_or_else(|| self.channel_id.to_string())
    }
}

#[derive(Debug, Clone)]
enum StreamAuthority {
    /// Keto `subscribe` on the channel and `view` on every event.
    Relations { subscribe_tuple: RelationTuple },
    /// Every delivered event's `channel.path` must equal the granted topic.
    Topic { path: String },
}

/// The client asked for an offset the broker no longer retains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resync {
    /// Inclusive high-water offset when the stream was reopened, `None` when
    /// the channel is empty. The reopened stream starts after it.
    pub head: Option<Offset>,
}

/// An authorized event stream, and the resync that preceded it, if any.
pub struct OpenedStream {
    pub stream: EventStream,
    pub resync: Option<Resync>,
}

/// `true` when `error` is the broker's "offset no longer retained" signal.
#[must_use]
pub fn is_resnapshot_required(error: &PortError) -> bool {
    matches!(error, PortError::NotFound(message) if message.starts_with("resnapshot_required:"))
}

pub struct SubscribePipeline<L, A, I> {
    broker: Arc<L>,
    authz: Arc<A>,
    identity: Arc<I>,
    topic_namespace: Option<Uuid>,
    replay: TicketReplayGuard,
    presence: Arc<PresenceTracker<L>>,
}

impl<L, A, I> SubscribePipeline<L, A, I>
where
    L: LogBroker,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    pub fn new(broker: Arc<L>, authz: Arc<A>, identity: Arc<I>) -> Self {
        Self {
            presence: Arc::new(PresenceTracker::new(Arc::clone(&broker))),
            broker,
            authz,
            identity,
            topic_namespace: None,
            replay: TicketReplayGuard::default(),
        }
    }

    /// Set the `UUIDv5` namespace that maps a `topics` claim path to its channel
    /// id. Without it, a token carrying `topics` is refused.
    #[must_use]
    pub fn with_topic_namespace(mut self, namespace: Option<Uuid>) -> Self {
        self.topic_namespace = namespace;
        self
    }

    /// Presence heartbeats and expiry for this pipeline's broker.
    #[must_use]
    pub fn presence(&self) -> &Arc<PresenceTracker<L>> {
        &self.presence
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
        let grant = self
            .authorize(req.channel_id, &req.bearer_token, CredentialKind::Bearer)
            .await?;
        let raw_stream = self
            .broker
            .subscribe(req.channel_id, grant.session_id.to_string(), req.from)
            .await?;
        Ok(self.authorized(raw_stream, grant))
    }

    /// Verify a credential and decide what it may read on `channel_id`.
    ///
    /// A token with a `topics` claim is authorized by the claim alone; one
    /// without is authorized by the `subscribe` relation tuple. A
    /// [`CredentialKind::Ticket`] must also carry `jti` and `iat`, live at most
    /// 60 seconds, and is consumed: a second use is refused.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Identity`] for an invalid, expired, replayed or
    /// over-long credential, and [`AppError::Forbidden`] when it does not
    /// cover the channel.
    #[instrument(name = "app::subscribe::authorize", skip(self, token), fields(channel_id = %channel_id))]
    pub async fn authorize(
        &self,
        channel_id: ChannelId,
        token: &str,
        credential: CredentialKind,
    ) -> Result<SubscriptionGrant, AppError> {
        let claims = self
            .identity
            .verify(token)
            .await
            .map_err(AppError::Identity)?;
        if credential == CredentialKind::Ticket {
            admit_ticket(&claims, &self.replay, unix_now()?)?;
        }
        let deadline = token_deadline(claims.expires_at)?;

        let (topic, authority) = if let Some(topics) = &claims.topics {
            let path = match_topic(topics, self.topic_namespace, channel_id)?;
            (Some(path.clone()), StreamAuthority::Topic { path })
        } else {
            let subscribe_tuple = RelationTuple {
                tenant_id: claims.tenant_id,
                subject: claims.subject.clone(),
                relation: "subscribe".to_owned(),
                object: channel_id.to_string(),
            };
            let allowed = self
                .authz
                .check(&subscribe_tuple)
                .await
                .map_err(AppError::Broker)?;
            if !allowed {
                return Err(AppError::Forbidden(format!(
                    "subject {} may not subscribe to channel {}",
                    claims.subject, channel_id
                )));
            }
            (None, StreamAuthority::Relations { subscribe_tuple })
        };

        Ok(SubscriptionGrant {
            channel_id,
            tenant_id: claims.tenant_id,
            subject: claims.subject,
            session_id: claims.session_id,
            topic,
            deadline,
            authority,
        })
    }

    /// Open the granted channel from `from` (inclusive).
    ///
    /// When the broker no longer retains `from`, the stream is reopened after
    /// the current head and the returned [`OpenedStream::resync`] says so.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Broker`] if the broker subscription fails.
    #[instrument(name = "app::subscribe::open", skip(self, grant), fields(channel_id = %grant.channel_id))]
    pub async fn open(
        &self,
        grant: &SubscriptionGrant,
        from: Offset,
    ) -> Result<OpenedStream, AppError> {
        let consumer_id = grant.session_id.to_string();
        match self
            .broker
            .subscribe(grant.channel_id, consumer_id, from)
            .await
        {
            Ok(raw_stream) => Ok(OpenedStream {
                stream: self.authorized(raw_stream, grant.clone()),
                resync: None,
            }),
            Err(error) if is_resnapshot_required(&error) => self.open_at_head(grant).await,
            Err(error) => Err(AppError::Broker(error)),
        }
    }

    /// Reopen the granted channel after its current head, for a client whose
    /// offset is no longer retained.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Broker`] if the head cannot be read or the broker
    /// subscription fails.
    pub async fn open_at_head(&self, grant: &SubscriptionGrant) -> Result<OpenedStream, AppError> {
        let head = self.broker.head_offset(grant.channel_id).await?;
        let start = head.map_or(Offset::BEGINNING, Offset::next);
        let raw_stream = self
            .broker
            .subscribe(grant.channel_id, grant.session_id.to_string(), start)
            .await?;
        Ok(OpenedStream {
            stream: self.authorized(raw_stream, grant.clone()),
            resync: Some(Resync { head }),
        })
    }

    fn authorized(&self, raw_stream: EventStream, grant: SubscriptionGrant) -> EventStream {
        let state = AuthorizedStream {
            raw_stream,
            authz: Arc::clone(&self.authz),
            authority: grant.authority,
            tenant_id: grant.tenant_id,
            subject: grant.subject,
            deadline: grant.deadline,
            terminated: false,
        };
        Box::pin(stream::unfold(state, next_authorized::<A>))
    }
}

struct AuthorizedStream<A> {
    raw_stream: EventStream,
    authz: Arc<A>,
    authority: StreamAuthority,
    tenant_id: TenantId,
    subject: String,
    deadline: Instant,
    terminated: bool,
}

fn unix_now() -> Result<u64, AppError> {
    Ok(unix_now_duration()?.as_secs())
}

fn unix_now_duration() -> Result<Duration, AppError> {
    SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        AppError::Identity(PortError::PermissionDenied(
            "system clock is before the Unix epoch".to_owned(),
        ))
    })
}

fn token_deadline(expires_at: u64) -> Result<Instant, AppError> {
    let now = unix_now_duration()?;
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

        let subscribe_tuple = match &state.authority {
            StreamAuthority::Topic { path } => {
                if envelope.channel.path == *path && Instant::now() < state.deadline {
                    return Some((Ok(envelope), state));
                }
                continue;
            }
            StreamAuthority::Relations { subscribe_tuple } => subscribe_tuple,
        };

        match check_before_deadline(&*state.authz, subscribe_tuple, state.deadline).await {
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
