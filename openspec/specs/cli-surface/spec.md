# cli-surface Specification

## Purpose
TBD - created by archiving change p17-c007. Update Purpose after archive.
## Requirements
### Requirement: The CLI MUST implement its advertised broker and CDC operations

The `frf` CLI MUST provide a broker-offset inspect command and CDC slot management, so its
surface matches what its help/docs advertise. `frf broker offsets` reads a consumer's
stored offset; `frf cdc slot create|drop` manages the logical replication slot.

#### Scenario: broker offsets reads the stored consumer offset

- **WHEN** an operator runs `frf broker offsets --channel <uuid> --consumer <id>`
- **THEN** the CLI reports the consumer's stored offset, or that none is stored yet

#### Scenario: cdc slot create/drop manages the replication slot

- **WHEN** an operator runs `frf cdc slot create --slot <name>`
- **THEN** the slot is created if absent (and reported as already-existing otherwise)
- **AND** `frf cdc slot drop --slot <name>` removes it (reporting if absent)

