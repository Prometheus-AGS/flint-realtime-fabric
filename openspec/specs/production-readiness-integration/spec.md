# production-readiness-integration Specification

## Purpose
TBD - created by archiving change pri-c001-build-policy. Update Purpose after archive.

## Requirements

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

### Requirement: Resolve release decisions and complete dependency qualification research

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c003-scope-research-contract` are satisfied and its accepted local verification is executed
- **THEN** An accepted release matrix names required profiles, clients, source schemas/keys/tenants, historical/delete semantics, scale/retention/RTO/RPO, ASO budgets and profile-specific authorization lifetime endpoints.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Pinned API, compatibility and maintenance/security evidence exists for each retained dependency; unsupported choices trigger a revised proposal, not invented feasibility. Reconcile Assess provenance finding through focused independent review.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
