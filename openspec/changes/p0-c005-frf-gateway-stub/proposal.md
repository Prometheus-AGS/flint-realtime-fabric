# p0-c005 — frf-gateway Stub

## Affected crates
- `crates/frf-gateway` (new, binary crate)

## Dependency-rule impact
`frf-gateway` is the outermost layer — it may depend on all inner crates (frf-domain, frf-ports, frf-app, frf-proto, and adapter crates). At this stub stage it depends only on `frf-proto` and `frf-domain` (for types in health response). No adapter crates yet — their wiring is Phase 1. `anyhow` is allowed at this binary edge. **Semver impact:** none on inner crates — frf-gateway is a binary, not a library.

## Phase 0 exit criterion satisfied
_frf-gateway boots and serves GET /healthz → 200; cargo test passes._

## What this change does
1. Creates `crates/frf-gateway/Cargo.toml` (workspace member, binary target, `[dependencies]` = axum 0.8.8 + tokio + tonic + frf-proto + frf-domain + anyhow + tracing + tracing-subscriber).
2. Writes `crates/frf-gateway/src/main.rs`:
   - `tracing_subscriber` initialization.
   - Axum router with `GET /healthz` returning `{"status":"ok"}`.
   - Empty tonic `Server::builder()` with a stub service (no ops).
   - `axum_server` or `tokio::net::TcpListener` binding on `0.0.0.0:8080`.
   - WS upgrade handler at `/ws` that echoes frames back (no business logic).
3. Integration test in `crates/frf-gateway/tests/health.rs` that starts the server and asserts `GET /healthz` returns 200.
4. File size: `main.rs` ≤ 200 lines; echo handler split to `src/ws.rs` if needed.

## Non-goals
- No business logic, no port wiring, no real auth.
- No Iggy connection (Phase 1).
- No CRDT, no federation, no media.
- No admin UI serving (Phase 4).
- WS echo is a protocol smoke-test only — not the production mux.
