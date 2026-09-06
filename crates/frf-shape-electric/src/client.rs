//! The upstream Electric HTTP exchange.
//!
//! Electric's shape protocol is preserved verbatim: the request carries `table`, `columns`,
//! `where`, and — when resuming — `handle` and `offset`; the response carries the shape
//! handle, the next offset, an up-to-date marker and a must-refetch signal. The facade
//! constrains what may be asked, then forwards; it does not reinterpret the protocol or
//! rewrite the body.
//!
//! [`ElectricUpstream`] exists so the facade's request construction and response mapping are
//! testable without a live Electric server — the seam is the HTTP exchange itself, not the
//! logic around it.

use async_trait::async_trait;

use crate::error::ShapeError;

/// Electric's response to one shape request, before mapping onto the port type.
#[derive(Debug, Clone)]
pub struct UpstreamResponse {
    /// `electric-handle` response header.
    pub handle: String,
    /// `electric-offset` response header.
    pub next_offset: String,
    /// `electric-up-to-date` marker — the initial snapshot is complete.
    pub up_to_date: bool,
    /// HTTP 409 or an explicit must-refetch signal: discard and rebuild.
    pub must_refetch: bool,
    /// The response payload, unmodified.
    pub body: Vec<u8>,
}

/// One Electric HTTP exchange.
#[async_trait]
pub trait ElectricUpstream: Send + Sync + 'static {
    /// Perform a shape request against Electric.
    ///
    /// `query` carries the fully server-derived parameters; the implementation adds nothing
    /// of its own beyond transport concerns.
    ///
    /// # Errors
    ///
    /// [`ShapeError::Upstream`] if the exchange fails or returns an unusable response.
    async fn get_shape(&self, query: &[(String, String)]) -> Result<UpstreamResponse, ShapeError>;
}

/// A live Electric server over HTTP.
pub struct HttpElectric {
    client: reqwest::Client,
    base_url: String,
}

impl HttpElectric {
    /// Build a client against `base_url` (the Electric server's origin).
    ///
    /// # Errors
    ///
    /// [`ShapeError::Upstream`] if the HTTP client cannot be constructed.
    pub fn new(
        base_url: impl Into<String>,
        timeout: std::time::Duration,
    ) -> Result<Self, ShapeError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| ShapeError::Upstream(e.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }
}

/// Electric signals "your handle is gone, start over" with 409.
const HTTP_CONFLICT: u16 = 409;

#[async_trait]
impl ElectricUpstream for HttpElectric {
    #[tracing::instrument(name = "electric::get_shape", skip(self, query), fields(params = query.len()))]
    async fn get_shape(&self, query: &[(String, String)]) -> Result<UpstreamResponse, ShapeError> {
        let url = format!("{}/v1/shape", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .query(query)
            .send()
            .await
            .map_err(|e| ShapeError::Upstream(e.to_string()))?;

        let status = response.status();
        let must_refetch = status.as_u16() == HTTP_CONFLICT;
        if !status.is_success() && !must_refetch {
            // Status class only — an upstream body may carry row data.
            return Err(ShapeError::Upstream(format!("status {}", status.as_u16())));
        }

        let header = |name: &str| -> String {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_owned()
        };
        let handle = header("electric-handle");
        let next_offset = header("electric-offset");
        let up_to_date = response.headers().contains_key("electric-up-to-date");

        let body = response
            .bytes()
            .await
            .map_err(|e| ShapeError::Upstream(e.to_string()))?
            .to_vec();

        Ok(UpstreamResponse {
            handle,
            next_offset,
            up_to_date,
            must_refetch,
            body,
        })
    }
}
