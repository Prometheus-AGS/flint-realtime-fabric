# sfu-mode-consistency Specification

## Purpose
TBD - created by archiving change p18-c003. Update Purpose after archive.
## Requirements
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

### Requirement: A sovereign compose override exposes the media UDP path
The deployment SHALL provide an opt-in compose override that runs the gateway in
`SFU_MODE=sovereign` with a mapped UDP media port and advertised candidate IP, without altering
the production hosted default.

#### Scenario: Sovereign override applied
- **WHEN** the stack is brought up with `-f compose.yml -f compose.sovereign.yml`
- **THEN** the gateway runs sovereign with a UDP port mapped and `MEDIA_ADVERTISE_IP` set.

#### Scenario: Default unchanged
- **WHEN** the stack is brought up with `compose.yml` alone
- **THEN** the gateway stays `SFU_MODE=hosted` with no UDP mapping.

