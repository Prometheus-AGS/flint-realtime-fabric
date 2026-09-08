#![deny(warnings)]
#![warn(clippy::pedantic)]

use std::sync::Arc;

use anyhow::{Context as _, Result};
use frf_app::SyncUseCase;
use frf_app::{AuthzUseCase, EntityUseCase, PublishUseCase, SubscribePipeline};
use frf_broker_iggy::IggyBroker;
use frf_crdt::{InMemoryCrdtStore, LoroDeltaApplier};
use frf_domain::{Channel, TenantId, ids::ChannelId};
use frf_gateway::authz_backend::ConfiguredAuthzProvider;
use frf_gateway::config::{AuthzBackend, PolicyEngineMode};
use frf_gateway::{
    AppState, GatewayConfig, agent_grpc_service::AgentGrpcService,
    authz_grpc_service::AuthzGrpcService, entity_grpc_service::EntityGrpcService,
    entity_store_mem::InMemoryEntityStore, grpc_service::SpineGrpcService,
    signal_service::SpineSignalService, sync_grpc_service::SyncGrpcService,
};
use frf_identity_ory::OryIdentityVerifier;
use frf_librefang::LibreFangBus;
use frf_media_livekit::LiveKitSignaling;
use frf_media_str0m::StrOmSignaler;
use frf_policy_cedar::CedarPolicyEngine;
use frf_ports::{
    BoxedPolicyProvider, DynMediaSignaler, DynPolicyProvider, LogBroker, NoOpPolicyProvider,
};
use frf_postgres_cdc::{CdcConfig, PostgresCdcConsumer};
use frf_store_redb::RedbOpStore;
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

mod federation;

fn init_telemetry() -> Result<Option<TracerProvider>> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otlp_endpoint {
        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "frf-gateway".to_owned());

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .context("build OTLP span exporter")?;

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

        Ok(Some(provider))
    } else {
        fmt().with_env_filter(EnvFilter::from_default_env()).init();
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let tracer_provider = init_telemetry()?;

    // Install the Prometheus recorder before any metric is recorded so the
    // /metrics endpoint has data. Non-fatal: a failure only disables metrics.
    if let Err(e) = frf_gateway::routes::metrics::install_recorder() {
        tracing::warn!(error = %e, "metrics disabled — Prometheus recorder not installed");
    }

    let config = GatewayConfig::from_env()?;
    // Fail fast on semantically-invalid config (e.g. hosted SFU with empty
    // LiveKit creds) rather than booting into a silently-broken state.
    config.validate()?;
    let broker = Arc::new(IggyBroker::new(&config.iggy_connection_string).await?);

    // Pre-create the fixture channel used by integration and Layer 3 E2E tests.
    // IggyBroker::publish requires the stream + topic to exist before publishing.
    // ensure_channel is idempotent — safe to call on every restart.
    {
        use uuid::Uuid;
        // Fixture tenant 00000000-0000-0000-0000-000000000001, built infallibly
        // from its integer value (no parse, no panic).
        let fixture_tenant_uuid = Uuid::from_u128(1);
        let fixture_channel = Channel {
            id: ChannelId::new(),
            tenant_id: TenantId::from_uuid(fixture_tenant_uuid),
            path: "entities".into(),
        };
        if let Err(e) = broker.ensure_channel(fixture_channel).await {
            tracing::warn!(error = %e, "fixture channel pre-creation failed (non-fatal)");
        }
    }

    let authz = Arc::new(match config.authz_backend {
        AuthzBackend::VerifiedIdentity => ConfiguredAuthzProvider::verified_identity(),
        AuthzBackend::Keto => {
            ConfiguredAuthzProvider::keto(&config.keto_base_url, &config.keto_namespace)
        }
    });
    let identity = Arc::new(if let Some(issuer) = &config.jwt_issuer {
        OryIdentityVerifier::with_issuer(&config.gateway_jwks_url, &config.jwt_audience, issuer)
    } else {
        tracing::warn!(
            "JWT_ISSUER is not set — token issuer (iss) will NOT be validated. \
             Set JWT_ISSUER in production so only your IdP's tokens are trusted."
        );
        OryIdentityVerifier::new(&config.gateway_jwks_url, &config.jwt_audience)
    });

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

    let federation_bridges = federation::build_federation_bridges(&config);

    let action_policy = Arc::new(build_policy_provider(&config)?);

    // For SFU_MODE=sovereign, compose the str0m media engine + ADR-007 authz once and share it
    // across the gRPC signal service and the /ws/v1/signal inbound path (p23-c003). Building it
    // here (not inline in the gRPC wiring) lets the WS route drive the same bridge. This does
    // NOT flip the gate — end-to-end media is unproven (deferred); hosted stays the media path.
    let media_bridge = if config.sfu_mode == frf_gateway::SfuMode::Sovereign {
        // p24-c001: for a real browser↔gateway ICE path the media socket must bind a
        // reachable address + advertise a host-reachable candidate IP on a fixed UDP port.
        // MediaConfig::from_env reads MEDIA_BIND_ADDR / MEDIA_ADVERTISE_IP / MEDIA_UDP_PORT
        // (defaulting to 0.0.0.0 : ephemeral when unset).
        let transport = Arc::new(frf_media_str0m::StrOmTransport::with_config(
            frf_media_str0m::MediaConfig::from_env(),
        ));
        Some(Arc::new(
            frf_gateway::media_bridge::MediaTransportBridge::new(transport)
                .with_authz(Arc::clone(&authz) as Arc<dyn frf_ports::AuthzProvider>),
        ))
    } else {
        None
    };

    #[cfg(feature = "shape-facade")]
    let (shape_facade, shape_resolver) = build_shape_facade(&authz)?;
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
        media_bridge,
        #[cfg(feature = "shape-facade")]
        shape_facade,
        #[cfg(feature = "shape-facade")]
        shape_resolver,
        config: Arc::new(config),
    });

    federation::spawn_federation_ingest_tasks(&state);

    let app = frf_gateway::build_router(Arc::clone(&state));
    let grpc_task = spawn_grpc_server(Arc::clone(&state))?;

    tracing::info!("frf-gateway listening on {bind_addr}");
    let listener = TcpListener::bind(bind_addr).await?;

    // Graceful shutdown: on SIGTERM/SIGINT, axum stops accepting new connections
    // and DRAINS in-flight requests (and WS streams) before `serve` returns,
    // rather than aborting them mid-flight.
    axum::serve(listener, app)
        .with_graceful_shutdown(frf_gateway::shutdown_signal())
        .await?;
    tracing::info!("HTTP server drained; shutting down background tasks");

    drain_background_tasks(&shutdown_tx, cdc_task, grpc_task, tracer_provider).await;

    Ok(())
}

