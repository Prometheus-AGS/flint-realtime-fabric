use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::{MediaSignaler, PortError, SignalStream};
use livekit_api::services::room::{RoomClient, SendDataOptions};
use tokio::sync::broadcast;
use tokio_stream::StreamExt as _;
use tracing::instrument;

use crate::{config::LiveKitConfig, error::LiveKitError};

/// Per-session signal channel entry for fan-out.
#[derive(Clone)]
struct SessionChannel {
    tx: broadcast::Sender<SignalEnvelope>,
}

/// `LiveKit` hosted SFU adapter implementing [`MediaSignaler`].
///
/// # Directionality (v1 status)
///
/// - **Outbound (`send_signal`) is fully cross-node:** it publishes to the
///   `LiveKit` server via the `send_data` API, which fans the JSON payload out to
///   every participant in the tenant-namespaced room across all nodes.
/// - **Inbound (`subscribe_signals`)** serves each session from a local broadcast
///   channel. By default that channel receives only this process's own outbound
///   signals; **cross-node inbound relay is available** via
///   [`LiveKitSignaling::start_inbound_relay`] (see [`crate::inbound`]), which pumps a
///   [`crate::inbound::LiveKitDataSource`] into the same per-session fan-out.
///
/// CAPABILITY STATUS (p19-c005): the inbound-relay plumbing + trait seam are present and
/// unit-tested. The `libwebrtc`-backed data source (the `LiveKit` realtime SDK) is gated
/// behind the off-by-default `realtime` cargo feature and is proven end-to-end only
/// against a live `LiveKit` server (integration-gated) — so the default gateway build
/// stays light. Cross-node egress and single-process signaling work today.
pub struct LiveKitSignaling {
    config: LiveKitConfig,
    client: Arc<RoomClient>,
    sessions: Arc<dashmap::DashMap<SessionId, SessionChannel>>,
}

impl LiveKitSignaling {
    /// Construct from explicit config.
    #[must_use]
    pub fn new(config: LiveKitConfig) -> Self {
        let client = Arc::new(RoomClient::with_api_key(
            &config.server_url,
            &config.api_key,
            &config.api_secret,
        ));
        Self {
            config,
            client,
            sessions: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Construct from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required env var is absent.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self::new(LiveKitConfig::from_env()?))
    }

    /// Fan a signal out to subscribed session channels: unicast to `to_session`
    /// when set, otherwise broadcast to every subscribed session. Shared by the
    /// outbound `send_signal` path and the inbound cross-node relay so both deliver
    /// through the same per-session broadcast topology.
    pub(crate) fn fan_out(&self, signal: &SignalEnvelope) {
        if let Some(target_session) = signal.to_session {
            if let Some(entry) = self.sessions.get(&target_session) {
                let _ = entry.tx.send(signal.clone());
            }
        } else {
            for entry in self.sessions.iter() {
                let _ = entry.tx.send(signal.clone());
            }
        }
    }
}

#[async_trait]
impl MediaSignaler for LiveKitSignaling {
    async fn send_signal(&self, signal: SignalEnvelope) -> Result<(), PortError> {
        let span = tracing::info_span!(
            "livekit::send_signal",
            room_id = %signal.room_id,
            kind = ?signal.kind,
        );
        let _guard = span.enter();

        let room_name = self
            .config
            .namespaced_room(signal.tenant_id.as_uuid(), &signal.room_id);

        let payload = serde_json::to_vec(&signal).map_err(LiveKitError::Serialization)?;

        drop(_guard);

        // livekit-api's `send_data` uses `rand::rng()` (ThreadRng, !Send) in its
        // async body. Run it in a blocking thread to avoid the Send bound issue.
        let client = Arc::clone(&self.client);
        let room_name_owned = room_name.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                client
                    .send_data(&room_name_owned, payload, SendDataOptions::default())
                    .await
                    .map_err(|e| LiveKitError::Api(e.to_string()))
            })
        })
        .await
        .map_err(|e| PortError::Transport(format!("spawn_blocking join error: {e}")))??;

        // Fan out locally to any subscriber session channel.
        self.fan_out(&signal);

        Ok(())
    }

    #[instrument(name = "livekit::subscribe_signals", skip(self))]
    async fn subscribe_signals(
        &self,
        session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        let (tx, rx) = broadcast::channel(64);
        self.sessions.insert(session_id, SessionChannel { tx });

        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|item| match item {
                Ok(env) => Some(Ok(env)),
                Err(_) => None,
            });

        Ok(Box::pin(stream))
    }

    #[instrument(name = "livekit::remove_session", skip(self))]
    async fn remove_session(
        &self,
        session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        self.sessions.remove(&session_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_config() -> LiveKitConfig {
        LiveKitConfig {
            api_key: "key".into(),
            api_secret: "secret".into(),
            server_url: "https://example.livekit.cloud".into(),
            room_prefix: "frf/".into(),
        }
    }

    #[test]
    fn namespaced_room_contains_tenant_and_room() {
        let config = test_config();
        let tid = Uuid::nil();
        let name = config.namespaced_room(&tid, "room-abc");
        assert!(name.contains(&tid.to_string()));
        assert!(name.contains("room-abc"));
    }

    #[tokio::test]
    async fn subscribe_then_remove_session_cleans_up() {
        let adapter = LiveKitSignaling::new(test_config());
        let session_id = SessionId::new();
        let tenant_id = TenantId::from_uuid(Uuid::nil());

        let _stream = adapter
            .subscribe_signals(session_id, tenant_id)
            .await
            .expect("subscribe should succeed");

        assert!(adapter.sessions.contains_key(&session_id));

        adapter
            .remove_session(session_id, TenantId::from_uuid(Uuid::nil()))
            .await
            .expect("remove_session should succeed");

        assert!(!adapter.sessions.contains_key(&session_id));
    }
}
