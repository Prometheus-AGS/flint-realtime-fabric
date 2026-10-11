## ADDED Requirements

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
