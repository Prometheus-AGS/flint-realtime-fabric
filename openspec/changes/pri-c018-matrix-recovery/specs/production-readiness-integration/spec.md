## ADDED Requirements

### Requirement: Qualify Matrix directions and restart cursors

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c018-matrix-recovery` are satisfied and its accepted local verification is executed
- **THEN** Inbound and outbound events reach intended rooms/channels after bridge restart with bounded deduplication and durable cursor continuity.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Echo-loop suppression, cross-tenant denial, target outage and unconfigured direction errors are asserted against the actual selected local server version.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
