#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod agent;
pub mod entity;
pub mod envelope;
pub mod ids;
pub mod presence;
pub mod routed_observer;
pub mod signal;
pub mod sync;

pub use agent::{AgentEvent, AgentEventKind, AgentProtocol};
pub use entity::{ChangeOp, EntityChange};
pub use envelope::{Channel, Cursor, EventEnvelope, EventKind, Offset};
pub use ids::{AgentId, ChannelId, CursorId, EntityId, SessionId, TenantId};
pub use presence::{Presence, PresenceStatus};
pub use routed_observer::{
    AccountId, ActionId, CausalTrace, DeliveryId, HandlerId, NativeMessageId, ObserverDelivery,
    PrincipalId, ProjectionClass, ProviderId, ROUTED_OBSERVER_PROFILE_V1, RevisionId, RoomId,
    RouteId, RoutedObserverEnvelopeV1, RoutedProfileError, RoutedSelection, RoutedSource, SenderId,
    SourceOccurrenceId, SubscriberCursorId, SubscriberId, ThreadId, WorkspaceId,
};
pub use signal::{SfuMode, SignalEnvelope, SignalKind};
pub use sync::{SyncOp, SyncOpKind};
