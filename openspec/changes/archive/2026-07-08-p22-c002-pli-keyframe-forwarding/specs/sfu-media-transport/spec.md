# sfu-media-transport (delta)

## ADDED Requirements

### Requirement: The SFU MUST forward keyframe requests from receivers to senders

The sovereign SFU MUST relay an inbound `Event::KeyframeRequest` from one session to the room's
other member(s) and apply it via `Writer::request_keyframe` (guarded by
`is_request_keyframe_possible`), so a desynced/late peer can be served a keyframe. The request
travels receiver→sender (opposite to media) over the same per-session forwarding channel
(carrying a `ForwardedFrame` enum). Renegotiation and the real "peer decodes after PLI" proof
are browser-gated and deferred; `SFU_MODE=sovereign` stays gated off.

#### Scenario: a keyframe request routes to the other room member(s)

- **WHEN** a session forwards a keyframe request in a room
- **THEN** the other member(s) receive it (not the requester)

#### Scenario: an unsupported keyframe kind is skipped safely

- **WHEN** `request_keyframe` is not possible for a session/kind
- **THEN** it is skipped without panic (guarded by `is_request_keyframe_possible`)
