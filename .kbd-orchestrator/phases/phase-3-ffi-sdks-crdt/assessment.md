# Assessment: Phase 3 — CRDT Core + Offline Persistence + FFI SDK Tier

> RFC-FRF-002 · Prometheus AGS
> Assessed: 2026-06-19
> Phase: phase-3-ffi-sdks-crdt

---

## 1. Phase Context

Phase 3 adds the CRDT engine, on-device op-log, and FFI SDK tier (Swift, Kotlin/Java, Dart).

Exit criterion: *"Mobile app edits offline, reconnects, converges; identical merge on Swift + Kotlin + Dart."*

Three open decisions carried from prior phases must be resolved **before implementation begins**:

1. **CRDT engine: Loro vs automerge-rs** — propagates into every FFI binding
2. **UniFFI version + language coverage** — confirmed 0.31.2; Swift + Kotlin supported natively; Java via Kotlin binding
3. **flutter_rust_bridge version** — confirmed 2.11.1 installed

---

## 2. Toolchain Inventory

### On-machine tools (confirmed present)

| Tool | Version | Status |
|---|---|---|
| Rust / cargo | 1.85+ | ✅ (MSRV pinned in workspace) |
| Swift | 6.3.2 (swiftlang) | ✅ macOS arm64 |
| Flutter | 3.45.0-beta | ✅ (beta channel — acceptable for FFI gen) |
| flutter_rust_bridge_codegen | 2.11.1 | ✅ installed |
| wasm-pack | 0.13.1 | ✅ installed (for `frf-wasm` if needed) |
| Go | 1.25.0 | ✅ |
| Node.js / pnpm | 24.x / 11.5.2 | ✅ |

### Missing / not confirmed

| Tool | Status | Action |
|---|---|---|
| `kotlinc` / Android SDK | ❌ not found on PATH | Install via `sdkman` or Android Studio for Kotlin build tests; UniFFI generates `.kt` files so CI only needs `kotlinc` to compile-check |
| `uniffi-bindgen` CLI | ✅ `uniffi_bindgen = "0.31.2"` on crates.io; run via `cargo run -p uniffi-bindgen` | No separate install needed |
| SurrealDB running locally | Not assessed — needed for `frf-store-surreal` integration tests | Start via Docker for integration tests |

### Confirmed latest versions (as of 2026-06-19)

| Package | Version |
|---|---|
| `loro` (crates.io) | 1.13.1 |
| `loro-ffi` (crates.io) | 1.13.1 |
| `automerge` (crates.io) | 0.10.0 |
| `redb` (crates.io) | 4.1.0 |
| `surrealdb` (crates.io) | 3.1.5 |
| `uniffi` (crates.io) | 0.31.2 |
| `flutter_rust_bridge` | 2.11.1 |

---

## 3. CRDT Engine Decision

**Status: OPEN — must be resolved at plan kickoff.**

| Factor | Loro 1.13.1 | automerge 0.10.0 |
|---|---|---|
| Rust crate maturity | ✅ active, 1.x stable | ✅ active, 0.x (pre-1.0) |
| FFI story | `loro-ffi` crate (same org, same version) — designed for Swift/Kotlin via UniFFI | `automerge` has no first-party UniFFI crate; would need manual FFI wrapping |
| `loro-ffi` UniFFI support | ✅ explicit: generates Swift + Kotlin bindings | ❌ not available out-of-box |
| Performance | Generally faster, especially rich-document range ops | Mature ecosystem, slower on large docs |
| Sync protocol | Loro Sync Protocol (binary, incremental, mergeable) | Automerge Sync Protocol (same shape) |
| WASM | ✅ `loro` has WASM targets | ✅ `automerge-wasm` exists |
| Ecosystem | Younger but purpose-built for mobile FFI | Older, broader community, automerge-repo |

**Recommendation for Phase 3: Loro.**

`loro-ffi` provides first-party UniFFI bindings matching our Swift/Kotlin requirement. Using it avoids hand-rolling FFI shims. The `automerge` route would require wrapping the core manually in `#[uniffi::export]` macros — equivalent work but with less upstream support. Commit this decision in `docs/decisions/adr-001-crdt-engine.md` before plan execution.

---

## 4. Codebase Inventory — What Exists vs. What Phase 3 Needs

### ✅ Already present (reuse, do not rebuild)

