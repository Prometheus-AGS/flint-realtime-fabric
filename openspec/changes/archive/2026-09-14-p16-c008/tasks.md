# Tasks — p16-c008

- [x] Enable tonic_web + accept_http1 for SpineService
- [x] Add /ws/v1/signal route wired to the signal service
- [x] Embed admin-ui in gateway (rust-embed/ServeDir)
- [x] Add login/auth flow in admin-ui; token flows into every call
- [x] Reconcile default gateway URL/port (UI + gateway config)
- [x] E2E: Entities page subscribes against a real gateway

## Summary

Wired the admin-UI ↔ gateway path end to end (audit C2/C3/H9/H10/H11). Done in two
passes per operator decision: transport first, then UI/embed/E2E.

## Task 1 — gRPC-web transport (C2)

Added `tonic-web = "0.14"`. `spawn_grpc_server` now
`.accept_http1(true).layer(GrpcWebLayer::new())` and registers **Spine + Signal +
Agent** (previously only Agent). The browser's `SpineService/Subscribe` now has a real
Connect/gRPC-web endpoint. `SyncService` deferred to c013 (needs a `SyncUseCase` not
built in main).

## Task 2 — /ws/v1/signal route (C3)

New `routes/signal.rs`: an Axum WebSocket route that subscribes to the `MediaSignaler`
for a fresh session and streams JSON `SignalFrame`s (camelCase, matching the admin-UI
shape). Auth via `?token=` query param (browser WS cannot set an Authorization header),
verified through the identity verifier; dev-endpoints + `DEV_NO_AUTH` falls back to
`?tenant=`. Registered at `/ws/v1/signal`. Compiles + clippy clean both feature configs.

## Task 3 — embed admin-UI (H9)

Added `rust-embed` + `mime_guess`. New `routes/admin_ui.rs` embeds `admin-ui/dist` and
serves it as the router `.fallback` — API routes take precedence; unmatched GETs return
UI assets or fall back to `index.html` (SPA client routing). 3 unit tests
(`root_serves_embedded_index_html`, `unknown_path_falls_back_to_index`,
`asset_is_served_with_its_mime`) pass.

## Task 4 — login flow + token attachment (H10)

- **Transport interceptor** (`infrastructure/gateway.ts`): reads `accessToken` from the
  auth store and adds `Authorization: Bearer` to every Connect call. Required
  `@connectrpc/connect@^1.7.0` (the `Interceptor` type's home; connect-web's peer).
- **Auth feature** built to the admin-UI architecture: `authService` (token persistence
  + `restoreToken`), `useAuth` hook (coordinates), `LoginGate` component (renders a
  token-entry form; gates the app). `App.tsx` restores a persisted token on boot and
  wraps the app in `LoginGate`.
- Signaling WS now passes the token as `?token=` (browser WS can't set headers).
- Not a full OIDC redirect flow (needs flint-gate login endpoints) — documented; this is
  a functional token-entry gate matching the current dev-passthrough reality.
- All 6 touched TS files lint clean + typecheck. (Two pre-existing lint errors in
  untouched files — `p7-smoke.spec.ts`, `useEntitySubscription.ts` — remain, from
  commit a6e0e51.)

## Task 5 — URL reconciliation (H11)

`spineClient` default `http://localhost:4000` → `http://localhost:9090` (Connect/gRPC-web
is served on the gRPC port, not the Axum HTTP port; compose maps it to host 29090). The
`/ws/v1/signal` service uses a separate `VITE_GATEWAY_WS_URL` (default 8080, compose
28080) since it is an Axum WS route on the HTTP port. Both documented inline.

## Task 6 — E2E

New `e2e/p16-admin-gateway.spec.ts`:
- Layer 1 (no gateway): login gate renders unauthenticated; entering a token dismisses
  it; the gated app renders with a seeded token.
- Layer 2 (gated behind `SKIP_INTEGRATION`/`GATEWAY_URL`): Entities page mounts against a
  real gateway with no hard connection error.

Added a Playwright `globalSetup` that seeds a dev token into `storageState` so the
existing app-content specs render past the new LoginGate; the gate-specific tests opt out
via `test.use({ storageState: { cookies: [], origins: [] } })`. `.auth/` gitignored.

**Environment note:** the Layer-1 tests could not execute in this session because port
5173 is held by an unrelated SSH tunnel (Playwright `reuseExistingServer` connected to a
foreign "Flint Gate Admin" app, not the FRF admin-UI whose title is
"Flint Realtime Fabric — Admin"). The spec itself lint-clean + typechecks; it runs in a
clean CI/dev environment. Same class of limitation as the DinD-gated integration tests.

## Verification

- `cargo clippy -p frf-gateway --lib --bins` (default + dev-endpoints) → exit 0
- `cargo fmt --check -p frf-gateway` → clean
- `cargo test -p frf-gateway --lib` → 7/7 (incl. 3 new admin_ui embed tests)
- admin-ui: `pnpm typecheck` → exit 0; ESLint on all touched files → exit 0
