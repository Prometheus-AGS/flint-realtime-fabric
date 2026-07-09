# federation-config (delta)

## ADDED Requirements

### Requirement: The gateway MUST wire the ATProto outbound writer when PDS credentials are configured

The gateway MUST build the ATProto bridge with an outbound writer
(`AtProtoBridge::with_writer`) rather than inbound-only when the ATProto PDS-writer
environment variables are set (`ATPROTO_PDS_URL`, `ATPROTO_PDS_IDENTIFIER`,
`ATPROTO_PDS_APP_PASSWORD`), so outbound federated writes reach the PDS. Boot-time
validation MUST enforce **all-or-none**:
if any one of the three PDS-writer vars is set, all three MUST be set, else the gateway
fails fast. The app-password is a secret — read from the environment only, never logged.
When none of the PDS-writer vars are set, the bridge stays inbound-only (unchanged v1
behavior).

#### Scenario: PDS writer configured wires outbound

- **WHEN** all three PDS-writer vars are set and federation is enabled
- **THEN** the ATProto bridge is built with an outbound writer
- **AND** the wiring log names the PDS URL but not the app-password

#### Scenario: partial PDS-writer config fails to boot

- **WHEN** exactly one or two of the three PDS-writer vars are set
- **THEN** config validation returns an error naming the missing variable
- **AND** the gateway does not start

#### Scenario: no PDS-writer config stays inbound-only

- **WHEN** none of the PDS-writer vars are set
- **THEN** the ATProto bridge is built inbound-only (unchanged)
