#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod authz;
pub mod entity;
pub mod entity_type_watch;
pub mod error;
pub mod publish;
pub mod shape;
pub mod subscribe;
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
pub use publish::{PublishRequest, PublishUseCase};
pub use shape::{ShapeCatalog, ShapePolicy, ShapeRequestTime, ShapeUseCase, ShapeUseCaseError};
pub use subscribe::{SubscribePipeline, SubscribeRequest};
pub use sync::{SyncRequest, SyncResult, SyncUseCase};
