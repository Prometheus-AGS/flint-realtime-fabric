//! `GET /v1/shape` — the authorized relational replication facade (ADR-009).
//!
//! This interface verifies identity and delegates authorization, server policy, handle
//! binding, and the Electric exchange to the application use case. Nothing here trusts the
//! request for identity, practice scope, table, columns, or row predicates.
//!
//! Feature-gated behind `shape-facade`, off by default. A bounded local Gate/FRF/Electric
//! exchange, expired-handle denial, client-segment isolation and a persisted gate-summary
//! transition passed on 2026-09-08; ADR-009 keeps the lane uncertified until measured
//! revocation and deployment-specific topology criteria are proved.

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use frf_app::{ShapeRequestTime, ShapeUseCaseError};
use frf_ports::{Cursor, PortError, ShapeBodyStream, ShapeProtocol, ShapeRequest, ShapeResponse};
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::Stream;

use crate::AppStateArc;

/// Client-facing query parameters.
///
/// Parse Electric's initial, current, and continuation cursor forms without guessing when a
/// client supplies only half of a continuation.
fn parse_cursor(params: &HashMap<String, String>) -> Result<Cursor, ()> {
    match (params.get("handle"), params.get("offset")) {
        (None, None) => Ok(Cursor::Initial),
        (None, Some(offset)) if offset == "-1" => Ok(Cursor::Initial),
        (None, Some(offset)) if offset == "now" => Ok(Cursor::Now),
        (Some(handle), Some(offset)) => Ok(Cursor::Resume {
            handle: handle.clone(),
            offset: offset.clone(),
        }),
        _ => Err(()),
    }
}

fn protocol(headers: &HeaderMap, params: &HashMap<String, String>) -> Result<ShapeProtocol, ()> {
    let if_none_match = headers
        .get(header::IF_NONE_MATCH)
        .map(HeaderValue::to_str)
        .transpose()
        .map_err(|_| ())?
        .map(str::to_owned);
    Ok(ShapeProtocol {
        live: params.get("live").cloned(),
        cursor: params.get("cursor").cloned(),
        if_none_match,
    })
}

/// Extract the bearer token from an `Authorization` header.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

