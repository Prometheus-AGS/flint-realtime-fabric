# media-e2e

## ADDED Requirements

### Requirement: A runner exercises the live decoded-media proof and records its outcome
The project SHALL provide a runner that boots the sovereign stack, seeds the media `view` grant,
runs the decode harness, and records the actual `framesDecoded` outcome as the gate-flip input.

#### Scenario: Live run recorded
- **WHEN** the runner executes against the sovereign stack
- **THEN** its actual result (decoded or the concrete failure) is recorded, never fabricated.
