// Dart binding smoke test — verifies the flutter_rust_bridge surface is callable.
//
// Run after `./sdks/dart/build_dart.sh`:
//   cd sdks/dart && flutter test ../../tests/crdt/dart_smoke_test.dart
//
// CI: included in the `frb-dart` Dagger stage.

import 'package:frf_dart/frf_dart.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('crdt_new_snapshot returns non-empty bytes', () async {
    await RustLib.init();

    final snapshot = await crdtNewSnapshot();
    expect(snapshot, isNotEmpty, reason: 'new snapshot must be non-empty');
  });

  test('crdt_snapshot_version on empty input returns 0', () async {
    await RustLib.init();

    final version = await crdtSnapshotVersion(snapshot: Uint8List(0));
    expect(version, equals(0), reason: 'empty snapshot version must be 0');
  });

  test('crdt_apply_delta with empty delta is idempotent', () async {
    await RustLib.init();

    final snapshot = await crdtNewSnapshot();
    final merged = await crdtApplyDelta(
      existing: Uint8List.fromList(snapshot),
      delta: Uint8List(0),
    );
    expect(merged, isNotEmpty, reason: 'apply empty delta must return non-empty bytes');
  });
}
