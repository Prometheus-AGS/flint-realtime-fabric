# Plan — Phase 3: CRDT Core + Offline Persistence + FFI SDK Tier

> RFC-FRF-002 · Prometheus AGS
> Planned: 2026-06-19
> Backend: OpenSpec

---

## Ordering Rationale

Phase 3 has a strict dependency chain driven by the CRDT engine choice and the
FFI scaffold. Every mobile SDK binding fans out from `frf-ffi`, which requires
`frf-crdt` to exist. The plan is therefore strictly bottom-up:

```
ADR (CRDT decision)
  → workspace deps update + OpStore port
    → frf-crdt (LoroCrdtStore)
      → frf-store-redb (OpStore impl)
      → frf-store-surreal (CrdtStore impl)
        → SyncUseCase (frf-app)
          → SyncGrpcService (frf-gateway)
            → frf-ffi (UniFFI scaffold)
              → Swift SDK
              → Kotlin SDK
              → Dart SDK (flutter_rust_bridge)
                → CI codegen pipeline (Dagger)
                  → Offline roundtrip integration test
```

---

## Change List (13 changes)

| # | Change ID | Title | Depends on |
|---|---|---|---|
| 1 | `p3-c001-crdt-adr` | Loro engine ADR + workspace deps | — |
| 2 | `p3-c002-op-store-port` | `OpStore` port trait in `frf-ports` | p3-c001 |
| 3 | `p3-c003-frf-crdt` | `frf-crdt` crate — Loro adapter implementing `CrdtStore` | p3-c002 |
| 4 | `p3-c004-frf-store-redb` | `frf-store-redb` crate — on-device op-log | p3-c002 |
| 5 | `p3-c005-frf-store-surreal` | `frf-store-surreal` crate — server checkpoint store | p3-c001 |
| 6 | `p3-c006-sync-use-case` | `SyncUseCase` in `frf-app` | p3-c003, p3-c004 |
| 7 | `p3-c007-sync-grpc-service` | `SyncGrpcService` in `frf-gateway` | p3-c006, p3-c005 |
| 8 | `p3-c008-frf-ffi` | `frf-ffi` UniFFI scaffold | p3-c007 |
| 9 | `p3-c009-sdk-swift` | UniFFI-generated Swift SDK + XCFramework script | p3-c008 |
| 10 | `p3-c010-sdk-kotlin` | UniFFI-generated Kotlin SDK + Gradle wrapper | p3-c008 |
| 11 | `p3-c011-sdk-dart` | flutter_rust_bridge Dart SDK | p3-c008 |
| 12 | `p3-c012-ci-codegen` | Dagger codegen pipeline (UniFFI + FRB) | p3-c009, p3-c010, p3-c011 |
| 13 | `p3-c013-offline-roundtrip` | Offline CRDT roundtrip integration test | p3-c012 |

---

## Parallel Execution Opportunities

After p3-c002, three independent chains can run in parallel:
- p3-c003 (frf-crdt) → p3-c006
- p3-c004 (frf-store-redb) → p3-c006
- p3-c005 (frf-store-surreal) → p3-c007 (unblocks after p3-c006)

After p3-c008 (frf-ffi), all three SDK changes are independent:
- p3-c009 (Swift)
- p3-c010 (Kotlin)
- p3-c011 (Dart)

---

## Phase Exit Criterion

> Mobile app edits offline, reconnects, converges; identical merge on Swift +
> Kotlin + Dart.

Verified by p3-c013 offline roundtrip integration test.

---

## Open Decisions Committed in This Plan

| Decision | Committed Value |
|---|---|
| CRDT engine | **Loro 1.13.1** (see p3-c001 ADR) |
| UniFFI approach | proc-macro (`#[uniffi::export]`) — no `.udl` file |
| `frf-wasm` | Deferred to Phase 4 (not on mobile exit-criterion path) |
| Java SDK | Consumed from Kotlin binding (no separate SDK) |
| `kotlinc` gap | Documented in p3-c010; Kotlin compile-check deferred to CI container |
