# media-authz Specification

## Purpose
TBD - created by archiving change p23-c001-media-authz-adr. Update Purpose after archive.
## Requirements
### Requirement: Media fan-out is authorized per participant at room-join
The system SHALL authorize media room-join with `check(subject, "view", room)` where the subject
is the **authenticated identity** (JWT subject) when available, so the grant is stable and
seedable; it falls back to the session id only when no authenticated subject is present.

#### Scenario: Authenticated subject authorized
- **WHEN** a join carries a verified JWT subject with a `view` grant on the room
- **THEN** it is admitted and receives fan-out.

#### Scenario: Unauthorized subject rejected
- **WHEN** the subject has no `view` grant (or the check errors)
- **THEN** the join is denied, fail-closed.

### Requirement: The media path authorization boundary is documented in the security model
The security model SHALL describe the media path's JWT authentication, per-participant Keto
`view` authorization at room-join, and tenant isolation, in `docs/SECURITY.md` §1–§5.

#### Scenario: Media boundary documented
- **WHEN** a reader consults docs/SECURITY.md
- **THEN** §1/§2/§5 describe the media/signal channel's JWT gate, the Keto `view` room-join
  check (ADR-007), and `(TenantId, room)` isolation.

