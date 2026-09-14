//! Process bootstrap: telemetry installation and well-known channel pre-creation.
//!
//! Extracted from `main.rs` in `p38-c004` to bring that file back under the
//! 500-line cap (`constraints.md` R5, BLOCKING). Behaviour is unchanged — these
//! are the same two functions, moved verbatim.

use anyhow::{Context as _, Result};
use frf_broker_iggy::IggyBroker;
use frf_domain::{Channel, TenantId, ids::ChannelId};
use frf_ports::LogBroker;
use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, fmt};

/// Install the tracing subscriber, with an OTLP layer when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
///
/// Returns the `TracerProvider` when OTLP is active so the caller can shut it
/// down on exit; `None` when only the fmt layer is installed.
///
/// # Errors
///
/// Returns an error if the OTLP span exporter cannot be built.
pub(crate) fn init_telemetry() -> Result<Option<TracerProvider>> {
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

/// Pre-create the well-known entities channel that the E2E clients subscribe to.
///
/// `ensure_channel` is idempotent — safe to call on every restart.
///
/// This previously ran inline in `main` with `ChannelId::new()`, creating
/// `channel-<random-uuid>` on every boot: a stream nobody could address, while a
/// `warn!` made the failure look benign. The id must match
/// [`ChannelId::WELL_KNOWN_ENTITIES`] or no subscriber can reach the entity feed.
///
/// # Errors
///
/// Returns an error if the channel cannot be created. This is deliberately fatal:
/// a gateway that boots without the entities channel serves no events.
pub(crate) async fn ensure_entities_channel(broker: &IggyBroker) -> Result<()> {
    // Fixture tenant 00000000-0000-0000-0000-000000000001, built infallibly from its
    // integer value (no parse, no panic). The shared `1` with the channel id above is
    // coincidental — distinct types, unrelated meanings.
    let fixture_channel = Channel {
        id: ChannelId::WELL_KNOWN_ENTITIES,
        tenant_id: TenantId::from_uuid(uuid::Uuid::from_u128(1)),
        path: "entities".into(),
    };
    let channel_id = fixture_channel.id;

    // Not "non-fatal": if this fails the entities channel does not exist and every
    // subscriber to it receives nothing. Fail the boot rather than serving a gateway
    // that looks healthy and delivers no events.
    broker
        .ensure_channel(fixture_channel)
        .await
        .with_context(|| {
            format!("failed to pre-create the well-known entities channel {channel_id}")
        })?;

    tracing::info!(%channel_id, "entities channel ready");
    Ok(())
}
