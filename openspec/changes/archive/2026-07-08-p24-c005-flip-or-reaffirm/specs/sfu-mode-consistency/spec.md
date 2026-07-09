# sfu-mode-consistency

## MODIFIED Requirements

### Requirement: SFU_MODE=sovereign is enabled only once decoded media is proven end-to-end
The gateway SHALL NOT advertise `SFU_MODE=sovereign` as a production media path until a real
receiver has observed decoded media relayed by the sovereign gateway; absent that proof it
defaults to hosted and boots with a warning.

#### Scenario: Decode proof failed in the live run
- **WHEN** the c004 live decode run did not observe `framesDecoded > 0`
- **THEN** `SFU_MODE=sovereign` stays gated off and the concrete blocker is recorded in
  SECURITY §6 + the phase sign-off.
