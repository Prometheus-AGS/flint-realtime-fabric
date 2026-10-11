## ADDED Requirements

### Requirement: Deliver committed typed CDC changes without checkpoint gaps

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c008-cdc-commit-mapping` are satisfied and its accepted local verification is executed
- **THEN** Real INSERT/UPDATE/DELETE and rollback/multi-row transactions exercise enrolled non-first/composite/non-UUID keys and accepted tenant modes; unsupported mappings fail enrollment, not silent delivery.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Crash between publication and LSN advancement permits defined deduplication without loss; poison rows, schema change, unchanged TOAST and missing old-key data cannot be skipped beneath an acknowledged checkpoint.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
