# ADR-010: Versioned entity-type watch and recovery contract

## Status

Accepted contract — 2026-09-15. Runtime implementation is not certified.

## Context

The frozen v1 `EntityService` can read or watch one entity ID. Forge needs a
schema-qualified entity-type stream and currently uses an in-process Postgres
LISTEN/NOTIFY adapter because no Fabric type-watch RPC exists. A production
replacement must join a consistent authorized snapshot to committed durable
history, resume after interruption, represent PostgreSQL keys and values without
loss, and fail explicitly when retained history cannot satisfy a checkpoint.

The c003 release contract fixes the surrounding decisions: delivery is durable
and at least once; history is retained for at least 24 hours and at least twice
the measured restore-plus-catch-up interval; the first enrollment fixtures cover
a non-first key, a composite key and a non-UUID key; deletes expose only an
authorized invalidation; lag cannot drop silently. Current v1 files remain
byte-identical to the hashes in the c006 baseline.

The Forge and PEM mapping accepted before proto edits and generation is recorded
in `consumer-acceptance.md` at their exact clean revisions. Their transport
implementations remain later c011/c012 work.

## Decision

### Version and service boundary

Add `flint.v2.EntityService.WatchEntityType`. Do not edit or overload
`flint.v1.EntityService`. A server can expose v1 and v2 together. Rollback removes
the v2 route and binary while leaving v1 data and calls untouched.

The request contains one server-approved `EntityType` (`schema`, table `name`,
projection name), one tenant from the verified identity context, and exactly one
start mode:

- `snapshot`: authorized consistent snapshot followed by live history;
- `live`: mutations committed after admission, without a snapshot; or
- `resume`: one server-issued `ClientCheckpoint`.

An absent start, multiple starts, invalid identifier, unknown projection,
tenant mismatch, client-selected predicate/column, or checkpoint from another
type/tenant/subject fails before data delivery. The checkpoint token is opaque,
versioned, integrity-protected, and bound to the effective identity, entity type,
tenant, projection and retention generation. Clients store it and echo it; they
do not construct it from source or broker positions.

### Distinct identities and positions

These fields have different jobs and are never collapsed:

| Field | Meaning | Consumer behavior |
|---|---|---|
| source epoch | identity of the logical source history | epoch change requires resnapshot |
| commit LSN | PostgreSQL commit location | diagnostic/source ordering, never client-incremented |
| transaction index | zero-based mutation order within one commit | preserves transaction order |
| event ID | deterministic identity of one committed mutation | deduplicate at-least-once delivery |
| broker partition/offset | durable log position | partition-local ordering and replay evidence |
| client checkpoint | opaque authorized resume boundary | persist atomically with applied rows |

The producer sets `event_id` to `frfevent:v1:` plus lowercase hex SHA-256 of an
RFC 8785 canonical JSON object with keys `commit_lsn` (unsigned decimal string),
`entity_type` (`schema.name@projection`), `epoch`, `key` (canonical ID), and
`transaction_index` (JSON integer). Snapshot row IDs use the same construction
with keys `barrier_lsn`, `entity_type`, `epoch`, and `key`, prefixed
`frfsnapshot:v1:`. Republishing or redelivering the same source mutation retains
the ID. Neither is a random delivery-attempt identifier.

Source mutations become visible only after PostgreSQL COMMIT. Every mutation in
a transaction has the same commit LSN and increasing transaction index. The
publisher preserves that order in one broker partition. Ordering is guaranteed
within a source transaction and broker partition; no total order across
partitions is promised. Consumers deduplicate by event ID and apply mutations
idempotently before committing the returned client checkpoint.

### Snapshot barrier and replay

For `snapshot`, the server establishes one consistent PostgreSQL snapshot and
its WAL barrier before emitting rows. The first `WatchAccepted` includes the
barrier. Snapshot rows are ordered by canonical key. `SnapshotComplete` repeats
the barrier and supplies the checkpoint that describes the complete snapshot.
Live mutations then begin strictly after the barrier. Capturing history before
reading rows and replaying changes after the barrier closes the snapshot/live
race. At-least-once duplicates at a retry boundary are allowed and retain their
stable IDs.

For `resume`, the server validates the checkpoint scope, version, source epoch
and retained broker position before `WatchAccepted`. It resumes strictly after
the checkpoint boundary. An unknown version, scope mismatch, epoch change or
position older than retained history emits one terminal `ResnapshotRequired`
frame and closes. The server never substitutes the oldest surviving record.

Retention is at least 86,400 seconds and at least twice the measured worst-case
restore plus catch-up duration. c007 owns durable replay proof and c016/c023 own
the measured recovery and release thresholds.

### Canonical source mapping

Enrollment is an explicit schema-qualified allowlist. It fails when a table has
no primary key, no configured tenant mapping, insufficient replica identity, an
unsupported source type, duplicate projection fields, or a projection that can
expose an unapproved column.

