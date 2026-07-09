# admin-auth Specification

## Purpose
TBD - created by archiving change p18-c005. Update Purpose after archive.
## Requirements
### Requirement: The admin UI MUST handle token expiry and Unauthorized

The admin UI's token gate MUST be expiry-aware: it MUST reject an already-expired token
at login, decode `exp` to know when a token expires, log out automatically when a live
token expires, and clear the token on a gateway Unauthenticated response. The login copy
MUST be honest that this is a token-entry gate, not an interactive OIDC login (which is
deferred). The single `useAuthStore.accessToken` remains the source all consumers read.

#### Scenario: an expired token is rejected at login

- **WHEN** an operator submits a JWT whose `exp` is in the past
- **THEN** login returns false and the gate shows an error instead of authenticating

#### Scenario: a live token triggers logout at expiry

- **WHEN** a stored token reaches its `exp`
- **THEN** the token is cleared and the app returns to the login gate

#### Scenario: a gateway Unauthenticated response clears the token

- **WHEN** a gateway call returns Unauthenticated
- **THEN** the interceptor clears the stored token so the UI re-prompts for login

### Requirement: The admin-UI interactive-login path MUST be decided by an ADR before implementation

An interactive OIDC login for the admin UI MUST NOT be implemented until an ADR names the
IdP path and it is Accepted, because no authorization server exists in the stack today (no
Kratos/Hydra; flint-gate has no `authorize` endpoint). The ADR MUST present the IdP options
(Kratos/Hydra in compose vs. a flint-gate authorization-code endpoint), recommend one, and
record the hardened token gate as the interim. No admin-UI OIDC flow is written while no IdP
is deployed.

#### Scenario: the IdP decision is recorded before any OIDC code

- **WHEN** the admin-UI interactive login is planned
- **THEN** an ADR states the constraint, presents the IdP options, and recommends one
- **AND** the token gate is named as the interim until the ADR is Accepted and an IdP exists

#### Scenario: no OIDC flow is shipped against a missing IdP

- **WHEN** no authorization server is deployed
- **THEN** the admin UI ships no interactive OIDC login (the hardened token gate remains)

