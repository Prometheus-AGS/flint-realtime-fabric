# sfu-signaling Specification

## Purpose
TBD - created by archiving change p18-c001. Update Purpose after archive.
## Requirements
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

### Requirement: The gateway relays sovereign trickle ICE candidates to the browser
For `SFU_MODE=sovereign`, the `/ws/v1/signal` endpoint SHALL relay the media engine's local trickle
ICE candidates and connection-state to the browser, and accept the browser's inbound ICE
candidates, so ICE can complete between the browser and the SFU.

#### Scenario: Bidirectional trickle
- **WHEN** a browser negotiates an offer with the sovereign SFU
- **THEN** the browser receives the SFU's `ice-candidate` frames and the SFU receives the browser's,
  and ICE reaches `connected`.

### Requirement: Media room participants join the same room for fan-out
Media peers SHALL send a `RoomJoin` for a shared room after negotiating, so the `RoomRouter` fans
one participant's RTP out to the others.

#### Scenario: Sender and receiver share a room
- **WHEN** both peers offer + `RoomJoin` the same room
- **THEN** the SFU relays the sender's media to the receiver.

