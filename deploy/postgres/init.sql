-- Postgres initialization for CDC logical replication
-- Creates the publication used by frf-postgres-cdc.

CREATE PUBLICATION frf_pub FOR ALL TABLES;

-- Grant replication privilege so the frf user can open a replication connection.
ALTER USER frf REPLICATION;

-- Pre-create the logical replication slot (idempotent).
-- The slot is auto-created at runtime by ensure_replication_slot, but pre-creating
-- it here avoids a race on first gateway startup before the slot exists.
SELECT pg_create_logical_replication_slot('frf_slot', 'pgoutput')
WHERE NOT EXISTS (
    SELECT 1 FROM pg_replication_slots WHERE slot_name = 'frf_slot'
);
