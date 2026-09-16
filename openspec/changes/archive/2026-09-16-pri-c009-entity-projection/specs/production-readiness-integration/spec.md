## ADDED Requirements

### Requirement: Populate durable v1 entity reads and watches

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c009-entity-projection` are satisfied and its accepted local verification is executed
- **THEN** A database commit reaches existing v1 GetEntity and WatchEntity through real adapters; projector readiness remains false until retained broker backlog is applied; snapshot overlap does not omit or duplicate state incorrectly.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Gateway/projector restart preserves state and cursor coherence; delete removes current state; v1 wire remains unchanged and required per-event object authorization is applied.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
