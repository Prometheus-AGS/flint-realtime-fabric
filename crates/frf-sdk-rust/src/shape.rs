//! Client for the FRF authorized shape facade — `GET /v1/shape` (ADR-009).
//!
//! # Why this is not an `ElectricSQL` client
//!
//! The request contract is inverted relative to talking to Electric directly.
//! The caller does **not** name the table, the columns, or the predicate: Gate
//! and the facade derive allowed rows, columns and tenant scope server-side
//! from verified identity. A client-supplied predicate is useful validation but
//! cannot enforce access against a modified client.
//!
//! So a request carries a **shape id** plus opaque protocol echoes, and nothing
//! else. A catalog entry declaring `allowed_params: []` rejects any further
//! query parameter with `400` — including the `table`, `columns` and `where`
//! that an off-the-shelf Electric client always sends.
//!
//! # What the facade preserves
//!
//! Electric's HTTP protocol verbatim — status, `electric-handle`,
//! `electric-offset`, `electric-up-to-date`, `electric-must-refetch`, and the
//! message body. This parses Electric's wire format while never speaking to
//! Electric and never holding a credential Electric would accept.
//!
//! # Availability
//!
//! Behind the `shape-facade` feature, off by default, mirroring the gateway's
//! own gate. ADR-009 keeps the lane uncertified: measured revocation passed
//! locally, deployment-specific topology proofs remain open.

use std::time::Duration;

use serde::Deserialize;

use crate::error::SdkError;

/// Default per-request timeout. A `live=true` long-poll can legitimately take
/// longer; override it with [`ShapeClient::with_timeout`].
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// One Electric protocol frame.
///
/// Control frames (`up-to-date`, `must-refetch`) carry no row and are filtered
/// out before a caller sees them; what remains carries `value`.
#[derive(Debug, Clone, Deserialize)]
pub struct ShapeMessage {
    /// Frame metadata. `operation` is `insert` / `update` / `delete`.
    #[serde(default)]
    pub headers: ShapeMessageHeaders,
    /// Electric's row key.
    #[serde(default)]
    pub key: Option<String>,
    /// The row itself, as raw JSON. Left untyped: the column set is decided by
    /// server-side policy, so the client cannot know it at compile time.
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Frame metadata carried by [`ShapeMessage`].
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ShapeMessageHeaders {
    /// `insert`, `update` or `delete`.
    #[serde(default)]
    pub operation: Option<String>,
    /// Present on control frames only.
    #[serde(default)]
    pub control: Option<String>,
}

impl ShapeMessage {
    /// Does this frame carry a row, rather than being a control frame?
    #[must_use]
    fn carries_row(&self) -> bool {
        self.headers.control.is_none() && self.value.is_some()
    }

    /// Is this a delete?
    ///
    /// Deletes are **returned**, not dropped. A replica that rebuilds on
    /// `must_refetch` may ignore them; a caller with a removal path must not
    /// have that decision made for it.
    #[must_use]
    pub fn is_delete(&self) -> bool {
        self.headers.operation.as_deref() == Some("delete")
    }
}

/// The opaque position in a shape's stream.
///
/// Both fields are echoed back verbatim on the next request, and neither is
/// interpretable by the client: `offset` is Electric's LSN-derived cursor and
/// `handle` identifies the server-side shape instance. Constructing one by hand
/// is always wrong — take it from a [`ShapeFrame`].
///
/// The two travel together by construction because the facade's `parse_cursor`
/// rejects a `handle` without an `offset` with `400`. Keeping them in one
/// struct makes that invalid request unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeCursor {
    /// Server-side shape instance id.
    pub handle: String,
    /// Position within that instance's stream.
    pub offset: String,
}

/// Optional per-request protocol controls the facade accepts.
///
/// Every field here is a *protocol echo*, not a query. Anything else added to
/// the request is narrowing input, which a shape declaring `allowed_params: []`
/// refuses with `400`.
#[derive(Debug, Clone, Default)]
pub struct ShapeRequestOptions {
    /// Electric long-polling mode (`live=true`).
    pub live: bool,
    /// Electric cache-busting cursor, echoed from a prior response.
    pub cursor: Option<String>,
    /// Conditional request validator, sent as `If-None-Match`.
    pub if_none_match: Option<String>,
}

