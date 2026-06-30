#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::sync::Arc;

use anyhow::{Context as _, Result};
use frf_app::{PublishUseCase, SubscribePipeline};
use frf_authz_keto::KetoAuthzProvider;
use frf_bridge_atproto::AtProtoBridge;
use frf_bridge_matrix::MatrixBridge;
use frf_bridge_matrix::client::ReqwestMatrixClient;
use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, TenantId, ids::ChannelId};
use frf_gateway::config::PolicyEngineMode;
use frf_gateway::{AppState, GatewayConfig, agent_grpc_service::AgentGrpcService};
use frf_identity_ory::OryIdentityVerifier;
use frf_librefang::LibreFangBus;
use frf_media_livekit::LiveKitSignaling;
use frf_media_str0m::StrOmSignaler;
use frf_policy_cedar::CedarPolicyEngine;
use frf_ports::{
    BoxedPolicyProvider, DynMediaSignaler, DynPolicyProvider, FederationBridge, FederationProtocol,
    LogBroker, NoOpPolicyProvider,
};
use frf_postgres_cdc::{CdcConfig, PostgresCdcConsumer};
use futures_util::StreamExt as _;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::TracerProvider;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, fmt};

fn init_telemetry() -> Option<TracerProvider> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otlp_endpoint {
        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "frf-gateway".to_owned());

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .expect("build OTLP span exporter");

        let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

        let provider = TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(resource)
            .build();

        global::set_tracer_provider(provider.clone());

        let tracer = provider.tracer("frf-gateway");
        let otel_layer = OpenTelemetryLayer::new(tracer);

        tracing_subscriber::registry()
            .with(otel_layer)
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();

        Some(provider)
    } else {
        fmt().with_env_filter(EnvFilter::from_default_env()).init();
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _tracer_provider = init_telemetry();

    let config = GatewayConfig::from_env()?;
    let broker = Arc::new(IggyBroker::new(&config.iggy_connection_string).await?);

    // Pre-create the fixture channel used by integration and Layer 3 E2E tests.
    // IggyBroker::publish requires the stream + topic to exist before publishing.
    // ensure_channel is idempotent — safe to call on every restart.
    {
        use uuid::Uuid;
        let fixture_tenant_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .expect("fixture tenant UUID is a compile-time constant");
        let fixture_channel = Channel {
            id: ChannelId::new(),
            tenant_id: TenantId::from_uuid(fixture_tenant_uuid),
            path: "entities".into(),
        };
        if let Err(e) = broker.ensure_channel(fixture_channel).await {
            tracing::warn!(error = %e, "fixture channel pre-creation failed (non-fatal)");
        }
    }

    let authz = Arc::new(KetoAuthzProvider::new(
        &config.keto_base_url,
        &config.keto_namespace,
    ));
    let identity = Arc::new(OryIdentityVerifier::new(
        &config.oathkeeper_jwks_url,
        &config.jwt_audience,
    ));

    let subscribe_pipeline = Arc::new(SubscribePipeline::new(
        Arc::clone(&broker),
        Arc::clone(&authz),
        Arc::clone(&identity),
    ));
    let publish_usecase = Arc::new(PublishUseCase::new(
        Arc::clone(&broker),
        Arc::clone(&authz),
        Arc::clone(&identity),
    ));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let cdc_task = spawn_cdc_consumer(&config, Arc::clone(&broker), shutdown_rx.clone())?;

    let media_signaler = Arc::new(build_media_signaler(&config));
    let agent_bus = Arc::new(LibreFangBus::start_with_config(
        config.registry_idle_secs,
        config.registry_sweep_interval_secs,
    )?);
    let bind_addr = config.bind_addr;

    let federation_bridges = build_federation_bridges(&config);

    let action_policy = Arc::new(build_policy_provider(&config)?);

    let state = Arc::new(AppState {
        subscribe_pipeline,
        publish_usecase,
        media_signaler,
        agent_bus,
        identity: Arc::clone(&identity),
        authz: Arc::clone(&authz),
        log_broker: Arc::clone(&broker),
        action_policy,
        federation_bridges,
        config: Arc::new(config),
    });

    spawn_federation_ingest_tasks(&state);

    let app = frf_gateway::build_router(Arc::clone(&state));
    let grpc_task = spawn_grpc_server(Arc::clone(&state))?;

    tracing::info!("frf-gateway listening on {bind_addr}");
    let listener = TcpListener::bind(bind_addr).await?;

    tokio::select! {
        result = axum::serve(listener, app) => { result?; }
        _ = tokio::signal::ctrl_c() => { tracing::info!("received ctrl-c, shutting down"); }
    }

    let _ = shutdown_tx.send(true);

    if let Some(task) = cdc_task {
        let _ = task.await;
    }
    if let Some(task) = grpc_task {
        task.abort();
    }

    if let Some(provider) = _tracer_provider {
        if let Err(e) = provider.shutdown() {
            tracing::warn!(error = %e, "OTEL tracer provider shutdown error");
        }
    }

    Ok(())
}

