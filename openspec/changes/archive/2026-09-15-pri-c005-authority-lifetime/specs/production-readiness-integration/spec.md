## ADDED Requirements

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
