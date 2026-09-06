//! Policy resolution + authorization — the only path that produces an
//! [`AuthorizedShapeRequest`].
//!
//! ADR-009 requires that **every** request and **every continuation** be authorized: a
//! resumed cursor carries no standing permission, because a subject's membership can be
//! revoked between chunks. This module enforces that by construction — `resolve` runs the
//! Keto check on each call, and the adapter's `fetch` accepts nothing else. There is no
//! "authorized once" state to go stale.
//!
//! The check is a live `AuthzProvider::check` per call rather than a cached grant. ADR-009
//! rejects ADR-007's room-lifetime cache for ASO precisely because a grant held until
//! disconnect cannot meet a bounded revocation window; caching here would reintroduce that
//! defect on the relational lane.

use frf_ports::{AuthorizedShapeRequest, AuthzProvider, RelationTuple, ShapeRequest};

use crate::error::ShapeError;
use crate::policy::ShapeCatalog;

/// Resolves a client [`ShapeRequest`] into an authorized, server-derived request.
pub struct ShapeResolver {
    catalog: ShapeCatalog,
    authz: std::sync::Arc<dyn AuthzProvider>,
    scope_column: String,
}

impl ShapeResolver {
    /// Build a resolver over a declared catalog and an authorization provider.
    ///
    /// `scope_column` is the column carrying the practice scope (e.g. `practice_id`). It is
    /// server configuration, never client input.
    pub fn new(
        catalog: ShapeCatalog,
        authz: std::sync::Arc<dyn AuthzProvider>,
        scope_column: impl Into<String>,
    ) -> Self {
        Self {
            catalog,
            authz,
            scope_column: scope_column.into(),
        }
    }

