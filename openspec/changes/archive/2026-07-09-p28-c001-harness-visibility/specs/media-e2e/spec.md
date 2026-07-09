# media-e2e

## ADDED Requirements

### Requirement: A stalled decode run reports its diagnostics
The decode harness SHALL surface its ICE/connection diagnostics on failure (not be killed before
they print), and the runner SHALL capture the gateway's media logs before teardown.

#### Scenario: A failed decode run is diagnosable
- **WHEN** the decode run fails to reach `framesDecoded > 0`
- **THEN** the assertion message reports `ice`/candidate counts and the gateway str0m logs are saved.
