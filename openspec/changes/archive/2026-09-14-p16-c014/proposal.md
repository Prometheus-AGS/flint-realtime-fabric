# p16-c014 — FFI transport for Swift/Kotlin/Dart

## Status: DONE

## Goal
G3.2 (C6) — phase-16-production-hardening

## Problem
frf-ffi exposes only 3 CRDT byte functions; Swift/Kotlin SDKs are CRDT-only and the Dart bridge dir is empty (package does not compile). Mobile clients cannot connect/auth/subscribe/publish.

## Solution
1. Extend frf-ffi with connect/auth/subscribe/publish over the frf-sdk-rust core
2. Regenerate Swift/Kotlin bindings
3. Fix the empty Dart bridge dir so pub get succeeds

## Files Changed
- `crates/frf-ffi/src/lib.rs`
- `sdks/swift/`
- `sdks/kotlin/`
- `sdks/dart/`

## Risk
HIGH — depends-on c011; UniFFI/frb surface is broad.
