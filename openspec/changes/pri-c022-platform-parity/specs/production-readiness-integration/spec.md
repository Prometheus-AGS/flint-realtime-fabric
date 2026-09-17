## ADDED Requirements

### Requirement: Qualify required native and generated SDK transports

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c022-platform-parity` are satisfied and its accepted local verification is executed
- **THEN** Each required platform installs the packaged artifact and performs authenticated publish/watch/reconnect; supported offline CRDT behavior survives process replacement.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Dart async remains blocked until a real transport proof passes; unsupported platforms remain excluded with explicit reasons, without claiming full phase closure or parity.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
