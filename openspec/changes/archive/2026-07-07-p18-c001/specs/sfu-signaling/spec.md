# sfu-signaling (delta)

## ADDED Requirements

### Requirement: SFU signaling MUST route to the target peer, not the sender

The str0m signaling adapter MUST deliver a signal to its intended recipient: unicast to
`to_session` when set, otherwise fan out to every OTHER member of `room_id`. It MUST NOT
echo a signal back to its sender. Room membership is tracked so broadcast reaches all
peers; removing a session deregisters it from its room.

#### Scenario: unicast reaches the target peer, not the sender

- **WHEN** peer A sends a signal with `to_session = B`
- **THEN** peer B receives it (tagged from A)
- **AND** peer A does NOT receive its own signal

#### Scenario: broadcast reaches all other room members

- **WHEN** a peer sends a signal with `to_session = None` into a room
- **THEN** every other member of that room receives it
- **AND** the sender does not receive its own broadcast

#### Scenario: unicast to an unknown session fails

- **WHEN** a signal targets a `to_session` that is not registered
- **THEN** the call returns NotFound