/// What one [`ShapeClient::fetch_shape`] call observed.
#[derive(Debug, Clone)]
pub struct ShapeFrame {
    /// Row frames in upstream order, control frames removed. Deletes retained.
    pub messages: Vec<ShapeMessage>,
    /// The cursor to send next; `None` when the server returned no complete one.
    pub cursor: Option<ShapeCursor>,
    /// The stream has caught up — `electric-up-to-date` was present.
    pub up_to_date: bool,
    /// The history this cursor pointed into is gone (`409`, or
    /// `electric-must-refetch`).
    ///
    /// The caller must discard local state for this shape and restart cold.
    /// Retrying with the same cursor cannot succeed, which is why no cursor is
    /// returned alongside this.
    pub must_refetch: bool,
    /// `304` — the conditional request matched; nothing changed.
    pub not_modified: bool,
}

/// Reads one shape at a time from the authorized facade.
///
/// Cheap to clone conceptually but not `Clone`: hold one per base URL. The
/// inner `reqwest::Client` owns a connection pool, so reusing a single instance
/// matters more than passing it around.
pub struct ShapeClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl ShapeClient {
    /// Build a client pointed at **Gate's** base URL.
    ///
    /// Never Electric's own URL: ADR-009 restricts direct Electric access to an
    /// operator-loopback diagnostic and never a client path, so pointing this
    /// at Electric bypasses the authorization boundary entirely.
    ///
    /// `token` is the bearer credential. A browser-side caller that lets Gate
    /// mint the downstream JWT from a session cookie passes `None` and relies
    /// on cookie propagation; a service caller passes `Some`.
    ///
    /// # Errors
    ///
    /// [`SdkError::InvalidEndpoint`] if the HTTP client cannot be constructed
    /// (TLS backend unavailable, or an invalid timeout).
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Result<Self, SdkError> {
        Self::with_timeout(base_url, token, DEFAULT_TIMEOUT)
    }

