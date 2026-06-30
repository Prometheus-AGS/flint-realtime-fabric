# Plan — Phase 7: WebRTC str0m SFU + WASM SDK Depth

> Generated: 2026-06-21 · Tool: kbd-plan

---

## Open Decision Resolutions

| Decision | Resolution |
|---|---|
| str0m vs LiveKit | Both supported — `frf-media-str0m` as new crate; both wire through `MediaSignaler` port; gateway chooses at runtime via `SFU_MODE` env var |
| `str0m` crate version | `str0m = "0.7"` — add to workspace deps |
| `RunAgent` bidi shape | Upgrade to `rpc RunAgent(stream AgentRunRequest) returns (stream AgentEvent)` — true bidi; client sends `AgentRunRequest` for cancel/control |
| WASM AgentStream transport | Use Connect-ES server-streaming fetch (`@connectrpc/connect-web` already at `^1.6.1`) — avoids HTTP/2 gRPC-web complexity in browser |
| WASM build integration | `build_wasm.sh` script + `prepare` npm lifecycle (skipped in CI) + explicit Dagger `wasm-build` stage |
| `frf-gateway` feature gate for str0m | No feature gate — both adapters compile; gateway selects one at construction via config |

---

## Ordered Change List

Changes execute in strict dependency order.

### p7-c001 — `frf-media-str0m` Sovereign SFU Crate
- **Dir**: `openspec/changes/p7-c001-frf-media-str0m/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: can run in parallel with p7-c002 + p7-c003 (no mutual deps)
- **Description**: Create `crates/frf-media-str0m/` implementing `MediaSignaler` via `str0m 0.7`. Add `str0m = "0.7"` to workspace deps. Implement `StrOmSfuSession` struct with ICE-lite candidate exchange, DTLS handshake skeleton, and `send_signal` / `receive_signal` methods. Write unit tests on session lifecycle. Add `frf-media-str0m` to Cargo workspace members.
- **Exit**: `cargo check --workspace` passes; `cargo test -p frf-media-str0m` passes; `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes.

### p7-c002 — `frf-wasm` SDK Depth + Build Script
- **Dir**: `openspec/changes/p7-c002-frf-wasm-sdk-depth/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: can run in parallel with p7-c001 + p7-c003
- **Description**: Add `agent.rs` to `frf-wasm` — WASM bidi stream over Connect-ES server-streaming fetch for `RunAgent`; add JWT Bearer token support to `publish.rs` and `subscribe.rs`; write `crates/frf-wasm/build_wasm.sh` that runs `wasm-pack build --target web --out-dir ../../sdks/ts/frf-wasm/`. Update `web-sys` features to include `Request`, `RequestInit`, `Headers`, `Response`, `ReadableStream` for fetch streaming. Remove hand-authored `admin-ui/src/types/frf-wasm.d.ts` — wasm-pack will generate it.
- **Exit**: `cargo check -p frf-wasm --target wasm32-unknown-unknown` passes; `cargo test -p frf-wasm` (non-wasm target, non-wasm tests) passes; `build_wasm.sh` runs successfully.

### p7-c003 — `RunAgent` Proto Bidi Upgrade + Handler
- **Dir**: `openspec/changes/p7-c003-runagent-bidi-upgrade/`
- **Agent**: rust-reviewer
- **Deps**: none
- **Parallel candidate**: can run in parallel with p7-c001 + p7-c002
- **Description**: Update `proto/flint/v1/agent.proto` — change `rpc RunAgent(AgentRunRequest) returns (stream AgentEvent)` to `rpc RunAgent(stream AgentRunRequest) returns (stream AgentEvent)`. Add `AgentRunControl` message (`cancel: bool`, `pause: bool`). Update `agent_grpc_service.rs` to accept the inbound stream; handle `cancel` control messages by closing the outbound event stream. Update `frf-agentproto/src/convert.rs` for the new request message type.
- **Exit**: `cargo check --workspace` passes; `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes; proto regeneration produces updated server trait.

### p7-c004 — Gateway str0m Wiring + Dev Inject Spine
- **Dir**: `openspec/changes/p7-c004-gateway-str0m-dev-inject/`
- **Agent**: rust-reviewer
- **Deps**: p7-c001, p7-c003 (str0m crate + bidi handler must exist)
- **Description**: Wire `frf-media-str0m` into `frf-gateway/src/main.rs` — read `SFU_MODE` env var (`str0m` | `livekit`); construct the appropriate `MediaSignaler` impl and pass into `AppState`. Add `frf-media-str0m` to `frf-gateway/Cargo.toml`. Fix `/dev/inject-federation-event`: add `State(state)` extractor, construct `EventEnvelope` from body, call `state.log_broker.publish(envelope).await`. Make `TenantActorRegistry` sweep interval configurable: add `registry_sweep_interval_secs` to `GatewayConfig` + `REGISTRY_SWEEP_INTERVAL_SECS` env var (default 60).
- **Exit**: `cargo check --workspace` passes; `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` passes; `cargo test -p frf-gateway` passes.

