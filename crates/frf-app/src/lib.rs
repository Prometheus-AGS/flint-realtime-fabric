#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod authz;
pub mod entity;
pub mod error;
pub mod publish;
pub mod subscribe;
pub mod sync;

pub use authz::{AuthzRequest, AuthzUseCase};
pub use entity::{EntityRequest, EntityUseCase};
// EntityRequest is shared by both get and watch (each carries entity_id, tenant_id, token).
pub use error::AppError;
pub use publish::{PublishRequest, PublishUseCase};
pub use subscribe::{SubscribePipeline, SubscribeRequest};
pub use sync::{SyncRequest, SyncResult, SyncUseCase};