    /// As [`ShapeClient::new`], with an explicit per-request timeout.
    ///
    /// Raise it for `live=true` long-polls, which hold the connection open
    /// until the server has something to say.
    ///
    /// # Errors
    ///
    /// [`SdkError::InvalidEndpoint`] if the HTTP client cannot be constructed.
    pub fn with_timeout(
        base_url: impl Into<String>,
        token: Option<String>,
        timeout: Duration,
    ) -> Result<Self, SdkError> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| SdkError::InvalidEndpoint(error.to_string()))?;
        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token,
        })
    }

    /// Fetch one frame of one shape.
    ///
    /// Pass `cursor` to continue a stream, or `None` to start cold. Because
    /// [`ShapeCursor`] holds both halves, this only ever sends both or neither
    /// — the half-cursor the facade rejects cannot be expressed.
    ///
    /// # Errors
    ///
    /// - [`SdkError::ShapeGrantExpired`] (`401`) — recoverable; re-establish
    ///   the session.
    /// - [`SdkError::ShapeForbidden`] (`403`) — refused, or the handle belongs
    ///   to another grant.
    /// - [`SdkError::UnknownShape`] (`404`) — catalog miss, **or** a gateway
    ///   built without `--features shape-facade`.
    /// - [`SdkError::InvalidShapeRequest`] (`400`) — disallowed narrowing
    ///   parameter.
    /// - [`SdkError::ShapeUpstream`] (`5xx`, transport failure, or an
    ///   unparseable body) — retryable with backoff.
    #[tracing::instrument(
        name = "sdk::shape::fetch",
        skip(self, cursor, options),
        fields(shape = %shape, resuming = cursor.is_some())
    )]
    pub async fn fetch_shape(
        &self,
        shape: &str,
        cursor: Option<&ShapeCursor>,
        options: &ShapeRequestOptions,
    ) -> Result<ShapeFrame, SdkError> {
        let response = self.send(shape, cursor, options).await?;
        let status = response.status().as_u16();

        // Consumes the response on a refusal — a `400` body carries the
        // facade's explanation — and hands it back otherwise.
        let response = Self::check_refusals(shape, status, response).await?;

        let must_refetch = status == 409
            || header_str(response.headers(), "electric-must-refetch") == Some("true");
        let next_cursor = read_cursor(response.headers());
        let up_to_date = response.headers().contains_key("electric-up-to-date");

        if must_refetch {
            // No cursor: the caller must restart cold, and handing one back
            // would invite a retry that cannot succeed.
            return Ok(ShapeFrame {
                messages: Vec::new(),
                cursor: None,
                up_to_date: false,
                must_refetch: true,
                not_modified: false,
            });
        }

        // `304` carries no body and must report `not_modified`, which the
        // fall-through below cannot express.
        //
        // `204` is deliberately NOT special-cased. The facade's metadata-only
        // reply carries no body, so the fall-through reads `""`, and
        // `parse_messages` returns an empty vec — field-for-field identical to
        // what a dedicated branch would produce. A branch here would be dead
        // code that no test could distinguish from its absence, which is worse
        // than none: it invites a test that appears to guard it and does not.
        if status == 304 {
            return Ok(ShapeFrame {
                messages: Vec::new(),
                cursor: next_cursor,
                up_to_date,
                must_refetch: false,
                not_modified: status == 304,
            });
        }

        let body = response
            .text()
            .await
            .map_err(|error| SdkError::ShapeUpstream(error.without_url().to_string()))?;
        let messages = parse_messages(&body)?;

        tracing::debug!(rows = messages.len(), "shape frame parsed");

        Ok(ShapeFrame {
            messages,
            cursor: next_cursor,
            up_to_date,
            must_refetch: false,
            not_modified: false,
        })
    }

    /// Stream one shape's body frame-by-frame, rather than buffering it.
    ///
    /// For `live=true` long-polls, where the server holds the connection open
    /// and emits frames as they occur. Not available on `wasm32`: `reqwest`'s
    /// browser backend has no `bytes_stream`, so the buffered
    /// [`ShapeClient::fetch_shape`] is the only path there.
    ///
    /// The caller receives raw byte chunks in upstream order and is responsible
    /// for framing them; this deliberately does not re-implement Electric's
    /// chunk boundaries.
    ///
    /// # Errors
    ///
    /// The same refusals as [`ShapeClient::fetch_shape`], raised before the
    /// first chunk. Per-chunk transport failures surface as
    /// [`SdkError::ShapeUpstream`] within the stream.
    #[cfg(not(target_arch = "wasm32"))]
    #[tracing::instrument(
        name = "sdk::shape::stream",
        skip(self, cursor, options),
        fields(shape = %shape, resuming = cursor.is_some())
    )]
    pub async fn fetch_shape_streaming(
        &self,
        shape: &str,
        cursor: Option<&ShapeCursor>,
        options: &ShapeRequestOptions,
    ) -> Result<
        (
            ShapeStreamHead,
            impl futures_util::Stream<Item = Result<bytes::Bytes, SdkError>> + use<>,
        ),
        SdkError,
    > {
        use futures_util::StreamExt as _;

        let response = self.send(shape, cursor, options).await?;
        let status = response.status().as_u16();

        let response = Self::check_refusals(shape, status, response).await?;

        let head = ShapeStreamHead {
            cursor: read_cursor(response.headers()),
            up_to_date: response.headers().contains_key("electric-up-to-date"),
            must_refetch: status == 409
                || header_str(response.headers(), "electric-must-refetch") == Some("true"),
            not_modified: status == 304,
        };

        let body = response.bytes_stream().map(|chunk| {
            // `without_url` strips the URL, which carries the shape id and any
            // echoed cursor, out of the error text before it can reach a log.
            chunk.map_err(|error| SdkError::ShapeUpstream(error.without_url().to_string()))
        });

        Ok((head, body))
    }

    /// Issue the request. Shared by the buffered and streaming paths so the
    /// request contract is described in exactly one place.
    async fn send(
        &self,
        shape: &str,
        cursor: Option<&ShapeCursor>,
        options: &ShapeRequestOptions,
    ) -> Result<reqwest::Response, SdkError> {
        // Only protocol echoes accompany the shape id.
        let mut query: Vec<(&str, &str)> = vec![("shape", shape)];
        if let Some(cursor) = cursor {
            query.push(("handle", &cursor.handle));
            query.push(("offset", &cursor.offset));
        }
        if options.live {
            query.push(("live", "true"));
        }
        if let Some(cursor_param) = options.cursor.as_deref() {
            query.push(("cursor", cursor_param));
        }

        let mut request = self
            .http
            .get(format!("{}/v1/shape", self.base_url))
            .query(&query)
            .header(reqwest::header::ACCEPT, "application/json");

        if let Some(token) = self.token.as_deref() {
            request = request.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(etag) = options.if_none_match.as_deref() {
            request = request.header(reqwest::header::IF_NONE_MATCH, etag);
        }

        request
            .send()
            .await
            .map_err(|error| SdkError::ShapeUpstream(error.without_url().to_string()))
    }

    /// Map the facade's refusal statuses onto distinct errors.
    ///
    /// Each is a different server decision. Collapsing them into one error is
    /// what turns a failed sync into a guessing game.
    /// Takes the response by value on the refusal path because a `400` body
    /// carries the facade's explanation, and reading it consumes the response.
    /// Returns the response untouched when the status is one a caller can use.
    async fn check_refusals(
        shape: &str,
        status: u16,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, SdkError> {
        match status {
            401 => Err(SdkError::ShapeGrantExpired),
            403 => Err(SdkError::ShapeForbidden),
            404 => Err(SdkError::UnknownShape(shape.to_owned())),
            400 => {
                // The facade's message names the offending key class without
                // disclosing values, so it is safe to carry through.
                let detail = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "no detail".to_owned());
                Err(SdkError::InvalidShapeRequest(detail))
            }
            // 200/204/304/409 are all meaningful; everything else is upstream.
            200 | 204 | 304 | 409 => Ok(response),
            other => Err(SdkError::ShapeUpstream(format!(
                "unexpected status {other}"
            ))),
        }
    }
}

