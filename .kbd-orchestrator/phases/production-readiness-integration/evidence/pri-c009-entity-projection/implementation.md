# c009 implementation — durable entity projection

Status: **COMPLETE**

## Runtime path

The gateway now composes a dedicated `frf-projection-surreal` adapter for the
`EntityStore` port whenever CDC is enabled. A single projector consumes the
well-known entity channel, decodes c008's canonical committed mutations, derives
stable entity IDs, and transactionally stores each state change with its source
epoch and broker cursor before acknowledging Iggy.

Snapshot installation writes state and its checkpoint together. Projector
startup resumes inclusively; offsets at or below the stored checkpoint are
duplicates. A source-epoch mismatch or update without a base row fails closed
with `resnapshot_required` rather than manufacturing state.

The existing v1 `EntityService` reads and watches this store. Watch delivery
performs an object-level authorization check for every event and terminates on
revocation. Gateway readiness combines CDC and projector readiness, and shutdown
drains both tasks.

## Deployment path

The full Compose profile includes pinned SurrealDB, a durable volume, a readiness
probe, explicit credentials, and gateway projection settings. The deployment
renderer requires the new image and secrets. Environment and recovery docs cover
cursor/source-epoch lifetime, backup alignment, resnapshot, and singleton
operation.

The local acceptance runner owns PostgreSQL, Iggy, and SurrealDB, uses immutable
images and per-run credentials, records source and log hashes, and removes only
its Compose project.
