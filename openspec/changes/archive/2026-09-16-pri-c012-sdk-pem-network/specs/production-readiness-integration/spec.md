## ADDED Requirements

### Requirement: Deliver the runnable PGlite–Forge–Electric prototype loop

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c012-sdk-pem-network` are satisfied and its accepted local verification is executed
- **THEN** A create, update or delete initiated against PGlite is submitted through authenticated Forge, committed to PostgreSQL, returned through Electric and Fabric's authorized shape facade, reconciled into PGlite and observed by PEM without manual refresh.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** A second authenticated client observes canonical state, tenant and field projection remain enforced, and the configured LISTEN fallback keeps Forge GraphQL realtime usable when the Fabric watch source is rolled back.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
