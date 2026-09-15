## ADDED Requirements

### Requirement: Provide an isolated local integration runner

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c002-local-fixtures` are satisfied and its accepted local verification is executed
- **THEN** A real authenticated publish reaches a subscribed gateway client; disabling delivery makes that assertion fail before restoration passes.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Missing prerequisites, zero required scenarios and skipped required tests exit nonzero; cleanup touches only the run-owned containers/volumes/slots.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