| Artifact | Location | Notes |
|---|---|---|
| `CrdtStore` port trait | `crates/frf-ports/src/crdt_store.rs` | `checkpoint`, `restore`, `purge` — engine-agnostic `Bytes` payload |
| `SyncOp` / `SyncOpKind` domain types | `crates/frf-domain/src/sync.rs` | Engine-agnostic delta: `payload: Vec<u8>` |
| `SyncService` proto | `proto/flint/v1/sync.proto` | Bidi RPC `Sync(stream SyncRequest) returns (stream SyncResponse)` |
| `SyncCheckpoint` proto message | `proto/flint/v1/sync.proto` | Maps to `CrdtSnapshot` in domain |
| `frf-ports` federation / media port stubs | `crates/frf-ports/src/{federation,media}.rs` | Pre-declared port traits for later phases |
| `PostgresCdcConsumer` | `crates/frf-postgres-cdc/src/consumer.rs` | **REVISED**: consumer loop is actually implemented and compiles (`cargo check -p frf-postgres-cdc` passes). Phase 1/2 debt was overstated — the `pg_walstream` crate provides the `LogicalReplicationStream` API; the `run_until_shutdown` loop is functional. Integration test requires a live Postgres with logical replication enabled. |

### ❌ Missing — Phase 3 must create

| Artifact | Target location | Notes |
|---|---|---|
| `frf-crdt` crate | `crates/frf-crdt/` | Loro engine adapter implementing `CrdtStore` + merge logic; encodes/decodes Loro binary as `SyncOp.payload` |
| `frf-store-redb` crate | `crates/frf-store-redb/` | On-device op-log: persist unsynced `SyncOp` queue; implement `OpStore` port; replay on reconnect |
| `OpStore` port trait | `crates/frf-ports/src/op_store.rs` | `queue_op`, `drain_pending`, `mark_synced` — on-device write-ahead ops before reconnect |
| `SyncUseCase` | `crates/frf-app/src/sync.rs` | Apply incoming CRDT delta, merge with local state, emit `SyncOp` back; wire `CrdtStore` + `OpStore` |
| `SyncGrpcService` | `crates/frf-gateway/src/sync_service.rs` | tonic impl of `SyncService` bidi stream; delegates to `SyncUseCase` |
| `frf-ffi` crate | `crates/frf-ffi/` | UniFFI scaffold; exports `subscribe`, `publish`, `sync_apply`, CRDT merge API to Swift/Kotlin |
| `frf-wasm` crate | `crates/frf-wasm/` | `wasm-bindgen` scaffold; optional in-browser CRDT; lower priority than FFI |
| Swift SDK | `sdks/swift/` | UniFFI-generated Swift bindings + XCFramework build script |
| Kotlin SDK | `sdks/kotlin/` | UniFFI-generated Kotlin bindings + Gradle wrapper |
| Dart SDK | `sdks/dart/` | flutter_rust_bridge 2.x generated bindings over `frf-ffi` core |
| `frf-store-surreal` crate | `crates/frf-store-surreal/` | Server-side CRDT checkpoint store on SurrealDB 3.x; implements `CrdtStore` |
| CI codegen step | `dagger/` | `buf generate` + `pnpm -r build` + UniFFI bindgen in Dagger pipeline |
| CRDT engine ADR | `docs/decisions/adr-001-crdt-engine.md` | Decision record for Loro vs automerge-rs |

### ⚠️ Partially present — needs extension

| Artifact | Current state | Phase 3 delta |
|---|---|---|
| `frf-gateway/src/grpc_service.rs` | `SpineService` impl only | Add `SyncGrpcService` tonic impl; register `SyncServiceServer` |
| `frf-gateway/src/main.rs` | Wires `IggyBroker`, `KetoAuthz`, `OryIdentity` | Add `SurrealCrdtStore` + `RedbOpStore` + `SyncUseCase` wiring |
| `frf-domain/src/sync.rs` | `SyncOp`, `SyncOpKind` types defined | No changes needed |
| `crates/frf-ports/src/lib.rs` | `CrdtStore`, `LogBroker`, `AuthzProvider`, `IdentityVerifier` | Add `OpStore` port; re-export |
| Workspace `Cargo.toml` | 9 member crates | Add `frf-crdt`, `frf-store-redb`, `frf-store-surreal`, `frf-ffi`, `frf-wasm` as members |

---

## 5. Dependency Analysis

### `frf-crdt` dependency chain

```
frf-domain (SyncOp, EntityId, TenantId, CrdtSnapshot)
frf-ports  (CrdtStore trait, PortError)
    ↑ implemented by
frf-crdt   (LoroCrdtStore + merge engine)
    ↑ imported by
frf-app    (SyncUseCase)
    ↑ imported by
frf-gateway (SyncGrpcService)
```

Loro crates to add to workspace:
```toml
loro = { version = "1.13.1" }
loro-ffi = { version = "1.13.1" }
redb = { version = "4.1.0" }
surrealdb = { version = "3.1.5" }
uniffi = { version = "0.31.2" }
```

### `frf-ffi` structure

`frf-ffi` exports a **thin** Rust API surface (not the full domain). Only stable,
FFI-safe types cross the boundary. Internal port traits and `Arc<>` adapters stay
Rust-side. The UniFFI `.udl` file defines the exported interface.

