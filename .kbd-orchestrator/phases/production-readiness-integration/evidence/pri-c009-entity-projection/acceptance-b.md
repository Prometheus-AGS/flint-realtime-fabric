# c009 acceptance B — restart, delete, wire, and authorization

Captured: `2026-09-16T13:28:51Z`

Status: **PASS**

## Restart and cursor coherence

The source-bound live run recorded in `acceptance-a.md` stopped the first
projector after an update, opened a new SurrealDB adapter against the same
database, and started a new projector from the persisted cursor. The v1
`GetEntity` call returned the persisted update before the projector restarted.
Inclusive replay of the cursor message emitted no duplicate watch event.

## Delete semantics

The same run committed a source-row delete. `WatchEntity` received a v1
`DELETE`, and repeated direct v1 `GetEntity` calls converged to no current
entity. The mutation and cursor were stored transactionally before the Iggy
acknowledgement.

## Per-event authorization

The watch was established while the object-level `view` relation was allowed.
Authorization was then revoked before a subsequent committed update. The next
event caused a fresh `AuthzProvider::check`; the stream returned gRPC
`PERMISSION_DENIED`, exposed no protected payload, and terminated. The local
unit regression `watch_rechecks_object_authority_for_each_event` independently
checks that the stream ends after revocation.

## v1 compatibility

No file under `proto/` or `crates/frf-proto/` changed. The live proof invokes
the existing generated `fv1::entity_service_server::EntityService` methods and
existing `GetEntityRequest`, `WatchEntityRequest`, and `EntityChange` messages.
The change extends internal ports and deployment requirements without changing
the v1 protobuf wire contract.

Acceptance marker:

```text
ENTITY_PROJECTION_PASS get=true watch=true backlog_ready=true overlap_dedup=true restart=true delete=true per_event_auth=true transaction_cancel=true
```
