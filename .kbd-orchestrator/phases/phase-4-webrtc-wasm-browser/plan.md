# Plan — Phase 4: WebRTC Media Plane + WASM Browser SDK

> RFC-FRF-002 · Prometheus AGS
> Planned: 2026-06-19
> Change backend: OpenSpec

---

## ADR Recorded

**WebRTC SFU: LiveKit (hosted)** — confirmed by operator before planning.
`SFU_MODE_HOSTED = 2` in frozen `signal.proto`. `frf-media-str0m` is out of
Phase 4 scope. The sovereign str0m adapter may be revisited in a future phase
if a self-hosted SFU requirement emerges.

---

## Change Summary

| # | Change ID | Title | Depends on | Est. complexity |
|---|---|---|---|---|
| 1 | `p4-c001-cdc-gateway-wiring` | Wire PostgresCdcConsumer into frf-gateway | p1-c006, p1-c007 | Small |
| 2 | `p4-c002-frf-media-livekit` | LiveKit hosted SFU adapter (new crate) | p2-c007, p3-c007 | Medium |
| 3 | `p4-c003-signal-grpc-service` | SpineSignalService bidi gRPC in gateway | p4-c002 | Medium-High |
| 4 | `p4-c004-frf-wasm` | frf-wasm: wasm-bindgen browser SDK (new crate) | p3-c003, p2-c005 | Medium |
| 5 | `p4-c005-admin-ui-signaling` | Admin UI signaling feature + CRDT demo | p4-c003, p4-c004 | Medium |
| 6 | `p4-c006-e2e-browser-smoke` | Phase 4 exit-criterion E2E browser smoke test | p4-c001, p4-c004, p4-c005 | Small |

**Total: 6 changes, 32 tasks**

---

## Ordering Rationale

Changes follow the dependency graph: CDC wiring (1) is independent and closes HIGH
debt immediately. LiveKit adapter (2) must exist before the signaling gRPC service
(3) can be mounted. WASM SDK (4) is independent of signaling and can be built in
parallel with (2)+(3). Admin UI (5) requires both the mounted service and the WASM
SDK. E2E smoke (6) is last and requires CDC (1), WASM (4), and UI (5) all green.

Parallel execution is possible for changes (2) and (4) — they have no shared
dependency. An executor may begin (4) immediately after (1) while (2) and (3) are
in progress.

---

## New Crates

| Crate | Type | Port implemented |
|---|---|---|
| `crates/frf-media-livekit` | Infrastructure adapter | `MediaSignaling` (new in `frf-ports`) |
| `crates/frf-wasm` | Interface / WASM SDK | N/A (browser client interface) |

Both crates must be added to the workspace `Cargo.toml` `members` array.

---

## Workspace `Cargo.toml` Additions

```toml
# New workspace.dependencies entries:
livekit = "0.4"          # confirm current version at execution
wasm-bindgen = "0.2.100"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["WebSocket", "console"] }
```

---

## Security Constraints (must hold across all changes)

- JWT verified at gateway boundary (SpineSignalService, p4-c003) — never pass
  unverified claims downstream
- Keto `check(subject, "view", object_id)` per event fan-out — existing
  `SubscribePipeline` handles this; signal relay adds Keto tenant membership check
- Never log JWT payloads, relation tuples, or tenant identifiers in debug output
- `SignalEnvelope` contents are never persisted (spine delivers transiently)
- Room IDs are namespaced by tenant UUID in `LiveKitSignaling` adapter

---

## Open Decision (verify at execution)

| Item | Action | Needed before |
|---|---|---|
| Confirm `@connectrpc/connect` 1.6.1 + `wasm-bindgen` 0.2.100 compatibility | `npm show @connectrpc/connect version` + `cargo search wasm-bindgen` | p4-c004 |
| Confirm `livekit` crate current version | `cargo search livekit` | p4-c002 |

---

## Phase 4 Exit Criterion

> Browser client (via `frf-wasm` + Connect-ES) subscribes to an entity stream,
> edits offline, and reconnects via WebSocket mux; CDC event from PostgreSQL WAL
> flows end-to-end through the spine to the browser.

Satisfied by: p4-c001 (CDC end-to-end) + p4-c004 (WASM subscribe) + p4-c006
(E2E smoke confirms the full path).

---

## Phase Gate

Per protocol: halt at Phase 4 boundary after p4-c006 passes. Report to operator.
Do not begin Phase 5 (agent protocols / BossFang) without explicit approval.
