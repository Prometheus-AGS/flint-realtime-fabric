# Tasks — p6-c006: Gateway Federation Wiring + Admin UI Debt

## Part A: Federation Wiring

- [ ] Read `crates/frf-gateway/src/lib.rs` to understand current `AppState<L,A,I,M,B>` struct
- [ ] Read `crates/frf-gateway/src/main.rs` to understand current startup sequence
- [ ] Read `crates/frf-gateway/Cargo.toml` to check current deps
- [ ] Add `frf-bridge-matrix` and `frf-bridge-atproto` path deps to `crates/frf-gateway/Cargo.toml`
- [ ] Update `crates/frf-gateway/src/lib.rs`:
  - Add `use frf_ports::FederationBridge` import
  - Add `federation_bridges: Vec<Arc<dyn FederationBridge + Send + Sync>>` field to `AppState`
  - Add `pub type AppStateArc<L, A, I, M, B> = Arc<AppState<L, A, I, M, B>>` type alias
- [ ] Update `crates/frf-gateway/src/main.rs`:
  - Add `MatrixBridge` and `AtProtoBridge` construction (feature-gated or config-driven)
  - Populate `federation_bridges` vec in `AppState` construction
  - Add background ingest loop for each bridge after `AppState` is built
  - Add `federated_event_to_envelope(event: FederatedEvent) -> EventEnvelope` conversion fn
- [ ] Remove `#[allow(clippy::type_complexity)]` from `routes/agents.rs`, `routes/publish.rs`, `routes/subscribe.rs`
- [ ] Replace `State<Arc<AppState<...>>>` with `State<AppStateArc<...>>` in all three files
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`

## Part B: Admin UI Debt

- [ ] Read `admin-ui/src/features/agents/services/agentWebSocket.ts` to find hardcoded token
- [ ] Read `admin-ui/src/features/agents/stores/agentEventStore.ts` to understand store structure
- [ ] Read `admin-ui/e2e/phase5-smoke.spec.ts` ring-buffer test that uses `window.__agentEventStore`
- [ ] Create `admin-ui/src/features/auth/stores/authStore.ts` with minimal `useAuthStore` stub
- [ ] Update `agentWebSocket.ts` to import and use `useAuthStore.getState().accessToken`
- [ ] Update `agentEventStore.ts` to export on `window.__frf_dev` when `import.meta.env.DEV`
- [ ] Update `admin-ui/e2e/phase5-smoke.spec.ts` ring-buffer test: `window.__agentEventStore` → `window.__frf_dev?.agentEventStore`
- [ ] Run `pnpm typecheck` in `admin-ui/`
- [ ] Run `pnpm build` in `admin-ui/`
