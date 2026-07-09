# media-e2e

## MODIFIED Requirements

### Requirement: A runner exercises the live decoded-media proof and records its outcome
The runner SHALL be executed against the authenticated sovereign stack and its actual
`framesDecoded` outcome recorded as the gate-flip input, never fabricated.

#### Scenario: Authenticated live run recorded
- **WHEN** the authenticated runner executes against the sovereign gateway
- **THEN** its actual result (decoded, or the concrete blocker) is recorded honestly.
