# Assessment — Phase 7: WebRTC str0m SFU + WASM SDK Depth

> Generated: 2026-06-21 · Tool: kbd-assess

---

## Codebase Scan Summary

### Workspace Inventory (Phase 7 relevant crates)

| Crate | Status |
|---|---|
| `frf-domain` | EXISTS — no changes needed for Phase 7 goals |
| `frf-ports` | EXISTS — `MediaSignaler` and `FederationBridge` ports already defined |
| `frf-app` | EXISTS — no changes needed |
| `frf-proto` | EXISTS — `agent.proto` server-streaming only; `signal.proto` bidi |
| `frf-gateway` | EXISTS — `agent_grpc_service.rs` server-streaming; `routes/dev.rs` stub |
| `frf-agentproto` | EXISTS — `domain_to_proto` + `domain_from_proto` complete |
| `frf-librefang` | EXISTS — `TenantActorRegistry` implemented; 60s sweep hardcoded |
| `frf-media-livekit` | EXISTS — LiveKit adapter wired; `MediaSignaler` port implemented |
| **`frf-media-str0m`** | **MISSING — entire crate does not exist** |
| `frf-wasm` | EXISTS (skeleton) — `crdt_apply_delta` only; missing publish/subscribe/AgentStream |
| `frf-bridge-matrix` | EXISTS (stub) — `ReqwestMatrixClient` REST stub, no live room subscription |
| `frf-bridge-atproto` | EXISTS — Jetstream WS consumer; write path unsupported |

---

## Gap Analysis by Goal

### Goal 1: `frf-media-str0m` Sovereign SFU

**Status: ENTIRE CRATE MISSING**

`frf-media-str0m` appears in `CLAUDE.md` workspace structure and in `crates/` plans but has never been created. Only `frf-media-livekit` exists.

The `MediaSignaler` port is fully defined in `frf-ports`. The proto definition for `SignalService` is a bidi-streaming RPC over `signal.proto`. `frf-gateway/src/signal_grpc_service.rs` exists but dispatches to `frf-media-livekit` only.

**Work needed:**
1. Create `crates/frf-media-str0m/` implementing `MediaSignaler` via `str0m` crate
2. Add `str0m` to workspace deps (confirm latest crates.io version)
3. Implement `SfuSession` struct — ICE candidate exchange, DTLS handshake, media session lifecycle
4. Wire into gateway via feature gate (`frf-media-str0m` vs `frf-media-livekit` config)

---

### Goal 2: `frf-wasm` Browser SDK Depth

**Status: SKELETON — crdt_apply_delta only**

Current exports (confirmed):
- `crdt::crdt_apply_delta` — WASM wrapper over `frf-crdt`
- `publish::PublishClient` (bare `fetch`, no JWT, no Connect-ES)
- `subscribe::SubscribeClient` (raw WebSocket, no JWT, no retry)

Missing:
- `agent::AgentStream` — no bidi gRPC client for `RunAgent`
- JWT Bearer token support in all clients
- Connect-ES integration for unary RPCs
- Reconnection / backoff logic in `subscribe`
- `build_wasm.sh` — does not exist; admin-ui references it but nothing runs it
- Auto-generated TypeScript `.d.ts` from wasm-pack (currently hand-authored stubs)

**Work needed:**
1. Add `agent.rs` — WASM bidi gRPC stream via `@connectrpc/connect-web` (or `fetch` + streaming response)
2. Patch `publish.rs` + `subscribe.rs` to accept and forward `Authorization: Bearer <token>`
3. Add `build_wasm.sh` — `wasm-pack build --target web --out-dir ../../sdks/ts/frf-wasm`
4. Replace hand-authored `admin-ui/src/types/frf-wasm.d.ts` with wasm-pack generated types

---

### Goal 3: `RunAgent` Bidi Upgrade

**Status: SERVER SIDE COMPLETE — client side absent, not true bidi**

`crates/frf-gateway/src/agent_grpc_service.rs`:
- JWT boundary guard ✓
- Subscribe-time Keto check ✓
- Event filtering by `agent_id` + `session_id` ✓
- Server-streaming return type: `Pin<Box<dyn Stream<Item = Result<ProtoAgentEvent, Status>>>>` ✓

