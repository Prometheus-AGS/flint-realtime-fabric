# media-authz

## MODIFIED Requirements

### Requirement: Media fan-out is authorized per participant at room-join
The system SHALL call `check(subject, "view", room)` before admitting a participant to media
fan-out, denying membership on a negative or errored check.

#### Scenario: Authorized participant admitted
- **WHEN** an authenticated session joins a room and Keto `view` returns true
- **THEN** it is added to `(tenant, room)` membership and receives fan-out.

#### Scenario: Unauthorized participant rejected
- **WHEN** Keto `view` returns false (or errors) for the join
- **THEN** the session is not added to membership and receives no media.
