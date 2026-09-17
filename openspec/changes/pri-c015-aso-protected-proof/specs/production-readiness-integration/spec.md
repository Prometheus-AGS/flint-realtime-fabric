## ADDED Requirements

### Requirement: Qualify the protected shape-to-SQL-to-PEM path

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c015-aso-protected-proof` are satisfied and its accepted local verification is executed
- **THEN** Crash before SQL checkpoint rolls back rows/checkpoints; crash after SQL commit before PEM publication recovers coherently only after current authorization. Logout/practice switch fences stale owners and clears the old graph.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Interrupted shape bodies, expiry/refetch, revocation timing, direct-backend isolation and forbidden/local-only field egress pass for each selected topology; no competing generic-watch writer handles the same clinical rows. Inspect the owning ASO repository rules before edits; any modified file must be at most 500 lines, splitting replica-runtime.ts if that file is touched and preserving its owner fences. Untouched oversized files remain explicitly tracked.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
