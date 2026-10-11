## ADDED Requirements

### Requirement: Qualify capacity, recovery and rotation per profile

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c016-operations-recovery` are satisfied and its accepted local verification is executed
- **THEN** Local campaigns meet accepted subscription/change-rate/latency/retention/RTO/RPO targets with explicit resource allocation, no required skips and recorded recovery positions.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Restoration and credential/certificate rotation preserve required authorization and durable state; bounded backpressure/cancellation and alerts prevent false-ready/no-progress operation. Repeat affected receipts after consumer or profile changes.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
