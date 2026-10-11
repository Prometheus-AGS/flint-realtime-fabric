# Decisions — pri-c009-entity-projection

- Keep `frf-store-surreal` dedicated to `CrdtStore`; use a separate
  `frf-projection-surreal` adapter for `EntityStore` so the architecture retains
  one port per adapter.
- Store an entity mutation and its projection cursor in the same SurrealDB
  transaction, then acknowledge Iggy. Retry inclusively from the stored cursor.
- Treat source-epoch mismatch and an update without a projected base row as a
  resnapshot requirement. Do not invent partial state.
- Use the existing v1 `EntityService` unchanged and provide the durable store at
  composition time. Re-check the object `view` relation for every watch event.
- Couple projector readiness and shutdown to the singleton CDC runtime.
- Capture Iggy's channel high-water mark only after subscription is active, and
  publish readiness only after the projector applies and acknowledges that
  boundary.
- Prove behavior through owned real adapters and immutable container images;
  compile-only and CI results are not acceptance evidence.
