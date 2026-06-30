# Assessment — Phase 4: WebRTC Media Plane + WASM Browser SDK

> RFC-FRF-002 · Prometheus AGS
> Assessed: 2026-06-19
> Assessor: kbd-assess

---

## Prerequisites Status

| Prerequisite | Status | Notes |
|---|---|---|
| WebRTC SFU decision | **RESOLVED** | **LiveKit** (hosted). Operator confirmed. |
| wasm-bindgen + Connect-ES version | **PENDING** | Must confirm before frf-wasm scaffold |
| frf-postgres-cdc WAL loop | **MOSTLY DONE** | Consumer loop is fully implemented — see below |

### SFU Decision: LiveKit (Hosted)

`signal.proto` already defines `SFU_MODE_HOSTED = 2` (LiveKit) and `SFU_MODE_SOVEREIGN = 1`
(str0m). With LiveKit confirmed, the sovereign str0m adapter (`frf-media-str0m`) is out of Phase 4
scope. The Phase 4 media adapter is **`frf-media-livekit`** only.

The `SignalService.Signal` bidi RPC is already defined in `proto/flint/v1/signal.proto`. The proto
is frozen — no changes needed.

---

## Gap Analysis

### Gap 1: `frf-postgres-cdc` — Actual vs Assumed State

**Assessment finding: WAL consumer loop IS implemented.** Prior Phase 3 reflection flagged this as
HIGH debt (stub), but inspection shows `crates/frf-postgres-cdc/src/consumer.rs` contains a
complete `run_until_shutdown` loop with:

- `LogicalReplicationStream` setup via `pg_walstream 0.6`
- `ensure_replication_slot` + `start` + `into_stream`
- `tokio::select!` loop handling shutdown signal and `next_event`
- `translate_event` dispatching INSERT/UPDATE/DELETE → `decode_insert/update/delete`
- `broker.publish(envelope)` + `update_applied_lsn` + `offset.next()`

`decode.rs` has full column-to-JSON decode logic with unit tests for INSERT/UPDATE/DELETE/null-PK/
invalid-UUID cases.

**Actual gap:** `frf-postgres-cdc` is NOT wired into `frf-gateway`. `main.rs` does not construct a
`PostgresCdcConsumer` or spawn its task. This is a wiring gap, not an implementation gap.

**Effort:** Small — add `frf-postgres-cdc` to workspace members if missing, wire consumer into
gateway startup, spawn `run_until_shutdown` as a background Tokio task.

**Note:** `frf-postgres-cdc` is already in the workspace `members` list.

---

### Gap 2: `frf-wasm` — Does Not Exist

**Status: Missing entirely.**

No `crates/frf-wasm/` directory. The `wasm32-unknown-unknown` target is already in
`rust-toolchain.toml`. The workspace `Cargo.toml` does not list `frf-wasm`.

**What is needed:**
- New crate `crates/frf-wasm` with `wasm-bindgen`, `js-sys`, `web-sys` deps
- Connect-ES browser transport binding (confirm version: `@connectrpc/connect` + `@bufbuild/protobuf`)
- Exports: `subscribe(channel, callback)`, `publish(channel, payload)`, `crdt_apply_delta(delta)`
- wasm-pack build configuration
- `sdks/ts/` output directory for generated `.js` + `.d.ts`

**Effort:** Medium. Pure new crate, no existing code to modify.

---

### Gap 3: `frf-media-livekit` — Does Not Exist

**Status: Missing entirely.**

No `crates/frf-media-livekit/` directory. No LiveKit dependencies in `Cargo.toml`.

**What is needed:**
- New crate `crates/frf-media-livekit` implementing a `MediaSignaling` port (or using the
  `SignalBroker` port pattern consistent with `frf-ports`)
- LiveKit Rust SDK dependency: `livekit` crate (or HTTP-based LiveKit Cloud API client)
- `RoomJoin` / `RoomLeave` / ICE candidate relay via `SignalEnvelope` on the spine
- `SFU_MODE_HOSTED` routing in the gateway's SignalService handler

**Effort:** Medium. LiveKit has a Rust SDK; the adapter wraps it behind the port trait.

---

### Gap 4: Signaling gRPC Service in `frf-gateway` — Not Wired

**Status: Proto defined, service not implemented.**

`signal.proto` defines `SignalService { rpc Signal(stream SignalEnvelope) returns (stream SignalEnvelope) }`.
`frf-proto` will code-generate the tonic server/client stubs from this. The gateway
`build_router` does not yet mount a gRPC service or tonic `Router`.

**What is needed:**
- `SpineSignalService` struct implementing tonic's generated `SignalServiceServer` trait
- bidi streaming handler: receive `SignalEnvelope` frames, route to LiveKit adapter, fan out to
  subscriber sessions via the spine
