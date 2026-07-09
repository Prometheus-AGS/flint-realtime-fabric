# sfu-mode-consistency

## MODIFIED Requirements

### Requirement: SFU_MODE=sovereign is enabled only once decoded media is proven end-to-end
The gateway SHALL enable `SFU_MODE=sovereign` as a production media path once, and only once, a real
receiver observes `getStats().framesDecoded > 0` for media relayed by the sovereign gateway; absent
that proof it stays gated off with recorded rationale.

#### Scenario: Decoded media observed
- **WHEN** the live run observes `framesDecoded > 0`
- **THEN** `SFU_MODE=sovereign` is flipped on and SECURITY §6 marks the media plane functional.

#### Scenario: Decode still not observed
- **WHEN** the live run does not observe a decoded frame
- **THEN** the gate stays off and the concrete blocker is recorded.
