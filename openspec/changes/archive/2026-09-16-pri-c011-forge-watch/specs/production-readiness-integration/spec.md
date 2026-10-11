## ADDED Requirements

### Requirement: Replace Forge's Fabric stub with safe GraphQL delivery

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c011-forge-watch` are satisfied and its accepted local verification is executed
- **THEN** A real SQL write reaches an authenticated GraphQL subscription with only permitted fields; denied rows, composite/key changes, deletes and truncated events behave as specified.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Disconnect/reconnect, revocation, backend failure and switch/rollback are explicit and do not silently lose updates or double-apply them. OQ-FRF-1 closes only after this receipt passes.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
