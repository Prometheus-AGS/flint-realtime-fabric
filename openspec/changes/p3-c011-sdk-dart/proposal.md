# p3-c011 — flutter_rust_bridge Dart SDK

## Phase
phase-3-ffi-sdks-crdt

## Depends on
p3-c008 (frf-ffi crate built; flutter_rust_bridge_codegen 2.11.1 installed)

## Directory
`sdks/dart/`

## What this change does

Generates the Dart/Flutter SDK from the `frf-ffi` Rust crate using
`flutter_rust_bridge_codegen 2.11.1`. Produces a Dart package consumable by any
Flutter app.

### Generation approach

`flutter_rust_bridge_codegen generate` reads the public `#[frb]`-annotated surface
of `frf-ffi` and emits:
- `sdks/dart/lib/src/rust/frb_generated.dart` — generated async Dart bridge
- `sdks/dart/lib/src/rust/frb_generated.io.dart` — native platform loader
- `sdks/dart/ios/Classes/FrbBridgePlugin.swift` — optional iOS plugin (for Flutter plugin)
- `sdks/dart/android/src/main/java/...` — optional Android plugin stubs

### Flutter package manifest

```yaml
# sdks/dart/pubspec.yaml
name: frf_dart
description: Dart SDK for Flint Realtime Fabric (generated from Rust FFI)
version: 0.1.0
dependencies:
  flutter_rust_bridge: ^2.11.1
  ffi: ^2.1.0
dev_dependencies:
  flutter_test:
    sdk: flutter
```

### Hand-authored additions

- `sdks/dart/lib/frf_dart.dart` — re-exports public API from generated bridge
- `sdks/dart/GENERATED.md` — note that `lib/src/rust/` is generated; do not edit
- `sdks/dart/build_dart.sh` — runs `flutter_rust_bridge_codegen generate` then `flutter pub get`

## Non-goals

- Does not publish to pub.dev.
- Does not add Flutter demo app.
- Does not add iOS/Android plugin runner (those require Flutter project scaffolding — Phase 7).
