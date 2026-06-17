# p1-c004 — frf-authz-keto: AuthzProvider → Ory Keto

## Affected crates
- `crates/frf-authz-keto` (new — stub created by p1-c001)

## Dependency-rule impact
Layer 2 (infrastructure adapter). Imports `frf-domain` and `frf-ports`. Implements `AuthzProvider`. Must NOT import `frf-app` or any other adapter crate.

## What this change does

Implements the `AuthzProvider` port against the Ory Keto v0.12+ REST API.

### HTTP endpoints used
```
POST /relation-tuples/check          → check(tuple) → bool
POST /relation-tuples/batch/check    → batch check (used by fan-out filter)
PUT  /relation-tuples                → write(tuple)
DELETE /relation-tuples?...          → delete(tuple)
```

### Internal check cache
```rust
struct CheckCache {
    entries: DashMap<CacheKey, CacheEntry>,
}
struct CacheKey { subject: String, relation: String, object: String }
struct CacheEntry { allowed: bool, expires_at: Instant }
```
TTL: 60 seconds (configurable via `KetoBrokerConfig.check_ttl_secs`). On `delete()`, invalidate all entries matching the deleted tuple's `(relation, object)`.

### `KetoAuthzProvider` struct
```rust
pub struct KetoAuthzProvider {
    http: reqwest::Client,
    base_url: String,
    namespace: String,
    cache: Arc<CheckCache>,
}
```

### Module layout
```
crates/frf-authz-keto/src/
├── lib.rs
├── provider.rs    KetoAuthzProvider + AuthzProvider impl
├── cache.rs       CheckCache (DashMap + TTL)
└── types.rs       Keto REST request/response types
```

## Phase 1 exit criterion satisfied
`check/write/delete` tested against `httpmock` mock HTTP server. Cache hit avoids second HTTP call.

## Non-goals
- Does not implement userset expansion (`expand` is out of scope for Phase 1).
- Does not implement webhook-based cache invalidation (Phase 7).
- Does not manage Keto namespaces (pre-configured via Keto config file).
