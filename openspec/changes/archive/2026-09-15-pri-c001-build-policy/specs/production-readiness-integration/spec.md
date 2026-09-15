## ADDED Requirements

### Requirement: Restore the declared build matrix and local-only testing policy

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c001-build-policy` are satisfied and its accepted local verification is executed
- **THEN** Default, dev-endpoints and shape-facade checks plus applicable Clippy, format and SDK typecheck succeed locally against recorded source hashes.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Inventory every workflow/Dagger call path: CI performs build/lint/typecheck/format/package only; each removed runtime gate has a documented local invocation and is exercised by its owning later change.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
