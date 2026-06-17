#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod agent;
pub mod entity;
pub mod envelope;
pub mod ids;
pub mod presence;
pub mod signal;
pub mod sync;

pub use agent::{AgentEvent, AgentEventKind, AgentProtocol};
pub use entity::{ChangeOp, EntityChange};
pub use envelope::{Channel, Cursor, EventEnvelope, EventKind, Offset};
pub use ids::{AgentId, ChannelId, CursorId, EntityId, SessionId, TenantId};
pub use presence::{Presence, PresenceStatus};
pub use signal::{SfuMode, SignalEnvelope, SignalKind};
pub use sync::{SyncOp, SyncOpKind};
