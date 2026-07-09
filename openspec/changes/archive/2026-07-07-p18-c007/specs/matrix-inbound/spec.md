# matrix-inbound (delta)

## ADDED Requirements

### Requirement: Matrix inbound MUST stream real room events

The Matrix bridge inbound MUST long-poll the Client-Server `/sync` endpoint (threading the
`next_batch` token, bearer-authenticated, with reconnect backoff) and project each new
timeline event for the subscribed room into a FederatedEvent — replacing the previous
empty-stream stub. It requires no Tuwunel crate.

#### Scenario: sync response yields this room's timeline events

- **WHEN** a /sync response contains timeline events for the subscribed room
- **THEN** each is extracted and projected into a FederatedEvent
- **AND** events for other rooms are ignored

#### Scenario: a malformed or empty sync body yields no events

- **WHEN** a /sync body lacks the room / rooms structure
- **THEN** no events are produced and the loop continues (no panic)
