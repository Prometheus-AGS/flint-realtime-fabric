# entity-plane Specification

## Purpose
TBD - created by archiving change p17-c004. Update Purpose after archive.
## Requirements
### Requirement: EntityService MUST be served and auth-guarded

The gateway MUST serve `flint.v1.EntityService` (GetEntity unary, WatchEntity
server-stream) backed by an `EntityStore` port, and every read MUST be authorized:
the bearer JWT is verified, its tenant MUST equal the requested tenant, and the subject
MUST hold Keto `view` on the entity. No adapter is imported by `frf-domain`/`frf-app`;
proto↔domain conversion lives only in the gateway.

#### Scenario: authorized get returns the entity

- **WHEN** a caller with a valid token for the entity's tenant and `view` permission
  calls GetEntity
- **THEN** the latest EntityChange for that entity is returned

#### Scenario: cross-tenant read is rejected before the store

- **WHEN** a caller whose verified tenant differs from the requested tenant calls GetEntity
- **THEN** the call is rejected as permission-denied before the store is consulted

#### Scenario: missing view permission is rejected

- **WHEN** an authenticated same-tenant caller lacks Keto `view` on the entity
- **THEN** the call is rejected as permission-denied

