//! Versioned transport profile for an already selected channel route and observer.
//! Fabric carries this fact; it does not select a handler or authorize an effect.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Channel, EventEnvelope, EventKind, Offset, TenantId};

/// The JSON discriminator survives the frozen protobuf v1 custom-kind conversion.
pub const ROUTED_OBSERVER_PROFILE_V1: &str = "frf.routed-observer/1";

macro_rules! routed_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(transparent)]
        pub struct $name(String);

        impl $name {
            /// Create an opaque identity. Admission validates nonempty values.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

routed_id!(
    SourceOccurrenceId,
    "Stable, source-assigned occurrence identity."
);
routed_id!(
    NativeMessageId,
    "Provider-native message identity for redelivery."
);
routed_id!(ProviderId, "Source provider identity.");
routed_id!(
    AccountId,
    "Source account or configured adapter instance identity."
);
routed_id!(WorkspaceId, "Source workspace identity.");
routed_id!(RoomId, "Source room identity.");
routed_id!(ThreadId, "Source thread identity.");
routed_id!(SenderId, "Source sender identity.");
routed_id!(RouteId, "Stable selected route identity.");
routed_id!(HandlerId, "Already selected handler identity.");
routed_id!(PrincipalId, "Original requesting principal identity.");
routed_id!(SubscriberId, "Authorized observer identity.");
routed_id!(
    SubscriberCursorId,
    "Observer cursor identity, distinct from an Iggy group offset."
);
routed_id!(
    DeliveryId,
    "Identity of this observer delivery, distinct from its source occurrence."
);
routed_id!(ActionId, "Stable causal action identity.");
routed_id!(
    RevisionId,
    "Opaque source, binding, or policy revision identity."
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutedSource {
    pub occurrence_id: SourceOccurrenceId,
    pub native_message_id: NativeMessageId,
    pub tenant_id: TenantId,
    pub provider: ProviderId,
    pub account: AccountId,
    pub workspace: WorkspaceId,
    pub room: RoomId,
    pub thread: Option<ThreadId>,
    pub sender: SenderId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutedSelection {
    pub route_id: RouteId,
    pub handler_id: HandlerId,
    pub route_revision: RevisionId,
    pub binding_revision: RevisionId,
    pub policy_revision: RevisionId,
    pub original_principal: PrincipalId,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionClass {
    MetadataOnly,
    PolicyFiltered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverDelivery {
    pub delivery_id: DeliveryId,
    pub subscriber_id: SubscriberId,
    /// Owned by the subscriber's durable state, not `LogBroker::ack`.
    pub subscriber_cursor_id: SubscriberCursorId,
    pub classification: ProjectionClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalTrace {
    pub root_occurrence_id: SourceOccurrenceId,
    pub parent_action_id: Option<ActionId>,
    pub action_id: ActionId,
    pub depth: u8,
    pub fanout: u8,
    pub visited_routes: Vec<RouteId>,
}

/// Complete routed-observer payload stored inside the existing spine envelope.
/// It is not an executable command and does not confer authority on a recipient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedObserverEnvelopeV1 {
    pub profile: String,
    pub source: RoutedSource,
    pub selection: RoutedSelection,
    pub delivery: ObserverDelivery,
    pub causal: CausalTrace,
    /// Source-filtered projection. Raw credentials never belong here.
    pub projection: serde_json::Value,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutedProfileError {
    NotRoutedObserver,
    UnsupportedProfile(String),
    MissingField(&'static str),
    TenantMismatch,
    InvalidCorrelation,
    InvalidCausality,
    Serialization(String),
}

impl fmt::Display for RoutedProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRoutedObserver => f.write_str("event is not a routed observer delivery"),
            Self::UnsupportedProfile(profile) => write!(f, "unsupported routed profile: {profile}"),
            Self::MissingField(field) => write!(f, "missing routed profile field: {field}"),
            Self::TenantMismatch => {
                f.write_str("routed source tenant differs from transport channel")
            }
            Self::InvalidCorrelation => {
                f.write_str("transport correlation differs from source occurrence")
            }
            Self::InvalidCausality => {
                f.write_str("invalid routed observer causal budget or route history")
            }
            Self::Serialization(error) => write!(f, "routed profile serialization failed: {error}"),
        }
    }
}

impl std::error::Error for RoutedProfileError {}

impl RoutedObserverEnvelopeV1 {
    #[must_use]
    pub fn is_candidate(envelope: &EventEnvelope) -> bool {
        matches!(&envelope.kind, EventKind::Custom(kind) if kind.starts_with("frf.routed-observer/")
            || envelope.payload.get("profile").and_then(serde_json::Value::as_str)
                .is_some_and(|profile| profile.starts_with("frf.routed-observer/")))
    }

    /// Reject incomplete or forged profile data at the transport admission boundary.
    ///
    /// # Errors
    ///
    /// Returns a profile error for unsupported versions or missing identities.
    pub fn validate(&self) -> Result<(), RoutedProfileError> {
        if self.profile != ROUTED_OBSERVER_PROFILE_V1 {
            return Err(RoutedProfileError::UnsupportedProfile(self.profile.clone()));
        }

        for (name, value) in [
            ("source.occurrence_id", self.source.occurrence_id.as_str()),
            (
                "source.native_message_id",
                self.source.native_message_id.as_str(),
            ),
            ("source.provider", self.source.provider.as_str()),
            ("source.account", self.source.account.as_str()),
            ("source.workspace", self.source.workspace.as_str()),
            ("source.room", self.source.room.as_str()),
            ("source.sender", self.source.sender.as_str()),
            ("selection.route_id", self.selection.route_id.as_str()),
            ("selection.handler_id", self.selection.handler_id.as_str()),
            (
                "selection.route_revision",
                self.selection.route_revision.as_str(),
            ),
            (
                "selection.binding_revision",
                self.selection.binding_revision.as_str(),
            ),
            (
                "selection.policy_revision",
                self.selection.policy_revision.as_str(),
            ),
            (
                "selection.original_principal",
                self.selection.original_principal.as_str(),
            ),
            ("delivery.delivery_id", self.delivery.delivery_id.as_str()),
            (
                "delivery.subscriber_id",
                self.delivery.subscriber_id.as_str(),
            ),
            (
                "delivery.subscriber_cursor_id",
                self.delivery.subscriber_cursor_id.as_str(),
            ),
            (
                "causal.root_occurrence_id",
                self.causal.root_occurrence_id.as_str(),
            ),
            ("causal.action_id", self.causal.action_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(RoutedProfileError::MissingField(name));
            }
        }
        if self
            .source
            .thread
            .as_ref()
            .is_some_and(|id| id.as_str().trim().is_empty())
        {
            return Err(RoutedProfileError::MissingField("source.thread"));
        }
        if self
            .causal
            .parent_action_id
            .as_ref()
            .is_some_and(|id| id.as_str().trim().is_empty())
        {
            return Err(RoutedProfileError::MissingField("causal.parent_action_id"));
        }
        if self.causal.depth > 4
            || self.causal.fanout > 8
            || self
                .causal
                .visited_routes
                .iter()
                .any(|id| id.as_str().trim().is_empty())
            || self
                .causal
                .visited_routes
                .iter()
                .collect::<HashSet<_>>()
                .len()
                != self.causal.visited_routes.len()
        {
            return Err(RoutedProfileError::InvalidCausality);
        }
        Ok(())
    }

    /// Wrap the profile in the existing v1 transport envelope without changing proto v1.
    /// The transport event id and offset are not the source occurrence or observer cursor.
    ///
    /// # Errors
    ///
    /// Returns a profile error if source identity is incomplete or tenants differ.
    pub fn into_event_envelope(
        self,
        channel: Channel,
        offset: Offset,
    ) -> Result<EventEnvelope, RoutedProfileError> {
        self.validate()?;
        if self.source.tenant_id != channel.tenant_id {
            return Err(RoutedProfileError::TenantMismatch);
        }
        let occurrence_id = self.source.occurrence_id.to_string();
        let payload = serde_json::to_value(self)
            .map_err(|error| RoutedProfileError::Serialization(error.to_string()))?;
        let mut envelope = EventEnvelope::new(
            channel,
            offset,
            EventKind::Custom(ROUTED_OBSERVER_PROFILE_V1.to_owned()),
            payload,
        );
        envelope.correlation_id = Some(occurrence_id);
        Ok(envelope)
    }

    /// Decode a routed profile from an authorized transport event. Ordinary v1
    /// envelopes remain readable but cannot silently enter this profile.
    ///
    /// # Errors
    ///
    /// Returns a profile error for legacy events, malformed payloads, or tenant mismatch.
    pub fn from_event_envelope(envelope: &EventEnvelope) -> Result<Self, RoutedProfileError> {
        if !Self::is_candidate(envelope) {
            return Err(RoutedProfileError::NotRoutedObserver);
        }
        let routed: Self = serde_json::from_value(envelope.payload.clone())
            .map_err(|error| RoutedProfileError::Serialization(error.to_string()))?;
        routed.validate()?;
        if routed.source.tenant_id != envelope.channel.tenant_id {
            return Err(RoutedProfileError::TenantMismatch);
        }
        if envelope.correlation_id.as_deref() != Some(routed.source.occurrence_id.as_str()) {
            return Err(RoutedProfileError::InvalidCorrelation);
        }
        Ok(routed)
    }
}
