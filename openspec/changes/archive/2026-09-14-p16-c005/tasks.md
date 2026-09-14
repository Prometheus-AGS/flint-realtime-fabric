# Tasks — p16-c005

- [x] Add tower-http rate-limit layer to router
- [x] Add RequestBodyLimit layer
- [x] Add explicit CorsLayer policy
- [x] Integration test: over-limit request is rejected; CORS preflight succeeds

## Notes

### Implementation

`apply_security_layers(router, config)` (in `lib.rs`, generic over router state so it
runs before `with_state`) applies three layers to every route:

1. **Rate limit** — `tower_governor` `GovernorLayer` with a token bucket
   (`RATE_LIMIT_PER_SEC` default 50, `RATE_LIMIT_BURST` default 100).
2. **Body-size cap** — `tower_http::limit::RequestBodyLimitLayer`
   (`MAX_BODY_BYTES` default 1 MiB) → 413 on over-limit.
3. **CORS** — `tower_http::cors::CorsLayer` from an exact-match allowlist
   (`CORS_ALLOWED_ORIGINS`, comma-separated). Empty list = no cross-origin access.
   Allowed methods GET/POST/OPTIONS; headers Authorization + Content-Type.

New deps: workspace `tower-http` gained the `limit` feature; added
`tower_governor = "0.8"` (cached offline).

New config fields (`config.rs`): `rate_limit_per_sec`, `rate_limit_burst`,
`max_body_bytes`, `cors_allowed_origins`, parsed in an extracted
`middleware_config_from_env()` helper (kept `from_env` under the 100-line lint).

### Design decision — GlobalKeyExtractor, not per-IP

Initially used `SmartIpKeyExtractor` (per-IP via X-Forwarded-For). The integration
test surfaced a real production hazard: an IP extractor **fails closed with 500
("Unable To Extract Key!")** when it cannot determine a client IP — which happens on
any direct connection without a trusted `X-Forwarded-For`, and in `axum_test`.
Switched to `GlobalKeyExtractor`: a global token-bucket cap that works reliably
everywhere and never fails closed. Per-IP limiting is deferred until proxy-trust /
`ConnectInfo` config exists — documented inline. A global cap still directly closes
the audit's H2 "unbounded concurrency → DoS" concern.

### Tests (`crates/frf-gateway/tests/security_layers.rs`)

Added `build_security_test_router(config)` (minimal router + the real security layers)
so tests exercise the middleware without full adapter state. 4 tests, all pass:
- `body_within_limit_is_accepted` — under-cap POST → 200.
- `body_over_limit_is_rejected` — over-cap POST → 413.
- `cors_preflight_from_allowed_origin_is_permitted` — allowed origin reflected.
- `cors_origin_not_in_allowlist_is_not_reflected` — disallowed origin NOT reflected.

### Verification

- `cargo clippy --workspace --lib --bins` (default) → exit 0
- `cargo clippy -p frf-gateway --lib --bins --features dev-endpoints` → exit 0
- `cargo clippy -p frf-gateway --tests` → exit 0
- `cargo test -p frf-gateway --test security_layers` → 4/4 pass
- `cargo fmt --check` → clean

### Follow-up for docs (G5)

`RATE_LIMIT_PER_SEC`, `RATE_LIMIT_BURST`, `MAX_BODY_BYTES`, `CORS_ALLOWED_ORIGINS`
must be added to the env-var reference (c022). Note per-IP rate limiting as future work.
