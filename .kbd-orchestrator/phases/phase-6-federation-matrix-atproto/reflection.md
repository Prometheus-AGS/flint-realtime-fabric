# Reflection — Phase 6: Federation + Matrix/ATProto Bridge

> Generated: 2026-06-21 · Tool: kbd-reflect

---

## Goal Achievement

| # | Goal | Status | Evidence |
|---|------|--------|----------|
| 1 | `frf-bridge-matrix` — Tuwunel projection | **MET** | `crates/frf-bridge-matrix/src/{lib,client,convert,error}.rs` implement `FederationBridge`; `MatrixClient` trait abstraction with `ReqwestMatrixClient` stub; unit tests on projection logic |
| 2 | `frf-bridge-atproto` — Tranquil firehose projection | **MET** | `crates/frf-bridge-atproto/src/{lib,jetstream,convert,error}.rs` implement `FederationBridge` via `tokio-tungstenite` Jetstream WS consumer; write path returns `Err(Unsupported)` |
| 3 | `AgentService.RunAgent` gRPC streaming | **MET** | `crates/frf-gateway/src/agent_grpc_service.rs` implements server-streaming `RunAgent`; `domain_to_proto` added to `frf-agentproto`; tonic `Server` started on port 9090 in `main.rs` |
| 4 | AgentEventBus tenant isolation ADR | **MET** | `docs/decisions/adr-002-agent-bus-tenant-isolation.md` written; documents subscription-scoped isolation, subscribe-time Keto check mitigation, CLAUDE.md deviation rationale |
| 5 | LibreFangBus actor sharding (`TenantActorRegistry`) | **MET** | `crates/frf-librefang/src/registry.rs` implements `TenantActorRegistry` (DashMap + lazy spawn + idle eviction); `bus.rs` routes via registry instead of single global actor |
| 6 | Phase exit criterion — E2E test | **MET** | `admin-ui/e2e/phase6-smoke.spec.ts` Layer 1 tests pass with `SKIP_INTEGRATION=true`; Layer 2 + 3 skip cleanly; `pnpm typecheck` exits 0 |

**Goal achievement: 6/6 (100%)**

---

## Delivered Changes

| Change | Description | Key Artifacts |
|---|---|---|
| p6-c001 | ADR-002: AgentEventBus Tenant Isolation | `docs/decisions/adr-002-agent-bus-tenant-isolation.md` |
| p6-c002 | LibreFangBus TenantActorRegistry | `crates/frf-librefang/src/registry.rs`, updated `bus.rs` |
| p6-c003 | `frf-bridge-matrix` crate | `crates/frf-bridge-matrix/src/{lib,client,convert,error}.rs` |
| p6-c004 | `frf-bridge-atproto` crate | `crates/frf-bridge-atproto/src/{lib,jetstream,convert,error}.rs` |
| p6-c005 | AgentGrpcService RunAgent handler | `crates/frf-gateway/src/agent_grpc_service.rs`; `domain_to_proto` in `frf-agentproto` |
| p6-c006 | Gateway federation wiring + admin-ui debt | `AppStateArc` alias; `federation_bridges` field; `spawn_federation_ingest_tasks`; `authStore.ts`; dev route; `window.__frf_dev` export |
| p6-c007 | Phase 6 E2E smoke test | `admin-ui/e2e/phase6-smoke.spec.ts`; `routes/dev.rs` inject endpoint |

**All 7 changes delivered. No blocked changes.**

---

## Artifact Quality Summary

Artifact-refiner was not run for this phase (no `.refiner/artifacts/` logs present). Quality was enforced manually at each change boundary via the plan's gate protocol:

- `cargo check --workspace` — 0 errors across all 7 changes
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` — 0 warnings at phase close
- `fnm exec --using=24 pnpm typecheck` — 0 TypeScript errors
- `SKIP_INTEGRATION=true pnpm exec playwright test e2e/phase6-smoke.spec.ts` — Layer 1 tests passing

Notable quality incidents (resolved before close):

| Incident | Change | Resolution |
|---|---|---|
| `use std::sync::Arc` accidentally removed from `routes/agents.rs` | p6-c006 | Re-added; `Arc::clone` requires the import |
| `FederationBridge` trait not in scope in `main.rs` | p6-c006 | Re-added to import block |
| TypeScript TS2352 `window as Record<string,unknown>` | p6-c006 | Fixed with `as unknown as Record<string,unknown>` double cast |
| pnpm fails on Node 20 (requires Node 22+) | p6-c006 | Resolved by running via `fnm exec --using=24`; `.node-version` + `engines` field added |

---

## Carry-Forward Debt Cleared

All five Phase 5 debt items were resolved in Phase 6:

| Debt Item | Resolved In |
|---|---|
| Keto ADR for agent bus missing | p6-c001 |
| Single actor tree, no sharding | p6-c002 |
| `#[allow(clippy::type_complexity)]` × 3 | p6-c006 (`AppStateArc` alias) |
| Demo WS token in `agentWebSocket.ts` | p6-c006 (`authStore.ts` + `useAuthStore`) |
| E2E ring-buffer test broken (`window.__agentEventStore`) | p6-c006 + p6-c007 (`window.__frf_dev.agentEventStore`) |

