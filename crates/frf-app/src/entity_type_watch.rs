mod snapshot;
mod stream;

use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use frf_domain::{
    EntityKey, EntityTypeDelivery, EntityTypeSelector, Offset, SourcePosition, TenantId,
};
use frf_ports::{
    AuthzProvider, EntityStore, EntityTypeWatchSource, IdentityVerifier, PortError,
    ProjectionCursor, RelationTuple, VerifiedClaims,
};
use futures_util::{Stream, stream as future_stream};
use tokio::time::Instant;
use tracing::instrument;

use self::snapshot::{disclosed_barrier, snapshot_id};
use self::stream::{WatchProducer, spawn_bounded_stream};
use crate::AppError;
use crate::watch_checkpoint::{CheckpointCodec, CheckpointScope, CheckpointValidation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchCheckpoint {
    pub version: u32,
    pub token: Vec<u8>,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeWatchStart {
    Snapshot,
    Live,
    Resume(WatchCheckpoint),
}

#[derive(Debug, Clone)]
pub struct EntityTypeWatchRequest {
    pub entity_type: EntityTypeSelector,
    pub tenant_id: TenantId,
    pub start: TypeWatchStart,
    pub bearer_token: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatchAccepted {
    pub stream_id: String,
    pub entity_type: EntityTypeSelector,
    pub tenant_id: TenantId,
    pub retention_seconds: u64,
    pub buffer_capacity: u32,
    pub start_checkpoint: WatchCheckpoint,
    pub snapshot_barrier: Option<SourcePosition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotRow {
    pub snapshot_id: String,
    pub key: EntityKey,
    pub record: Vec<frf_domain::EntityField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResnapshotReason {
    HistoryExpired,
    SourceEpochChanged,
    CheckpointScopeMismatch,
    CheckpointUnsupported,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EntityTypeWatchFrame {
    Accepted(WatchAccepted),
    SnapshotRow(SnapshotRow),
    SnapshotComplete {
        barrier: SourcePosition,
        checkpoint: WatchCheckpoint,
        row_count: u64,
    },
    Mutation {
        delivery: EntityTypeDelivery,
        checkpoint: WatchCheckpoint,
    },
    CheckpointAdvanced(WatchCheckpoint),
    ResnapshotRequired {
        reason: ResnapshotReason,
        message: String,
    },
    Lagged {
        checkpoint_resumable: bool,
        last_safe_checkpoint: Option<WatchCheckpoint>,
    },
}

pub type EntityTypeWatchStream =
    Pin<Box<dyn Stream<Item = Result<EntityTypeWatchFrame, PortError>> + Send>>;
#[derive(Debug, Clone)]
pub struct EntityTypeWatchConfig {
    pub enrolled_types: Vec<EntityTypeSelector>,
    pub source_epoch: String,
    pub checkpoint_key: Vec<u8>,
    pub checkpoint_generation: u64,
    pub retention_seconds: u64,
    pub buffer_capacity: usize,
    pub authority_recheck: Duration,
}

pub struct EntityTypeWatchUseCase<S, P: ?Sized, A, I> {
    source: Arc<S>,
    projection: Arc<P>,
    authz: Arc<A>,
    identity: Arc<I>,
    config: EntityTypeWatchConfig,
    checkpoints: CheckpointCodec,
}

struct Admission {
    claims: VerifiedClaims,
    deadline: Instant,
    subscribe_tuple: RelationTuple,
}

impl<S, P, A, I> EntityTypeWatchUseCase<S, P, A, I>
where
    S: EntityTypeWatchSource,
    P: EntityStore + ?Sized,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    /// Build the v2 entity-type watch application service.
    ///
    /// # Errors
    ///
    /// Returns an error when checkpoint or bounded-buffer configuration is unsafe.
    pub fn new(
        source: Arc<S>,
        projection: Arc<P>,
        authz: Arc<A>,
        identity: Arc<I>,
        config: EntityTypeWatchConfig,
    ) -> Result<Self, String> {
        if config.buffer_capacity == 0 || config.buffer_capacity > u32::MAX as usize {
            return Err("entity watch buffer capacity must be between 1 and u32::MAX".to_owned());
        }
        if config.authority_recheck.is_zero() || config.authority_recheck > Duration::from_secs(5) {
            return Err("entity watch authority recheck must be within five seconds".to_owned());
        }
        let checkpoints =
            CheckpointCodec::new(&config.checkpoint_key, config.checkpoint_generation)?;
        Ok(Self {
            source,
            projection,
            authz,
            identity,
            config,
            checkpoints,
        })
    }

    /// Open an authorized snapshot/live or resumed entity-type stream.
    ///
    /// # Errors
    ///
    /// Fails before data delivery for invalid identity, tenant, enrollment or
    /// subscribe permission. Recovery failures are returned as terminal frames.
    #[instrument(name = "app::entity_type_watch", skip(self, request), fields(entity_type = %request.entity_type.canonical_name(), tenant_id = %request.tenant_id))]
    pub async fn watch(
        &self,
        request: EntityTypeWatchRequest,
    ) -> Result<EntityTypeWatchStream, AppError> {
        let Admission {
            claims,
            deadline,
            subscribe_tuple,
        } = self.admit(&request).await?;
        let scope = CheckpointScope {
            subject: &claims.subject,
            tenant_id: claims.tenant_id,
            entity_type: &request.entity_type,
            source_epoch: &self.config.source_epoch,
        };
        let consumer_id = format!("entity-type-watch-{}", uuid::Uuid::new_v4());
        let (raw, initial, last_offset) = match request.start.clone() {
            TypeWatchStart::Resume(checkpoint) => {
                let offset = match self.checkpoints.validate(&checkpoint, &scope) {
                    Ok(offset) => offset,
                    Err(error) => return Ok(terminal_resnapshot(error)),
                };
                let from = offset.map_or(Offset::BEGINNING, |value| Offset(value).next());
                let raw = match self.source.subscribe(consumer_id, from).await {
                    Ok(raw) => raw,
                    Err(error) => return history_or_error(error),
                };
                (raw, Vec::new(), offset)
            }
            TypeWatchStart::Live => {
                let head = self.source.head_offset().await?;
                let from = head.map_or(Offset::BEGINNING, Offset::next);
                let raw = self.source.subscribe(consumer_id, from).await?;
                (raw, Vec::new(), head.map(|offset| offset.0))
            }
            TypeWatchStart::Snapshot => {
                let snapshot = self
                    .projection
                    .snapshot_entity_type(&request.entity_type, request.tenant_id)
                    .await?;
                if snapshot
                    .cursor
                    .as_ref()
                    .is_some_and(|cursor| cursor.source_epoch != self.config.source_epoch)
                {
                    return Ok(single_frame(EntityTypeWatchFrame::ResnapshotRequired {
                        reason: ResnapshotReason::SourceEpochChanged,
                        message: "projection source epoch changed".to_owned(),
                    }));
                }
                let from = snapshot
                    .cursor
                    .as_ref()
                    .map_or(Offset::BEGINNING, |cursor| {
                        Offset(cursor.broker_offset).next()
                    });
                let raw = match self.source.subscribe(consumer_id, from).await {
                    Ok(raw) => raw,
                    Err(error) => return history_or_error(error),
                };
                let initial = self
                    .snapshot_frames(
                        &request,
                        &claims.subject,
                        snapshot.entities,
                        snapshot.cursor,
                    )
                    .await?;
                (raw, initial, None)
            }
        };
        let accepted = self.accepted_frame(&request, &scope, last_offset, &initial)?;
        let mut frames = Vec::with_capacity(initial.len() + 1);
        frames.push(accepted);
        frames.extend(initial);
        let producer = WatchProducer::new(
            Arc::clone(&self.authz),
            Arc::clone(&self.identity),
            self.checkpoints.clone(),
            request,
            claims.subject,
            subscribe_tuple,
            self.config.source_epoch.clone(),
            deadline,
            self.config.authority_recheck,
        );
        Ok(spawn_bounded_stream(
            raw,
            producer,
            frames,
            self.config.buffer_capacity,
        ))
    }

    fn accepted_frame(
        &self,
        request: &EntityTypeWatchRequest,
        scope: &CheckpointScope<'_>,
        last_offset: Option<u64>,
        initial: &[EntityTypeWatchFrame],
    ) -> Result<EntityTypeWatchFrame, AppError> {
        let start_checkpoint = self
            .checkpoints
            .issue(scope, last_offset)
            .map_err(PortError::Serialization)?;
        let snapshot_barrier = initial.iter().find_map(|frame| match frame {
            EntityTypeWatchFrame::SnapshotComplete { barrier, .. } => Some(barrier.clone()),
            _ => None,
        });
        Ok(EntityTypeWatchFrame::Accepted(WatchAccepted {
            stream_id: uuid::Uuid::new_v4().to_string(),
            entity_type: request.entity_type.clone(),
            tenant_id: request.tenant_id,
            retention_seconds: self.config.retention_seconds,
            buffer_capacity: u32::try_from(self.config.buffer_capacity)
                .map_err(|error| PortError::Serialization(error.to_string()))?,
            start_checkpoint,
            snapshot_barrier,
        }))
    }

    async fn admit(&self, request: &EntityTypeWatchRequest) -> Result<Admission, AppError> {
        validate_selector(&request.entity_type)?;
        if !self.config.enrolled_types.contains(&request.entity_type) {
            return Err(AppError::Forbidden(
                "entity type is not enrolled".to_owned(),
            ));
        }
        let claims = self
            .identity
            .verify(&request.bearer_token)
            .await
            .map_err(AppError::Identity)?;
        if claims.tenant_id != request.tenant_id {
            return Err(AppError::Forbidden(
                "entity watch tenant mismatch".to_owned(),
            ));
        }
        let deadline = token_deadline(claims.expires_at)?;
        let subscribe_tuple = RelationTuple {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            relation: "subscribe".to_owned(),
            object: request.entity_type.canonical_name(),
        };
        if !self.authz.check(&subscribe_tuple).await? {
            return Err(AppError::Forbidden(
                "entity type subscription is not permitted".to_owned(),
            ));
        }
        Ok(Admission {
            claims,
            deadline,
            subscribe_tuple,
        })
    }

    async fn snapshot_frames(
        &self,
        request: &EntityTypeWatchRequest,
        subject: &str,
        rows: Vec<EntityTypeDelivery>,
        cursor: Option<ProjectionCursor>,
    ) -> Result<Vec<EntityTypeWatchFrame>, AppError> {
        let mut authorized = Vec::with_capacity(rows.len());
        let mut earliest_undisclosed_offset = None;
        for delivery in rows {
            if delivery.mutation.tenant_id != request.tenant_id
                || !request.entity_type.matches(&delivery.mutation)
                || delivery.mutation.source.epoch != self.config.source_epoch
            {
                continue;
            }
            let tuple = object_view_tuple(request.tenant_id, subject, &delivery.mutation.key);
            if self.authz.check(&tuple).await? {
                authorized.push(delivery);
            } else {
                earliest_undisclosed_offset = Some(
                    earliest_undisclosed_offset.map_or(delivery.broker_offset, |offset: u64| {
                        offset.min(delivery.broker_offset)
                    }),
                );
            }
        }
        let barrier = disclosed_barrier(&self.config.source_epoch, &authorized);
        // Resume before the earliest retained in-scope row withheld by object
        // authorization. A later grant can then replay that row instead of
        // skipping it behind the projection's global cursor.
        let completion_offset = match earliest_undisclosed_offset {
            Some(offset) => offset.checked_sub(1),
            None => cursor.as_ref().map(|value| value.broker_offset),
        };
        let mut frames = Vec::with_capacity(authorized.len() + 1);
        for delivery in authorized {
            let record = delivery.mutation.record.ok_or_else(|| {
                PortError::Serialization("typed projection current row has no record".to_owned())
            })?;
            frames.push(EntityTypeWatchFrame::SnapshotRow(SnapshotRow {
                snapshot_id: snapshot_id(
                    &self.config.source_epoch,
                    &request.entity_type,
                    &delivery.mutation.key,
                    barrier.commit_lsn,
                )?,
                key: delivery.mutation.key,
                record,
            }));
        }
        let scope = CheckpointScope {
            subject,
            tenant_id: request.tenant_id,
            entity_type: &request.entity_type,
            source_epoch: &self.config.source_epoch,
        };
        let checkpoint = self
            .checkpoints
            .issue(&scope, completion_offset)
            .map_err(PortError::Serialization)?;
        let row_count = u64::try_from(frames.len()).unwrap_or(u64::MAX);
        frames.push(EntityTypeWatchFrame::SnapshotComplete {
            barrier,
            checkpoint,
            row_count,
        });
        Ok(frames)
    }
}

fn validate_selector(selector: &EntityTypeSelector) -> Result<(), AppError> {
    if [&selector.schema, &selector.name, &selector.projection]
        .into_iter()
        .all(|value| {
            !value.is_empty()
                && value.len() <= 63
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
    {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "invalid entity type identifier".to_owned(),
        ))
    }
}

pub(super) fn object_view_tuple(
    tenant_id: TenantId,
    subject: &str,
    key: &EntityKey,
) -> RelationTuple {
    RelationTuple {
        tenant_id,
        subject: subject.to_owned(),
        relation: "view".to_owned(),
        object: key.canonical_id.clone(),
    }
}

fn token_deadline(expires_at: u64) -> Result<Instant, AppError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        AppError::Identity(PortError::PermissionDenied(
            "system clock is before the Unix epoch".to_owned(),
        ))
    })?;
    let remaining = Duration::from_secs(expires_at)
        .checked_sub(now)
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| {
            AppError::Identity(PortError::PermissionDenied("token expired".to_owned()))
        })?;
    Instant::now().checked_add(remaining).ok_or_else(|| {
        AppError::Identity(PortError::PermissionDenied(
            "token expiry is outside the supported clock range".to_owned(),
        ))
    })
}

fn history_or_error(error: PortError) -> Result<EntityTypeWatchStream, AppError> {
    if matches!(&error, PortError::NotFound(message) if message.starts_with("resnapshot_required:"))
    {
        Ok(single_frame(EntityTypeWatchFrame::ResnapshotRequired {
            reason: ResnapshotReason::HistoryExpired,
            message: error.to_string(),
        }))
    } else {
        Err(error.into())
    }
}

fn terminal_resnapshot(validation: CheckpointValidation) -> EntityTypeWatchStream {
    let (reason, message) = match validation {
        CheckpointValidation::HistoryExpired => (
            ResnapshotReason::HistoryExpired,
            "checkpoint retention generation expired",
        ),
        CheckpointValidation::SourceEpochChanged => (
            ResnapshotReason::SourceEpochChanged,
            "checkpoint source epoch changed",
        ),
        CheckpointValidation::ScopeMismatch | CheckpointValidation::Invalid => (
            ResnapshotReason::CheckpointScopeMismatch,
            "checkpoint is invalid for this identity or entity type",
        ),
        CheckpointValidation::Unsupported => (
            ResnapshotReason::CheckpointUnsupported,
            "checkpoint version is unsupported",
        ),
    };
    single_frame(EntityTypeWatchFrame::ResnapshotRequired {
        reason,
        message: message.to_owned(),
    })
}

fn single_frame(frame: EntityTypeWatchFrame) -> EntityTypeWatchStream {
    Box::pin(future_stream::once(async move { Ok(frame) }))
}