Gaps:
- Not bidi — client cannot send cancel/control signals back
- `proto/flint/v1/agent.proto` defines `rpc RunAgent(AgentRunRequest) returns (stream AgentEvent)` — proto must be upgraded to `rpc RunAgent(stream AgentRunRequest) returns (stream AgentEvent)` for true bidi
- No WASM client — impossible to call from browser today
- No `SpineSignalService` integration test against a live tonic server (Phase 4 debt)

**Open decision:** Whether to make `RunAgent` fully bidi (both directions streaming) or keep server-streaming and add a separate `CancelAgent` unary RPC. The simplest upgrade is a single `stream AgentRunRequest` on the request side.

**Work needed:**
1. Update `proto/flint/v1/agent.proto` → bidi streaming
2. Update `agent_grpc_service.rs` to handle incoming client messages (cancel, pause)
3. Add WASM client in `frf-wasm/src/agent.rs`

---

### Goal 4: Admin-UI WebRTC + WASM Integration

**Status: PARTIAL — signaling panel exists, no ICE/DTLS UI, no WASM build integration**

Current admin-ui signaling feature (`admin-ui/src/features/signaling/`):
- `CrdtDemoButton.tsx` — dynamically imports `frf-wasm` and calls `crdt_apply_delta` ✓
- `SignalingPanel.tsx` — exists; details unclear without full read
- `signalingStore.ts`, `signalingService.ts`, `useSignalingStream.ts` — present

Gaps:
- No `WebRtcPanel.tsx` — no ICE gathering, offer/answer exchange, or media stream UI
- No WASM subscription binding — `subscribe.rs` exposed but not connected to Zustand
- `@prometheusags/frf-sdk` workspace package referenced in `package.json` but package source unclear
- No `postinstall` or Makefile step to run `build_wasm.sh` before admin-ui starts
- `frf-wasm` type declarations hand-authored; missing `publish`, `subscribe`, `AgentStream` exports

**Work needed:**
1. Write `crates/frf-wasm/build_wasm.sh` (Goal 2 dependency)
2. Add `postinstall` hook in `admin-ui/package.json` or a Makefile target
3. Add `WebRtcPanel.tsx` component for peer connection lifecycle
4. Add `useWebRtc.ts` hook managing `RTCPeerConnection`
5. Wire `AgentStream` consumer into agent activity panel (using WASM SDK or gRPC-web client)

---

### Goal 5: Dagger CI Admin-UI + Node 24

**Status: PARTIAL — pnpm workspace build runs; no WASM build, no E2E, no typecheck stage**

`dagger/codegen.ts` current pipeline stages:
1. `rust-build` — cargo build frf-ffi
2. `uniffi-swift` — UniFFI diff check
3. `uniffi-kotlin` — UniFFI diff check
4. `frb-dart` — flutter_rust_bridge diff check
5. `buf-generate` — proto codegen
6. `pnpm-build` — `pnpm -r build` workspace

Node version in pipeline: `node:20-slim` (hardcoded — conflicts with `.node-version: 24`)

Gaps:
- **No WASM build stage** before `pnpm-build`; if `frf-wasm` SDK output doesn't exist the build fails silently
- **No E2E test stage** — Playwright never runs in CI
- **Node 20 hardcoded** — must be `node:24-slim` to match repo `.node-version`
- **No typecheck stage** separate from build

**Work needed:**
1. Update `dagger/codegen.ts` — change `node:20-slim` → `node:24-slim`
2. Add `wasm-build` stage before `pnpm-build` that runs `build_wasm.sh`
3. Add `e2e-smoke` stage — `pnpm --filter admin-ui exec playwright test --grep "Phase.*UI layer"`

---

### Goal 6: Federation Dev Injection Spine Wiring

**Status: STUB — 202 response only, no broker publish**

`crates/frf-gateway/src/routes/dev.rs` handler:
```rust
pub async fn inject_federation_event(Json(body): Json<InjectFederationEventRequest>) -> impl IntoResponse {
    tracing::debug!(protocol = %body.protocol, source = %body.source, "dev: federation event injection accepted");
    StatusCode::ACCEPTED
}
```

The handler does not accept `State(state)` — it has no access to the `AppState` or `LogBroker`. There is no way to call `broker.publish()` without adding the state extractor.

