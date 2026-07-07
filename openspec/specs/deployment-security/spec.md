# deployment-security Specification

## Purpose
TBD - created by archiving change p17-c001. Update Purpose after archive.
## Requirements
### Requirement: Dev auth-bypass config MUST NOT be committed or deployable

The system MUST NOT commit or make deployable any dev auth-bypass configuration.
The local development override that enables the `dev-endpoints` build feature and
`DEV_NO_AUTH=true` MUST be untracked (gitignored) so no auth-bypass configuration or
signing-secret literal is ever committed, and it MUST carry a guard marking it
never-a-deploy-base. A tracked `.example` template ships only placeholders.

#### Scenario: committed tree contains no dev signing-secret literal

- **WHEN** the git-tracked tree is scanned for the dev signing-secret literal
- **THEN** the literal is absent from every tracked file
- **AND** `compose.override.yml` is gitignored (not tracked)
- **AND** `compose.override.example.yml` is tracked and contains only a placeholder
  secret sourced from the environment

#### Scenario: production image cannot honor the bypass

- **WHEN** the production image is built from the bare `compose.yml`
- **THEN** the `dev-endpoints` feature is absent, so `DEV_NO_AUTH` has no effect
- **AND** the override that would enable it is not part of the deployable artifact

### Requirement: Production builds MUST validate the JWT issuer

The gateway MUST require `JWT_ISSUER` in a production (non-`dev-endpoints`) build: boot-time
config validation fails when it is unset, because an unvalidated `iss` claim would accept
any JWKS-valid token regardless of issuer. Dev (`dev-endpoints`) builds relax this to a
startup warning so local work without a configured IdP still runs.

#### Scenario: production build refuses to boot without JWT_ISSUER

- **WHEN** a non-`dev-endpoints` binary validates its config with `JWT_ISSUER` unset
- **THEN** validation returns an error naming `JWT_ISSUER`
- **AND** the gateway does not start

#### Scenario: dev build warns but starts without JWT_ISSUER

- **WHEN** a `dev-endpoints` binary starts with `JWT_ISSUER` unset
- **THEN** config validation passes
- **AND** a startup warning states the issuer is not validated

