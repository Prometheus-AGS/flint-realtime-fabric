use std::sync::Arc;

use anyhow::{Context as _, Result};
use frf_app::{
    AuthzUseCase, EntityTypeWatchConfig, EntityTypeWatchUseCase, EntityUseCase, SyncUseCase,
};
use frf_crdt::{InMemoryCrdtStore, LoroDeltaApplier};
use frf_domain::EntityTypeSelector;
use frf_gateway::authz_backend::ConfiguredAuthzProvider;
use frf_gateway::{
    AppState, agent_grpc_service::AgentGrpcService, authz_grpc_service::AuthzGrpcService,
    entity_grpc_service::EntityGrpcService as V1EntityGrpcService,
    entity_type_grpc_service::EntityTypeGrpcService as V2EntityTypeGrpcService,
    grpc_service::SpineGrpcService, signal_service::SpineSignalService,
    sync_grpc_service::SyncGrpcService,
};
use frf_identity_ory::OryIdentityVerifier;
use frf_ports::{BoxedPolicyProvider, DynAgentEventBus, DynMediaSignaler};
use frf_postgres_cdc::CdcConfig;
use frf_store_redb::RedbOpStore;

use crate::configured_broker::ConfiguredLogBroker;

type GatewayState = AppState<
    ConfiguredLogBroker,
    ConfiguredAuthzProvider,
    OryIdentityVerifier,
    DynMediaSignaler,
    DynAgentEventBus,
    BoxedPolicyProvider,
>;

pub(crate) fn spawn(
    state: Arc<GatewayState>,
    entity_store: Arc<dyn frf_ports::EntityStore>,
) -> Result<Option<tokio::task::JoinHandle<()>>> {
    let Some(grpc_port) = state.config.grpc_port else {
        tracing::info!("gRPC server disabled (GRPC_PORT=0)");
        return Ok(None);
    };
    let grpc_addr: std::net::SocketAddr = format!("0.0.0.0:{grpc_port}").parse()?;
    let spine_svc = SpineGrpcService::new(Arc::clone(&state)).into_server();
    let signal_service = SpineSignalService::new(
        Arc::clone(&state.media_signaler),
        state.config.sfu_mode.into(),
    );
    let signal_service = match &state.media_bridge {
        Some(bridge) => signal_service.with_media_bridge(Arc::clone(bridge)),
        None => signal_service,
    };
    let signal_svc = signal_service.into_server();
    let sync_use_case = Arc::new(SyncUseCase::new(
        InMemoryCrdtStore::default(),
        RedbOpStore::in_memory().context("initialize redb in-memory op-store")?,
        LoroDeltaApplier,
    ));
    let sync_svc = SyncGrpcService::new(sync_use_case).into_server();
    let entity_use_case = Arc::new(EntityUseCase::new(
        Arc::clone(&entity_store),
        Arc::clone(&state.authz),
        Arc::clone(&state.identity),
    ));
    let entity_v1_svc = V1EntityGrpcService::new(entity_use_case).into_server();
    let entity_v2_svc = if state.config.cdc_enabled {
        let checkpoint_key = state
            .config
            .entity_watch_checkpoint_key
            .clone()
            .context("validated CDC configuration must include a watch checkpoint key")?;
        let source_epoch = state
            .config
            .cdc_source_epoch
            .clone()
            .context("validated CDC configuration must include a source epoch")?;
        let watch_enrollments = CdcConfig::parse_enrollments(
            state
                .config
                .cdc_enrollments_json
                .as_deref()
                .context("validated CDC configuration must include watch enrollments")?,
        )?
        .into_iter()
        .map(|enrollment| EntityTypeSelector {
            schema: enrollment.schema,
            name: enrollment.table,
            projection: enrollment.projection,
        })
        .collect();
        let watch_source = Arc::new(frf_watch_broker::BrokerEntityTypeWatchSource::new(
            Arc::clone(&state.log_broker),
        ));
        let watch_use_case = EntityTypeWatchUseCase::new(
            watch_source,
            entity_store,
            Arc::clone(&state.authz),
            Arc::clone(&state.identity),
            EntityTypeWatchConfig {
                enrolled_types: watch_enrollments,
                source_epoch,
                checkpoint_key: checkpoint_key.into_bytes(),
                checkpoint_generation: state.config.entity_watch_checkpoint_generation,
                retention_seconds: state.config.entity_watch_retention_seconds,
                buffer_capacity: state.config.entity_watch_buffer_capacity,
                authority_recheck: std::time::Duration::from_secs(1),
            },
        )
        .map_err(anyhow::Error::msg)?;
        Some(V2EntityTypeGrpcService::new(Arc::new(watch_use_case)).into_server())
    } else {
        tracing::info!("v2 entity-type watch disabled (CDC_ENABLED is false)");
        None
    };
    let authz_use_case = Arc::new(AuthzUseCase::new(
        Arc::clone(&state.authz),
        Arc::clone(&state.identity),
    ));
    let authz_svc = AuthzGrpcService::new(authz_use_case).into_server();
    let agent_svc = AgentGrpcService::new(state).into_server();
    tracing::info!("frf-gateway gRPC (+gRPC-web) listening on {grpc_addr}");
    Ok(Some(tokio::spawn(async move {
        if let Err(error) = tonic::transport::Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(spine_svc)
            .add_service(signal_svc)
            .add_service(sync_svc)
            .add_service(entity_v1_svc)
            .add_optional_service(entity_v2_svc)
            .add_service(authz_svc)
            .add_service(agent_svc)
            .serve(grpc_addr)
            .await
        {
            tracing::error!(error = %error, "gRPC server exited with error");
        }
    })))
}
