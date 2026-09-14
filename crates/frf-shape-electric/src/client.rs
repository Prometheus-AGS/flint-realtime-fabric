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
use frf_ports::{PortError, ShapeBodyStream, ShapeHeader, ShapeResponse};
use futures_util::StreamExt as _;
use reqwest::header::IF_NONE_MATCH;

use crate::error::ShapeError;

const PROTOCOL_HEADERS: [&str; 10] = [
    "electric-handle",
    "electric-offset",
    "electric-schema",
    "electric-up-to-date",
    "electric-must-refetch",
    "electric-cursor",
    "etag",
    "cache-control",
    "content-type",
    "vary",
];

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
    async fn get_shape(
        &self,
        query: &[(String, String)],
        if_none_match: Option<&str>,
    ) -> Result<ShapeResponse, ShapeError>;
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

#[async_trait]
impl ElectricUpstream for HttpElectric {
    #[tracing::instrument(name = "electric::get_shape", skip(self, query), fields(params = query.len()))]
    async fn get_shape(
        &self,
        query: &[(String, String)],
        if_none_match: Option<&str>,
    ) -> Result<ShapeResponse, ShapeError> {
        let url = format!("{}/v1/shape", self.base_url.trim_end_matches('/'));
        let mut request = self.client.get(&url).query(query);
        if let Some(etag) = if_none_match {
            request = request.header(IF_NONE_MATCH, etag);
        }
        let response = request
            .send()
            .await
            .map_err(|e| ShapeError::Upstream(e.to_string()))?;

        let status = response.status().as_u16();
        if !matches!(status, 200 | 304 | 409) {
            return Err(ShapeError::Upstream(format!("status {status}")));
        }
        let headers = response
            .headers()
            .iter()
            .filter(|(name, _)| {
                PROTOCOL_HEADERS
                    .iter()
                    .any(|allowed| name.as_str().eq_ignore_ascii_case(allowed))
            })
            .map(|(name, value)| ShapeHeader::new(name.as_str(), value.as_bytes()))
            .collect();

        let body: ShapeBodyStream = Box::pin(response.bytes_stream().map(|frame| {
            frame.map_err(|error| PortError::Transport(error.without_url().to_string()))
        }));

        Ok(ShapeResponse::streamed(status, headers, body))
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    #[test]
    fn protocol_allowlist_contains_electric_and_cache_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("electric-schema", HeaderValue::from_static("{}"));
        headers.insert("etag", HeaderValue::from_static("shape:1"));
        headers.insert("server", HeaderValue::from_static("private-electric"));
        let preserved = headers
            .iter()
            .filter(|(name, _)| {
                PROTOCOL_HEADERS
                    .iter()
                    .any(|allowed| name.as_str().eq_ignore_ascii_case(allowed))
            })
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(preserved, ["electric-schema", "etag"]);
    }

    #[tokio::test]
    async fn upstream_error_status_never_exposes_its_body() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("test server address");
        let protected_body = "protected relation aso.cases";
        let response = format!(
            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{protected_body}",
            protected_body.len()
        );
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await.expect("read request");
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        });

        let client = HttpElectric::new(
            format!("http://{address}"),
            std::time::Duration::from_secs(1),
        )
        .expect("build client");
        let error = client
            .get_shape(&[], None)
            .await
            .expect_err("an upstream error body must not cross the facade");

        assert!(matches!(error, ShapeError::Upstream(message) if message == "status 500"));
        server.await.expect("test server completed");
    }

    #[tokio::test]
    async fn must_refetch_response_preserves_header_and_control_body() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("test server address");
        let body = r#"[{"headers":{"control":"must-refetch"}}]"#;
        let response = format!(
            "HTTP/1.1 409 Conflict\r\nelectric-handle: replacement\r\nelectric-must-refetch: true\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await.expect("read request");
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write response");
        });

        let client = HttpElectric::new(
            format!("http://{address}"),
            std::time::Duration::from_secs(1),
        )
        .expect("build client");
        let response = client.get_shape(&[], None).await.expect("must-refetch");

        assert_eq!(response.status, 409);
        assert_eq!(response.header("electric-must-refetch"), Some(&b"true"[..]));
        let mut streamed = response.body;
        let mut received = Vec::new();
        while let Some(frame) = streamed.next().await {
            received.extend_from_slice(&frame.expect("body frame"));
        }
        assert_eq!(received, body.as_bytes());
        server.await.expect("test server completed");
    }

    #[tokio::test]
    async fn response_headers_return_before_the_streamed_body_finishes() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let address = listener.local_addr().expect("test server address");
        let (release_body, body_released) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await.expect("read request");
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nelectric-handle: streamed\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nfirst\r\n",
                )
                .await
                .expect("write headers and first frame");
            let _ = body_released.await;
            socket
                .write_all(b"6\r\nsecond\r\n0\r\n\r\n")
                .await
                .expect("write final frame");
        });

        let client = HttpElectric::new(
            format!("http://{address}"),
            std::time::Duration::from_secs(1),
        )
        .expect("build client");
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            client.get_shape(&[], None),
        )
        .await
        .expect("headers return without waiting for the final frame")
        .expect("streamed response");

        release_body.send(()).expect("release response body");
        let mut streamed = response.body;
        let mut received = Vec::new();
        while let Some(frame) = streamed.next().await {
            received.extend_from_slice(&frame.expect("body frame"));
        }

        assert_eq!(received, b"firstsecond");
        server.await.expect("test server completed");
    }
}
