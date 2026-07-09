# media-e2e

## ADDED Requirements

### Requirement: The decode proof runs on a real Linux host with native host networking
The decode proof SHALL be runnable on a Linux host (GitHub Actions `ubuntu-latest`) where host
networking is native and containers reach the host at the docker0 bridge gateway, so the browser and
SFU share one real network stack without the same-host candidate-address confusions of the local
macOS/Colima environment.

#### Scenario: CI decode job runs the full proof
- **WHEN** the `decode-proof` workflow_dispatch job runs on `ubuntu-latest`
- **THEN** it builds the gateway image, boots the sovereign stack, runs the Playwright decode, and asserts `framesDecoded > 0`.
