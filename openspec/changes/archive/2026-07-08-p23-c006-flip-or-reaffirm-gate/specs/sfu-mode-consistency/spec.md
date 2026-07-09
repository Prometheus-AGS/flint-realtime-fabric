# sfu-mode-consistency

## ADDED Requirements

### Requirement: SFU_MODE=sovereign is enabled only once decoded media is proven end-to-end
The gateway SHALL NOT advertise `SFU_MODE=sovereign` as a production media path until a real
receiver has observed decoded media relayed by the sovereign gateway; absent that proof it
defaults to hosted and boots with a warning.

#### Scenario: Decode unproven in environment
- **WHEN** the decoded-media proof has not run against a live sovereign gateway
- **THEN** `SFU_MODE=sovereign` stays gated off and the deferral is documented in SECURITY §6.
