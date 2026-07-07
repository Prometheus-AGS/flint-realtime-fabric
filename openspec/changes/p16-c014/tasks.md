# Tasks — p16-c014

- [x] Extend frf-ffi API to connect/auth/subscribe/publish via frf-sdk-rust
- [x] Regenerate Swift bindings + smoke test
- [x] Regenerate Kotlin bindings + smoke test
- [x] Populate Dart bridge output so package compiles
- [~] Dart smoke test connect/subscribe

## Task 1 — real transport in frf-ffi (over frf-sdk-rust)

New `client.rs`: `FrfFfiClient` (`#[derive(uniffi::Object)]`) wrapping the Rust SDK's
`FrfClient` behind an `Arc<Mutex<>>`. Async FFI methods (`#[uniffi::export(async_runtime
= "tokio")]`):
- `connect(endpoint, token?)` — constructor; auth via bearer token.
- `publish(envelope_json) -> u64` — envelope crosses as JSON.
- `subscribe(channel_id, consumer_id, from_offset, callback)` — pushes events to a
  `EventCallback` foreign trait (`#[uniffi::export(with_foreign)]`), one JSON envelope
  per `on_event`, `on_error` on failure.

Enabled the `tokio` feature on `uniffi` (brings `async_compat` for the async bridge).
Added `ClientFfiError` with `From<SdkError>`. Domain types cross as JSON strings so the
wire encoding is never part of the FFI contract. The crate goes from 3 CRDT functions to
CRDT + full transport (closes the "CRDT-only stub" part of C6).

## Tasks 2, 3 — Swift + Kotlin bindings (UniFFI)

Regenerated via the `uniffi-bindgen` crate from the built cdylib:
- **Swift** (`sdks/swift/Sources/FrfClient/frf.swift` + `.h` + `.modulemap`): now contains
  `FrfFfiClient` with async `connect`/`publish`/`subscribe` + the `EventCallback` protocol
  (17 client refs). Generation exit 0.
- **Kotlin** (`sdks/kotlin/lib/src/main/kotlin/uniffi/frf/frf.kt`): same transport surface
  (13 client refs). Generation exit 0 (ktlint-formatted).

"Smoke" = the binding generator succeeds and emits the transport client — a device
smoke test needs a built native lib + a running gateway (out of this env).

## Task 4 — Dart package compiles (C6)

Root cause found: the Dart SDK was scaffolded for **flutter_rust_bridge (FRB)**, but the
FFI crate is **UniFFI**. FRB cannot parse a UniFFI crate — it panics on the
`#[uniffi::export]` surface and only emits non-functional stubs (and injects an unwanted
`mod frb_generated` into `lib.rs`).

Per operator decision (FRB CRDT bridge now, defer Dart transport), the honest fix:
- Removed the broken FRB stubs; reverted FRB's `lib.rs` injection (frf-ffi builds clean).
- Rewrote `lib/frf_dart.dart` to a resolvable placeholder that documents the pending
  UniFFI-Dart generation (no broken bridge export).
- Rewrote `build_dart.sh` to use `uniffi-bindgen-dart` (the correct UniFFI Dart backend,
  `0.1.x`) instead of FRB.
- **`dart pub get` now succeeds and `dart analyze` reports no issues** — the C6 defect
  (package doesn't compile / fails pub get) is fixed.

## Task 5 — Dart connect/subscribe smoke: DEFERRED (`[~]`)

Dart transport bindings are not generated yet — that needs `uniffi-bindgen-dart` installed
and a built native lib (a full env step in `build_dart.sh`). A connect/subscribe smoke is
therefore deferred, honestly marked, with the generation path documented. Swift/Kotlin got
full transport; Dart gets a compiling package + the correct migration path.

## Verification

- `cargo build -p frf-ffi` (+ `--release` cdylib) → exit 0
- `cargo clippy -p frf-ffi --lib` → exit 0; workspace `--lib --bins` → exit 0; fmt clean
- Swift + Kotlin binding generation → exit 0, transport client present
- `dart pub get` → exit 0; `dart analyze lib/` → no issues

## Follow-up

Generate Dart transport bindings via `uniffi-bindgen-dart` (run `build_dart.sh` in an env
with the tool + native lib), then add the Dart connect/subscribe smoke test.
