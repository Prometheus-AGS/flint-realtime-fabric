## ADDED Requirements

### Requirement: Make PGlite outbound sync durable and restart-safe

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c013-sync-persistence` are satisfied and its accepted local verification is executed
- **THEN** A mutation created while Forge or Electric is unavailable survives client restart, is submitted when connectivity returns, and converges through canonical Electric/Fabric state in PGlite and PEM.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Retries do not duplicate database effects, rejection or version conflict is visible and recoverable, two clients converge after create/update/delete, and tenant isolation remains enforced.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
