# media-authz

## MODIFIED Requirements

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
