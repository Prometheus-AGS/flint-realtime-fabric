use std::net::SocketAddr;

use anyhow::Context;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfuMode {
    /// Sovereign SFU using `str0m` (no cloud dependency).
    Sovereign,
    /// Hosted SFU using `LiveKit`.
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
    pub gateway_jwks_url: String,
    pub jwt_audience: String,
    /// Expected JWT issuer (`iss`). Env: `JWT_ISSUER`. When set, tokens with a
    /// missing/mismatched issuer are rejected. Optional for backward compatibility,
    /// but SHOULD be set in production.
    pub jwt_issuer: Option<String>,
    // CDC configuration — all optional (enabled via CDC_ENABLED=true)
    pub cdc_enabled: bool,
    pub cdc_replication_url: Option<String>,
    pub cdc_slot_name: Option<String>,
    pub cdc_publication_name: Option<String>,
    pub cdc_tenant_id: Option<Uuid>,
    pub cdc_channel_path: Option<String>,
    /// SFU mode: "sovereign" → `str0m`, "hosted" → `LiveKit` (default: "hosted").
    pub sfu_mode: SfuMode,
    /// How long a tenant actor may be idle before eviction (default 300s).
    /// Env: `REGISTRY_IDLE_SECS`.
    pub registry_idle_secs: u64,
    /// How often the eviction sweep runs (default 60s).
    /// Env: `REGISTRY_SWEEP_INTERVAL_SECS`.
    pub registry_sweep_interval_secs: u64,
    /// Master opt-in for federation. Env: `FEDERATION_ENABLED` (default false).
    /// Matrix and `ATProto` bridges are only wired when this is true — federation
    /// is half-implemented for v1 (Matrix: outbound only; `ATProto`: inbound only),
    /// so it must be explicitly enabled, never silently active.
    pub federation_enabled: bool,
    /// Tenant that ingested federated events are stamped with.
    /// Env: `FEDERATION_TENANT_ID` (UUID). Required when `federation_enabled`.
    pub federation_tenant_id: Option<Uuid>,
    /// Channel that ingested federated events are published to.
    /// Env: `FEDERATION_CHANNEL_ID` (UUID). Required when `federation_enabled`.
    pub federation_channel_id: Option<Uuid>,
    // Matrix bridge — enabled when MATRIX_HOMESERVER_URL and MATRIX_ACCESS_TOKEN are set.
    pub matrix_homeserver_url: Option<String>,
    pub matrix_access_token: Option<String>,
    pub matrix_room_id: Option<String>,
    // ATProto bridge — enabled when ATPROTO_JETSTREAM_URL is set.
    pub atproto_jetstream_url: Option<String>,
    pub atproto_collections: Vec<String>,
    /// Action policy engine selection. Env: `POLICY_ENGINE` (`none` | `cedar`).
    pub policy_engine: PolicyEngineMode,
    /// Per-client request rate limit (requests per second). Env: `RATE_LIMIT_PER_SEC`
    /// (default 50).
    pub rate_limit_per_sec: u64,
    /// Per-client burst allowance above the sustained rate. Env: `RATE_LIMIT_BURST`
    /// (default 100).
    pub rate_limit_burst: u32,
    /// Maximum request body size in bytes. Env: `MAX_BODY_BYTES` (default 1 MiB).
    pub max_body_bytes: usize,
    /// Allowed CORS origins (exact matches). Env: `CORS_ALLOWED_ORIGINS`
    /// (comma-separated). Empty → CORS disabled (no cross-origin browser access).
    pub cors_allowed_origins: Vec<String>,
}

/// When the `dev-endpoints` feature is active, returns true if `DEV_NO_AUTH=true`
/// is set in the environment, bypassing JWT verification for publish/subscribe.
///
/// In non-dev-endpoints builds this function does not exist — the compiler
/// makes it impossible to call from production code paths.
#[cfg(feature = "dev-endpoints")]
#[must_use]
pub fn dev_no_auth() -> bool {
    std::env::var("DEV_NO_AUTH").is_ok_and(|v| v.eq_ignore_ascii_case("true"))
}

