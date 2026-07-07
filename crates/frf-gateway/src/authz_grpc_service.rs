use std::sync::Arc;

use frf_app::{AppError, AuthzRequest, AuthzUseCase};
use frf_domain::TenantId;
use frf_ports::{AuthzProvider, IdentityVerifier, RelationTuple};
use frf_proto::fv1::{
    self,
    authz_service_server::{AuthzService, AuthzServiceServer},
};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

/// gRPC service implementing `flint.v1.AuthzService`.
///
/// A thin transport over the `AuthzUseCase`, which verifies the caller's token and
/// enforces tenant-equality before delegating to the Keto-backed `AuthzProvider`. The
/// concrete adapter types (`A`, `I`) are wired in `main.rs`.
pub struct AuthzGrpcService<A, I> {
    use_case: Arc<AuthzUseCase<A, I>>,
}

impl<A, I> AuthzGrpcService<A, I>
where
    A: AuthzProvider,
    I: IdentityVerifier,
{
    #[must_use]
    pub fn new(use_case: Arc<AuthzUseCase<A, I>>) -> Self {
        Self { use_case }
    }

    #[must_use]
    pub fn into_server(self) -> AuthzServiceServer<Self> {
        AuthzServiceServer::new(self)
    }
}

fn app_error_to_status(err: AppError) -> Status {
    match err {
        AppError::Forbidden(msg) => Status::permission_denied(msg),
        AppError::Unauthorized(msg) => Status::unauthenticated(msg),
        AppError::Identity(e) => Status::unauthenticated(e.to_string()),
        AppError::Broker(e) => Status::internal(e.to_string()),
        _ => Status::internal("unexpected error"),
    }
}

fn extract_bearer<T>(request: &Request<T>) -> Result<String, Status> {
    request
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
        .ok_or_else(|| Status::unauthenticated("missing Bearer token"))
}

/// Convert a proto `RelationTuple` into the domain tuple, parsing the tenant UUID.
fn proto_tuple_to_domain(t: fv1::RelationTuple) -> Result<RelationTuple, Status> {
    let tenant_id = Uuid::parse_str(&t.tenant_id)
        .map(TenantId::from_uuid)
        .map_err(|_| {
            Status::invalid_argument(format!("invalid tenant_id UUID: {}", t.tenant_id))
        })?;
    Ok(RelationTuple {
        tenant_id,
        subject: t.subject,
        relation: t.relation,
        object: t.object,
    })
}

fn build_request<T>(
    request: &Request<T>,
    tuple: Option<fv1::RelationTuple>,
) -> Result<AuthzRequest, Status> {
    let bearer_token = extract_bearer(request)?;
    let proto_tuple = tuple.ok_or_else(|| Status::invalid_argument("missing tuple"))?;
    Ok(AuthzRequest {
        tuple: proto_tuple_to_domain(proto_tuple)?,
        bearer_token,
    })
}

#[tonic::async_trait]
impl<A, I> AuthzService for AuthzGrpcService<A, I>
where
    A: AuthzProvider + Send + Sync + 'static,
    I: IdentityVerifier + Send + Sync + 'static,
{
    #[instrument(name = "grpc::authz_check", skip(self, request))]
    async fn check(
        &self,
        request: Request<fv1::CheckRequest>,
    ) -> Result<Response<fv1::CheckResponse>, Status> {
        let tuple = request.get_ref().tuple.clone();
        let req = build_request(&request, tuple)?;
        let allowed = self
            .use_case
            .check(req)
            .await
            .map_err(app_error_to_status)?;
        Ok(Response::new(fv1::CheckResponse { allowed }))
    }

    #[instrument(name = "grpc::authz_write", skip(self, request))]
    async fn write_relation(
        &self,
        request: Request<fv1::WriteRelationRequest>,
    ) -> Result<Response<fv1::WriteRelationResponse>, Status> {
        let tuple = request.get_ref().tuple.clone();
        let req = build_request(&request, tuple)?;
        self.use_case
            .write(req)
            .await
            .map_err(app_error_to_status)?;
        Ok(Response::new(fv1::WriteRelationResponse {}))
    }

    #[instrument(name = "grpc::authz_delete", skip(self, request))]
    async fn delete_relation(
        &self,
        request: Request<fv1::DeleteRelationRequest>,
    ) -> Result<Response<fv1::DeleteRelationResponse>, Status> {
        let tuple = request.get_ref().tuple.clone();
        let req = build_request(&request, tuple)?;
        self.use_case
            .delete(req)
            .await
            .map_err(app_error_to_status)?;
        Ok(Response::new(fv1::DeleteRelationResponse {}))
    }
}
