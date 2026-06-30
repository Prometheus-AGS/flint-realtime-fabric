# p4-c001 — Wire PostgresCdcConsumer into frf-gateway

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p1-c006-frf-postgres-cdc (CDC consumer fully implemented)
p1-c007-gateway-subscription-mux (gateway AppState established)

## Crates affected
`frf-postgres-cdc`, `frf-gateway`

## Dependency-rule impact
`frf-postgres-cdc` is an infrastructure adapter (Layer 2). `frf-gateway` is the
interface layer (Layer 3) and the only place adapter wiring is permitted. This
change adds `frf-postgres-cdc` as a dependency of `frf-gateway` only — no domain
or port crates are modified. Dependency rule: safe.

## What this change does

The WAL logical replication consumer in `frf-postgres-cdc` is fully implemented
but not wired into the gateway startup sequence. This change:

1. Adds `frf-postgres-cdc` as a `[dependencies]` entry in `frf-gateway/Cargo.toml`
2. Extends `GatewayConfig` with CDC connection parameters (replication URL, slot
   name, publication name, tenant ID, channel path) sourced from environment
3. Constructs a `PostgresCdcConsumer<IggyBroker>` in `frf-gateway/src/main.rs`
4. Spawns `consumer.run_until_shutdown(shutdown_rx)` as a background Tokio task
5. Wires the same `watch::Sender<bool>` shutdown signal used by the gateway HTTP
   listener to gracefully stop the CDC loop on SIGTERM

After this change, WAL events from PostgreSQL flow through `LogBroker::publish`
into the Iggy spine, closing the end-to-end CDC path required by the Phase 4
exit criterion.

## Non-goals
- Does not implement SurrealDB CDC (Phase 5 concern)
- Does not add schema migration tooling
- Does not change any port trait signatures
- Does not add Postgres as a direct dependency to frf-domain or frf-app