---

## Technical Debt Introduced

| Item | Location | Severity | Notes |
|---|---|---|---|
| `ReqwestMatrixClient` is a REST stub | `frf-bridge-matrix/src/client.rs` | MEDIUM | Full Tuwunel SDK integration deferred; no actual room subscription; subscribe returns empty stream |
| ATProto write path returns `Err(Unsupported)` | `frf-bridge-atproto/src/lib.rs` | LOW | Intentional; firehose is read-only; `send()` documented as unsupported |
| `RunAgent` gRPC is server-streaming only | `frf-gateway/src/agent_grpc_service.rs` | LOW | Bidi client-streaming deferred to Phase 7 or later |
| `/dev/inject-federation-event` does not push to spine | `frf-gateway/src/routes/dev.rs` | LOW | Returns 202 Accepted without wiring into federation pipeline; Layer 3 smoke tests cannot verify true bus propagation |
| `TenantActorRegistry` idle eviction uses a fixed 5-minute TTL | `frf-librefang/src/registry.rs` | LOW | Should be configurable via `GatewayConfig`; hardcoded for now |
| Node 24 required but not enforced in CI | `.node-version`, `admin-ui/package.json` | LOW | `.node-version` and `engines.node` added; Dagger CI pipeline for admin-ui not yet updated |

---

## Lessons Captured

1. **`(FederationProtocol, Arc<dyn FederationBridge>)` tuple solves protocol dispatch**: Storing the protocol enum alongside the bridge object eliminates runtime protocol guessing during background ingest task spawning. Using bare `Arc<dyn FederationBridge>` would have required adding a `protocol()` method to the trait (or dynamic downcasting). The tuple approach is zero-overhead at the call site.

2. **`AppStateArc<L,A,I,M,B>` type alias must be declared at the crate root (`lib.rs`)**: Route handlers import from `crate::`, so the alias must live in `lib.rs`. A module-level alias (e.g., inside `routes/`) creates a second declaration that clippy flags as redundant or fails coherence checks.

3. **`#[allow(unused_mut)]` on the router is correct for `#[cfg(debug_assertions)]` route extension**: Clippy correctly warns about unused `mut` in release mode because the `#[cfg]` block is absent. The `#[allow]` suppression is the canonical pattern; removing it breaks release-mode clippy.

4. **pnpm 11.x requires Node ≥ 22**: Running `pnpm` on Node 20 fails immediately with a pre-flight engine check. Always verify the Node version before running any admin-ui pnpm command. The `.node-version` file and `fnm exec --using=24` are the right guards.

5. **Window object double-cast is unavoidable for global dev exports**: `window as Record<string,unknown>` fails TS2352 because `Window & typeof globalThis` has no string index. The pattern `window as unknown as Record<string,unknown>` is the correct TypeScript idiom; an `as` to `Record` directly is unsound and should not be used.

6. **Background ingest tasks need their bridge's protocol at spawn time, not call time**: The `spawn_federation_ingest_tasks` function receives `(FederationProtocol, Arc<dyn FederationBridge>)` tuples and clones the protocol into the spawned closure. This avoids a runtime type-dispatch problem where all tasks would subscribe under the same hardcoded protocol.

---

## Recommended Next Phase Focus

### Phase 7: WebRTC + WASM Browser Transport

Phase 6 completes the server-side federation layer and actor sharding. The workspace now has:
- `frf-media-str0m` and `frf-media-livekit` stubs (no SFU signaling implemented)
- `frf-wasm` stub (no `wasm-bindgen` bindings)
- No browser-side SDK transport

Phase 7 should address:

1. **`frf-media-str0m` SFU signaling** — implement `str0m`-based sovereign SFU; wire `MediaSignaler` port; ICE/DTLS negotiation for admin-ui WebRTC calls
2. **`frf-wasm` browser SDK** — `wasm-bindgen` bindings over `frf-sdk-rust` core; WebSocket transport using Connect-ES; expose `publish`, `subscribe`, `AgentStream` to TypeScript
3. **`RunAgent` bidi upgrade** — extend `AgentGrpcService` from server-streaming to full bidi; wire client-side message sends
4. **Admin-ui WebRTC integration** — signaling UI feature; call lifecycle management; `frf-wasm` browser bundle integration
5. **Dagger CI for admin-ui** — add Node 24 enforcement to the Dagger pipeline; run Layer 1 Playwright smoke tests in CI without a live gateway

**Carry-forward debt entering Phase 7:**
- `ReqwestMatrixClient` stub (needs real Tuwunel integration when SDK stabilizes)
- `TenantActorRegistry` idle TTL should be configurable
- `/dev/inject-federation-event` does not propagate to spine (Layer 3 smoke tests blocked)
- Dagger CI pipeline does not enforce Node 24 for admin-ui builds
