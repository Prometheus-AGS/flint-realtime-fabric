# sfu-mode-consistency (delta)

## ADDED Requirements

### Requirement: Signal envelopes MUST report the configured SFU mode

The SignalService MUST stamp the SFU mode the gateway is actually configured with onto
signal envelopes (both proto→domain and domain→proto), not a hardcoded constant. The
gateway's configured mode is threaded into the service at construction.

#### Scenario: configured Sovereign is reported as Sovereign on the wire

- **WHEN** the gateway runs with SFU_MODE=sovereign and converts a signal
- **THEN** the domain and proto sfu_mode both report Sovereign (not the old hardcoded Hosted)

#### Scenario: configured Hosted is reported as Hosted

- **WHEN** the gateway runs with SFU_MODE=hosted and converts a signal
- **THEN** the sfu_mode reports Hosted
