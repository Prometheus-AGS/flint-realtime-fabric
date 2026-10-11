## ADDED Requirements

### Requirement: Qualify ATProto ingestion and outbound recovery

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c019-atproto-recovery` are satisfied and its accepted local verification is executed
- **THEN** Supported directions resume after disconnect/process restart without missed acknowledged events or unbounded duplication.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** Outbound retries, self-echo, tenant/channel routing and unavailable/unconfigured targets have explicit assertions; faithful protocol mocks are labeled and cannot substitute for target-version qualification.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