    /// Resolve and authorize one request.
    ///
    /// `scope` is the subject's practice, resolved upstream from verified claims — not from
    /// the request body. Authorization asks Keto whether this subject holds the shape's
    /// relation on that scope, and a denial *or a check error* refuses the request
    /// (fail-closed): a subject that cannot be authorized must not receive clinical rows.
    ///
    /// # Errors
    ///
    /// [`ShapeError::UnknownShape`] for an undeclared shape, [`ShapeError::ParamNotAllowed`]
    /// or [`ShapeError::InvalidParam`] for a widening or malformed parameter, and
    /// [`ShapeError::Unauthorized`] when the check denies or fails.
    #[tracing::instrument(
        name = "shape_facade::resolve",
        skip(self, request, scope),
        fields(shape = %request.shape, params = request.params.len())
    )]
    pub async fn resolve(
        &self,
        request: &ShapeRequest,
        scope: &str,
    ) -> Result<AuthorizedShapeRequest, ShapeError> {
        let policy = self.catalog.get(&request.shape)?;

        // Authorize before composing: a denied subject learns nothing about the shape's
        // schema from the error it receives.
        let tuple = RelationTuple {
            tenant_id: request.tenant_id,
            subject: request.subject.clone(),
            relation: policy.relation.clone(),
            object: format!("{}:{scope}", policy.object_namespace),
        };
        match self.authz.check(&tuple).await {
            Ok(true) => {}
            Ok(false) => return Err(ShapeError::Unauthorized),
            Err(e) => {
                // Fail closed, and log the class only — never the subject or scope.
                tracing::warn!(error = %e, "shape authz check failed — denying");
                return Err(ShapeError::Unauthorized);
            }
        }

        let where_clause = policy.compose_where(&self.scope_column, scope, &request.params)?;

        Ok(AuthorizedShapeRequest {
            table: policy.table.clone(),
            columns: policy.columns.clone(),
            where_clause,
            cursor: request.cursor.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::ShapePolicy;
    use async_trait::async_trait;
    use frf_domain::TenantId;
    use frf_ports::{Cursor, PortError};
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct StubAuthz {
        allow: bool,
        fail: bool,
        calls: AtomicUsize,
    }

    impl StubAuthz {
        fn new(allow: bool, fail: bool) -> Arc<Self> {
            Arc::new(Self {
                allow,
                fail,
                calls: AtomicUsize::new(0),
            })
        }
        fn allowing() -> Arc<Self> {
            Self::new(true, false)
        }
        fn denying() -> Arc<Self> {
            Self::new(false, false)
        }
        fn failing() -> Arc<Self> {
            Self::new(false, true)
        }
    }

    /// Coerce a concrete stub into the trait object the resolver stores.
    fn erased(a: &Arc<StubAuthz>) -> Arc<dyn AuthzProvider> {
        Arc::clone(a) as Arc<dyn AuthzProvider>
    }

    #[async_trait]
    impl AuthzProvider for StubAuthz {
        async fn check(&self, _tuple: &RelationTuple) -> Result<bool, PortError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(PortError::Timeout);
            }
            Ok(self.allow)
        }
        async fn write(&self, _tuple: RelationTuple) -> Result<(), PortError> {
            Ok(())
        }
        async fn delete(&self, _tuple: RelationTuple) -> Result<(), PortError> {
            Ok(())
        }
    }

    fn catalog() -> ShapeCatalog {
        let mut m = BTreeMap::new();
        m.insert(
            "prior_auth".to_owned(),
            ShapePolicy {
                table: "prior_auth_request".to_owned(),
                columns: vec!["id".to_owned(), "status".to_owned()],
                allowed_params: vec!["status".to_owned()],
                relation: "view".to_owned(),
                object_namespace: "practice".to_owned(),
            },
        );
        ShapeCatalog::new(m)
    }

    fn request(cursor: Cursor) -> ShapeRequest {
        ShapeRequest {
            tenant_id: TenantId::new(),
            subject: "user-1".to_owned(),
            shape: "prior_auth".to_owned(),
            params: vec![],
            cursor,
        }
    }

    #[tokio::test]
    async fn an_authorized_request_is_scoped_to_the_server_derived_practice() {
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::allowing()), "practice_id");
        let out = resolver
            .resolve(&request(Cursor::Initial), "p-1")
            .await
            .expect("authorized");

        assert_eq!(out.table, "prior_auth_request");
        assert_eq!(out.columns, vec!["id".to_owned(), "status".to_owned()]);
        assert_eq!(out.where_clause, "practice_id = 'p-1'");
    }

    #[tokio::test]
    async fn a_denied_subject_gets_no_rows() {
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::denying()), "practice_id");
        let err = resolver
            .resolve(&request(Cursor::Initial), "p-1")
            .await
            .expect_err("denied");
        assert!(matches!(err, ShapeError::Unauthorized));
    }

    #[tokio::test]
    async fn an_authz_failure_denies_rather_than_admits() {
        // Fail-closed: an unreachable Keto must not become an open door.
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::failing()), "practice_id");
        let err = resolver
            .resolve(&request(Cursor::Initial), "p-1")
            .await
            .expect_err("must deny on error");
        assert!(matches!(err, ShapeError::Unauthorized));
    }

    #[tokio::test]
    async fn every_continuation_is_re_authorized_not_just_the_first_request() {
        // The ADR-009 requirement that distinguishes this lane from ADR-007's cached grant:
        // a resumed cursor carries no standing permission.
        let authz = StubAuthz::allowing();
        let resolver = ShapeResolver::new(catalog(), erased(&authz), "practice_id");

        resolver
            .resolve(&request(Cursor::Initial), "p-1")
            .await
            .expect("first");
        resolver
            .resolve(
                &request(Cursor::Resume {
                    handle: "h-1".to_owned(),
                    offset: "5".to_owned(),
                }),
                "p-1",
            )
            .await
            .expect("continuation");

        assert_eq!(
            authz.calls.load(Ordering::SeqCst),
            2,
            "the continuation must trigger its own check"
        );
    }

    #[tokio::test]
    async fn revocation_between_chunks_stops_delivery_on_the_next_continuation() {
        // Same subject, same cursor: only the authorization changed.
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::denying()), "practice_id");
        let err = resolver
            .resolve(
                &request(Cursor::Resume {
                    handle: "h-1".to_owned(),
                    offset: "5".to_owned(),
                }),
                "p-1",
            )
            .await
            .expect_err("revoked");
        assert!(matches!(err, ShapeError::Unauthorized));
    }

    #[tokio::test]
    async fn a_client_cannot_widen_the_shape_by_supplying_the_scope_column() {
        let mut req = request(Cursor::Initial);
        req.params = vec![("practice_id".to_owned(), "p-2".to_owned())];
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::allowing()), "practice_id");

        let err = resolver
            .resolve(&req, "p-1")
            .await
            .expect_err("must reject");
        assert!(matches!(err, ShapeError::ParamNotAllowed(k) if k == "practice_id"));
    }

    #[tokio::test]
    async fn an_undeclared_shape_is_refused_before_any_upstream_call() {
        let mut req = request(Cursor::Initial);
        req.shape = "not_declared".to_owned();
        let resolver = ShapeResolver::new(catalog(), erased(&StubAuthz::allowing()), "practice_id");

        let err = resolver
            .resolve(&req, "p-1")
            .await
            .expect_err("must reject");
        assert!(matches!(err, ShapeError::UnknownShape(_)));
    }
}