/// Signal background tasks to stop, await the CDC task, abort the gRPC server and flush
/// telemetry. Extracted from `main` to keep it within the line budget.
async fn drain_background_tasks(
    shutdown_tx: &tokio::sync::watch::Sender<bool>,
    cdc_task: Option<tokio::task::JoinHandle<()>>,
    grpc_task: Option<tokio::task::JoinHandle<()>>,
    tracer_provider: Option<TracerProvider>,
) {
    let _ = shutdown_tx.send(true);

    if let Some(task) = cdc_task {
        let _ = task.await;
    }
    if let Some(task) = grpc_task {
        task.abort();
    }

    if let Some(provider) = tracer_provider
        && let Err(e) = provider.shutdown()
    {
        tracing::warn!(error = %e, "OTEL tracer provider shutdown error");
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
            // Sovereign signaling relay. The str0m MEDIA engine (StrOmTransport) is composed
            // separately and driven from the signal path via MediaTransportBridge (p22-c003).
            //
            // p36-c004: the end-to-end browser decode proof PASSES. A two-browser Chromium run
            // against this engine observed `inbound-rtp.framesDecoded > 0` on the authenticated
            // path (see docs/PHASE-36-LOCAL-DECODE-RESULT.md). The gate that phases 16-36 held
            // shut is therefore open: this is no longer a warning path.
            //
            // Scope of that proof, stated precisely: it was a LOCAL run on a single Compose
            // bridge, both peers inside the network, with coturn available. It demonstrates the
            // relay decodes real media. It is NOT a multi-host, NAT-traversal, or scale result,
            // and the advertised-candidate configuration is topology-sensitive (MEDIA_ADVERTISE_IP
            // must resolve to an address the peer can actually pair with — on a dual-stack bridge
            // a hostname can resolve IPv6-first and silently strand ICE in `new`).
            tracing::info!(
                "SFU_MODE=sovereign: signaling + str0m media engine composed; end-to-end decode \
                 proven locally (p36-c004, framesDecoded > 0). Verify MEDIA_ADVERTISE_IP is \
                 peer-reachable for your topology."
            );
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
            ConfiguredAuthzProvider,
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

    // Register every service that has a server implementation. SpineService is
    // what the browser admin UI calls over Connect/gRPC-web (Subscribe/Publish).
    // SyncService (CRDT sync) is wired here with an in-memory CRDT store + redb
    // op-log + Loro applier. EntityService (read plane) is wired with an in-memory
    // entity store. AuthzService uses the configured authorization adapter.
    // All six proto services now have server implementations.
    let spine_svc = SpineGrpcService::new(Arc::clone(&state)).into_server();
    // For SFU_MODE=sovereign, compose the str0m media engine (StrOmTransport) alongside the
    // signaling relay and drive it from the signal path via MediaTransportBridge. This does
    // NOT flip the gate — end-to-end media is unproven (deferred); hosted stays the media path.
    let signal_service = SpineSignalService::new(
        Arc::clone(&state.media_signaler),
        state.config.sfu_mode.into(),
    );
    let signal_service = match &state.media_bridge {
        // Reuse the single sovereign bridge shared with the /ws/v1/signal inbound path (p23-c003)
        // so both transports drive the same str0m engine + ADR-007 authz.
        Some(bridge) => signal_service.with_media_bridge(Arc::clone(bridge)),
        None => signal_service,
    };
    let signal_svc = signal_service.into_server();

    // SyncService: CRDT sync over bidi streaming. In-memory stores are the
    // default; persistent deployments swap in frf-store-surreal / a redb file.
    let sync_use_case = Arc::new(SyncUseCase::new(
        InMemoryCrdtStore::default(),
        RedbOpStore::in_memory().context("initialize redb in-memory op-store")?,
        LoroDeltaApplier,
    ));
    let sync_svc = SyncGrpcService::new(sync_use_case).into_server();

    // EntityService: read side of the entity plane. In-memory store is the default;
    // reads are auth-guarded (identity + tenant-equality + Keto `view`) in the use-case.
    // Built before `agent_svc` because that call consumes `state`.
    let entity_use_case = Arc::new(EntityUseCase::new(
        Arc::new(InMemoryEntityStore::new()),
        Arc::clone(&state.authz),
        Arc::clone(&state.identity),
    ));
    let entity_svc = EntityGrpcService::new(entity_use_case).into_server();

    // AuthzService: check/write/delete relation tuples against Keto. Each op verifies the
    // caller's token and enforces tenant-equality before delegating to the provider.
    let authz_use_case = Arc::new(AuthzUseCase::new(
        Arc::clone(&state.authz),
        Arc::clone(&state.identity),
    ));
    let authz_svc = AuthzGrpcService::new(authz_use_case).into_server();

    let agent_svc = AgentGrpcService::new(state).into_server();
    tracing::info!("frf-gateway gRPC (+gRPC-web) listening on {grpc_addr}");

    Ok(Some(tokio::spawn(async move {
        // `accept_http1(true)` + GrpcWebLayer lets browsers reach these services
        // via Connect-Web / gRPC-web (HTTP/1.1), not just native HTTP/2 gRPC.
        if let Err(e) = tonic::transport::Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(spine_svc)
            .add_service(signal_svc)
            .add_service(sync_svc)
            .add_service(entity_svc)
            .add_service(authz_svc)
            .add_service(agent_svc)
            .serve(grpc_addr)
            .await
        {
            tracing::error!(error = %e, "gRPC server exited with error");
        }
    })))
}

/// Compose the ADR-009 relational replication lane from environment configuration.
///
/// Returns `(None, None)` unless **both** `SHAPE_ELECTRIC_URL` and `SHAPE_CATALOG_PATH` are
/// set. A half-configured deployment therefore gets no `/v1/shape` route at all, rather than
/// one backed by an empty catalog — an empty catalog would reject every shape, which looks
/// like a client bug instead of a deployment mistake.
///
/// `SHAPE_SCOPE_COLUMN` defaults to `practice_id`; it names the column carrying the
/// practice scope and is server configuration, never client input.
/// The composed ADR-009 lane: the transport and its resolver, or `None` when unconfigured.
/// Both are always present or both absent — a facade without a resolver would be an
/// unauthorized read path.
#[cfg(feature = "shape-facade")]
type ShapeLane = (
    Option<Arc<dyn frf_ports::ShapeFacade>>,
    Option<Arc<frf_shape_electric::ShapeResolver>>,
);

#[cfg(feature = "shape-facade")]
fn build_shape_facade(authz: &Arc<ConfiguredAuthzProvider>) -> anyhow::Result<ShapeLane> {
    use anyhow::Context as _;

    let (Ok(url), Ok(catalog_path)) = (
        std::env::var("SHAPE_ELECTRIC_URL"),
        std::env::var("SHAPE_CATALOG_PATH"),
    ) else {
        tracing::info!(
            "ADR-009 shape facade not configured (SHAPE_ELECTRIC_URL / SHAPE_CATALOG_PATH unset) — lane disabled"
        );
        return Ok((None, None));
    };

    let raw = std::fs::read_to_string(&catalog_path)
        .with_context(|| format!("reading shape catalog from {catalog_path}"))?;
    let catalog =
        frf_shape_electric::ShapeCatalog::from_json(&raw).context("parsing shape catalog")?;
    let shape_count = catalog.len();

    let timeout = std::time::Duration::from_secs(
        std::env::var("SHAPE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
    );
    let upstream =
        frf_shape_electric::HttpElectric::new(&url, timeout).context("building Electric client")?;
    let facade = frf_shape_electric::ElectricShapeFacade::new(upstream);

    let scope_column =
        std::env::var("SHAPE_SCOPE_COLUMN").unwrap_or_else(|_| "practice_id".to_owned());
    let resolver = frf_shape_electric::ShapeResolver::new(
        catalog,
        Arc::clone(authz) as Arc<dyn frf_ports::AuthzProvider>,
        scope_column,
    );

    // Count and endpoint only — never the catalog contents.
    tracing::warn!(
        shapes = shape_count,
        "ADR-009 shape facade ENABLED — this lane is not certified; the live Electric \
         exchange is unverified and ASO's replica schema is not finalized"
    );

    Ok((Some(Arc::new(facade)), Some(Arc::new(resolver))))
}
