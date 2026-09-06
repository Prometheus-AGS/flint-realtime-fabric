//! `GET /v1/shape` — the authorized relational replication facade (ADR-009).
//!
//! This is the composition point where (shape × authz) meet: the adapter is a pure transport
//! and the resolver holds the policy, but only this route binds them to a *verified* subject
//! and a *server-resolved* practice scope. Nothing here trusts the request body for identity
//! or scope.
//!
//! Feature-gated behind `shape-facade`, off by default. ADR-009 keeps this lane disabled
//! until its verification criteria are proved, and the live Electric exchange has not been
//! verified against a running server.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use frf_ports::{Cursor, ShapeRequest};
use std::collections::HashMap;

use crate::AppStateArc;

/// Client-facing query parameters.
///
/// `handle` and `offset` are Electric's own values echoed back; everything else is treated as
/// narrowing input and validated against the shape's allow-list. Note what is absent: no
/// table, no column list, no filter expression. Those are server-derived, so a client has no
/// vocabulary in which to widen its request.
fn parse_cursor(params: &HashMap<String, String>) -> Cursor {
    match (params.get("handle"), params.get("offset")) {
        (Some(handle), Some(offset)) => Cursor::Resume {
            handle: handle.clone(),
            offset: offset.clone(),
        },
        _ => Cursor::Initial,
    }
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
/// The practice scope comes from the verified claims' `scope`, never from the query string:
/// a client that supplies its own scope is not trusted, and one whose token carries no scope
/// is refused rather than defaulted.
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
    A: Send + Sync + 'static,
    I: frf_ports::IdentityVerifier + Send + Sync + 'static,
    M: Send + Sync + 'static,
    B: Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    let Some(facade) = state.shape_facade.as_ref() else {
        // The lane is compiled in but not configured — not an error, just disabled.
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(resolver) = state.shape_resolver.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    // JWT verification at this boundary — never trust claims downstream.
    let Ok(claims) = state.identity.verify(&token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // The practice scope is authoritative server state. A token without one cannot be
    // scoped, so it is refused rather than silently granted a default practice.
    let Some(scope) = claims.scope.as_deref() else {
        tracing::warn!("shape request with no scope claim — denying");
        return StatusCode::FORBIDDEN.into_response();
    };

    let Some(shape) = params.get("shape").cloned() else {
        return (StatusCode::BAD_REQUEST, "missing shape").into_response();
    };

    // Everything except the protocol cursor and the shape id is narrowing input.
    let narrowing: Vec<(String, String)> = params
        .iter()
        .filter(|(k, _)| !matches!(k.as_str(), "shape" | "handle" | "offset"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let request = ShapeRequest {
        tenant_id: claims.tenant_id,
        subject: claims.subject.clone(),
        shape,
        params: narrowing,
        cursor: parse_cursor(&params),
    };

    // Resolve + authorize. This runs on every request *and every continuation* — a resumed
    // cursor carries no standing permission (ADR-009).
    let authorized = match resolver.resolve(&request, scope).await {
        Ok(a) => a,
        Err(frf_shape_electric::ShapeError::Unauthorized) => {
            return StatusCode::FORBIDDEN.into_response();
        }
        Err(frf_shape_electric::ShapeError::UnknownShape(_)) => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            // Parameter and policy failures are the caller's to fix; the message names keys
            // and classes only, never values.
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };

    match facade.fetch(&authorized).await {
        Ok(chunk) => {
            let mut headers = HeaderMap::new();
            insert_header(&mut headers, "electric-handle", &chunk.handle);
            insert_header(&mut headers, "electric-offset", &chunk.next_offset);
            if chunk.snapshot_complete {
                insert_header(&mut headers, "electric-up-to-date", "");
            }
            if chunk.must_refetch {
                // Preserve Electric's refetch contract so the client rebuilds the affected
                // replica generation instead of merging into stale rows.
                insert_header(&mut headers, "electric-must-refetch", "true");
                return (StatusCode::CONFLICT, headers, chunk.body).into_response();
            }
            (StatusCode::OK, headers, chunk.body).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "shape fetch failed");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

/// Insert a header, dropping it if the value is not a valid header value.
fn insert_header(headers: &mut HeaderMap, name: &'static str, value: &str) {
    if let Ok(v) = axum::http::HeaderValue::from_str(value) {
        headers.insert(name, v);
    }
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
            Cursor::Resume {
                handle: "h-1".to_owned(),
                offset: "17".to_owned()
            }
        );
    }

    #[test]
    fn a_partial_cursor_falls_back_to_initial_rather_than_guessing() {
        let params = HashMap::from([("handle".to_owned(), "h-1".to_owned())]);
        assert_eq!(parse_cursor(&params), Cursor::Initial);
    }

    #[test]
    fn no_cursor_means_a_cold_start() {
        assert_eq!(parse_cursor(&HashMap::new()), Cursor::Initial);
    }
}
