## ADDED Requirements

### Requirement: Close sovereign media decode and protected fan-out

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c020-sovereign-decode` are satisfied and its accepted local verification is executed
- **THEN** Repeated two-peer campaigns show increasing receiver framesDecoded plus audio progress, including late join/reconnect and multiple rooms; ICE/RTP alone is insufficient.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Revocation/expiry removes protected fan-out within c003's accepted bound; wrong-room/tenant traffic is denied and task/socket counts return to baseline. Keep production mode gated until proof.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
