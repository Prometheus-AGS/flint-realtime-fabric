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

### Requirement: Make selected deployment profiles portable and truthful

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c004-deployment-profiles` are satisfied and its accepted local verification is executed
- **THEN** Clean local installation of each selected profile reaches semantic readiness with real dependency handshakes; missing issuer/keys/broker/auth authority fails visibly.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Pinned images, declared secret sources, exposed ports and feature flags match the profile; release artifacts contain no dev authorization bypass. Each profile declares TLS termination, certificate/key/trust sources and external/internal network boundaries; valid HTTPS succeeds, invalid or missing TLS inputs fail startup/readiness, and prohibited plaintext or backend-bypass access is denied.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof

### Requirement: Enforce identity and authorization on selected exposed lanes

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c005-authority-lifetime` are satisfied and its accepted local verification is executed
- **THEN** Real Gate-issued tokens allow only intended issuer/audience/tenant/subject and object; same-tenant unauthorized subjects, cross-tenant access, authority loss, key rotation and stale refill races are exercised locally.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** ASO authoritative commit/expiry to last server-produced protected frame or cancellation preventing the next frame is at most 5000 ms, followed by new-request denial; direct Kratos revocation uses the documented observation/ASO-denial start. All other lanes meet the lifetime contract accepted in c003.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof

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

### Requirement: Expose durable broker positions and commit-controlled replay

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c007-broker-replay` are satisfied and its accepted local verification is executed
- **THEN** Crash after poll but before durable consumer checkpoint replays the event; restart after acknowledgement respects defined cursor inclusivity and duplicate rules.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Producer restart does not reuse a replay position; independent consumers, partition boundaries, retention expiry and cancellation behave as the frozen contract requires. Record actual server durability/flush configuration.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof

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
