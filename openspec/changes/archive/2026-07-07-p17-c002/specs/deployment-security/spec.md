# deployment-security (delta)

## ADDED Requirements

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
