# media-e2e

## MODIFIED Requirements

### Requirement: A runner exercises the live decoded-media proof and records its outcome
The authenticated runner SHALL be executed against the live sovereign stack and its actual
`framesDecoded` outcome recorded as the gate-flip input, never fabricated.

#### Scenario: Authenticated live run reaches the media path
- **WHEN** the runner executes against the built + booted sovereign gateway
- **THEN** its actual result (decoded, or the concrete media-transport blocker) is recorded honestly.
