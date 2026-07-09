/// Hand-written transport shim over the generated UniFFI bindings (p18-c009).
///
/// The UniFFI Dart generator (`uniffi-bindgen-dart` 0.1.3) cannot emit the async
/// constructor/callback ABI, so the generated `FrfFfiClient.connect` throws
/// `UnsupportedError` at runtime (see `src/rust/frf.dart` and `GENERATED.md`). This shim
/// wraps the bindings so consumers get:
///   - the **working synchronous CRDT surface** (`FrfCrdt`), delegating to the generated
///     `crdtApplyDelta` / `crdtNewSnapshot` / `crdtSnapshotVersion`; and
///   - a **stable transport API** (`FrfTransport`) that, for the not-yet-generated async
///     methods, throws a clear, actionable [`FrfTransportUnavailable`] rather than the
///     opaque generated `UnsupportedError`.
///
/// This is deliberately honest: hand-lowering the full UniFFI async runtime (RustFuture
/// poll/complete/free + foreign-callback vtables + RustBuffer marshaling) in `dart:ffi`
/// is a large, fragile undertaking; shipping a half-working hand-rolled async runtime
/// would risk silent FFI memory corruption. Until `uniffi-bindgen-dart` fixes async
/// codegen (0.1.3 is the latest and does not), the transport surface is documented as
/// unavailable on Dart while the CRDT surface is fully usable.
library frf_transport;

import 'dart:typed_data';

import 'rust/frf.dart' as gen;

/// Thrown by [FrfTransport] methods that depend on the not-yet-generated UniFFI async
/// ABI. Carries an actionable message rather than the generated `UnsupportedError`.
class FrfTransportUnavailable implements Exception {
  const FrfTransportUnavailable(this.method);

  /// The transport method that is unavailable (e.g. `connect`).
  final String method;

  @override
  String toString() =>
      'FrfTransportUnavailable: `$method` is not available on the Dart SDK yet. '
      'The uniffi-bindgen-dart 0.1.3 generator cannot emit the async transport ABI '
      '(see GENERATED.md). Use the Rust/TS/Go/Swift/Kotlin SDKs for the transport '
      'surface; the Dart CRDT surface (FrfCrdt) is fully usable.';
}

/// The working, synchronous CRDT surface — a thin typed wrapper over the generated
/// top-level functions. These are fully functional (the generator handles sync calls).
class FrfCrdt {
  const FrfCrdt._();

  /// Apply a CRDT delta to an existing snapshot, returning the merged snapshot bytes.
  static Uint8List applyDelta(Uint8List existing, Uint8List delta) =>
      gen.crdtApplyDelta(existing, delta);

  /// Create a new, empty CRDT snapshot.
  static Uint8List newSnapshot() => gen.crdtNewSnapshot();

  /// Read the version counter from a CRDT snapshot.
  static int snapshotVersion(Uint8List snapshot) =>
      gen.crdtSnapshotVersion(snapshot);
}

/// The intended transport API (connect / subscribe / publish / ack).
///
/// Every method currently throws [FrfTransportUnavailable] because the underlying async
/// UniFFI bindings are not generated. The API shape is stable so consumers can code
/// against it now and pick up a working implementation when the generator (or a future
/// full hand-lowering) lands — without changing call sites.
class FrfTransport {
  const FrfTransport._();

  /// Connect to the gateway. **Unavailable** — see [FrfTransportUnavailable].
  static Future<FrfTransport> connect(String endpoint, String? token) {
    throw const FrfTransportUnavailable('connect');
  }

  /// Publish an event envelope (JSON). **Unavailable.**
  Future<int> publish(String envelopeJson) {
    throw const FrfTransportUnavailable('publish');
  }

  /// Subscribe to a channel; events are delivered to [onEvent]. **Unavailable.**
  void subscribe(
    String channelId,
    String consumerId,
    int fromOffset,
    void Function(String envelopeJson) onEvent,
    void Function(String message) onError,
  ) {
    throw const FrfTransportUnavailable('subscribe');
  }

  /// Acknowledge consumption up to [offset]. **Unavailable.**
  Future<void> ack(String channelId, String consumerId, int offset) {
    throw const FrfTransportUnavailable('ack');
  }
}
