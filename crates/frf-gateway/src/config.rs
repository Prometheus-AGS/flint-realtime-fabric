use std::net::SocketAddr;

use anyhow::Context;

pub struct GatewayConfig {
    pub bind_addr: SocketAddr,
    pub iggy_connection_string: String,
    pub keto_base_url: String,
    pub keto_namespace: String,
    pub oathkeeper_jwks_url: String,
    pub jwt_audience: String,
}

impl GatewayConfig {
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

        Ok(Self {
            bind_addr,
            iggy_connection_string,
            keto_base_url,
            keto_namespace,
            oathkeeper_jwks_url,
            jwt_audience,
        })
    }
}
