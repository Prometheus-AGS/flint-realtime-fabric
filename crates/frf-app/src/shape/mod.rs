//! Authorized Electric shape use case.
//!
//! This module owns the application policy that combines a verified ASO grant,
//! live authorization, a server-defined shape catalog, and the outbound
//! [`ShapeFacade`]. Electric handles are bound to the exact verified grant that
//! first received them, so an opaque cursor cannot be replayed across subjects,
//! practices, sessions, or projection revisions.

mod continuation;
mod lease;
mod policy;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use frf_domain::{SessionId, TenantId};
use frf_ports::{
    AuthorizedShapeRequest, AuthzProvider, Cursor, PortError, RelationTuple, ShapeFacade,
    ShapeLease, ShapeRequest, ShapeResponse, VerifiedClaims,
};
use tokio::time::Instant;

pub use policy::{ShapeCatalog, ShapePolicy};

/// Maximum lifetime of one protected HTTP response before the client must reconnect.
pub const MAX_ACTIVE_LEASE: Duration = Duration::from_millis(1_750);

/// Paired wall and monotonic request timestamps captured by the trusted interface.
///
/// The monotonic timestamp must be sampled before the wall timestamp. Anchoring grant expiry to
/// that earlier instant ensures scheduling delay cannot extend a response beyond the JWT expiry.
#[derive(Debug, Clone, Copy)]
pub struct ShapeRequestTime {
    epoch: Duration,
    monotonic: Instant,
}

impl ShapeRequestTime {
    /// Pair a precise Unix timestamp with the monotonic instant sampled immediately before it.
    #[must_use]
    pub const fn new(epoch: Duration, monotonic: Instant) -> Self {
        Self { epoch, monotonic }
    }
}

