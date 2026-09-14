# p6-c006 — Gateway Federation Wiring + Admin UI Debt

## Summary

Wire the two federation bridge crates into `frf-gateway` as background ingest
tasks, and address the three carry-forward debt items from Phase 5: the demo
WS token in `agentWebSocket.ts`, the ring-buffer E2E store export, and the
`#[allow(clippy::type_complexity)]` in three route handlers.

## Part A: Federation Bridge Wiring in frf-gateway

### AppState federation dimension

Do NOT add a 6th generic. Instead, add a `federation_bridges` side-channel
to `AppState` as a `Vec<Arc<dyn FederationBridge + Send + Sync>>`:

```rust
// frf-gateway/src/lib.rs
pub struct AppState<L, A, I, M, B> {
    // existing 5 generics unchanged
    pub log_broker: Arc<L>,
    pub authz: Arc<A>,
    pub identity: Arc<I>,
    pub media: Arc<M>,
    pub agent_bus: Arc<B>,
    // NEW — object-safe side channel, not a generic
    pub federation_bridges: Vec<Arc<dyn FederationBridge + Send + Sync>>,
}
```

### Background ingest task (main.rs)

For each bridge, spawn a background task that consumes `bridge.subscribe()`
and publishes each `FederatedEvent` onto the Iggy log broker as an
`EventEnvelope`:

```rust
for bridge in &state.federation_bridges {
    let bridge = Arc::clone(bridge);
    let broker = Arc::clone(&state.log_broker);
    tokio::spawn(async move {
        let mut stream = bridge.subscribe();
        while let Some(result) = stream.next().await {
            match result {
                Ok(federated_event) => {
                    let envelope = federated_event_to_envelope(federated_event);
                    if let Err(e) = broker.publish(envelope).await {
                        tracing::error!(error = %e, "federation ingest publish failed");
                    }
                }
                Err(e) => tracing::warn!(error = %e, "federation event error"),
            }
        }
    });
}
```

### Cargo deps

Add to `frf-gateway/Cargo.toml`:
```toml
frf-bridge-matrix = { path = "../frf-bridge-matrix" }
frf-bridge-atproto = { path = "../frf-bridge-atproto" }
frf-ports = { path = "../frf-ports" }  # already present — verify
```

## Part B: Phase 5 Debt Resolution

### 1. `AppStateArc` type alias (removes 3× type_complexity allows)

In `frf-gateway/src/lib.rs`, add:

```rust
pub type AppStateArc<L, A, I, M, B> = Arc<AppState<L, A, I, M, B>>;
```

Remove the 3 `#[allow(clippy::type_complexity)]` from:
- `routes/agents.rs`
- `routes/publish.rs`
- `routes/subscribe.rs`

Update all `State(state): State<Arc<AppState<L, A, I, M, B>>>` extractor
annotations to `State(state): State<AppStateArc<L, A, I, M, B>>`.

### 2. Demo WS token in `agentWebSocket.ts`

In `admin-ui/src/features/agents/services/agentWebSocket.ts`, replace the
hardcoded demo token with a token sourced from the auth store:

```typescript
// Before: const token = "demo-token";
// After:
import { useAuthStore } from '../../auth/stores/authStore';
const token = useAuthStore.getState().accessToken ?? '';
```

If `useAuthStore` doesn't exist yet, create a minimal stub:
```typescript
// admin-ui/src/features/auth/stores/authStore.ts
import { create } from 'zustand';
interface AuthState { accessToken: string | null; }
export const useAuthStore = create<AuthState>(() => ({ accessToken: null }));
```

### 3. Ring-buffer E2E store export

In `admin-ui/src/features/agents/stores/agentEventStore.ts`, export the store
on `window.__frf_dev` in development mode:

```typescript
if (import.meta.env.DEV) {
  (window as Record<string, unknown>).__frf_dev = {
    ...((window as Record<string, unknown>).__frf_dev as object ?? {}),
    agentEventStore: useAgentEventStore,
  };
}
```

Update the Phase 5 E2E ring-buffer test to use `window.__frf_dev.agentEventStore`
instead of `window.__agentEventStore`.

## Files Affected

- `crates/frf-gateway/Cargo.toml` (MODIFY — add bridge crate deps)
- `crates/frf-gateway/src/lib.rs` (MODIFY — `AppState`, `AppStateArc`, `FederationBridge` import)
- `crates/frf-gateway/src/main.rs` (MODIFY — federation bridge ingest tasks)
- `crates/frf-gateway/src/routes/agents.rs` (MODIFY — use `AppStateArc`)
- `crates/frf-gateway/src/routes/publish.rs` (MODIFY — use `AppStateArc`)
- `crates/frf-gateway/src/routes/subscribe.rs` (MODIFY — use `AppStateArc`)
- `admin-ui/src/features/agents/services/agentWebSocket.ts` (MODIFY — real token)
- `admin-ui/src/features/auth/stores/authStore.ts` (NEW — minimal stub)
- `admin-ui/src/features/agents/stores/agentEventStore.ts` (MODIFY — DEV export)
- `admin-ui/e2e/phase5-smoke.spec.ts` (MODIFY — update ring-buffer test)

## Quality Gates

- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes — 0 `type_complexity` allows remain in gateway routes
- [ ] `pnpm typecheck` in `admin-ui/` passes
- [ ] No hardcoded token strings in `agentWebSocket.ts`
- [ ] `window.__frf_dev` only set when `import.meta.env.DEV === true`
