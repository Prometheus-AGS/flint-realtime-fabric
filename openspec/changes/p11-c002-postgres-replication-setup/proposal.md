# p11-c002 — PostgreSQL Replication Privilege Setup

## Phase
phase-11-layer3-e2e-wasm-opt-cdc

## Summary

Grant the `REPLICATION` attribute to the `frf` Postgres user and pre-create
the `frf_slot` logical replication slot in `deploy/postgres/init.sql` so the
CDC consumer can connect and subscribe to WAL without requiring superuser.

## Files to Create/Modify

- `deploy/postgres/init.sql` — add:
  1. `ALTER USER frf REPLICATION;` — grant replication privilege
  2. `SELECT pg_create_logical_replication_slot('frf_slot', 'pgoutput') WHERE NOT EXISTS (SELECT 1 FROM pg_replication_slots WHERE slot_name = 'frf_slot');` — idempotent slot creation

## Design Notes

The `frf` user is created by the `postgres:17-alpine` image via the
`POSTGRES_USER` env var. By default it does NOT have the `REPLICATION`
attribute — without this, `pg_walstream` will fail with
`permission denied for REPLICATION`.

The slot pre-creation is idempotent: the `WHERE NOT EXISTS` guard prevents
errors on container restarts where the slot already exists from a prior run.
Alternatively `pg_walstream::ensure_replication_slot` creates it at runtime
— but only if the user has `SUPERUSER` or `REPLICATION + LOGIN`. Since
`init.sql` runs as the postgres superuser (initdb context), it's simpler and
more reliable to pre-grant here.

`pgoutput` is the standard Postgres logical decoding plugin required by
protocol version 2 (used in `ReplicationStreamConfig`).

## Exit Criteria

- `docker compose exec postgres psql -U frf -c "\du"` shows `frf` has
  `Replication` attribute
- `docker compose exec postgres psql -U frf -c "SELECT slot_name FROM pg_replication_slots"` shows `frf_slot`
- `docker compose logs gateway` shows no `permission denied` errors from CDC
  consumer on startup