/// Failures exposed by the authorized shape application boundary.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ShapeUseCaseError {
    /// The verified claims do not carry the required ASO replica grant.
    #[error("replica grant is not authorized")]
    Unauthorized,
    /// The verified grant has reached its expiry.
    #[error("replica grant has expired")]
    GrantExpired,
    /// The requested shape is not declared by server policy.
    #[error("shape is not declared")]
    UnknownShape,
    /// Client protocol or narrowing input is malformed.
    #[error("invalid shape request: {0}")]
    InvalidRequest(String),
    /// The supplied handle was issued under a different or expired grant.
    #[error("shape handle is not valid for this grant")]
    HandleMismatch,
    /// The authorization provider denied or could not evaluate the request.
    #[error("shape authorization denied")]
    Forbidden,
    /// The outbound Electric exchange failed or returned an invalid protocol response.
    #[error("shape upstream failed: {0}")]
    Upstream(PortError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HandleBinding {
    tenant_id: TenantId,
    subject: String,
    originating_session_id: Option<SessionId>,
    authorization_revision: Option<String>,
    projection_revision: Option<u32>,
    projection_ids: Vec<String>,
    shape: String,
    expires_at: u64,
}

impl HandleBinding {
    fn from_claims(claims: &VerifiedClaims, shape: &str) -> Self {
        let mut projection_ids = claims.projection_ids.clone();
        projection_ids.sort_unstable();
        Self {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            originating_session_id: claims.originating_session_id,
            authorization_revision: claims.authorization_revision.clone(),
            projection_revision: claims.projection_revision,
            projection_ids,
            shape: shape.to_owned(),
            expires_at: claims.expires_at,
        }
    }

    fn matches(&self, claims: &VerifiedClaims, shape: &str, now_epoch: Duration) -> bool {
        let mut projection_ids = claims.projection_ids.clone();
        projection_ids.sort_unstable();
        Duration::from_secs(self.expires_at) > now_epoch
            && self.tenant_id == claims.tenant_id
            && self.subject == claims.subject
            && self.originating_session_id == claims.originating_session_id
            && self.authorization_revision == claims.authorization_revision
            && self.projection_revision == claims.projection_revision
            && self.projection_ids == projection_ids
            && self.shape == shape
    }

    fn same_authority(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.subject == other.subject
            && self.originating_session_id == other.originating_session_id
            && self.authorization_revision == other.authorization_revision
            && self.projection_revision == other.projection_revision
            && self.projection_ids == other.projection_ids
            && self.shape == other.shape
    }
}

type HandleBindings = HashMap<String, Vec<HandleBinding>>;

/// Executes one authorized Electric shape request.
pub struct ShapeUseCase<A> {
    catalog: ShapeCatalog,
    authz: Arc<A>,
    facade: Arc<dyn ShapeFacade>,
    handle_bindings: Arc<Mutex<HandleBindings>>,
    active_lease: Duration,
}

impl<A> ShapeUseCase<A>
where
    A: AuthzProvider,
{
    /// Build the use case from server-owned policy and an outbound facade.
    #[must_use]
    pub fn new(catalog: ShapeCatalog, authz: Arc<A>, facade: Arc<dyn ShapeFacade>) -> Self {
        Self {
            catalog,
            authz,
            facade,
            handle_bindings: Arc::new(Mutex::new(HashMap::new())),
            active_lease: MAX_ACTIVE_LEASE,
        }
    }

    /// Tighten the active response lease for a deployment or deterministic test.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeUseCaseError::InvalidRequest`] for a zero duration or a value above the
    /// recorded RA06 component budget.
    pub fn with_active_lease(mut self, active_lease: Duration) -> Result<Self, ShapeUseCaseError> {
        if active_lease.is_zero() || active_lease > MAX_ACTIVE_LEASE {
            return Err(ShapeUseCaseError::InvalidRequest(
                "active response lease must be between 1 ms and 1750 ms".to_owned(),
            ));
        }
        self.active_lease = active_lease;
        Ok(self)
    }

    /// Verify, authorize, constrain, and fetch one shape response.
    ///
    /// `request_time` pairs the precise Unix time with an earlier monotonic sample supplied by the
    /// trusted interface layer. Keeping both explicit prevents scheduling delay from extending the
    /// grant and makes expiry behavior deterministic in application tests.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeUseCaseError`] when the grant, handle, policy,
    /// authorization decision, or upstream protocol response is invalid.
    #[tracing::instrument(
        name = "app::shape",
        skip(self, claims, request),
        fields(shape = %request.shape)
    )]
    pub async fn execute(
        &self,
        claims: &VerifiedClaims,
        request: ShapeRequest,
        request_time: ShapeRequestTime,
    ) -> Result<ShapeResponse, ShapeUseCaseError> {
        let scope = claims
            .aso_replica_scope()
            .map_err(|_| ShapeUseCaseError::Unauthorized)?;
        let grant_expiry = Duration::from_secs(claims.expires_at);
        let Some(grant_lifetime) = grant_expiry.checked_sub(request_time.epoch) else {
            return Err(ShapeUseCaseError::GrantExpired);
        };
        if grant_lifetime.is_zero() {
            return Err(ShapeUseCaseError::GrantExpired);
        }
        if !claims
            .projection_ids
            .iter()
            .any(|projection| projection == &request.shape)
        {
            return Err(ShapeUseCaseError::Forbidden);
        }

        self.validate_handle(claims, &request, request_time.epoch)?;

        let policy = self.catalog.get(&request.shape)?;
        let tuple = RelationTuple {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            relation: policy.relation.clone(),
            object: format!("{}:{scope}", policy.object_namespace),
        };
        let Some(grant_deadline) = request_time.monotonic.checked_add(grant_lifetime) else {
            return Err(ShapeUseCaseError::Unauthorized);
        };
        if grant_deadline <= Instant::now() {
            return Err(ShapeUseCaseError::GrantExpired);
        }
        let response_deadline = (request_time.monotonic + self.active_lease).min(grant_deadline);
        let lease: Arc<dyn ShapeLease> = Arc::new(lease::GrantLease::new(
            Arc::clone(&self.authz),
            tuple,
            grant_deadline,
        ));
        let where_clause = policy.compose_where(&scope, &request.params)?;
        let authorized = AuthorizedShapeRequest {
            table: policy.table.clone(),
            columns: policy.columns.clone(),
            where_clause,
            cursor: request.cursor.clone(),
            protocol: request.protocol.clone(),
        };
        let protected_exchange = async {
            lease::revalidate_once(&lease)
                .await
                .map_err(|_| ShapeUseCaseError::Forbidden)?;
            let mut response = self
                .facade
                .fetch(&authorized)
                .await
                .map_err(ShapeUseCaseError::Upstream)?;
            lease::revalidate_once(&lease)
                .await
                .map_err(|_| ShapeUseCaseError::Forbidden)?;
            let response_handle = validated_response_handle(&response)?;
            let previous_handle = match &request.cursor {
                Cursor::Resume { handle, .. } => Some(handle.clone()),
                Cursor::Initial | Cursor::Now => None,
            };
            let continuation = continuation::ContinuationSettlement::new(
                Arc::clone(&self.handle_bindings),
                previous_handle,
                response_handle,
                HandleBinding::from_claims(claims, &request.shape),
                request_time.epoch.as_secs(),
            );
            response.body = lease::protect(
                response.body,
                Arc::clone(&lease),
                response_deadline,
                move |outcome| continuation.settle(outcome),
            );
            Ok(response)
        };
        let response = match tokio::time::timeout_at(response_deadline, protected_exchange).await {
            Ok(result) => result?,
            Err(_) => return Ok(ShapeResponse::new(204, Vec::new(), Vec::new())),
        };

        Ok(response)
    }

    fn validate_handle(
        &self,
        claims: &VerifiedClaims,
        request: &ShapeRequest,
        now_epoch: Duration,
    ) -> Result<(), ShapeUseCaseError> {
        let Cursor::Resume { handle, .. } = &request.cursor else {
            return Ok(());
        };
        let bindings = continuation::lock_bindings(&self.handle_bindings);
        let Some(handle_bindings) = bindings.get(handle) else {
            return Err(ShapeUseCaseError::HandleMismatch);
        };
        if handle_bindings
            .iter()
            .any(|binding| binding.matches(claims, &request.shape, now_epoch))
        {
            Ok(())
        } else {
            Err(ShapeUseCaseError::HandleMismatch)
        }
    }
}

fn validated_response_handle(
    response: &ShapeResponse,
) -> Result<Option<String>, ShapeUseCaseError> {
    let handle = response
        .header("electric-handle")
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|_| invalid_upstream("electric-handle is not UTF-8"))?
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    if matches!(response.status, 200 | 409) && handle.is_none() {
        return Err(invalid_upstream(
            "Electric response omitted electric-handle",
        ));
    }
    Ok(handle)
}

fn invalid_upstream(message: &str) -> ShapeUseCaseError {
    ShapeUseCaseError::Upstream(PortError::Transport(message.to_owned()))
}

#[cfg(test)]
mod tests;
