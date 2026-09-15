# c008 implementation record

## Boundary

The CDC adapter now validates a server-owned, schema-qualified enrollment
against the live PostgreSQL catalog before declaring readiness. The mapping
requires a primary key, adequate replica identity, supported published columns,
an accepted tenant mode and an ordinary table. Publications using `FOR ALL
TABLES`, row filters, missing CRUD actions or missing enrolled tables fail
closed.

The decoder preserves typed canonical values and ordered composite keys. It
buffers row changes under BEGIN and publishes only after COMMIT. A source
position made from epoch, transaction end LSN and transaction index creates a
stable event ID. DELETE is an invalidation and does not fabricate a record.
The source position stays in the CDC payload; `LogBroker::publish` assigns and
returns the unique durable broker offset for each mutation.

## Checkpoint boundary

Every commit is revalidated against the catalog, then all mutations publish in
source order. Only after the entire commit succeeds does the consumer update
the applied LSN. Decode, catalog, transaction and broker errors terminate the
consumer before feedback can acknowledge the affected commit. Restart replay
uses stable IDs and the durable broker's deduplication contract.

## Deployment

Gateway configuration now requires a source epoch and JSON enrollment when CDC
is enabled. The deployment creates an empty explicit publication for migrations
to populate. The environment guide and runbook document enrollment, recovery,
slot recreation and the local acceptance runner.

The live fixture is owned by the gateway composition crate, which already
composes the CDC and broker adapters. The CDC adapter depends only on inward
domain and port crates; it does not import another adapter for tests.

All touched Rust and test files remain below the repository's 500-line limit.
