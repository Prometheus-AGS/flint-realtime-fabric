# c010 implementation summary

The gateway now serves the frozen `flint.v2.EntityService.WatchEntityType`
contract through a generated tonic server and client. Admission validates the
identity, tenant, enrolled type and subscribe relation. Every matching object is
authorized again before its mutation crosses the application boundary.

PostgreSQL committed mutations retain canonical type, key, record and source
positions through Iggy. The SurrealDB projection stores the typed delivery, v1
materialized entity and inclusive cursor in one transaction. Snapshot mode reads
the typed rows with that cursor and replays from the next broker offset; signed
checkpoints bind subject, tenant, type, projection, source epoch and generation.

Each subscriber uses a bounded queue and cancellation-owned producer task.
Overflow emits terminal lag, idle identity/authority checks run within five
seconds, permission loss discards buffered protected frames, and source errors
follow frames that were already queued. The v2 service is registered only when
a real checkpoint key is configured; no fallback secret exists.

The existing `flint.v1.EntityService` remains separately registered and its
protobuf files retain their baseline hashes. A v1-only projection snapshot can
continue advancing after upgrade; typed snapshots fail closed until a complete
typed row is projected rather than stopping the shared v1 projector.
