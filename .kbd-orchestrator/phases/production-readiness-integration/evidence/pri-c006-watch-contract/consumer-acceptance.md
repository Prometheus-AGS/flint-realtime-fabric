# c006 consumer acceptance before code generation

Recorded: `2026-09-15T14:24:06Z`

Status: **ACCEPTED CONTRACT INPUT** for the c006 v2 schema. This record precedes
the first c006 edit under `proto/flint/` and the first v2 code-generation run.
It does not claim that Forge or PEM have implemented the new transport; those
are c011 and c012 gates.

## Authority

The phase operator accepted Q4–Q6 in c003 and authorized execution of this
phase. This record translates those decisions into consumer-visible behavior
against clean Forge revision `dc313be3a044c65b05d845a9c34350bf5ca3ca3e`
and clean PEM revision `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565`.
The relevant source hashes are fixed in `baseline.md`.

## Request contract

- Forge supplies one validated schema-qualified entity type and one tenant from
  the verified subscriber context. A client cannot supply a predicate, table,
  projection, authorization subject or arbitrary column list.
- A first request can ask for an authorized snapshot followed by live changes,
  or resume from one server-issued opaque checkpoint. Supplying both is invalid.
- PEM stores the server-issued checkpoint as a batch boundary. It does not
  manufacture or increment source LSNs or broker offsets.
- Cancellation uses transport cancellation and must release the server-side
  subscription promptly; cancellation is not encoded as an entity mutation.

## Response contract

- Every mutation identifies the schema-qualified entity type, tenant, canonical
  ordered key, stable event identity, committed source epoch/LSN and transaction
  ordinal, broker partition/offset, and the server-issued client checkpoint.
- Insert/update/upsert frames carry an allowlisted typed row projection. Delete
  frames carry only the authorized key and projection identity; they never carry
  historical/old row contents.
- Snapshot rows use the same typed entity/key representation as live frames.
  `snapshot_complete` identifies the consistent source barrier. Live changes
  begin strictly after that barrier; at-least-once duplicates retain event IDs.
- Ordering is guaranteed within one source transaction and broker partition.
  No global ordering across partitions is promised. Consumers apply idempotently
  and deduplicate by event identity.
- A checkpoint from another tenant/type, an unknown source epoch, or history
  older than retention yields a terminal `resnapshot_required` control frame.
  It never silently begins at the oldest retained event.
- A bounded-buffer overflow yields a terminal lag control frame and closes the
  stream. The control tells the consumer whether its last checkpoint is still
  resumable or a new authorized snapshot is required.
- Unauthorized rows and historical mutations never enter a data frame. The
  server may advance its internal scan and issue only an opaque progress
  checkpoint, without key, entity payload, event ID, source LSN or broker
  position, so a denied record cannot be inferred from typed mutation metadata.

## Mapping accepted by Forge

- `SubscriptionSpec.entity_type` maps to the schema-qualified request type.
- `SubscriptionSpec.tenant` maps to the tenant selector derived from identity.
- Insert/update/upsert typed fields map to Forge JSON only after canonical value
  validation; unsupported values terminate rather than being silently dropped.
- Delete keys map to the existing RLS re-query/invalidation path. Missing row
  data is intentional and cannot be treated as an empty upsert.
- Status controls map to explicit unavailable/resnapshot/lag errors; an empty
  stream is never reported as a healthy subscription.

## Mapping accepted by PEM

- Canonical entity type and key map to PEM `type` and stable `id`. Composite keys
  use the contract's canonical key encoding and never field concatenation.
- Mutation kind maps to PEM insert/update/delete/upsert. A typed row maps to
  `data`; delete omits `data`.
- The opaque server checkpoint maps to PEM's batch-level cursor
  `{handle, offset}` without exposing source or broker offsets as a client-owned
  counter. c012 owns the packed SDK representation and network proof.
- `resnapshot_required` triggers an authorized rebuild. Lag and cancellation do
  not publish partial batches or advance the durable PEM checkpoint.

## Compatibility acceptance

- The six current `proto/flint/v1/*.proto` hashes in `baseline.md` remain exact.
- The new RPC uses package `flint.v2`; it neither edits nor overloads v1
  `EntityService.WatchEntity`.
- v1 and v2 generated Rust modules coexist. Existing v1 callers retain their
  wire and source namespace; adopting v2 is an explicit consumer change.
- Domain/port additions are public pre-1.0 API additions requiring at least a
  minor release at c023. No adapter implementation is added in c006, preserving
  one-port-per-adapter ownership for later changes.
