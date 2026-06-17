# Goals — Phase 1: Core Infrastructure Adapters

- Implement `frf-broker-iggy`: the `LogBroker` port backed by the GQAdonis Apache Iggy fork — this unlocks the durable event spine that all other adapters, CDC, and fan-out depend on
- Implement `frf-authz-keto`: the `AuthzProvider` port backed by Ory Keto (Zanzibar ReBAC) — tenant isolation and per-event RLS enforcement
- Implement `frf-identity-ory`: the `IdentityVerifier` port backed by Ory Kratos (identity) and Oathkeeper (JWT verification at the gateway boundary)
- Wire the first live subscription path in `frf-gateway`: `LogBroker.subscribe()` → WebSocket fan-out → Keto RLS check per event
- Begin `frf-postgres-cdc`: logical replication slot consumer that produces `EventEnvelope` facts onto the Iggy spine
- Confirm version currency: tonic 0.14 vs 0.15, Connect-ES, Iggy fork branch state
