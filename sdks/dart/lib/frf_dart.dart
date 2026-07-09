/// Dart SDK for Flint Realtime Fabric.
///
/// Two surfaces:
///   - **CRDT** (`FrfCrdt`, from `src/transport.dart`) — fully working, wraps the
///     generated sync FFI (`crdtApplyDelta` / `crdtNewSnapshot` / `crdtSnapshotVersion`).
///   - **Transport** (`FrfTransport`) — the intended connect/subscribe/publish/ack API.
///     Currently throws `FrfTransportUnavailable`: the `uniffi-bindgen-dart` 0.1.3
///     generator cannot emit the async transport ABI (see `GENERATED.md`). The API shape
///     is stable so call sites won't change when a working impl lands.
///
/// The generated UniFFI bindings (`src/rust/frf.dart`) are re-exported too, but prefer the
/// `FrfCrdt` / `FrfTransport` shim — it is the honest, stable surface. Regenerate the
/// bindings after any change to `frf-ffi` with `./build_dart.sh`; do not hand-edit
/// `src/rust/frf.dart`.
library frf_dart;

// The hand-written shim: FrfCrdt (working) + FrfTransport (stable API, currently
// unavailable) + FrfTransportUnavailable.
export 'src/transport.dart';

// The generated UniFFI bindings — CRDT functions and the (broken) transport client.
// Hidden symbols are re-provided by the shim above with an honest surface.
export 'src/rust/frf.dart';
