use std::sync::Arc;

use async_trait::async_trait;
use frf_ports::{AuthzProvider, PortError, RelationTuple};
use tracing::instrument;

use crate::cache::{CacheKey, CheckCache};
use crate::types::{CheckResponse, RelationTupleBody};

const DEFAULT_CHECK_TTL: u64 = 60;

pub struct KetoAuthzProvider {
    http: reqwest::Client,
    base_url: String,
    namespace: String,
    cache: Arc<CheckCache>,
    check_ttl_secs: u64,
}

impl KetoAuthzProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
            namespace: namespace.into(),
            cache: CheckCache::new(),
            check_ttl_secs: DEFAULT_CHECK_TTL,
        }
    }

    fn tuple_body(&self, tuple: &RelationTuple) -> RelationTupleBody {
        RelationTupleBody {
            namespace: self.namespace.clone(),
            object: tuple.object.clone(),
            relation: tuple.relation.clone(),
            subject_id: tuple.subject.clone(),
        }
    }
}

#[async_trait]
impl AuthzProvider for KetoAuthzProvider {
    /// Return `true` if the subject holds the relation on the object.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the HTTP request fails.
    /// Returns [`PortError::Serialization`] if the response body cannot be decoded.
    #[instrument(name = "port::AuthzProvider::check", skip(self, tuple), fields(relation = %tuple.relation))]
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        let key = CacheKey(
            tuple.subject.clone(),
            tuple.relation.clone(),
            tuple.object.clone(),
        );

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }

        let body = self.tuple_body(tuple);
        let url = format!("{}/relation-tuples/check", self.base_url);

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;

        let check: CheckResponse = resp
            .json()
            .await
            .map_err(|e| PortError::Serialization(e.to_string()))?;

        self.cache.insert(key, check.allowed, self.check_ttl_secs);

        Ok(check.allowed)
    }

    /// Write (grant) a relation tuple.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the HTTP request fails or returns a non-2xx status.
    #[instrument(name = "port::AuthzProvider::write", skip(self, tuple), fields(relation = %tuple.relation))]
    async fn write(&self, tuple: RelationTuple) -> Result<(), PortError> {
        let body = self.tuple_body(&tuple);
        let url = format!("{}/relation-tuples", self.base_url);

        let resp = self
            .http
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            return Err(PortError::Transport(format!(
                "keto write returned HTTP {status}"
            )));
        }

        Ok(())
    }

    /// Delete (revoke) a relation tuple and invalidate the cache for that object.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the HTTP request fails or returns a non-2xx status.
    #[instrument(name = "port::AuthzProvider::delete", skip(self, tuple), fields(relation = %tuple.relation))]
    async fn delete(&self, tuple: RelationTuple) -> Result<(), PortError> {
        let url = format!(
            "{}/relation-tuples?namespace={}&object={}&relation={}&subject_id={}",
            self.base_url, self.namespace, tuple.object, tuple.relation, tuple.subject
        );

        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            return Err(PortError::Transport(format!(
                "keto delete returned HTTP {status}"
            )));
        }

        self.cache.invalidate_object(&tuple.relation, &tuple.object);

        Ok(())
    }
}
