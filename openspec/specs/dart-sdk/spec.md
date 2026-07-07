# dart-sdk Specification

## Purpose
TBD - created by archiving change p17-c008. Update Purpose after archive.
## Requirements
### Requirement: The Dart SDK MUST expose the generated UniFFI bindings

The Dart SDK MUST generate its bindings from the `frf-ffi` UniFFI crate via
`uniffi-bindgen-dart` (never flutter_rust_bridge), commit the generated `frf.dart`, and
re-export it from the package entry point so `dart pub get` and a package import resolve.
Any generator limitation MUST be documented honestly rather than presented as working.

#### Scenario: package resolves and exposes the generated API

- **WHEN** a consumer runs `dart pub get` and imports `package:frf_dart/frf_dart.dart`
- **THEN** resolution succeeds and the generated CRDT API is available
- **AND** `dart analyze` reports no issues for hand-authored code

#### Scenario: generator limitations are documented, not hidden

- **WHEN** the generator emits a type-incorrect async/callback surface
- **THEN** the limitation is recorded in GENERATED.md and the transport surface is not
  claimed to work

