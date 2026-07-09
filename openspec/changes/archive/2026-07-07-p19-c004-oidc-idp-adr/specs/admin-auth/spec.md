# admin-auth (delta)

## ADDED Requirements

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