/// Serve one shape chunk for a verified, authorized subject.
///
/// The practice scope comes from the verified tenant claim after the dedicated
/// ASO scope, revision, projection allowlist, and session linkage are checked.
// `Query` deserializes into a concrete `HashMap<String, String>`; the hasher cannot be
// generalized at an axum extractor boundary, so the pedantic lint does not apply here.
#[allow(clippy::implicit_hasher)]
#[tracing::instrument(name = "route::shape", skip(state, headers, params))]
pub async fn get_shape<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response
where
    L: Send + Sync + 'static,
    A: frf_ports::AuthzProvider + Send + Sync + 'static,
    I: frf_ports::IdentityVerifier + Send + Sync + 'static,
    M: Send + Sync + 'static,
    B: Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    let Some(usecase) = state.shape_usecase.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    // JWT verification at this boundary — never trust claims downstream.
    let Ok(claims) = state.identity.verify(&token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("absent")
        .to_owned();

    let Some(shape) = params.get("shape").cloned() else {
        return (StatusCode::BAD_REQUEST, "missing shape").into_response();
    };

    let Ok(cursor) = parse_cursor(&params) else {
        return (StatusCode::BAD_REQUEST, "invalid shape cursor").into_response();
    };
    let Ok(protocol) = protocol(&headers, &params) else {
        return (StatusCode::BAD_REQUEST, "invalid conditional request").into_response();
    };

    // Everything except the protocol cursor and the shape id is narrowing input.
    let narrowing: Vec<(String, String)> = params
        .iter()
        .filter(|(k, _)| {
            !matches!(
                k.as_str(),
                "shape" | "handle" | "offset" | "live" | "cursor"
            )
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let request = ShapeRequest {
        tenant_id: claims.tenant_id,
        subject: claims.subject.clone(),
        shape,
        params: narrowing,
        cursor,
        protocol,
    };

    let request_started_at = tokio::time::Instant::now();
    let Ok(request_epoch) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        tracing::error!("system clock is before the Unix epoch");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let request_time = ShapeRequestTime::new(request_epoch, request_started_at);

    match usecase.execute(&claims, request, request_time).await {
        Ok(mut response) => {
            response.body = ObservedShapeBody::wrap(response.body, request_id, claims.expires_at);
            if let Ok(response) = electric_response(response) {
                response
            } else {
                tracing::warn!("shape upstream returned invalid HTTP metadata");
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
        Err(ShapeUseCaseError::Unauthorized | ShapeUseCaseError::HandleMismatch) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(ShapeUseCaseError::GrantExpired) => StatusCode::UNAUTHORIZED.into_response(),
        Err(ShapeUseCaseError::UnknownShape) => StatusCode::NOT_FOUND.into_response(),
        Err(ShapeUseCaseError::InvalidRequest(message)) => {
            (StatusCode::BAD_REQUEST, message).into_response()
        }
        Err(ShapeUseCaseError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ShapeUseCaseError::Upstream(e)) => {
            tracing::warn!(error = %e, "shape fetch failed");
            StatusCode::BAD_GATEWAY.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "unrecognized shape use case failure");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Records the lifecycle of the actual protected FRF body without recording token or row data.
///
/// The elapsed values come from one monotonic clock owned by this response. The paired wall-clock
/// value lets a mounted verifier compare the verified JWT `exp` with the interval during which the
/// protected producer was active. A terminal event is emitted at the same boundary that returns the
/// final frame, cancellation error, normal completion, or consumer drop to Axum.
struct ObservedShapeBody {
    inner: ShapeBodyStream,
    request_id: String,
    grant_expiry_epoch_seconds: u64,
    origin: tokio::time::Instant,
    frames: u64,
    last_frame_elapsed_ns: Option<u64>,
    settled: bool,
}

impl ObservedShapeBody {
    fn wrap(
        inner: ShapeBodyStream,
        request_id: String,
        grant_expiry_epoch_seconds: u64,
    ) -> ShapeBodyStream {
        tracing::info!(
            target: "frf_shape_stream",
            event = "opened",
            request_id = %request_id,
            grant_expiry_epoch_seconds,
            observed_epoch_ns = observed_epoch_ns(),
            elapsed_monotonic_ns = 0_u64,
            frames = 0_u64,
            "protected shape stream lifecycle"
        );
        Box::pin(Self {
            inner,
            request_id,
            grant_expiry_epoch_seconds,
            origin: tokio::time::Instant::now(),
            frames: 0,
            last_frame_elapsed_ns: None,
            settled: false,
        })
    }

    fn elapsed_ns(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn record_terminal(&mut self, event: &'static str) {
        if self.settled {
            return;
        }
        self.settled = true;
        tracing::info!(
            target: "frf_shape_stream",
            event,
            request_id = %self.request_id,
            grant_expiry_epoch_seconds = self.grant_expiry_epoch_seconds,
            observed_epoch_ns = observed_epoch_ns(),
            elapsed_monotonic_ns = self.elapsed_ns(),
            frames = self.frames,
            last_frame_elapsed_ns = self.last_frame_elapsed_ns,
            "protected shape stream lifecycle"
        );
    }
}

impl Stream for ObservedShapeBody {
    type Item = Result<bytes::Bytes, PortError>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        match this.inner.as_mut().poll_next(context) {
            Poll::Ready(Some(Ok(frame))) => {
                this.frames = this.frames.saturating_add(1);
                let elapsed_monotonic_ns = this.elapsed_ns();
                this.last_frame_elapsed_ns = Some(elapsed_monotonic_ns);
                tracing::info!(
                    target: "frf_shape_stream",
                    event = "frame",
                    request_id = %this.request_id,
                    grant_expiry_epoch_seconds = this.grant_expiry_epoch_seconds,
                    observed_epoch_ns = observed_epoch_ns(),
                    elapsed_monotonic_ns,
                    frames = this.frames,
                    "protected shape stream lifecycle"
                );
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Ready(Some(Err(error))) => {
                this.record_terminal("cancelled");
                Poll::Ready(Some(Err(error)))
            }
            Poll::Ready(None) => {
                this.record_terminal("completed");
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for ObservedShapeBody {
    fn drop(&mut self) {
        self.record_terminal("dropped");
    }
}

fn observed_epoch_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
        .unwrap_or(0)
}

fn electric_response(response: ShapeResponse) -> Result<Response, ()> {
    let status = StatusCode::from_u16(response.status).map_err(|_| ())?;
    let mut headers = HeaderMap::new();
    for preserved in response.headers {
        let name = HeaderName::from_bytes(preserved.name.as_bytes()).map_err(|_| ())?;
        if name == header::CACHE_CONTROL {
            continue;
        }
        let value = HeaderValue::from_bytes(&preserved.value).map_err(|_| ())?;
        headers.append(name, value);
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok((status, headers, Body::from_stream(response.body)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cursor_with_both_handle_and_offset_resumes() {
        let params = HashMap::from([
            ("handle".to_owned(), "h-1".to_owned()),
            ("offset".to_owned(), "17".to_owned()),
        ]);
        assert_eq!(
            parse_cursor(&params),
            Ok(Cursor::Resume {
                handle: "h-1".to_owned(),
                offset: "17".to_owned()
            })
        );
    }

    #[test]
    fn a_partial_cursor_is_rejected() {
        let params = HashMap::from([("handle".to_owned(), "h-1".to_owned())]);
        assert_eq!(parse_cursor(&params), Err(()));
    }

    #[test]
    fn no_cursor_means_a_cold_start() {
        assert_eq!(parse_cursor(&HashMap::new()), Ok(Cursor::Initial));
    }

    #[test]
    fn now_offset_starts_at_the_current_log_position() {
        let params = HashMap::from([("offset".to_owned(), "now".to_owned())]);
        assert_eq!(parse_cursor(&params), Ok(Cursor::Now));
    }

    #[test]
    fn downstream_shape_response_cannot_inherit_shared_cache_policy() {
        let response = electric_response(ShapeResponse::new(
            200,
            vec![
                frf_ports::ShapeHeader::new("cache-control", b"public, max-age=3600"),
                frf_ports::ShapeHeader::new("etag", b"shape:1"),
            ],
            b"[]".to_vec(),
        ))
        .expect("valid response");

        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        assert_eq!(
            response.headers().get(header::ETAG),
            Some(&HeaderValue::from_static("shape:1"))
        );
    }

    #[tokio::test]
    async fn electric_response_preserves_status_headers_and_frame_order() {
        let body: frf_ports::ShapeBodyStream = Box::pin(futures_util::stream::iter([
            Ok(bytes::Bytes::from_static(b"first")),
            Ok(bytes::Bytes::from_static(b"-second")),
            Ok(bytes::Bytes::from_static(b"-third")),
        ]));
        let response = electric_response(ShapeResponse::streamed(
            409,
            vec![
                frf_ports::ShapeHeader::new("electric-handle", b"replacement"),
                frf_ports::ShapeHeader::new("electric-must-refetch", b"true"),
                frf_ports::ShapeHeader::new("content-type", b"application/json"),
            ],
            body,
        ))
        .expect("valid Electric response");

        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            response.headers().get("electric-handle"),
            Some(&HeaderValue::from_static("replacement"))
        );
        assert_eq!(
            response.headers().get("electric-must-refetch"),
            Some(&HeaderValue::from_static("true"))
        );
        let body = axum::body::to_bytes(response.into_body(), 32)
            .await
            .expect("ordered body");
        assert_eq!(body, b"first-second-third".as_slice());
    }
}
