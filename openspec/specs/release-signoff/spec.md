# release-signoff Specification

## Purpose
TBD - created by archiving change p17-c010. Update Purpose after archive.
## Requirements
### Requirement: The phase MUST close with a verified release sign-off

Phase-17 MUST end with a re-run of the full gate suite on the current tree, a security
spot-check confirming the phase-16 security fixes hold, and a release sign-off note
recording zero CRITICAL findings with HIGH triaged. Deferred planes MUST be re-affirmed
honestly, not presented as shipped.

#### Scenario: gates pass and sign-off records the result

- **WHEN** the phase-close verification runs the fmt/clippy/check/test gates
- **THEN** all gates pass and a sign-off note records zero CRITICAL, HIGH triaged, and the
  deferred planes with their rationale

#### Scenario: security fixes still hold

- **WHEN** the spot-check re-verifies c001/c002
- **THEN** no committed secret exists, the override is untracked, and JWT_ISSUER is
  mandatory in production

