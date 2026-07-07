# sdk-parity Specification

## Purpose
TBD - created by archiving change p17-c006. Update Purpose after archive.
## Requirements
### Requirement: SDKs MUST bind all live services and FFI MUST be resilient

The Rust SDK MUST expose typed clients for every live gateway service (Sync, Agent,
Signal, Entity, Authz), and the TS/Go/C# thin wrappers MUST include Entity and Authz now
that their servers exist. The FFI/mobile subscribe path MUST use the resilient
reconnecting subscription and MUST expose `ack`, so mobile clients get the same
reconnect/replay parity as the Rust/TS/Go SDKs.

#### Scenario: Rust SDK exposes all five non-Spine service clients

- **WHEN** a caller constructs `ServiceClients`
- **THEN** it can obtain sync, agent, signal, entity, and authz clients, each with the
  bearer-token interceptor applied

#### Scenario: FFI subscribe reconnects and exposes ack

- **WHEN** an FFI client subscribes and the transport drops
- **THEN** the subscription reconnects with backoff and resumes from the last offset
- **AND** the client can call `ack` to acknowledge consumption