fn build_federation_bridges(
    config: &GatewayConfig,
) -> Vec<(FederationProtocol, Arc<dyn FederationBridge + Send + Sync>)> {
    let mut bridges: Vec<(FederationProtocol, Arc<dyn FederationBridge + Send + Sync>)> =
        Vec::new();

    if let (Some(url), Some(token), Some(room)) = (
        &config.matrix_homeserver_url,
        &config.matrix_access_token,
        &config.matrix_room_id,
    ) {
        let client = ReqwestMatrixClient::new(url, token);
        let bridge = MatrixBridge::new(client, room, TenantId::new(), ChannelId::new());
        tracing::info!(room_id = %room, "Matrix federation bridge enabled");
        bridges.push((FederationProtocol::Matrix, Arc::new(bridge)));
    }

    if let Some(url) = &config.atproto_jetstream_url {
        let collections = config.atproto_collections.clone();
        let bridge = AtProtoBridge::new(url, collections, TenantId::new(), ChannelId::new());
        tracing::info!(jetstream_url = %url, "ATProto federation bridge enabled");
        bridges.push((FederationProtocol::AtProto, Arc::new(bridge)));
    }

    bridges
}

fn spawn_federation_ingest_tasks(
    state: &Arc<
        AppState<
            IggyBroker,
            KetoAuthzProvider,
            OryIdentityVerifier,
            DynMediaSignaler,
            LibreFangBus,
            BoxedPolicyProvider,
        >,
    >,
) {
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

fn spawn_cdc_consumer(
    config: &GatewayConfig,
    broker: Arc<IggyBroker>,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<Option<tokio::task::JoinHandle<()>>> {
    if !config.cdc_enabled {
        return Ok(None);
    }

    let replication_url = config
        .cdc_replication_url
        .clone()
        .context("CDC_REPLICATION_URL must be set when CDC_ENABLED=true")?;
    let slot_name = config
        .cdc_slot_name
        .clone()
        .context("CDC_SLOT_NAME must be set when CDC_ENABLED=true")?;
    let publication_name = config
        .cdc_publication_name
        .clone()
        .context("CDC_PUBLICATION_NAME must be set when CDC_ENABLED=true")?;
    let tenant_uuid = config
        .cdc_tenant_id
        .context("CDC_TENANT_ID must be set when CDC_ENABLED=true")?;
    let channel_path = config
        .cdc_channel_path
        .clone()
        .context("CDC_CHANNEL_PATH must be set when CDC_ENABLED=true")?;

    let cdc_config = CdcConfig::new(
        replication_url,
        slot_name,
        publication_name,
        TenantId::from_uuid(tenant_uuid),
        channel_path,
    );
    let consumer = PostgresCdcConsumer::new(cdc_config, broker);
    tracing::info!("starting CDC consumer");
    Ok(Some(tokio::spawn(async move {
        if let Err(e) = consumer.run_until_shutdown(shutdown_rx).await {
            tracing::error!(error = %e, "CDC consumer exited with error");
        }
    })))
}

fn build_policy_provider(config: &GatewayConfig) -> Result<BoxedPolicyProvider> {
    match config.policy_engine {
        PolicyEngineMode::Cedar => {
            tracing::info!("action policy engine: Cedar");
            let engine = CedarPolicyEngine::new()
                .map_err(|e| anyhow::anyhow!("failed to load Cedar policy: {e}"))?;
            Ok(BoxedPolicyProvider(Arc::new(engine) as DynPolicyProvider))
        }
        PolicyEngineMode::None => {
            tracing::info!("action policy engine: no-op (all permitted)");
            Ok(BoxedPolicyProvider(
                Arc::new(NoOpPolicyProvider) as DynPolicyProvider
            ))
        }
    }
}

fn build_media_signaler(config: &GatewayConfig) -> DynMediaSignaler {
    use frf_gateway::SfuMode;

    match config.sfu_mode {
        SfuMode::Sovereign => {
            tracing::info!("SFU mode: sovereign (str0m)");
            DynMediaSignaler::new(Arc::new(StrOmSignaler::new()))
        }
        SfuMode::Hosted => {
            tracing::info!("SFU mode: hosted (LiveKit)");
            let lk = LiveKitSignaling::from_env().unwrap_or_else(|e| {
                tracing::warn!(error = %e, "LiveKit env vars absent — signaling disabled");
                LiveKitSignaling::new(frf_media_livekit::LiveKitConfig {
                    api_key: String::new(),
                    api_secret: String::new(),
                    server_url: String::new(),
                    room_prefix: String::from("frf/"),
                })
            });
            DynMediaSignaler::new(Arc::new(lk))
        }
    }
}

fn spawn_grpc_server(
    state: Arc<
        AppState<
            IggyBroker,
            KetoAuthzProvider,
            OryIdentityVerifier,
            DynMediaSignaler,
            LibreFangBus,
            BoxedPolicyProvider,
        >,
    >,
) -> Result<Option<tokio::task::JoinHandle<()>>> {
    let Some(grpc_port) = state.config.grpc_port else {
        tracing::info!("gRPC server disabled (GRPC_PORT=0)");
        return Ok(None);
    };

    let grpc_addr: std::net::SocketAddr = format!("0.0.0.0:{grpc_port}").parse()?;
    let agent_svc = AgentGrpcService::new(state).into_server();
    tracing::info!("frf-gateway gRPC listening on {grpc_addr}");

    Ok(Some(tokio::spawn(async move {
        if let Err(e) = tonic::transport::Server::builder()
            .add_service(agent_svc)
            .serve(grpc_addr)
            .await
        {
            tracing::error!(error = %e, "gRPC server exited with error");
        }
    })))
}
