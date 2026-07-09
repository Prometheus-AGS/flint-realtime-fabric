# sfu-signaling

## ADDED Requirements

### Requirement: Media room participants join the same room for fan-out
Media peers SHALL send a `RoomJoin` for a shared room after negotiating, so the `RoomRouter` fans
one participant's RTP out to the others.

#### Scenario: Sender and receiver share a room
- **WHEN** both peers offer + `RoomJoin` the same room
- **THEN** the SFU relays the sender's media to the receiver.
