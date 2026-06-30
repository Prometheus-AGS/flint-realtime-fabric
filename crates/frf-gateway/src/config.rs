use std::net::SocketAddr;

use anyhow::Context;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfuMode {
    /// Sovereign SFU using str0m (no cloud dependency).
    Sovereign,
    /// Hosted SFU using LiveKit.
    Hosted,
}

/// Selects the `ActionPolicyProvider` implementation at startup.
///
/// Controlled by the `POLICY_ENGINE` env var.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEngineMode {
    /// No-op: every action is permitted (default, zero overhead).
    None,
    /// Cedar: in-memory `PolicySet` loaded from the bundled `policy.cedar`.
    Cedar,
}

pub struct GatewayConfig {
    pub bind_addr: SocketAddr,
    /// gRPC server port (default 9090). Set `GRPC_PORT=0` to disable.
    pub grpc_port: Option<u16>,
    pub iggy_connection_string: String,
    pub keto_base_url: String,
    pub keto_namespace: String,
    pub oathkeeper_jwks_url: String,
    pub jwt_audience: String,
    // CDC configuration — all optional (enabled via CDC_ENABLED=true)
    pub cdc_enabled: bool,
    pub cdc_replication_url: Option<String>,
    pub cdc_slot_name: Option<String>,
    pub cdc_publication_name: Option<String>,
    pub cdc_tenant_id: Option<Uuid>,
    pub cdc_channel_path: Option<String>,
    /// SFU mode: "sovereign" → str0m, "hosted" → LiveKit (default: "hosted").
    pub sfu_mode: SfuMode,
    /// How long a tenant actor may be idle before eviction (default 300s).
    /// Env: `REGISTRY_IDLE_SECS`.
    pub registry_idle_secs: u64,
    /// How often the eviction sweep runs (default 60s).
    /// Env: `REGISTRY_SWEEP_INTERVAL_SECS`.
    pub registry_sweep_interval_secs: u64,
    // Matrix bridge — enabled when MATRIX_HOMESERVER_URL and MATRIX_ACCESS_TOKEN are set.
    pub matrix_homeserver_url: Option<String>,
    pub matrix_access_token: Option<String>,
    pub matrix_room_id: Option<String>,
    // ATProto bridge — enabled when ATPROTO_JETSTREAM_URL is set.
    pub atproto_jetstream_url: Option<String>,
    pub atproto_collections: Vec<String>,
    /// Action policy engine selection. Env: `POLICY_ENGINE` (`none` | `cedar`).
    pub policy_engine: PolicyEngineMode,
}

impl GatewayConfig {
    /// Construct a minimal `GatewayConfig` suitable for unit and integration tests.
    ///
    /// All string fields are set to placeholder values; numeric fields use
    /// permissive defaults (no gRPC port, no CDC).
    #[must_use]
    pub fn test_default() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".parse().expect("valid addr"),
            grpc_port: None,
            iggy_connection_string: "test://iggy".to_owned(),
            keto_base_url: "http://localhost:4466".to_owned(),
            keto_namespace: "default".to_owned(),
            oathkeeper_jwks_url: "http://localhost:4456/.well-known/jwks.json".to_owned(),
            jwt_audience: "test".to_owned(),
            cdc_enabled: false,
            cdc_replication_url: None,
            cdc_slot_name: None,
            cdc_publication_name: None,
            cdc_tenant_id: None,
            cdc_channel_path: None,
            sfu_mode: SfuMode::Sovereign,
            registry_idle_secs: 300,
            registry_sweep_interval_secs: 60,
            matrix_homeserver_url: None,
            matrix_access_token: None,
            matrix_room_id: None,
            atproto_jetstream_url: None,
            atproto_collections: vec![],
            policy_engine: PolicyEngineMode::None,
        }
    }

    /// Load gateway configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required environment variable is missing or if
    /// `BIND_ADDR` cannot be parsed as a [`SocketAddr`].
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
            .parse::<SocketAddr>()
            .context("BIND_ADDR must be a valid socket address")?;

        let iggy_connection_string = std::env::var("IGGY_CONNECTION_STRING")
            .context("IGGY_CONNECTION_STRING must be set")?;

        let keto_base_url = std::env::var("KETO_BASE_URL").context("KETO_BASE_URL must be set")?;

        let keto_namespace =
            std::env::var("KETO_NAMESPACE").unwrap_or_else(|_| "default".to_owned());

        let oathkeeper_jwks_url =
            std::env::var("OATHKEEPER_JWKS_URL").context("OATHKEEPER_JWKS_URL must be set")?;

        let jwt_audience = std::env::var("JWT_AUDIENCE").context("JWT_AUDIENCE must be set")?;

        let cdc_enabled =
            std::env::var("CDC_ENABLED").is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

        let cdc_tenant_id = std::env::var("CDC_TENANT_ID")
            .ok()
            .map(|v| Uuid::parse_str(&v).context("CDC_TENANT_ID must be a valid UUID"))
            .transpose()?;

        let grpc_port = match std::env::var("GRPC_PORT").as_deref() {
            Ok("0") => None,
            Ok(v) => Some(
                v.parse::<u16>()
                    .context("GRPC_PORT must be a valid port number (0–65535)")?,
            ),
            Err(_) => Some(9090),
        };

        let atproto_collections = std::env::var("ATPROTO_COLLECTIONS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();

        let sfu_mode = match std::env::var("SFU_MODE")
            .unwrap_or_else(|_| "hosted".to_owned())
            .to_ascii_lowercase()
            .as_str()
        {
            "sovereign" => SfuMode::Sovereign,
            _ => SfuMode::Hosted,
        };

        Ok(Self {
            bind_addr,
            grpc_port,
            iggy_connection_string,
            keto_base_url,
            keto_namespace,
            oathkeeper_jwks_url,
            jwt_audience,
            cdc_enabled,
            cdc_replication_url: std::env::var("CDC_REPLICATION_URL").ok(),
            cdc_slot_name: std::env::var("CDC_SLOT_NAME").ok(),
            cdc_publication_name: std::env::var("CDC_PUBLICATION_NAME").ok(),
            cdc_tenant_id,
            cdc_channel_path: std::env::var("CDC_CHANNEL_PATH").ok(),
            sfu_mode,
            registry_idle_secs: std::env::var("REGISTRY_IDLE_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            registry_sweep_interval_secs: std::env::var("REGISTRY_SWEEP_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            matrix_homeserver_url: std::env::var("MATRIX_HOMESERVER_URL").ok(),
            matrix_access_token: std::env::var("MATRIX_ACCESS_TOKEN").ok(),
            matrix_room_id: std::env::var("MATRIX_ROOM_ID").ok(),
            atproto_jetstream_url: std::env::var("ATPROTO_JETSTREAM_URL").ok(),
            atproto_collections,
            policy_engine: match std::env::var("POLICY_ENGINE")
                .unwrap_or_default()
                .to_ascii_lowercase()
                .as_str()
            {
                "cedar" => PolicyEngineMode::Cedar,
                _ => PolicyEngineMode::None,
            },
        })
    }
}
