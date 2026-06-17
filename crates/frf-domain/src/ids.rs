use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            #[must_use]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

uuid_newtype!(
    ChannelId,
    "Unique identifier for a pub/sub channel on the event spine."
);
uuid_newtype!(EventId, "Unique identifier for a single event envelope.");
uuid_newtype!(
    CursorId,
    "Unique identifier for a read cursor on a channel."
);
uuid_newtype!(EntityId, "Unique identifier for a domain entity.");
uuid_newtype!(AgentId, "Unique identifier for an agent (AG-UI / A2A).");
uuid_newtype!(SessionId, "Unique identifier for a client session.");
uuid_newtype!(
    TenantId,
    "Unique identifier for a tenant (top-level isolation boundary)."
);
