# p11-c001 — CDC Compose Wiring

## Phase
phase-11-layer3-e2e-wasm-opt-cdc

## Summary

Wire the `CDC_ENABLED` and related environment variables into the `gateway`
service in `compose.yml` so `PostgresCdcConsumer` activates when the compose
stack starts. Use a well-known fixture UUID for `CDC_TENANT_ID`.

## Files to Create/Modify

- `compose.yml` — add CDC env vars to `gateway` service `environment:` block:
  - `CDC_ENABLED: "true"`
  - `CDC_REPLICATION_URL: "postgres://frf:frf@postgres:5432/frf?replication=database"`
  - `CDC_SLOT_NAME: "frf_slot"`
  - `CDC_PUBLICATION_NAME: "frf_pub"`
  - `CDC_TENANT_ID: "00000000-0000-0000-0000-000000000001"`
  - `CDC_CHANNEL_PATH: "entities"`

## Design Notes

The `CDC_REPLICATION_URL` uses the special `?replication=database` query param
required by `pg_walstream` / libpq for logical replication connections. The
host is `postgres` (compose service name, not `localhost`).

`CDC_TENANT_ID` uses a fixture UUID (`00000000-...0001`) — a well-known test
value that makes CDC events traceable in integration tests.

`CDC_CHANNEL_PATH` matches the path the entity subscribe route listens on.

The gateway's `spawn_cdc_consumer` will call `ensure_replication_slot` on
startup, auto-creating `frf_slot` if it doesn't exist — but only if the
connecting user has `REPLICATION` privilege (addressed in c002).

## Exit Criteria

- `docker compose config` validates without error after the env var additions
- Gateway service environment block contains all 6 CDC vars
- `docker compose up -d gateway` starts without config error (CDC consumer may
  fail to connect if postgres isn't ready, but the gateway itself must start)
