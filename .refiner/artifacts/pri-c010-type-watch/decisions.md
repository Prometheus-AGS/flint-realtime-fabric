# Decisions — pri-c010-type-watch

- Keep `flint.v1.EntityService` byte-identical and expose the frozen contract as
  `flint.v2.EntityService.WatchEntityType` through the generated tonic surface.
- Implement one dedicated `EntityTypeWatchSource` adapter over the existing
  `LogBroker`; extend the existing `EntityStore` projection capability instead
  of giving one adapter a second port.
- Persist each lossless typed mutation with the v1 materialized row and cursor in
  one SurrealDB transaction. Snapshot one type and its inclusive cursor under the
  same projection writer boundary, then subscribe at the following broker offset.
- Encrypt and authenticate opaque checkpoints with AES-256-GCM, binding subject,
  tenant, entity type, projection, source epoch, version, and retention
  generation. Never expose raw positions in an authorization-filtered
  checkpoint-only frame.
- Use one bounded queue and a separate terminal control signal per subscriber.
  Buffer overflow requires resnapshot; client drop aborts the producer task.
- Prove runtime behavior with owned local PostgreSQL, Iggy, SurrealDB, and tonic
  fixtures using immutable container image digests.
- Keep v2 service registration conditional on validated CDC enablement; a stray
  checkpoint key must not expose an inert endpoint.
- For snapshot rows withheld by object authorization, place the encrypted
  completion boundary immediately before the earliest withheld retained offset
  so a later grant and resume cannot skip that row.
- Race token expiry through every awaited authorization or enqueue boundary and
  recheck identity plus subscription authority after object authorization,
  immediately before live output.
