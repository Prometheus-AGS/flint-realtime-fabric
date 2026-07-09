# sfu-mode-consistency

## ADDED Requirements

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
