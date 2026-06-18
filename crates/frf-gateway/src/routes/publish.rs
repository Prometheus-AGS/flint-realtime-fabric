use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use frf_app::{AppError, PublishRequest};
use frf_domain::EventEnvelope;
use frf_ports::{AuthzProvider, IdentityVerifier, LogBroker};
use serde::Serialize;
use tracing::instrument;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub offset: u64,
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
}

/// Publish an event to the spine.
///
/// Verifies the JWT, then forwards the envelope to the `PublishUseCase`.
/// Returns the assigned offset on success.
///
/// # Errors
///
/// Returns 401 if the `Authorization: Bearer <token>` header is missing or invalid.
/// Returns 500 if the broker rejects the envelope.
#[instrument(name = "http::publish", skip(state, headers, envelope))]
pub async fn publish_event<L, A, I>(
    State(state): State<Arc<AppState<L, A, I>>>,
    headers: HeaderMap,
    Json(envelope): Json<EventEnvelope>,
) -> impl IntoResponse
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    let Some(token) = bearer_token(&headers) else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    let req = PublishRequest {
        envelope,
        bearer_token: token,
    };

    match state.publish_usecase.execute(req).await {
        Ok(offset) => Json(PublishResponse { offset: offset.0 }).into_response(),
        Err(e) => {
            let status = match &e {
                AppError::Unauthorized(_) | AppError::Identity(_) => {
                    axum::http::StatusCode::UNAUTHORIZED
                }
                AppError::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string()).into_response()
        }
    }
}
