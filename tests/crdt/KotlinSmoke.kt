// Kotlin binding smoke test — verifies the UniFFI-generated surface is callable.
//
// Run after `./sdks/kotlin/build_jni.sh` from the sdks/kotlin directory:
//   ./gradlew test
//
// CI: included in the `uniffi-kotlin` Dagger stage.

import uniffi.frf.crdtNewSnapshot
import uniffi.frf.crdtSnapshotVersion
import uniffi.frf.crdtApplyDelta

fun main() {
    // crdtSnapshotVersion on empty bytes returns 0
    val emptyVersion = crdtSnapshotVersion(byteArrayOf())
    check(emptyVersion == 0UL) { "empty snapshot must have version 0, got $emptyVersion" }

    // crdtNewSnapshot produces non-empty bytes
    val snapshot = crdtNewSnapshot()
    check(snapshot.isNotEmpty()) { "new snapshot must be non-empty" }

    // crdtSnapshotVersion on a fresh snapshot returns 0 (no ops)
    val freshVersion = crdtSnapshotVersion(snapshot)
    check(freshVersion == 0UL) { "fresh snapshot must have version 0, got $freshVersion" }

    // crdtApplyDelta with empty delta is idempotent
    val merged = crdtApplyDelta(snapshot, byteArrayOf())
    check(merged.isNotEmpty()) { "apply empty delta must return non-empty bytes" }

    println("Kotlin smoke test passed.")
}
