# atproto-outbound (delta)

## ADDED Requirements

### Requirement: ATProto outbound MUST write records to a PDS

The ATProto bridge MUST support authenticated outbound writes: when a PDS writer is
configured, `send` authenticates a session (com.atproto.server.createSession) and writes
the event as a record (com.atproto.repo.createRecord with the account DID as repo),
replacing the previous unimplemented Err. Without a writer the bridge is inbound-only and
`send` returns a clear "not configured" error. A stale session is cleared on 401 so the
next call re-authenticates.

#### Scenario: a configured bridge writes a record to the PDS

- **WHEN** a bridge with a PDS writer sends an event
- **THEN** it creates a session, then POSTs createRecord with the DID as repo and the
  event payload as the record
- **AND** the write succeeds against the PDS

#### Scenario: an inbound-only bridge reports the write is unconfigured

- **WHEN** a bridge without a PDS writer sends an event
- **THEN** it returns an error stating outbound write is not configured
