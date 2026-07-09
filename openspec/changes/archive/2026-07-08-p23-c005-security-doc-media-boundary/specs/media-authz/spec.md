# media-authz

## ADDED Requirements

### Requirement: The media path authorization boundary is documented in the security model
The security model SHALL describe the media path's JWT authentication, per-participant Keto
`view` authorization at room-join, and tenant isolation, in `docs/SECURITY.md` §1–§5.

#### Scenario: Media boundary documented
- **WHEN** a reader consults docs/SECURITY.md
- **THEN** §1/§2/§5 describe the media/signal channel's JWT gate, the Keto `view` room-join
  check (ADR-007), and `(TenantId, room)` isolation.
