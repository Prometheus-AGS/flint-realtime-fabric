use std::sync::Arc;

use async_trait::async_trait;
use frf_domain::{AgentEvent, SessionId, SignalEnvelope, TenantId};
use frf_gateway::{GatewayConfig, GatewayProfile, SfuMode};
use frf_media_livekit::{LiveKitConfig, LiveKitSignaling};
use frf_media_str0m::StrOmSignaler;
use frf_ports::{
    AgentEventBus, AgentEventStream, DynMediaSignaler, MediaSignaler, PortError, SignalStream,
};

const DISABLED_MESSAGE: &str = "lane is disabled for this gateway profile";

pub(crate) struct InactiveMediaSignaler;

#[async_trait]
impl MediaSignaler for InactiveMediaSignaler {
    async fn send_signal(&self, _signal: SignalEnvelope) -> Result<(), PortError> {
        Err(PortError::PermissionDenied(DISABLED_MESSAGE.to_owned()))
    }

    async fn subscribe_signals(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<SignalStream, PortError> {
        Err(PortError::PermissionDenied(DISABLED_MESSAGE.to_owned()))
    }

    async fn remove_session(
        &self,
        _session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        Err(PortError::PermissionDenied(DISABLED_MESSAGE.to_owned()))
    }
}

pub(crate) struct InactiveAgentBus;

#[async_trait]
impl AgentEventBus for InactiveAgentBus {
    async fn publish(&self, _event: AgentEvent) -> Result<(), PortError> {
        Err(PortError::PermissionDenied(DISABLED_MESSAGE.to_owned()))
    }

    async fn subscribe(&self, _tenant_id: &str) -> Result<AgentEventStream, PortError> {
        Err(PortError::PermissionDenied(DISABLED_MESSAGE.to_owned()))
    }
}

pub(crate) fn build_media_signaler(config: &GatewayConfig) -> DynMediaSignaler {
    if config.profile == GatewayProfile::ShapeOnly {
        tracing::info!("media signaling lane disabled for shape-only profile");
        return DynMediaSignaler::new(Arc::new(InactiveMediaSignaler));
    }

    match config.sfu_mode {
        SfuMode::Sovereign => {
            tracing::info!(
                "SFU_MODE=sovereign: signaling + str0m media engine composed; end-to-end decode \
                 proven locally (p36-c004, framesDecoded > 0). Verify MEDIA_ADVERTISE_IP is \
                 peer-reachable for your topology."
            );
            DynMediaSignaler::new(Arc::new(StrOmSignaler::new()))
        }
        SfuMode::Hosted => {
            tracing::info!("SFU mode: hosted (LiveKit)");
            let livekit = LiveKitSignaling::from_env().unwrap_or_else(|error| {
                tracing::warn!(%error, "LiveKit env vars absent — signaling disabled");
                LiveKitSignaling::new(LiveKitConfig {
                    api_key: String::new(),
                    api_secret: String::new(),
                    server_url: String::new(),
                    room_prefix: String::from("frf/"),
                })
            });
            DynMediaSignaler::new(Arc::new(livekit))
        }
    }
}
