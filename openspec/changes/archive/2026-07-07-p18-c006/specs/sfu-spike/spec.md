# sfu-spike (delta)

## ADDED Requirements

### Requirement: The str0m negotiation round-trip MUST be proven before the full SFU build

The str0m dependency MUST be current (0.21), and a spike MUST prove the `Rtc` negotiation
round-trip (accept an SDP offer, produce a valid SDP answer) with an automated test. The
live UDP/ICE/DTLS media loop is out of scope for the spike and remains documented as
deferred; `SFU_MODE=sovereign` stays gated off until media actually flows.

#### Scenario: a real offer negotiates to a valid answer

- **WHEN** a real SDP offer is passed to the negotiation spike
- **THEN** a well-formed SDP answer is produced carrying the offered media line

#### Scenario: an invalid offer is rejected

- **WHEN** a malformed SDP is passed
- **THEN** the spike returns an InvalidOffer error

#### Scenario: sovereign mode stays gated off

- **WHEN** the spike lands
- **THEN** SFU_MODE defaults to hosted and sovereign selection still warns that no media flows
