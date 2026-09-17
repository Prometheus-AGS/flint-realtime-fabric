## ADDED Requirements

### Requirement: Qualify hosted media on its selected profile

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c017-hosted-media` are satisfied and its accepted local verification is executed
- **THEN** Two clients show progressing decoded audio/video across late join and reconnect; wrong room/tenant/subject is denied and revoked participants stop under the accepted lifetime contract.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Any advertised cross-node relay passes a two-gateway campaign; absent configurations fail visibly. Local protocol fixtures alone cannot certify an untested production service version.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