impl GatewayConfig {
    /// Construct a minimal `GatewayConfig` suitable for unit and integration tests.
    ///
    /// All string fields are set to placeholder values; numeric fields use
    /// permissive defaults (no gRPC port, no CDC).
    #[must_use]
    pub fn test_default() -> Self {
        Self {
            // Infallible construction from constant octets — no parse, no panic.
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            grpc_port: None,
            iggy_connection_string: "test://iggy".to_owned(),
            keto_base_url: "http://localhost:4466".to_owned(),
            keto_namespace: "default".to_owned(),
            gateway_jwks_url: "http://localhost:4456/.well-known/jwks.json".to_owned(),
            jwt_audience: "test".to_owned(),
            // Set so `test_default` models a valid *production* config: validate()
            // requires JWT_ISSUER in non-`dev-endpoints` builds (tests build that way).
            jwt_issuer: Some("https://issuer.test".to_owned()),
            cdc_enabled: false,
            cdc_replication_url: None,
            cdc_slot_name: None,
            cdc_publication_name: None,
            cdc_tenant_id: None,
            cdc_channel_path: None,
            sfu_mode: SfuMode::Sovereign,
            registry_idle_secs: 300,
            registry_sweep_interval_secs: 60,
            federation_enabled: false,
            federation_tenant_id: None,
            federation_channel_id: None,
            matrix_homeserver_url: None,
            matrix_access_token: None,
            matrix_room_id: None,
            atproto_jetstream_url: None,
            atproto_collections: vec![],
            policy_engine: PolicyEngineMode::None,
            rate_limit_per_sec: 50,
            rate_limit_burst: 100,
            max_body_bytes: 1024 * 1024,
            cors_allowed_origins: vec![],
        }
    }