`EntityKey.parts` is non-empty and ordered by PostgreSQL primary-key ordinal,
independent of physical column order. Key parts cannot be null. The first
fixture covers a key column that is not the first table column, a composite key,
and a text key. `EntityRecord.fields` is ordered by source column ordinal and
contains only the named server-owned projection.

Source values map as follows:

| PostgreSQL family | v2 canonical value | Canonical rule |
|---|---|---|
| `bool` | `bool_value` | protobuf boolean |
| `int2`, `int4`, `int8` | `signed_integer` | exact signed 64-bit value |
| configured unsigned domain | `unsigned_integer` | exact unsigned 64-bit value |
| `float4`, `float8` | `float_value` | finite IEEE-754 only; reject NaN/infinity |
| `numeric`, `decimal` | `decimal_value` | base-10, no exponent, no redundant leading/trailing zero, zero is `0` |
| text families and enums | `text_value` | valid UTF-8 after database semantics |
| `bytea` | `bytes_value` | exact bytes |
| `uuid` | `uuid_value` | lowercase RFC 4122 hyphenated text |
| `timestamptz` | `timestamp_value` | UTC seconds/nanos; valid protobuf range |
| `date` | `date_value` | ISO `YYYY-MM-DD` |
| `time`, `timetz`, timestamp without zone | `time_value` | ISO text including required offset/type convention |
| `json`, `jsonb` | `json_value` | UTF-8 RFC 8785 JSON Canonicalization Scheme bytes |
| SQL null in non-key fields | `NULL_KIND_NULL` | explicit null kind |

Arrays, ranges, geometric/network types, composites, domains without an
explicit mapping, invalid UTF-8, non-finite floats and out-of-range timestamps
fail enrollment or poison the affected source transaction; they are never
stringified or silently dropped. Schema evolution pauses the enrollment until
the new schema/projection mapping is accepted.

`EntityKey.canonical_id` is `frfkey:v1:` followed by unpadded base64url of UTF-8
RFC 8785 canonical JSON. The JSON is an array in key ordinal order; each item is
`{"column":<name>,"kind":<one canonical kind name>,"value":<canonical textual
or base64url value>}`. This encoding distinguishes type, column boundaries and
composite parts, so consumers never concatenate fields. The contract fixture is
the cross-language authority for later SDK tests.

The configured tenant source value is canonicalized and compared to the
verified tenant claim at enrollment/read time. It is not taken from a client
predicate. `WatchAccepted` and every `EntityMutation` repeat the effective
tenant and entity type. A persisted mutation is therefore self-describing even
when the stream-admission frame is unavailable.

### Authorization and deletes

Admission requires verified identity plus current permission for the entity
type and tenant. Before every protected mutation, the server evaluates current
object authorization using the canonical key. Expiry, revocation or authority
failure closes the stream within the c003 five-second bound and no denied
payload is emitted.

Insert/update/upsert frames require a typed projected record. Delete frames must
omit it and carry only canonical key, operation, entity type, tenant, stable
event ID and positions after the object/key authorization decision. Historical or old row
contents never cross this interface. CDC must retain the tenant and key needed
to authorize a delete; inability to do so is a poison event and blocks the
checkpoint rather than emitting an unauthenticated invalidation.

Denied historical mutations advance only the server's internal scanner. The
server can periodically send `CheckpointAdvanced`, containing an opaque client
checkpoint and no source LSN, broker offset, event identity, entity key or
payload. This lets a consumer make progress without disclosing typed metadata
about denied records. The next allowed mutation can carry the later checkpoint.

### Lag and cancellation

Every subscriber has a bounded buffer. Overflow sends one terminal `Lagged`
frame when possible and closes. It identifies whether the last safe checkpoint
is still inside retained history; otherwise the client must resnapshot. Silent
drop, unbounded buffering and continuing after overflow are forbidden.

Client transport cancellation releases the broker subscription, snapshot and
authorization work. Once the server observes cancellation, it produces no new
frame. A concurrent frame already handed to the transport may arrive, so clients
fence callbacks by their local subscription generation and commit only complete
batches.

## Compatibility and semver

c006 adds a new protobuf package and generated Rust module. Existing v1 package,
RPC names, field numbers and bytes remain unchanged. This is additive on the
wire, but the public `frf-proto::fv2` module is a pre-1.0 public API addition and
requires at least a minor version at c023.

c006 does not change `frf-domain` or `frf-ports`. Later typed domain and watch
port additions must also be additive pre-1.0 minor changes. A future adapter for
the watch source implements one dedicated port; existing LogBroker or
EntityStore adapter types do not acquire a second port implementation. Gateway
composition owns the service wiring.

## Consequences

The contract makes the current gap implementable without mutating v1 and gives
Forge/PEM explicit replay, delete and error behavior. It also adds work: the CDC
mapper, broker replay, projection store, type-watch service and packed consumers
must prove this contract in c007–c012 before `FLINT_CHANGE_SOURCE=fabric` can
become the default. This ADR alone does not make the RPC production-ready.
