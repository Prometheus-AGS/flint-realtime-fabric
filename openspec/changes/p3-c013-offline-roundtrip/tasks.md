# Tasks — p3-c013 offline-roundtrip

- [ ] **T1** Create `tests/crdt/` directory
  - `mkdir -p tests/crdt`
  - Verification: directory exists

- [ ] **T2** Create `crates/frf-crdt/tests/roundtrip.rs` (Rust integration test)
  - Test `offline_roundtrip_converges`:
    1. Use `loro::LoroDoc` directly to create docs A and B
    2. Doc A: `doc.get_map("data").insert("name", "alice")`; export updates
    3. Doc B: `doc.get_map("data").insert("name", "bob")`; export updates
    4. Call `frf_crdt::apply_delta(a_snapshot, b_delta)` → merged_a
    5. Call `frf_crdt::apply_delta(b_snapshot, a_delta)` → merged_b
    6. Assert `merged_a == merged_b` (convergence property)
  - Test `apply_delta_idempotent`:
    - Apply same delta twice → same result (Loro guarantees idempotent imports)
  - Test `apply_empty_delta_noop`:
    - Apply empty delta → original snapshot unchanged
  - Verification: `cargo test -p frf-crdt --test roundtrip` exits 0; 3 tests pass

- [ ] **T3** Create `tests/crdt/swift_smoke.sh`
  - Shell script that:
    1. Builds `frf-ffi` dylib if not present
    2. Runs `uniffi-bindgen` to output `frf.swift` to `/tmp/`
    3. Runs `swiftc -parse /tmp/frf.swift` (parse-only — no runtime needed)
  - Verification: script exits 0; Swift file parses without errors

- [ ] **T4** Create `tests/crdt/kotlin_smoke.sh`
  - Shell script that:
    1. Confirms `sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt` exists
    2. Runs `./gradlew :lib:compileKotlin` from `sdks/kotlin/`
  - Verification: script exits 0; Kotlin compiles without errors

- [ ] **T5** Create `tests/crdt/dart_smoke.sh`
  - Shell script that:
    1. Runs `flutter analyze lib/` from `sdks/dart/`
    2. Runs `dart analyze lib/` fallback if flutter not on PATH
  - Verification: script exits 0; 0 Dart errors

- [ ] **T6** Create `tests/crdt/run_all.sh` — orchestrates T3 + T4 + T5
  - Sources each smoke script
  - Prints PASS/FAIL per platform
  - Exits 0 only if all pass
  - Verification: `./tests/crdt/run_all.sh` exits 0 after all three pass

- [ ] **T7** Add Rust convergence test to CI
  - Ensure `cargo test --workspace` includes `frf-crdt` integration tests
  - Verify `cargo test -p frf-crdt` exits 0 in workspace context
  - Verification: `cargo test --workspace` exits 0; roundtrip tests listed in output

- [ ] **T8** Document Phase 3 exit criterion as MET
  - Update `tests/crdt/README.md` with:
    - How to run: `cargo test -p frf-crdt`, `./tests/crdt/run_all.sh`
    - Exit criterion: offline CRDT roundtrip converges; Swift + Kotlin + Dart FFI surface callable
    - Status: VERIFIED
  - Verification: file exists; exit criterion documented
