# Reflection — Phase 3: CRDT Core + Offline Persistence + FFI SDK Tier

> RFC-FRF-002 · Prometheus AGS
> Reflected: 2026-06-19
> Phase: phase-3-ffi-sdks-crdt

---

## Goal Achievement

| Goal | Status | Notes |
|---|---|---|
| CRDT engine decision (Loro vs automerge-rs) | ✅ MET | Loro 1.13.1 chosen; ADR committed in p3-c001 |
| `frf-crdt` — Loro CrdtStore adapter | ✅ MET | `apply_delta`, `InMemoryCrdtStore`, `LoroDeltaApplier` implemented and tested |
| `frf-store-redb` — on-device op-log | ✅ MET | Composite key layout `entity[16]\|tenant[16]\|seq[8 BE]`; 4 async tests passing |
| `frf-store-surreal` — server checkpoint store | ✅ MET | `SurrealCrdtStore` with hex-encoded snapshots; UPSERT/SELECT/DELETE wired |
| `SyncUseCase` in `frf-app` | ✅ MET | `apply_server_delta`, `queue_local_op`, `pending_ops`, `snapshot`; mock-based tests |
| `SyncGrpcService` in `frf-gateway` | ✅ MET | Bidi streaming + `get_checkpoint`; wired via `SyncService` tonic trait |
| `frf-ffi` UniFFI scaffold | ✅ MET | proc-macro approach (`uniffi::setup_scaffolding!`); `crdt_apply_delta`, `crdt_new_snapshot`, `crdt_snapshot_version` |
| Swift SDK | ✅ MET | `Package.swift` + `build_xcframework.sh` (3 targets: iOS arm64, iOS sim x86, macOS arm64) |
| Kotlin SDK | ✅ MET | Gradle 8.7 wrapper + `build_jni.sh`; JNA dep wired; `.gitignore` for generated artifacts |
| Dart SDK | ✅ MET | `pubspec.yaml` + `build_dart.sh` (flutter_rust_bridge_codegen 2.11.1) |
| CI codegen pipeline | ✅ MET | Dagger TypeScript pipeline: 6 stages (rust-build, uniffi-swift, uniffi-kotlin, frb-dart, buf-generate, pnpm-build) |
| Offline CRDT roundtrip exit criterion | ✅ MET | 3 integration tests passing: offline_roundtrip_converges, apply_delta_idempotent, three_way_merge_converges |

**Goal achievement: 12/12 MET (100%)**

---

## Delivered Changes (13/13)

| Change | Artifact | Status |
|---|---|---|
| p3-c001-crdt-adr | `docs/decisions/crdt-engine.md` + workspace Loro deps | ✅ DONE |
| p3-c002-op-store-port | `frf-ports`: `OpStore` trait + `PendingOp` type | ✅ DONE |
| p3-c003-frf-crdt | `crates/frf-crdt`: `apply.rs`, `merge.rs`, `store.rs`, `error.rs` | ✅ DONE |
| p3-c004-frf-store-redb | `crates/frf-store-redb`: `store.rs`, `key.rs`, `error.rs` | ✅ DONE |
| p3-c005-frf-store-surreal | `crates/frf-store-surreal`: `store.rs`, `model.rs`, `error.rs` | ✅ DONE |
| p3-c006-sync-use-case | `crates/frf-app/src/sync.rs` — `SyncUseCase<S,O,A>` | ✅ DONE |
| p3-c007-sync-grpc-service | `crates/frf-gateway/src/sync_grpc_service.rs` | ✅ DONE |
| p3-c008-frf-ffi | `crates/frf-ffi/src/{lib,crdt,error}.rs` | ✅ DONE |
| p3-c009-sdk-swift | `sdks/swift/Package.swift`, `build_xcframework.sh`, `.gitignore` | ✅ DONE |
| p3-c010-sdk-kotlin | `sdks/kotlin/` — Gradle wrapper + `build_jni.sh` + `.gitignore` | ✅ DONE |
| p3-c011-sdk-dart | `sdks/dart/` — `pubspec.yaml`, `build_dart.sh`, `GENERATED.md` | ✅ DONE |
| p3-c012-ci-codegen | `dagger/codegen.ts`, `package.json`, `tsconfig.json` | ✅ DONE |
| p3-c013-offline-roundtrip | `crates/frf-crdt/tests/roundtrip.rs` + `tests/crdt/` smoke tests | ✅ DONE |

---

## Artifact Quality Summary

No artifact-refiner QA gate is wired for this project (`.refiner/` directory absent).
Quality was validated through:

| Quality Gate | Result |
|---|---|
| `cargo check --workspace` | ✅ PASS (clean, ~98s on cold cache) |
| `cargo test -p frf-crdt` | ✅ PASS (unit + integration, 7 tests) |
| `cargo test -p frf-crdt --test roundtrip` | ✅ PASS (3/3) |
| Architecture constraint (no adapter imports in frf-domain/frf-app) | ✅ PASS (verified at Cargo dep level) |
| `#[non_exhaustive]` on public enums | ✅ PASS |
| No `unwrap()`/`expect()` in library crates | ✅ PASS (only in tests) |
| File size limit (≤500 lines) | ✅ PASS (all files within limit) |

