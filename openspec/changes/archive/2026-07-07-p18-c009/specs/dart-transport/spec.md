# dart-transport (delta)

## ADDED Requirements

### Requirement: The Dart SDK MUST compile and expose a stable transport shim

The Dart SDK MUST compile (the generated `frf.dart`'s broken async/callback methods are
replaced with documented compile-valid throws) and MUST expose a hand-written shim: a
working `FrfCrdt` surface over the generated sync FFI, and a stable `FrfTransport` API
whose currently-unavailable async methods throw an actionable `FrfTransportUnavailable`
rather than an opaque error. The post-generation patch MUST be documented.

#### Scenario: the package compiles and analyzes clean

- **WHEN** `dart pub get` and `dart analyze` run on the package
- **THEN** dependencies resolve and no analyzer issues are reported

#### Scenario: FrfTransport surfaces an actionable error, FrfCrdt is reachable

- **WHEN** a consumer calls `FrfTransport.connect`
- **THEN** it throws `FrfTransportUnavailable` naming the method and the reason
- **AND** the `FrfCrdt` CRDT API is reachable through the package entry point
