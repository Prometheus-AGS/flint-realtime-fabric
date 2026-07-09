# sfu-mode-consistency

## MODIFIED Requirements

### Requirement: SFU_MODE=sovereign is enabled only once decoded media is proven end-to-end
The gateway SHALL NOT advertise `SFU_MODE=sovereign` as a production media path until a real
receiver has observed decoded media relayed by the sovereign gateway; absent that proof it
defaults to hosted and boots with a warning.

#### Scenario: Decode proof reaches the media path but does not complete
- **WHEN** the c002 run reaches the media exchange but no `framesDecoded > 0` is observed
- **THEN** `SFU_MODE=sovereign` stays gated off and the concrete media-transport blocker is
  recorded in SECURITY §6 + the phase sign-off.