    /// Validate SEMANTIC validity of the config beyond mere env-var presence, so
    /// a misconfigured deployment fails fast at boot rather than starting into a
    /// silently-broken state.
    ///
    /// Checks:
    /// - Hosted SFU (`SFU_MODE=hosted`) requires non-empty `LIVEKIT_API_KEY`,
    ///   `LIVEKIT_API_SECRET`, and `LIVEKIT_SERVER_URL` — otherwise signaling
    ///   would silently do nothing.
    /// - CDC (`CDC_ENABLED=true`) requires `CDC_REPLICATION_URL`, `CDC_SLOT_NAME`,
    ///   and `CDC_PUBLICATION_NAME`.
    /// - Federation (`FEDERATION_ENABLED=true`) requires `FEDERATION_TENANT_ID`
    ///   (otherwise ingested events land under a random tenant no one matches).
    ///
    /// # Errors
    ///
    /// Returns an error describing the first semantic violation found.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.sfu_mode == SfuMode::Hosted {
            for var in [
                "LIVEKIT_API_KEY",
                "LIVEKIT_API_SECRET",
                "LIVEKIT_SERVER_URL",
            ] {
                let present = std::env::var(var).is_ok_and(|v| !v.trim().is_empty());
                anyhow::ensure!(
                    present,
                    "SFU_MODE=hosted requires {var} to be set to a non-empty value \
                     (LiveKit signaling would otherwise be silently disabled)"
                );
            }
        }

        if self.cdc_enabled {
            anyhow::ensure!(
                self.cdc_replication_url.is_some(),
                "CDC_ENABLED=true requires CDC_REPLICATION_URL"
            );
            anyhow::ensure!(
                self.cdc_slot_name.is_some(),
                "CDC_ENABLED=true requires CDC_SLOT_NAME"
            );
            anyhow::ensure!(
                self.cdc_publication_name.is_some(),
                "CDC_ENABLED=true requires CDC_PUBLICATION_NAME"
            );
        }

        if self.federation_enabled {
            anyhow::ensure!(
                self.federation_tenant_id.is_some(),
                "FEDERATION_ENABLED=true requires FEDERATION_TENANT_ID \
                 (ingested events would otherwise land under a random per-boot tenant)"
            );
        }

        // In a production (non-`dev-endpoints`) build, JWT_ISSUER is mandatory: without
        // it the `iss` claim is unvalidated, so any JWKS-valid token — including one from
        // a different issuer — would be accepted. Dev builds keep the softer warning path
        // (see `main.rs`) so local work without a configured IdP still runs.
        #[cfg(not(feature = "dev-endpoints"))]
        anyhow::ensure!(
            self.jwt_issuer.is_some(),
            "JWT_ISSUER must be set in production so the token issuer (iss) is validated \
             and only your IdP's tokens are accepted. (This check is relaxed to a warning \
             only in `dev-endpoints` builds.)"
        );

        Ok(())
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

        let gateway_jwks_url =
            std::env::var("GATEWAY_JWKS_URL").context("GATEWAY_JWKS_URL must be set")?;

        let jwt_audience = std::env::var("JWT_AUDIENCE").context("JWT_AUDIENCE must be set")?;

        // Optional but recommended in production. When unset, issuer is not
        // validated and a warning is logged at startup (see main.rs).
        let jwt_issuer = std::env::var("JWT_ISSUER").ok().filter(|s| !s.is_empty());

        let cdc_enabled =
            std::env::var("CDC_ENABLED").is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

        let cdc_tenant_id = std::env::var("CDC_TENANT_ID")
            .ok()
            .map(|v| Uuid::parse_str(&v).context("CDC_TENANT_ID must be a valid UUID"))
            .transpose()?;

        let federation_enabled = std::env::var("FEDERATION_ENABLED")
            .is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");
        let federation_tenant_id = std::env::var("FEDERATION_TENANT_ID")
            .ok()
            .map(|v| Uuid::parse_str(&v).context("FEDERATION_TENANT_ID must be a valid UUID"))
            .transpose()?;
        let federation_channel_id = std::env::var("FEDERATION_CHANNEL_ID")
            .ok()
            .map(|v| Uuid::parse_str(&v).context("FEDERATION_CHANNEL_ID must be a valid UUID"))
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

        let (rate_limit_per_sec, rate_limit_burst, max_body_bytes, cors_allowed_origins) =
            Self::middleware_config_from_env();

        Ok(Self {
            bind_addr,
            grpc_port,
            iggy_connection_string,
            keto_base_url,
            keto_namespace,
            gateway_jwks_url,
            jwt_audience,
            jwt_issuer,
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
            federation_enabled,
            federation_tenant_id,
            federation_channel_id,
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
            rate_limit_per_sec,
            rate_limit_burst,
            max_body_bytes,
            cors_allowed_origins,
        })
    }

    /// Parse the security-middleware knobs (rate limit, body cap, CORS origins)
    /// from the environment, applying defaults. Extracted from `from_env` to keep
    /// that function focused.
    fn middleware_config_from_env() -> (u64, u32, usize, Vec<String>) {
        let rate_limit_per_sec = std::env::var("RATE_LIMIT_PER_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        let rate_limit_burst = std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        let max_body_bytes = std::env::var("MAX_BODY_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024 * 1024);
        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        (
            rate_limit_per_sec,
            rate_limit_burst,
            max_body_bytes,
            cors_allowed_origins,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn federation_is_off_by_default() {
        // Safe-by-default: federation must never be silently active. The gateway
        // only wires Matrix/ATProto bridges when this is explicitly enabled.
        let cfg = GatewayConfig::test_default();
        assert!(!cfg.federation_enabled);
        assert!(cfg.federation_tenant_id.is_none());
        assert!(cfg.federation_channel_id.is_none());
    }

    #[test]
    fn valid_default_config_passes_validation() {
        // test_default is Sovereign SFU + CDC off + federation off, with JWT_ISSUER set →
        // no unmet semantic requirements, so validation succeeds.
        GatewayConfig::test_default()
            .validate()
            .expect("default config should be valid");
    }

    // The issuer-mandatory rule only exists in production (non-`dev-endpoints`) builds;
    // in dev builds a missing issuer is a warning, not a hard error (see `main.rs`).
    #[cfg(not(feature = "dev-endpoints"))]
    #[test]
    fn production_config_without_jwt_issuer_fails_validation() {
        // Arrange: an otherwise-valid config missing JWT_ISSUER.
        let mut cfg = GatewayConfig::test_default();
        cfg.jwt_issuer = None;

        // Act
        let result = cfg.validate();

        // Assert: rejected, with a message naming the offending variable.
        let err = result.expect_err("missing JWT_ISSUER must fail validation in prod builds");
        assert!(
            err.to_string().contains("JWT_ISSUER"),
            "error should name JWT_ISSUER, got: {err}"
        );
    }

    #[test]
    fn cdc_enabled_without_replication_url_fails_fast() {
        let mut cfg = GatewayConfig::test_default();
        cfg.cdc_enabled = true;
        cfg.cdc_replication_url = None;
        let err = cfg.validate().expect_err("expected validation to fail");
        assert!(
            err.to_string().contains("CDC_REPLICATION_URL"),
            "error should name the missing field, got: {err}"
        );
    }

    #[test]
    fn cdc_enabled_with_all_fields_passes() {
        let mut cfg = GatewayConfig::test_default();
        cfg.cdc_enabled = true;
        cfg.cdc_replication_url = Some("postgres://x".to_owned());
        cfg.cdc_slot_name = Some("frf_slot".to_owned());
        cfg.cdc_publication_name = Some("frf_pub".to_owned());
        cfg.validate()
            .expect("CDC config with all fields should be valid");
    }

    #[test]
    fn federation_enabled_without_tenant_fails_fast() {
        let mut cfg = GatewayConfig::test_default();
        cfg.federation_enabled = true;
        cfg.federation_tenant_id = None;
        let err = cfg.validate().expect_err("expected validation to fail");
        assert!(err.to_string().contains("FEDERATION_TENANT_ID"));
    }

    #[test]
    fn hosted_sfu_without_livekit_creds_fails_fast() {
        // Hosted SFU requires LiveKit env. This test env does not set LIVEKIT_*,
        // so validation must fail fast rather than silently disabling signaling.
        // (Guard: skip if the env happens to have creds set, to avoid a flaky
        // false-negative in an environment that provides them.)
        if std::env::var("LIVEKIT_API_KEY").is_ok() {
            return;
        }
        let mut cfg = GatewayConfig::test_default();
        cfg.sfu_mode = SfuMode::Hosted;
        let err = cfg.validate().expect_err("expected validation to fail");
        assert!(err.to_string().contains("LIVEKIT_"));
    }
}
