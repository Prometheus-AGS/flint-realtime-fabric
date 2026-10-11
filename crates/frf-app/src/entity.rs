use std::sync::Arc;

use frf_domain::{EntityChange, EntityId, TenantId};
use frf_ports::{AuthzProvider, EntityChangeStream, EntityStore, IdentityVerifier, RelationTuple};
use futures_util::StreamExt as _;
use tracing::instrument;

use crate::error::AppError;

/// Request to read or watch a single entity within a tenant.
pub struct EntityRequest {
    pub entity_id: EntityId,
    pub tenant_id: TenantId,
    pub bearer_token: String,
}

struct AuthorizedEntity {
    tenant_id: TenantId,
    view_tuple: RelationTuple,
}

/// Application-layer use-case for the read side of the entity plane.
///
/// Wires three port traits, mirroring the publish/subscribe security model:
/// - `I: IdentityVerifier` — verifies the bearer JWT and yields the authoritative tenant.
/// - `A: AuthzProvider`    — configured visibility check on the entity.
/// - `S: EntityStore`      — reads the latest `EntityChange` and streams changes.
///
/// No adapter crate is imported here; the dependency inversion is enforced at the Cargo
/// level by `frf-app`'s `[dependencies]` section.
pub struct EntityUseCase<S: ?Sized, A, I> {
    store: Arc<S>,
    authz: Arc<A>,
    identity: Arc<I>,
}

impl<S, A, I> EntityUseCase<S, A, I>
where
    S: EntityStore + ?Sized,
    A: AuthzProvider,
    I: IdentityVerifier,
{
    pub fn new(store: Arc<S>, authz: Arc<A>, identity: Arc<I>) -> Self {
        Self {
            store,
            authz,
            identity,
        }
    }

    /// Verify the bearer token, enforce tenant-equality, and Keto `view` on the entity.
    /// Returns the verified `TenantId` (authoritative) on success.
    async fn authorize(&self, req: &EntityRequest) -> Result<AuthorizedEntity, AppError> {
        let claims = self
            .identity
            .verify(&req.bearer_token)
            .await
            .map_err(AppError::Identity)?;

        // Tenant-equality assertion (defense in depth beneath Keto): the verified JWT
        // tenant is authoritative; the request's tenant is caller-supplied and must match.
        if claims.tenant_id != req.tenant_id {
            return Err(AppError::Forbidden(format!(
                "subject {} (tenant {}) may not read an entity owned by tenant {}",
                claims.subject, claims.tenant_id, req.tenant_id
            )));
        }

        // Per-object RLS: the subject must have `view` on this specific entity.
        let view_tuple = RelationTuple {
            tenant_id: claims.tenant_id,
            subject: claims.subject.clone(),
            relation: "view".to_owned(),
            object: req.entity_id.to_string(),
        };
        let allowed = self
            .authz
            .check(&view_tuple)
            .await
            .map_err(AppError::Broker)?;
        if !allowed {
            return Err(AppError::Forbidden(format!(
                "subject {} may not view entity {}",
                claims.subject, req.entity_id
            )));
        }

        Ok(AuthorizedEntity {
            tenant_id: claims.tenant_id,
            view_tuple,
        })
    }

    /// Fetch the latest known change for an entity, after auth.
    ///
    /// # Errors
    ///
    /// [`AppError::Identity`] on an invalid token; [`AppError::Forbidden`] on a tenant
    /// mismatch or missing `view`; [`AppError::Broker`] on a store failure. `Ok(None)` is
    /// returned (not an error) when the entity is unknown to the tenant.
    #[instrument(name = "app::entity::get", skip(self, req), fields(
        entity_id = %req.entity_id,
        tenant_id = %req.tenant_id,
    ))]
    pub async fn get(&self, req: EntityRequest) -> Result<Option<EntityChange>, AppError> {
        let authorized = self.authorize(&req).await?;
        let change = self
            .store
            .get_entity(req.entity_id, authorized.tenant_id)
            .await?;
        Ok(change)
    }

    /// Stream subsequent changes for an entity, after auth.
    ///
    /// # Errors
    ///
    /// Same as [`get`](Self::get); additionally propagates a store error if the
    /// subscription cannot be established.
    #[instrument(name = "app::entity::watch", skip(self, req), fields(
        entity_id = %req.entity_id,
        tenant_id = %req.tenant_id,
    ))]
    pub async fn watch(&self, req: EntityRequest) -> Result<EntityChangeStream, AppError> {
        let authorized = self.authorize(&req).await?;
        let stream = self
            .store
            .watch_entity(req.entity_id, authorized.tenant_id)
            .await?;
        let authz = Arc::clone(&self.authz);
        let view_tuple = authorized.view_tuple;
        let checked = Box::pin(stream.then(move |item| {
            let authz = Arc::clone(&authz);
            let view_tuple = view_tuple.clone();
            async move {
                let change = match item {
                    Ok(change) => change,
                    Err(error) => return Err(error),
                };
                match authz.check(&view_tuple).await {
                    Ok(true) => Ok(change),
                    Ok(false) => Err(frf_ports::PortError::PermissionDenied(
                        "entity view authorization was revoked".to_owned(),
                    )),
                    Err(error) => Err(error),
                }
            }
        }));
        let guarded = futures_util::stream::unfold(
            (checked, false),
            |(mut checked, terminated)| async move {
                if terminated {
                    return None;
                }
                let item = checked.as_mut().next().await?;
                let terminated = item.is_err();
                Some((item, (checked, terminated)))
            },
        );
        Ok(Box::pin(guarded))
    }
}