/// Response metadata available before a streamed body is consumed.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ShapeStreamHead {
    /// The cursor to send next.
    pub cursor: Option<ShapeCursor>,
    /// `electric-up-to-date` was present.
    pub up_to_date: bool,
    /// `409` or `electric-must-refetch`: restart cold.
    pub must_refetch: bool,
    /// `304`: nothing changed.
    pub not_modified: bool,
}

/// Read a header as `&str`, ignoring values that are not valid UTF-8.
fn header_str<'headers>(
    headers: &'headers reqwest::header::HeaderMap,
    name: &str,
) -> Option<&'headers str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Take the next cursor, but only when both halves are present.
///
/// A half cursor is exactly what the facade rejects with `400`, so one must
/// never be handed back to a caller who would echo it.
fn read_cursor(headers: &reqwest::header::HeaderMap) -> Option<ShapeCursor> {
    match (
        header_str(headers, "electric-handle"),
        header_str(headers, "electric-offset"),
    ) {
        (Some(handle), Some(offset)) => Some(ShapeCursor {
            handle: handle.to_owned(),
            offset: offset.to_owned(),
        }),
        _ => None,
    }
}

/// Parse the body into row-bearing frames, dropping control frames.
fn parse_messages(body: &str) -> Result<Vec<ShapeMessage>, SdkError> {
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    let frames: Vec<ShapeMessage> = serde_json::from_str(body)
        .map_err(|error| SdkError::ShapeUpstream(format!("malformed shape body: {error}")))?;
    Ok(frames
        .into_iter()
        .filter(ShapeMessage::carries_row)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_frames_are_dropped_and_deletes_are_kept() {
        let body = r#"[
            {"headers":{"control":"up-to-date"}},
            {"headers":{"operation":"insert"},"key":"a","value":{"id":"1"}},
            {"headers":{"operation":"delete"},"key":"b","value":{"id":"2"}}
        ]"#;
        let messages = parse_messages(body).expect("parses");
        // A replica that rebuilds may ignore deletes; a general client must not
        // make that choice for the caller.
        assert_eq!(messages.len(), 2);
        assert!(messages[1].is_delete());
    }

    #[test]
    fn an_empty_body_is_not_an_error() {
        assert!(parse_messages("").expect("empty parses").is_empty());
        assert!(parse_messages("  ").expect("blank parses").is_empty());
    }

    #[test]
    fn a_malformed_body_is_upstream_not_a_refusal() {
        let error = parse_messages("{not json").expect_err("must reject");
        assert!(matches!(error, SdkError::ShapeUpstream(_)));
    }
}
