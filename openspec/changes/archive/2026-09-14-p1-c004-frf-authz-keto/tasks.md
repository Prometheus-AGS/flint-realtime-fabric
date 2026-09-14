# Tasks — p1-c004 frf-authz-keto

- [x] **T1** Create `crates/frf-authz-keto/src/types.rs`
  - Keto REST request/response structs (serde): `RelationTupleBody { namespace, object, relation, subject_id }`, `CheckResponse { allowed: bool }`, `BatchCheckRequest`, `BatchCheckResponse`
  - All derived: `Debug, Clone, Serialize, Deserialize`
  - Verification: `cargo check -p frf-authz-keto` exits 0

- [x] **T2** Create `crates/frf-authz-keto/src/cache.rs`
  - `CacheKey(String, String, String)` — `(subject, relation, object)` tuple newtype
  - `CacheEntry { allowed: bool, expires_at: std::time::Instant }`
  - `CheckCache` wrapping `DashMap<CacheKey, CacheEntry>`
  - Methods: `get(key) -> Option<bool>`, `insert(key, allowed, ttl_secs)`, `invalidate_object(relation, object)` — removes all entries where key matches `(*, relation, object)`
  - Unit tests in `#[cfg(test)]` block: `cache_hit_returns_value`, `expired_entry_returns_none`, `invalidate_removes_matching`
  - Verification: `cargo test -p frf-authz-keto -- cache` passes

- [x] **T3** Create `crates/frf-authz-keto/src/provider.rs`
  - `pub struct KetoAuthzProvider { http: reqwest::Client, base_url: String, namespace: String, cache: Arc<CheckCache>, check_ttl_secs: u64 }`
  - `impl KetoAuthzProvider { pub fn new(base_url: impl Into<String>, namespace: impl Into<String>) -> Self }`
  - `#[async_trait] impl AuthzProvider for KetoAuthzProvider`
    - `check`: consult cache first; on miss call `POST /relation-tuples/check`; populate cache; return result
    - `write`: `PUT /relation-tuples`; no cache update needed
    - `delete`: `DELETE /relation-tuples?...`; call `cache.invalidate_object(relation, object)`
  - `#[tracing::instrument(name = "port::AuthzProvider::<method>")]` on each impl method
  - Verification: `cargo check -p frf-authz-keto` exits 0

- [x] **T4** Update `crates/frf-authz-keto/src/lib.rs`
  - `pub mod cache; pub mod provider; pub mod types;`
  - `pub use provider::KetoAuthzProvider;`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-authz-keto` exits 0

- [x] **T5** Write integration tests with httpmock
  - File: `crates/frf-authz-keto/tests/keto_provider.rs`
  - Use `httpmock::MockServer` to mock Keto REST endpoints
  - Tests:
    - `check_returns_true_on_allowed_response`
    - `check_returns_false_on_denied_response`
    - `cache_hit_skips_http_call` — call check twice; assert server called once
    - `write_calls_put_endpoint`
    - `delete_calls_delete_endpoint_and_invalidates_cache`
  - Verification: `cargo test -p frf-authz-keto` — all 5 tests pass
