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

impl ChannelId {
    /// The well-known channel for entity changes.
    ///
    /// The Iggy stream name derives from this id **alone** (`channel-{id}`), so a
    /// publisher that mints a fresh [`ChannelId::new`] writes to a stream no
    /// subscriber can address. Publishers of the entity feed must use this value.
    ///
    /// This constant is load-bearing across four languages: `tests/e2e/smoke_test.sh`,
    /// `tests/e2e/ts/smoke.ts`, `tests/e2e/go/main.go`, `tests/e2e/csharp/Smoke.cs` and
    /// `admin-ui/e2e/phase4-smoke.spec.ts` all default `FRF_CHANNEL_ID` to
    /// `00000000-0000-0000-0000-000000000001`. Changing it silently breaks every one.
    ///
    /// The integer `1` is shared with the fixture *tenant*
    /// (`TenantId::from_uuid(Uuid::from_u128(1))`) purely by coincidence — they are
    /// distinct types and the collision is not meaningful. Do not "reconcile" them.
    pub const WELL_KNOWN_ENTITIES: Self = Self(Uuid::from_u128(1));
}
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
