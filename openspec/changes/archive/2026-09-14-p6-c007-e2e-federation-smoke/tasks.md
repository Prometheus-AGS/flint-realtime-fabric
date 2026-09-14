# Tasks — p6-c007: Phase 6 E2E Federation Smoke Test

- [ ] Read `admin-ui/e2e/phase5-smoke.spec.ts` to capture exact 3-layer pattern
- [ ] Read `admin-ui/playwright.config.ts` for base URL and skip-integration pattern
- [ ] Read `crates/frf-gateway/src/lib.rs` to understand route registration
- [ ] Create `crates/frf-gateway/src/routes/dev.rs` (dev-only injection endpoint):
  - `#[cfg(debug_assertions)] pub async fn inject_federation_event(...)` handler
  - Accepts `POST /dev/inject-federation-event` with JSON body `{ "protocol": "Matrix", ... }`
  - Injects into the federation bridge pipeline (or directly into the log broker)
  - Returns `404` outside `debug_assertions`
- [ ] Update `crates/frf-gateway/src/lib.rs`:
  - Register dev route under `#[cfg(debug_assertions)]`:
    ```rust
    #[cfg(debug_assertions)]
    let router = router.route("/dev/inject-federation-event", post(routes::dev::inject_federation_event));
    ```
- [ ] Create `admin-ui/e2e/phase6-smoke.spec.ts`:
  - Layer 1 (5 tests — no gateway):
    1. Federation section heading visible
    2. Empty state renders when no events
    3. Matrix badge visible with synthetic event via `page.evaluate` + `window.__frf_dev`
    4. ATProto badge visible with synthetic event
    5. Navigation to `#federation` works (or `#agents` if no separate route)
  - Layer 2 (2 tests — gated `SKIP_INTEGRATION`):
    6. gRPC port 9090 accepts connections (or health check)
    7. Gateway starts without federation-related startup errors
  - Layer 3 (3 tests — gated `SKIP_INTEGRATION`):
    8. POST `/dev/inject-federation-event` returns 200 in dev mode
    9. Injected event appears in entity stream within 2000ms
    10. Protocol label shows "Matrix"
  - All tests use `waitForSelector` or `expect.poll()` — no `page.waitForTimeout`
- [ ] Run `pnpm typecheck` in `admin-ui/`
- [ ] Run `pnpm exec playwright test e2e/phase6-smoke.spec.ts --project=chromium` with `SKIP_INTEGRATION=true`
- [ ] Verify Layer 2 + 3 tests show `test.skip` output cleanly
- [ ] Run `cargo check --workspace` (verify dev.rs compiles)
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
