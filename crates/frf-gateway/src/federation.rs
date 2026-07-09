//! Federation bridge wiring for the gateway binary.
//!
//! Builds Matrix/ATProto bridges from config (opt-in, half-implemented for v1) and spawns
//! the ingest tasks that pump inbound federated events onto the spine. Extracted from
//! `main.rs` to keep that file focused and under the 500-line module limit.

use std::sync::Arc;

use frf_authz_keto::KetoAuthzProvider;
use frf_bridge_atproto::{AtProtoBridge, PdsConfig};
use frf_bridge_matrix::MatrixBridge;
use frf_bridge_matrix::client::ReqwestMatrixClient;
use frf_broker_iggy::IggyBroker;
use frf_domain::{TenantId, ids::ChannelId};
use frf_gateway::{AppState, GatewayConfig};
use frf_identity_ory::OryIdentityVerifier;
use frf_librefang::LibreFangBus;
use frf_ports::{
    BoxedPolicyProvider, DynMediaSignaler, FederationBridge, FederationProtocol, LogBroker,
};
use futures_util::StreamExt as _;

/// The concrete `AppState` the gateway binary composes. Aliased so the ingest-task
/// signature stays readable.
type GatewayAppState = AppState<
    IggyBroker,
    KetoAuthzProvider,
    OryIdentityVerifier,
    DynMediaSignaler,
    LibreFangBus,
    BoxedPolicyProvider,
>;

pub(crate) fn build_federation_bridges(
    config: &GatewayConfig,
) -> Vec<(FederationProtocol, Arc<dyn FederationBridge + Send + Sync>)> {
    let mut bridges: Vec<(FederationProtocol, Arc<dyn FederationBridge + Send + Sync>)> =
        Vec::new();

    // Federation is HALF-implemented for v1 and OFF by default. Neither bridge is
    // fully bidirectional (Matrix: outbound send only, inbound is a stub;
    // ATProto: inbound jetstream only, outbound send gated on PDS config), so it
    // must be explicitly opted into. Nothing is silently wired.
    if !config.federation_enabled {
        if config.matrix_homeserver_url.is_some() || config.atproto_jetstream_url.is_some() {
            tracing::warn!(
                "Federation env vars are set but FEDERATION_ENABLED is not true — \
                 Matrix/ATProto bridges are NOT wired (federation is opt-in for v1)"
            );
        }
        return bridges;
    }

    // Configured tenant/channel for ingested events. Falls back to freshly-minted
    // IDs only with a loud warning — a random per-boot tenant means ingested
    // events land where no subscriber's JWT matches (they change every restart).
    let (tenant_id, channel_id) = resolve_federation_ids(config);

    if let (Some(url), Some(token), Some(room)) = (
        &config.matrix_homeserver_url,
        &config.matrix_access_token,
        &config.matrix_room_id,
    ) {
        let client = ReqwestMatrixClient::new(url, token);
        let bridge = MatrixBridge::new(client, room, tenant_id, channel_id);
        tracing::info!(
            room_id = %room,
            "Matrix federation bridge enabled (OUTBOUND send supported; INBOUND is a stub — no events until Tuwunel is wired)"
        );
        bridges.push((FederationProtocol::Matrix, Arc::new(bridge)));
    }

    if let Some(url) = &config.atproto_jetstream_url {
        let collections = config.atproto_collections.clone();
        let bridge = AtProtoBridge::new(url, collections, tenant_id, channel_id);
        // Wire the outbound PDS writer when all three writer vars are present. The
        // all-or-none invariant is enforced at boot in `GatewayConfig::validate`, so a
        // partial config never reaches here. The app-password is never logged.
        let bridge = if let (Some(pds_url), Some(identifier), Some(app_password)) = (
            config.atproto_pds_url.as_ref(),
            config.atproto_pds_identifier.as_ref(),
            config.atproto_pds_app_password.as_ref(),
        ) {
            let pds = PdsConfig {
                service_url: pds_url.clone(),
                identifier: identifier.clone(),
                app_password: app_password.clone(),
            };
            tracing::info!(
                jetstream_url = %url,
                pds_url = %pds_url,
                "ATProto federation bridge enabled (INBOUND jetstream + OUTBOUND PDS write)"
            );
            bridge.with_writer(pds, config.atproto_write_collection.clone())
        } else {
            tracing::info!(
                jetstream_url = %url,
                "ATProto federation bridge enabled (INBOUND jetstream supported; OUTBOUND send unconfigured — set ATPROTO_PDS_* to enable)"
            );
            bridge
        };
        bridges.push((FederationProtocol::AtProto, Arc::new(bridge)));
    }

    bridges
}

/// Resolve the tenant/channel that ingested federated events are stamped with.
/// Uses configured values; falls back to random IDs with a loud warning (which
/// makes federation effectively non-functional — for dev only).
fn resolve_federation_ids(config: &GatewayConfig) -> (TenantId, ChannelId) {
    let tenant_id = config.federation_tenant_id.map_or_else(
        || {
            tracing::warn!(
                "FEDERATION_TENANT_ID not set — using a random per-boot tenant. Ingested \
                 federated events will not match any subscriber and change every restart. \
                 Set FEDERATION_TENANT_ID in production."
            );
            TenantId::new()
        },
        TenantId::from_uuid,
    );
    let channel_id = config
        .federation_channel_id
        .map_or_else(ChannelId::new, ChannelId::from_uuid);
    (tenant_id, channel_id)
}

pub(crate) fn spawn_federation_ingest_tasks(state: &Arc<GatewayAppState>) {
    for (protocol, bridge) in &state.federation_bridges {
        let protocol = protocol.clone();
        let bridge = Arc::clone(bridge);
        let broker = Arc::clone(&state.log_broker);
        tokio::spawn(async move {
            match bridge.subscribe(protocol).await {
                Ok(mut stream) => {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(federated_event) => {
                                if let Err(e) = broker.publish(federated_event.envelope).await {
                                    tracing::error!(error = %e, "federation ingest publish failed");
                                }
                            }
                            Err(e) => tracing::warn!(error = %e, "federation event error"),
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "federation bridge subscribe failed"),
            }
        });
    }
}
