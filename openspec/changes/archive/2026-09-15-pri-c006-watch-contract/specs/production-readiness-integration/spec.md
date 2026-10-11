## ADDED Requirements

### Requirement: Freeze the versioned watch and recovery contract

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c006-watch-contract` are satisfied and its accepted local verification is executed
- **THEN** Contract scenarios cover accepted schema/key/tenant shapes, transaction commit, snapshot-to-live races, restart/replay, old cursors, unauthorized history and deletion/projection; consumer acceptance is recorded before codegen.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Frozen proto/flint/v1 files remain byte-identical; new-version generation and old/new compatibility validation succeed. Semver impact on frf-domain/frf-ports and one-port adapter boundaries are documented.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
