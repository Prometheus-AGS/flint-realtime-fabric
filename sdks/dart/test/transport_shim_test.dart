import 'package:frf_dart/frf_dart.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('FrfTransport shim', () {
    test('connect throws an actionable FrfTransportUnavailable', () {
      // The async transport ABI is not generated yet; the shim surfaces a clear,
      // documented error rather than the opaque generated UnsupportedError.
      expect(
        () => FrfTransport.connect('http://localhost:9090', null),
        throwsA(isA<FrfTransportUnavailable>()),
      );
    });

    test('FrfTransportUnavailable message names the method and the reason', () {
      const err = FrfTransportUnavailable('connect');
      final msg = err.toString();
      expect(msg, contains('connect'));
      expect(msg, contains('uniffi-bindgen-dart'));
    });
  });

  group('FrfCrdt surface is exported', () {
    test('the CRDT API is reachable through the package entry point', () {
      // We can reference the working CRDT surface without loading the native library
      // (the static methods exist on the type). This proves the shim re-exports it.
      expect(FrfCrdt.applyDelta, isA<Function>());
      expect(FrfCrdt.newSnapshot, isA<Function>());
      expect(FrfCrdt.snapshotVersion, isA<Function>());
    });
  });
}
