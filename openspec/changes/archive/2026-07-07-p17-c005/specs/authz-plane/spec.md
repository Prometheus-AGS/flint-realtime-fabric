# authz-plane (delta)

## ADDED Requirements

### Requirement: AuthzService MUST be served and tenant-scoped

The gateway MUST serve `flint.v1.AuthzService` (Check, WriteRelation, DeleteRelation)
delegating to the Keto-backed `AuthzProvider`. Every operation MUST verify the caller's
bearer token and reject when the verified tenant differs from the tuple's tenant. Relation
tuples MUST NOT be logged.

#### Scenario: check returns the provider's decision

- **WHEN** an authenticated same-tenant caller invokes Check on a tuple
- **THEN** the response `allowed` reflects the Keto decision

#### Scenario: write then delete round-trips

- **WHEN** an authenticated same-tenant caller writes a relation and then deletes it
- **THEN** both operations succeed against the provider

#### Scenario: cross-tenant relation op is rejected before the provider

- **WHEN** a caller whose verified tenant differs from the tuple's tenant invokes
  WriteRelation
- **THEN** the call is rejected as permission-denied before the provider is consulted