- Mount as a tonic service on the gateway (axum ↔ tonic composition via `axum::Router::route_service`)
- Gateway `AppState` extended with `Arc<dyn MediaSignaling>`

**Effort:** Medium-High. Bidi streaming with proper fan-out and session routing.

---

### Gap 5: Admin UI — Media Demo Feature Missing

**Status: `admin-ui/src/features/` only has `entities/`. No `signaling/` or `media/` feature.**

**What is needed:**
- New feature `admin-ui/src/features/signaling/` with:
  - `SignalingPanel` component: shows room join/leave state, ICE status
  - `useSignalingStream` hook: Connect-ES bidi stream to `SignalService`
  - Zustand store: active room, SFU mode, session participants
  - wasm-bound `crdt_apply_delta` call demo (entity + signaling together)
- Wire into existing entity stream page or as a separate demo page

**Effort:** Medium. UI only — no new backend wiring beyond what Gap 4 provides.

---

### Gap 6: Connect-ES + wasm-bindgen Version Confirmation

**Status: OPEN — must resolve at Phase 4 kickoff before `frf-wasm` scaffold.**

- `@connectrpc/connect` current stable: **1.6.1** (June 2026)
- `@bufbuild/protobuf`: **2.3.0**
- `wasm-bindgen`: current stable **0.2.100** (compatible with Rust 1.85)
- `wasm-pack`: **0.13.1**

These are current as of assessment date. Verify with `npm show @connectrpc/connect version` and
`cargo search wasm-bindgen` at execution time before writing lock files.

---

## Codebase State Summary

| Crate / Artifact | Exists | State |
|---|---|---|
| `crates/frf-postgres-cdc` | ✅ | Fully implemented; needs gateway wiring |
| `crates/frf-wasm` | ❌ | Missing — Phase 4 creates it |
| `crates/frf-media-livekit` | ❌ | Missing — Phase 4 creates it |
| `crates/frf-media-str0m` | ❌ | **Out of Phase 4 scope** (LiveKit chosen) |
| SignalService gRPC handler | ❌ | Proto frozen; handler not written |
| `admin-ui/src/features/signaling/` | ❌ | Missing — Phase 4 creates it |
| `proto/flint/v1/signal.proto` | ✅ | Frozen, correct |
| `wasm32-unknown-unknown` target | ✅ | In rust-toolchain.toml |
| `sdks/ts/` WASM output dir | ❌ | Does not exist yet |

---

## Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| LiveKit Rust SDK maturity | MEDIUM | Use `livekit` crate (official LiveKit Rust SDK); fallback to HTTP/REST LiveKit Cloud API if bidi WebSocket SDK is insufficient |
| wasm-bindgen async compatibility | MEDIUM | Test with `wasm-bindgen-futures`; use `spawn_local` for async JS interop |
| tonic ↔ axum bidi composition | MEDIUM | Use `tonic::transport::Server` behind `axum::Router::route_service` — known pattern, confirmed working with axum 0.8 |
| Connect-ES browser transport CORS | LOW | Configure `tower-http` CORS to allow Connect-ES `application/connect+proto` content-type |
| frf-crdt WASM build | LOW | Loro 1.13.1 already supports wasm32 target; test with `wasm-pack test` |

---

## Change Order (Recommended)

1. **Wire `frf-postgres-cdc` into gateway** — closes HIGH-priority debt, unblocks end-to-end CDC flow
2. **Scaffold `frf-media-livekit`** — creates the port + LiveKit adapter, required before signal service
3. **Implement `SpineSignalService` in `frf-gateway`** — mounts bidi gRPC, routes via LiveKit adapter
4. **Scaffold `crates/frf-wasm`** — wasm-bindgen crate with subscribe/publish/crdt_apply_delta
5. **Admin UI signaling feature** — React 19 component wiring Connect-ES bidi stream + entity stream
6. **E2E smoke test** — browser WASM client subscribes → CDC event flows end-to-end

This ordering respects dependency flow: CDC wiring before E2E, LiveKit adapter before signaling
service, WASM SDK before UI demo.

---

## Open Decision

| Decision | Owner | Needed Before |
|---|---|---|
| Confirm Connect-ES 1.6.1 + wasm-bindgen 0.2.100 compatibility | Engineer | Change 4 (frf-wasm) |

---

## Exit Criterion Readiness

The Phase 4 exit criterion is:
> "Browser client (via `frf-wasm` + Connect-ES) subscribes to an entity stream, edits offline,
> and reconnects via WebSocket mux; CDC event from PostgreSQL WAL flows end-to-end through the
> spine to the browser."

After the 6 changes above, this criterion is satisfiable. No proto changes needed (frozen at
`proto-v1`). No new workspace infrastructure needed beyond the two new crates.
