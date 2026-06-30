// Swift binding smoke test — verifies the UniFFI-generated surface is callable.
//
// Run after `./sdks/swift/build_xcframework.sh`:
//   swift -F sdks/swift/Sources/FrfClient \
//         -framework FrfClientFFI \
//         tests/crdt/SwiftSmoke.swift
//
// CI: included in the `uniffi-swift` Dagger stage once XCFramework is built.

import FrfClient

// crdt_snapshot_version on an empty buffer returns 0
let version = crdtSnapshotVersion(snapshot: [])
assert(version == 0, "empty snapshot must have version 0, got \(version)")

// crdt_new_snapshot produces a non-empty Loro snapshot
let snapshot = try! crdtNewSnapshot()
assert(!snapshot.isEmpty, "new snapshot must be non-empty")

// crdt_snapshot_version on a fresh snapshot returns 0 (no ops yet)
let freshVersion = crdtSnapshotVersion(snapshot: snapshot)
assert(freshVersion == 0, "fresh snapshot must have version 0, got \(freshVersion)")

// crdt_apply_delta with empty delta is idempotent
let merged = try! crdtApplyDelta(existing: snapshot, delta: [])
assert(!merged.isEmpty, "apply empty delta to snapshot must return non-empty bytes")

print("Swift smoke test passed.")
