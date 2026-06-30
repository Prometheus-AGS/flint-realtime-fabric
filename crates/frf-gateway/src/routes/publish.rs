use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use frf_app::{AppError, PublishRequest};
use frf_domain::EventEnvelope;
use frf_ports::{
    ActionPolicyProvider, AgentEventBus, AuthzProvider, IdentityVerifier, LogBroker, MediaSignaler,
};
use serde::Serialize;
use tracing::instrument;

use crate::AppStateArc;

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
/// 1. Verifies the JWT.
/// 2. Calls Cedar `is_permitted("Publish", channel_id)` — returns 403 on deny.
/// 3. Forwards the envelope to the `PublishUseCase`.
///
/// Returns the assigned offset on success.
///
/// # Errors
///
/// Returns 401 if the `Authorization: Bearer <token>` header is missing or invalid.
/// Returns 403 if the Cedar policy denies the action.
/// Returns 500 if the broker rejects the envelope.
#[instrument(name = "http::publish", skip(state, headers, envelope))]
pub async fn publish_event<L, A, I, M, B, P>(
    State(state): State<AppStateArc<L, A, I, M, B, P>>,
    headers: HeaderMap,
    Json(envelope): Json<EventEnvelope>,
) -> impl IntoResponse
where
    L: LogBroker + Send + Sync + 'static,
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
    M: MediaSignaler + 'static,
    B: AgentEventBus + 'static,
    P: ActionPolicyProvider + 'static,
{
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // JWT verification at this boundary — never trust claims downstream.
    let claims = match state.identity.verify(&token).await {
        Ok(c) => c,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // Cedar action-policy check: Cedar governs mutation ops (publish).
    // Keto governs visibility — do not conflate.
    let resource = envelope.channel.id.to_string();
    match state
        .action_policy
        .is_permitted(&claims.tenant_id, "Publish", &resource)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!(
                tenant_id = %claims.tenant_id,
                resource = %resource,
                "Cedar denied Publish action",
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "Cedar policy evaluation error");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let req = PublishRequest {
        envelope,
        bearer_token: token,
    };

    match state.publish_usecase.execute(req).await {
        Ok(offset) => Json(PublishResponse { offset: offset.0 }).into_response(),
        Err(e) => {
            let status = match &e {
                AppError::Unauthorized(_) | AppError::Identity(_) => StatusCode::UNAUTHORIZED,
                AppError::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string()).into_response()
        }
    }
}
