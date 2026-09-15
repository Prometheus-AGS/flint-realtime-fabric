## ADDED Requirements

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
