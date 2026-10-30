#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod authz;
pub mod entity;
pub mod entity_type_watch;
pub mod error;
pub mod presence;
pub mod publish;
pub mod shape;
pub mod subscribe;
pub mod subscribe_ticket;
pub mod sync;
mod watch_checkpoint;

pub use authz::{AuthzRequest, AuthzUseCase};
pub use entity::{EntityRequest, EntityUseCase};
pub use entity_type_watch::{
    EntityTypeWatchConfig, EntityTypeWatchFrame, EntityTypeWatchRequest, EntityTypeWatchStream,
    EntityTypeWatchUseCase, ResnapshotReason, SnapshotRow, TypeWatchStart, WatchAccepted,
    WatchCheckpoint,
};
// EntityRequest is shared by both get and watch (each carries entity_id, tenant_id, token).
pub use error::AppError;
pub use presence::{PRESENCE_SWEEP_INTERVAL, PRESENCE_TTL, PresenceTracker};
pub use publish::{PublishRequest, PublishUseCase};
pub use shape::{ShapeCatalog, ShapePolicy, ShapeRequestTime, ShapeUseCase, ShapeUseCaseError};
pub use subscribe::{
    OpenedStream, Resync, SubscribePipeline, SubscribeRequest, SubscriptionGrant,
    is_resnapshot_required,
};
pub use subscribe_ticket::{CredentialKind, MAX_TICKET_LIFETIME_SECS, topic_channel_id};
pub use sync::{SyncRequest, SyncResult, SyncUseCase};
