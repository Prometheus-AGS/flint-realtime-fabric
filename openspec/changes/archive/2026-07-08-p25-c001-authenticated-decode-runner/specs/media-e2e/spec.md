# media-e2e

## ADDED Requirements

### Requirement: The decode runner uses a real authenticated JWT
The decode-proof runner SHALL bring up the sovereign stack cleanly and authenticate the harness
with a real gateway-accepted JWT (not a DEV_NO_AUTH bypass), so the proof exercises the
authenticated media-authz path.

#### Scenario: Authenticated runner boots
- **WHEN** the runner runs
- **THEN** the sovereign stack (incl. flint-gate) comes up, a real JWT is obtained, the view grant
  is seeded for its subject, and the harness runs with that JWT.