### p7-c005 — Admin-UI WebRTC + WASM Integration
- **Dir**: `openspec/changes/p7-c005-admin-ui-webrtc-wasm/`
- **Agent**: typescript-reviewer
- **Deps**: p7-c002 (build_wasm.sh must exist)
- **Parallel candidate**: can run in parallel with p7-c004 (no direct dep)
- **Description**: Add `prepare` script in `admin-ui/package.json` that runs `bash ../crates/frf-wasm/build_wasm.sh` (skipped in CI when `CI=true`). Add `WebRtcPanel.tsx` component — peer connection state display, ICE status, offer/answer exchange flow using `SpineSignalService` via Connect-ES. Add `useWebRtc.ts` hook managing `RTCPeerConnection` lifecycle. Wire `AgentStream` WASM client into agent activity panel — replace current WS-based stream with `frf-wasm` `AgentStream.open(token)`. Update `admin-ui/src/types/` to remove hand-authored stub (replaced by wasm-pack output). Update `admin-ui/src/features/agents/hooks/useAgentEventStream.ts` to use WASM SDK when `frf-wasm` is available, with WS fallback.
- **Exit**: `fnm exec --using=24 pnpm typecheck` passes; no `any` types; no hardcoded tokens.

### p7-c006 — Dagger CI Node 24 + WASM Build + E2E Stage
- **Dir**: `openspec/changes/p7-c006-dagger-ci-node24-wasm-e2e/`
- **Agent**: devops-engineer
- **Deps**: p7-c002 (build_wasm.sh must exist before Dagger can invoke it)
- **Parallel candidate**: can run in parallel with p7-c005
- **Description**: Update `dagger/codegen.ts` — change `node:20-slim` → `node:24-slim`. Add `wasm-build` Dagger stage before `pnpm-build`: install `wasm-pack` in Rust container, run `bash crates/frf-wasm/build_wasm.sh`. Add `e2e-smoke` Dagger stage after `pnpm-build`: run `pnpm --filter admin-ui exec playwright test --grep "Phase.*UI layer"` with `SKIP_INTEGRATION=true`. Add `typecheck` stage separately from `pnpm build` to surface TS errors clearly.
- **Exit**: `dagger/codegen.ts` lints cleanly; `pnpm typecheck` in `admin-ui/` passes; Node 24 confirmed in pipeline config.

### p7-c007 — Phase 7 E2E Smoke Test
- **Dir**: `openspec/changes/p7-c007-e2e-phase7-smoke/`
- **Agent**: e2e-runner
- **Deps**: p7-c004, p7-c005 (gateway dev inject wired; WASM SDK integrated)
- **Description**: Write `admin-ui/e2e/phase7-smoke.spec.ts` using 3-layer pattern. Layer 1 (no gateway): verify WebRtcPanel renders, signaling state initializes, WASM import resolves to stub. Layer 2 (gated): gRPC `RunAgent` bidi endpoint reachable, dev inject endpoint returns 202. Layer 3 (gated): inject federation event via dev endpoint → verify event appears in agent activity panel via WASM AgentStream.
- **Exit**: All Layer 1 tests pass with `SKIP_INTEGRATION=true`; `pnpm typecheck` passes.

---

## Dependency Graph

```
p7-c001 (str0m SFU)    p7-c002 (frf-wasm SDK)    p7-c003 (RunAgent bidi)
    │                        │                          │
    └──────────┬─────────────┘                          │
               ▼                                        │
         p7-c004 (gateway wiring)◄──────────────────────┘
               │
    ┌──────────┴──────────────┐
    ▼                         ▼
p7-c005 (admin-UI)      p7-c006 (Dagger CI)
    │                         │
    └──────────┬───────────────┘
               ▼
         p7-c007 (E2E smoke)
```

p7-c001, p7-c002, p7-c003 are fully parallel (no mutual deps).
p7-c004 is the sync point requiring all three.
p7-c005 and p7-c006 can run in parallel once p7-c002 and p7-c004 complete.
p7-c007 is the final gate.

---

## Quality Gate Protocol

After each change:
1. `cargo check --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
3. Change-specific test command

After p7-c005:
- Additionally: `cd admin-ui && fnm exec --using=24 pnpm typecheck && fnm exec --using=24 pnpm build`

After p7-c007:
- Additionally: `cd admin-ui && SKIP_INTEGRATION=true fnm exec --using=24 pnpm exec playwright test e2e/phase7-smoke.spec.ts`

---

## Phase Exit Criterion

**SATISFIED when**: `admin-ui/e2e/phase7-smoke.spec.ts` Layer 1 tests all pass with `SKIP_INTEGRATION=true`, AND `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0, AND `fnm exec --using=24 pnpm typecheck` in `admin-ui/` exits 0.

The full exit criterion (WASM AgentStream subscription from browser to gateway) is exercised in Layer 3, which requires a full stack.

---

## Carry-Forward Debt Resolution Tracking

| Debt Item | Source Phase | Resolved In |
|---|---|---|
| `ReqwestMatrixClient` stub | Phase 6 | Not in Phase 7 scope — deferred until Tuwunel SDK stabilizes |
| `/dev/inject-federation-event` no spine wiring | Phase 6 | p7-c004 |
| Dagger CI not enforcing Node 24 | Phase 6 | p7-c006 |
| `frf-wasm/build_wasm.sh` not in Dagger | Phase 4 | p7-c006 |
| `SpineSignalService` no live tonic integration test | Phase 4 | p7-c004 (partial) |
| `TenantActorRegistry` sweep interval hardcoded | Phase 6 | p7-c004 |