**Work needed:**
1. Add `State(state): State<AppStateArc<...>>` to the handler signature
2. Construct an `EventEnvelope` from the JSON body
3. Call `state.log_broker.publish(envelope).await`
4. Update generic bounds on the handler function

---

### Goal 7: `TenantActorRegistry` Idle TTL Configurability

**Status: COMPLETE WITH MINOR DEBT**

The `spawn_eviction_task(idle_secs: u64)` function takes idle duration as a parameter — it is **not hardcoded**. However:
- The sweep interval (60 seconds) is hardcoded internally
- The caller's default is unknown until we trace `main.rs` (or wherever the registry is created)

**Work needed:**
1. Add `sweep_interval_secs: u64` parameter to `spawn_eviction_task` (or read from `GatewayConfig`)
2. Trace callsite in `main.rs` — confirm what default idle TTL is passed

---

## Open Decisions (Resolve Before Plan)

| Decision | Impact | Recommendation |
|---|---|---|
| `str0m` crate version | Blocks Goal 1 | Check crates.io for latest stable; confirm DTLS + ICE-lite support |
| `RunAgent` bidi proto change | Blocks Goal 3 | Upgrade to bidi (`stream request → stream response`); single unary `CancelAgent` is less idiomatic |
| WASM AgentStream transport | Blocks Goal 2 + 3 | Use `@connectrpc/connect-web` server-streaming fetch for `RunAgent`; avoids gRPC-web HTTP/2 requirement in browser |
| `frf-media-str0m` feature gate | Blocks Goal 1 + 4 | Gate behind `str0m` Cargo feature in `frf-gateway`; default to livekit for dev |
| WASM build integration in monorepo | Blocks Goal 4 + 5 | `postinstall` in `admin-ui/package.json` is fragile; prefer Makefile target + Dagger stage |

---

## Dependency Order

```
Goal 5 (Dagger: Node 24 fix)       Goal 6 (dev/inject wiring)
    │                                      │
    │                                      │
Goal 2 (frf-wasm build script + SDK) ─────┘
    │
    │  depends on Goal 2's build_wasm.sh
    ▼
Goal 1 (frf-media-str0m)         Goal 3 (RunAgent bidi proto)
    │                                 │
    └──────────┬───────────────────────┘
               ▼
Goal 4 (admin-UI WebRTC + WASM integration)
               │
               ▼
Goal 7 (TTL configurability — low priority; can run parallel to all)
```

Goals 1 and 3 can be developed in parallel once their respective proto/crate decisions are made. Goal 4 is the sync point that requires both. Goal 2's build script is a prerequisite for Goal 4 and 5.

---

## Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| `str0m` API instability — crate is pre-1.0 | HIGH | Pin to specific version; write thin adapter layer; implement ICE-lite only |
| Proto bidi change breaks existing gRPC clients | MEDIUM | Version the proto file; add `AgentRunControl` message type; keep server-streaming as fallback |
| WASM build requires wasm-pack which may not be in CI | MEDIUM | Add wasm-pack install to Dagger `wasm-build` stage |
| `@connectrpc/connect-web` streaming fetch requires HTTP/2 | MEDIUM | Gateway can upgrade to H2C on port 9090; Axum supports it via `axum-server` |
| admin-ui `postinstall` adding WASM build breaks `pnpm install` in CI | LOW | Use `prepare` lifecycle (skipped in CI) or explicit Makefile target |

---

## Assessment Verdict

Phase 7 is **buildable** with one critical blocker (missing `frf-media-str0m`) and two moderate blockers (WASM SDK agent client, dev inject spine wiring). The `MediaSignaler` port, `frf-wasm` skeleton, and `frf-agentproto` conversion layer are all complete foundations.

**Planned change count:** 7 changes
- p7-c001: `frf-media-str0m` crate (str0m SFU adapter)
- p7-c002: `frf-wasm` SDK depth (agent stream + JWT + build script)
- p7-c003: `RunAgent` proto bidi upgrade + handler
- p7-c004: admin-ui WebRTC + WASM integration
- p7-c005: Dagger CI Node 24 + WASM build + E2E stage
- p7-c006: `/dev/inject-federation-event` spine wiring
- p7-c007: Phase 7 E2E smoke test
