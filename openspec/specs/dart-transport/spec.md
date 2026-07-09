# dart-transport Specification

## Purpose
TBD - created by archiving change p18-c009. Update Purpose after archive.
## Requirements
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

### Requirement: The Dart async-transport deferral MUST carry a dated upstream check

The Dart async-transport deferral MUST be recorded with a dated upstream check rather than
an open-ended "coming soon," so the deferral does not silently rot. The documentation MUST
name the date checked, the installed `uniffi-bindgen-dart` version, whether a newer
async-fixing release was found, and the explicit follow-up trigger (regenerate and remove
the post-generation patch when a fixed generator ships). No async transport code is written
while the generator remains broken.

#### Scenario: the deferral note is dated and actionable

- **WHEN** the Dart async-transport deferral is documented
- **THEN** it names the check date, the installed generator version, and the removal trigger
- **AND** no async transport method is claimed to work while the generator is unfixed

