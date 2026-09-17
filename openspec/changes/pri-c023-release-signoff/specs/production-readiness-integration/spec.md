## ADDED Requirements

### Requirement: Produce source-bound release verdicts for each agreed profile

The production-readiness integration SHALL satisfy the following acceptance
outcomes for this slice before its applicable capability is certified.

#### Scenario: Required functional outcome is demonstrated

- **WHEN** the prerequisites for `pri-c023-release-signoff` are satisfied and its accepted local verification is executed
- **THEN** Every selected profile has exact repository/diff/lockfile/package/image/config/topology fingerprints, commands, timestamps, exits, scenario counts and artifact hashes; critical/high findings are resolved by an independent reviewer.

#### Scenario: Failure and compatibility boundaries are enforced

- **WHEN** the same candidate is checked against this slice's negative, recovery and compatibility conditions
- **THEN** A narrower profile may receive a scoped interim verdict while other capabilities remain blocked; c023 and the whole phase cannot close until all agreed phase goals are satisfied or the operator explicitly accepts scope reduction. No registry publication/deployment is inferred.

#### Scenario: Evidence is absent or stale

- **WHEN** a required decision, dependency, local proof or matching source/configuration fingerprint is missing
- **THEN** the affected implementation or certification gate SHALL remain open and no missing-service skip or historical receipt SHALL count as current proof
