## ADDED Requirements

### Requirement: Deliver the accepted admin authentication profile

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c021-admin-auth` are satisfied and its accepted local verification is executed
- **THEN** Selected admin login/token entry, expiry, logout and unauthorized access behavior work against real local identity services.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Interactive-login claims require the actual redirect/session flow; a documented operator-only profile is never reported as delivered interactive login.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