**Recurring issues caught and fixed during execution:**

- **`PortError::Internal` does not exist** — Used `Transport` variant throughout; caught at compile time.
- **`EntityId::new()` takes no args** — UUID v4 no-arg constructor; fixed in `frf-crdt` and `frf-store-redb` tests.
- **Byte-identity vs semantic convergence** — Loro snapshots embed per-export metadata (checksums). Integration test corrected to assert semantic equality (same resolved value per key) rather than byte identity.
- **redb missing trait imports** — `ReadableDatabase` + `ReadableTable` required explicitly; not re-exported from crate root.
- **`LoroValue::String(Arc<str>)`** — `into_value()` returns `Result<LoroValue, ValueOrContainer>`; helper `read_str` introduced to simplify test assertions.

---

## Technical Debt Introduced

| Debt Item | Severity | Carried to |
|---|---|---|
| `kotlinc` not on PATH — Kotlin compile-check deferred to CI container | LOW | Phase 7 (release pipeline) |
| `frf-wasm` (browser WASM SDK) not scaffolded — deferred per plan | LOW | Phase 4 |
| `frf-postgres-cdc` WAL consumer still stub (Phase 1+2 HIGH debt) | HIGH | Phase 4 |
| Swift/Kotlin/Dart smoke tests in `tests/crdt/` are documentation-only | LOW | Phase 7 (CI integration tests) |
| `SurrealDB` integration tests require a running SurrealDB instance — no Docker fixture wired | MEDIUM | Phase 4 |
| `pending_changes` field in `progress.json` not cleaned up (stale entries from c006–c008) | LOW | housekeeping |

---

## Lessons Captured

1. **Loro snapshot byte format is non-deterministic across exports** — Contains session-local metadata (checksums, timestamps per-export). Integration tests must assert *semantic convergence* (same resolved value), not byte equality. This is fundamentally different from deterministic CRDTs like Automerge's op-log format.

2. **UniFFI 0.31.2 proc-macro approach eliminates build.rs** — `uniffi::setup_scaffolding!("frf")` + `#[uniffi::export]` on each function is the correct path for 0.31.x. No `.udl` file, no `build.rs` `generate_scaffolding()`. The `[build-dependencies]` section is unused.

3. **redb 4.x requires explicit trait imports for read operations** — `ReadableDatabase` and `ReadableTable` must be in scope at the use site; they are not re-exported from the crate root. This is a footgun for new users.

4. **`SurrealValue` derive (surrealdb 3.x)** — SurrealDB 3.x uses `SurrealValue` (not `serde::DeserializeOwned`) for `.take()` results. `#[derive(SurrealValue)] #[surreal(crate = "surrealdb::types")]` is required.

5. **Hexadecimal encoding for binary in SurrealDB** — Storing Loro CRDT bytes as `Vec<u8>` in SurrealDB 3.x proved complex due to type serialization. Hex-encoding to `String` (via `hex = "0.4"`) and decoding on read is a clean, robust pattern.

6. **Phase-3 dependency chain was strictly serial from frf-ffi onward** — The three SDK changes (Swift/Kotlin/Dart) were independent and could have been parallelized. Executed sequentially due to session context limits; future phases should exploit parallel execution more aggressively.

---

## Open Decisions for Phase 4

| Decision | Must Resolve Before |
|---|---|
| `frf-wasm` WebAssembly WASM SDK strategy (`wasm-bindgen` vs `tsify` vs `wasm-pack` + `Connect-ES`) | Phase 4 kickoff |
| WebRTC SFU choice: str0m (sovereign) vs LiveKit (hosted) for the media plane | Phase 4 kickoff |
| BossFang actor topology and `ractor` version | Phase 5 kickoff |
| Federation bridge priority: Tuwunel (Matrix) vs Tranquil (ATProto) first | Phase 6 kickoff |
| `frf-postgres-cdc` completion (HIGH debt — blocks CDC-sourced entity events) | Phase 4, change 1 |

---

## Recommended Next Phase

**Phase 4: WebRTC Media Plane + WASM Browser SDK**

Focus: Close the `frf-wasm` gap (browser transport via `wasm-bindgen` + Connect-ES), implement the `frf-media-str0m` sovereign SFU signaling adapter, and wire `frf-postgres-cdc` (HIGH debt item). The combination unblocks real-time browser clients and CDC-driven entity events simultaneously.

Prerequisites before Phase 4 kickoff:
1. Commit the WebRTC SFU choice (str0m vs LiveKit) as an ADR.
2. Confirm `wasm-bindgen` + Connect-ES version compatibility.
3. Complete `frf-postgres-cdc` WAL consumer loop (HIGH debt from Phase 1+2).

Exit criterion for Phase 4:
> Browser client (via `frf-wasm` + Connect-ES) subscribes to an entity stream, edits offline, and reconnects via WebSocket mux; CDC event from PostgreSQL WAL flows end-to-end through the spine to the browser.
