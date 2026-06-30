# p14-c002 — Add `DEV_NO_AUTH` env var to gateway for dev-endpoints builds

> Phase: phase-14-stage10-dind-live-triage · Priority: HIGH

## Problem

`/v1/publish` and `/ws/v1/subscribe` require a `Bearer` token in the `Authorization` header. Phase 4/5 E2E specs POST without any auth header, returning `401 Unauthorized`. Minting real JWTs in the compose test environment is complex.

## Solution

Add a `DEV_NO_AUTH` environment variable that, when set to `true`, bypasses JWT verification in the `publish` and `subscribe` route handlers. This bypass is **strictly gated** on the `dev-endpoints` Cargo feature — if the feature flag is absent, the env var is ignored.

### Gateway config (`crates/frf-gateway/src/config.rs`)

Add:

```rust
#[cfg(feature = "dev-endpoints")]
pub fn dev_no_auth() -> bool {
    std::env::var("DEV_NO_AUTH")
        .map(|v| v == "true")
        .unwrap_or(false)
}
```

### Publish route (`crates/frf-gateway/src/routes/publish.rs`)

Before the bearer token check:

```rust
#[cfg(feature = "dev-endpoints")]
if crate::config::dev_no_auth() {
    // skip auth in dev mode
} else {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    // verify token...
}
#[cfg(not(feature = "dev-endpoints"))]
{
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    // verify token...
}
```

### Subscribe route (`crates/frf-gateway/src/routes/subscribe.rs`)

Same pattern applied to the WS upgrade handler.

### compose.yml

Add to the gateway service environment:

```yaml
DEV_NO_AUTH: "true"
```

## Files Changed

- `crates/frf-gateway/src/config.rs` — add `dev_no_auth()` fn under `#[cfg(feature = "dev-endpoints")]`
- `crates/frf-gateway/src/routes/publish.rs` — gate auth check on `dev_no_auth()`
- `crates/frf-gateway/src/routes/subscribe.rs` — gate auth check on `dev_no_auth()`
- `compose.yml` — add `DEV_NO_AUTH: "true"` to gateway env

## Acceptance Criteria

- [ ] `DEV_NO_AUTH=true` + `dev-endpoints` feature → publish returns non-401
- [ ] `DEV_NO_AUTH=false` + `dev-endpoints` feature → publish returns 401 without token
- [ ] `DEV_NO_AUTH=true` + no `dev-endpoints` feature → env var ignored, normal auth enforced
- [ ] `cargo clippy --workspace -- -D warnings` passes
