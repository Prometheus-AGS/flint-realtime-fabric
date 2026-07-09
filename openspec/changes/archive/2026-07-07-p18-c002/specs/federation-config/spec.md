# federation-config (delta)

## ADDED Requirements

### Requirement: Federation MUST require a configured channel when enabled

When `FEDERATION_ENABLED=true`, boot-time config validation MUST require
`FEDERATION_CHANNEL_ID` (as it already requires `FEDERATION_TENANT_ID`). Without it,
ingested federated events would land on a random per-boot channel that no subscriber's
JWT-matched channel would receive.

#### Scenario: federation enabled without channel id fails to boot

- **WHEN** federation is enabled, a tenant is set, but `FEDERATION_CHANNEL_ID` is unset
- **THEN** config validation returns an error naming `FEDERATION_CHANNEL_ID`
- **AND** the gateway does not start
