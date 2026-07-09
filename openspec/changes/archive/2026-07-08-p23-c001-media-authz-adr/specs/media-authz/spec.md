# media-authz

## ADDED Requirements

### Requirement: Media fan-out is authorized per participant at room-join
The system SHALL authorize each participant against the room before admitting them to media
fan-out, in addition to the JWT boundary and tenant isolation.

#### Scenario: Authorized participant admitted
- **WHEN** an authenticated session joins a room it is permitted (Keto `view`) to see
- **THEN** it is added to room membership and receives fan-out.

#### Scenario: Unauthorized participant rejected
- **WHEN** an authenticated session joins a room it is NOT permitted to see
- **THEN** it is not added to membership and receives no media.
