# Proposal

## Why

The C08 channel source can select one handler and several observers, but Fabric's existing event envelope does not preserve the source occurrence, route, recipient, grant revisions, or causal limits needed for replay-safe forwarding. The frozen protobuf v1 cannot acquire required fields in place.

## What Changes

- Add a versioned routed-observer JSON profile in `frf-domain`, carried by the existing custom `EventEnvelope` and checked by `frf-broker-iggy` on publish and replay.
- Preserve separate source occurrence, transport event, observer delivery, and subscriber cursor identities. Reject incomplete, cross-tenant, or incompatible profile records while leaving ordinary envelopes readable.
- Make `frf-gateway` reject `AgentRunControl.cancel` as unsupported runtime control and detach the observation stream with an explicit status.
- Pin the workspace's SurrealDB SDK and related lockfile graph to 3.3.0, matching the current convergence dependency decision.

## Capabilities

### New Capabilities

- `routed-observer-transport`: Versioned, lossless transport of an already authorized source-to-observer delivery, without assigning execution authority.

### Modified Capabilities

None.

## Impact

`frf-domain` adds a public domain type and `frf-broker-iggy` validates its persisted JSON; `frf-gateway` changes a misleading cancellation response. `frf-ports` and `frf-app` keep their existing `LogBroker` and per-event authorization seams. The dependency direction remains domain → port → adapter → gateway. The Phase 0 contract-freeze criterion remains satisfied: `proto-v1` is tagged and no frozen protobuf file changes. This is additive Rust/domain and JSON compatibility; non-Rust clients use the payload discriminator until a future explicitly versioned wire contract.

## Non-goals

Fabric does not choose handlers, issue Gate disclosure or delivery grants, maintain subscriber cursors, execute agents, post channel replies, or cancel a UAR run. This provider profile alone does not enable cross-host C08 routing.
