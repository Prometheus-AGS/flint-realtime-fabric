# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The gateway MUST drive the sovereign media transport from the signal path

For `SFU_MODE=sovereign` the gateway MUST compose both the signaling relay (`StrOmSignaler`)
and the media engine (`StrOmTransport`), and MUST drive the `MediaTransport` from the signal
path via a `MediaTransportBridge`: `Offer`→`create_session` (relay the answer),
`RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`, `RoomLeave`/`Hangup`→
`remove_session`. The bridge mapping MUST be unit-tested. Composing the media plane MUST NOT
flip the gate: `SFU_MODE=sovereign` stays gated off (media present, end-to-end proof
browser-gated) and hosted remains the production media path. The signal path's JWT boundary is
unchanged.

#### Scenario: an offer creates a media session and relays the answer

- **WHEN** a `SignalKind::Offer` envelope reaches the bridge for sovereign mode
- **THEN** `create_session` runs and an `Answer` envelope is produced to relay outbound

#### Scenario: composing the media plane does not flip the gate

- **WHEN** the gateway composes `StrOmSignaler` + `StrOmTransport` for sovereign mode
- **THEN** `SFU_MODE=sovereign` stays gated off (unproven end-to-end); the flip is deferred
