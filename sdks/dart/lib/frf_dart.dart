/// Dart SDK for Flint Realtime Fabric.
///
/// The Rust FFI crate (`frf-ffi`) is **UniFFI-based**, exposing the CRDT functions
/// (`crdtApplyDelta`, `crdtNewSnapshot`, `crdtSnapshotVersion`) and the full transport
/// client (`FrfFfiClient`: connect / publish / subscribe / ack, with `EventCallback`).
/// The Dart bindings re-exported below are generated from it via `uniffi-bindgen-dart` —
/// the same UniFFI surface as the Swift and Kotlin bindings.
///
/// Regenerate after any change to `frf-ffi` with `./build_dart.sh` (requires
/// `uniffi-bindgen-dart` and the Rust toolchain). Do not hand-edit `src/rust/frf.dart`.
library frf_dart;

export 'src/rust/frf.dart';
