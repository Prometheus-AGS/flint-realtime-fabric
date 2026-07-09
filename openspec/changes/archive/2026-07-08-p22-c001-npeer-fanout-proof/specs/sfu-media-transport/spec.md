# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The room router MUST fan out media to all other room members

`RoomRouter::forward` MUST deliver a frame from one session to **every other** member of its
room (not just one), and never to the sender. This N-peer fan-out MUST be proven by a test with
3+ members. RID/simulcast handling is out of scope until a real multi-quality stream needs it.

#### Scenario: a frame fans out to all other room members

- **WHEN** three sessions are in a room and one forwards a frame
- **THEN** the other two each receive it and the sender does not
