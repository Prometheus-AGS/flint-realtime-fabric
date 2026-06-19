# p3-c013 — Offline CRDT roundtrip integration test

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c012 (all SDKs generated and CI codegen pipeline green)

## Directory
`tests/crdt/`

## What this change does

Adds the Phase 3 exit-criterion integration test: a Rust-level offline roundtrip
test that proves CRDT merge convergence, plus minimal binding smoke tests for
Swift, Kotlin, and Dart to confirm the FFI surface is callable.

### Test 1 — Rust offline roundtrip (`tests/crdt/roundtrip.rs`)

Simulates two offline devices (in-memory redb instances) diverging, then merging:

1. Device A: create Loro doc, set `name = "Alice"`, export snapshot + op
2. Device B: create fresh Loro doc, set `name = "Bob"`, export snapshot + op
3. Apply A's delta to B via `apply_delta` → merged doc
4. Apply B's delta to A via `apply_delta` → merged doc
5. Assert: both merged docs have both keys (`name` field keeps LWW winner, `author` diverges to conflict-resolved value per Loro semantics)
6. Assert: merged bytes from A == merged bytes from B (convergence)

### Test 2 — Swift binding smoke test (`tests/crdt/SwiftSmoke.swift`)

```swift
import FrfClient
let v = frfVersion()
assert(!v.isEmpty)
let result = try applyCrdtDelta(existing: [], delta: [])
// empty-to-empty is valid (Loro handles it gracefully)
```

### Test 3 — Kotlin binding smoke test (`tests/crdt/KotlinSmoke.kt`)

```kotlin
import uniffi.frf.*
val v = frfVersion()
check(v.isNotEmpty())
```

### Test 4 — Dart binding smoke test (`tests/crdt/dart_smoke_test.dart`)

```dart
import 'package:frf_dart/frf_dart.dart';
void main() {
  final v = frfVersion();
  assert(v.isNotEmpty);
}
```

## Non-goals

- Does not test WebRTC peer sync (Phase 4).
- Does not test SurrealDB checkpoint persistence (covered by frf-store-surreal integration tests in p3-c005).
- Does not test multi-user concurrent edit conflicts beyond binary convergence.