### `frf-store-redb` vs `frf-store-surreal`

- `frf-store-redb` — on-device embedded op-log (phone, desktop app); implements `OpStore`
- `frf-store-surreal` — server-side CRDT checkpoint store; implements `CrdtStore`

These are two different ports / adapters. Do not conflate.

---

## 6. Gap Summary (Ordered by Dependency)

| # | Gap | Severity | Phase |
|---|---|---|---|
| 1 | CRDT engine decision not committed | BLOCKER | Must be ADR before any code |
| 2 | `OpStore` port trait missing from `frf-ports` | HIGH | Needed before `frf-store-redb` |
| 3 | `frf-crdt` crate does not exist | HIGH | Core CRDT adapter |
| 4 | `frf-store-redb` crate does not exist | HIGH | On-device op-log |
| 5 | `frf-store-surreal` crate does not exist | HIGH | Server-side checkpoint store |
| 6 | `SyncUseCase` not in `frf-app` | HIGH | CRDT merge application logic |
| 7 | `SyncGrpcService` not in `frf-gateway` | HIGH | Server-side sync endpoint |
| 8 | `frf-ffi` crate does not exist | HIGH | FFI scaffold for Swift/Kotlin/Dart |
| 9 | Swift SDK not generated | HIGH | Mobile client |
| 10 | Kotlin SDK not generated | HIGH | Mobile client |
| 11 | Dart SDK not generated | HIGH | Flutter client |
| 12 | `kotlinc` not on PATH | MEDIUM | Needed for Kotlin binding compile-check in CI |
| 13 | `frf-wasm` crate does not exist | LOW | Optional browser CRDT; lower priority |
| 14 | CI codegen pipeline not in Dagger | MEDIUM | Carried from Phase 2 |
| 15 | `frf-postgres-cdc` integration test (live Postgres) | LOW | Crate compiles; needs infra to run |

---

## 7. Revised Debt from Phase 1/2

| Item | Revised Status |
|---|---|
| `frf-postgres-cdc` consumer loop "dead code" | **RETRACTED**: `run_until_shutdown` is implemented and compiles. The debt was that it lacked an *integration test* against live Postgres, not that it was unimplemented. This is LOW severity — the loop is ready; wire it to a gateway actor in Phase 7 when CDC infra is available. |
| `PublishUseCase` missing Keto authz | **RESOLVED** in p2-c001. |
| `SpineGrpcService.ack()` no-op stub | Carried to Phase 3 — resolve when Iggy offset-commit API is used. |

---

## 8. Open Questions for Plan

1. **CRDT engine decision** — confirm Loro as the choice; write ADR; plan can then proceed.
2. **`frf-wasm` priority** — include in Phase 3 or defer to Phase 4? Assessment recommendation: defer to Phase 4 (not on the critical path for mobile exit criterion).
3. **`frf-store-surreal` vs `frf-store-redb` in Phase 3** — both are needed for the full offline loop, but `frf-store-surreal` requires a live SurrealDB 3.x in tests. Recommend including a basic implementation with an integration-test gate (similar to CDC).
4. **UniFFI `.udl` vs proc-macro approach** — UniFFI 0.31 supports both `#[uniffi::export]` proc-macro and `.udl` file. Recommendation: proc-macro only (no `.udl` file) for simpler maintenance.
5. **`kotlinc` availability** — plan should note that generated `.kt` files need `kotlinc` to compile-check. If not available, defer Kotlin compile gate to CI container.

---

## 9. Assessment Conclusion

Phase 3 has **strong foundations** in the domain layer (`SyncOp`, `CrdtSnapshot`, port traits) and proto contract (`SyncService`). The CRDT engine, op-log, FFI scaffold, and all three mobile SDK bindings need to be built from scratch.

The CRDT engine decision is the **highest-priority unresolved item** — everything in Phase 3 fans out from it. The ADR should be the first change in the plan.

`frf-postgres-cdc` debt is retracted — the crate is functional, just untested against live infra.

Recommended plan order:
1. ADR: commit Loro as CRDT engine
2. `OpStore` port trait → workspace deps update
3. `frf-crdt` (LoroCrdtStore adapter)
4. `frf-store-redb` (on-device op-log)
5. `frf-store-surreal` (server checkpoint store)
6. `SyncUseCase` in `frf-app`
7. `SyncGrpcService` + gateway wiring
8. `frf-ffi` UniFFI scaffold
9. Swift SDK gen + XCFramework
10. Kotlin SDK gen
11. Dart SDK gen via flutter_rust_bridge
12. CI codegen pipeline in Dagger
13. Offline roundtrip integration test (Swift + Kotlin + Dart)
