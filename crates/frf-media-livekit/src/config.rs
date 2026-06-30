use anyhow::Context as _;

/// Configuration for the `LiveKit` hosted SFU adapter.
#[derive(Debug, Clone)]
pub struct LiveKitConfig {
    pub api_key: String,
    pub api_secret: String,
    pub server_url: String,
    /// Prefix applied to room IDs to namespace by tenant.
    /// Room IDs are formed as `{room_prefix}{tenant_id}/{room_id}`.
    pub room_prefix: String,
}

impl LiveKitConfig {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required variable is absent.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_key: std::env::var("LIVEKIT_API_KEY").context("LIVEKIT_API_KEY must be set")?,
            api_secret: std::env::var("LIVEKIT_API_SECRET")
                .context("LIVEKIT_API_SECRET must be set")?,
            server_url: std::env::var("LIVEKIT_SERVER_URL")
                .context("LIVEKIT_SERVER_URL must be set")?,
            room_prefix: std::env::var("LIVEKIT_ROOM_PREFIX").unwrap_or_else(|_| "frf/".to_owned()),
        })
    }

    /// Compute a tenant-namespaced `LiveKit` room name.
    ///
    /// Enforces tenant isolation at the room-naming layer: two tenants with the
    /// same logical room ID get distinct `LiveKit` rooms.
    #[must_use]
    pub fn namespaced_room(&self, tenant_id: &uuid::Uuid, room_id: &str) -> String {
        format!("{}{}/{}", self.room_prefix, tenant_id, room_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn tenant_namespaced_room_id_contains_tenant() {
        let config = LiveKitConfig {
            api_key: "key".into(),
            api_secret: "secret".into(),
            server_url: "https://example.livekit.cloud".into(),
            room_prefix: "frf/".into(),
        };
        let tid = Uuid::nil();
        let room = config.namespaced_room(&tid, "meeting-001");
        assert!(
            room.contains(&tid.to_string()),
            "expected tenant UUID in room name: {room}"
        );
        assert!(
            room.contains("meeting-001"),
            "expected room_id in room name: {room}"
        );
    }
}
