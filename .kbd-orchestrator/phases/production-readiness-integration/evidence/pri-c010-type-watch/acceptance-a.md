# c010 acceptance A — authorized committed type delivery

Captured: `2026-09-16T22:39:30Z`

Status: **PASS**

The application isolation regression uses an intentionally overbroad projection
fixture that returns rows for other tenants and entity types. The snapshot path
independently filters those rows before object authorization and payload
construction, proving isolation does not depend on projection query correctness.

## Real adapter and transport proof

Command:

```sh
./scripts/run-entity-type-watch-integration.sh --preflight-only
./scripts/run-entity-type-watch-integration.sh
```

The source-bound run created an owned local PostgreSQL 17, Iggy, and SurrealDB
Compose project and served the generated `flint.v2.EntityService` over a real
tonic TCP listener. Two independently authorized generated clients subscribed
before a PostgreSQL transaction committed. Both received the same stable event
from WAL -> Iggy -> durable typed projection -> RPC. A third same-tenant subject
was denied during admission.

- Run: `20260916223916-49993`
- Exit: `0`
- Candidate artifact-set SHA-256:
  `e76a442f83ab1d1a000e9d7125826e1e263bf7baedaf758d365b131d4d674962`
- Test-log SHA-256:
  `7baa130212aadca11a97502caff08bc1183ca3705110d98b6e1a426056055b42`
- Cleanup: `owned-resources-removed`
- Marker:
  `ENTITY_TYPE_WATCH_PASS tonic=true subscribers=2 snapshot=true resume=true delete_key_only=true unauthorized_denied=true`

## Scope and disclosure proof

`cargo test --offline -p frf-app --test entity_type_watch --locked` passed all
three tests. The `authorized_subscribers_share_type_history_without_cross_scope_payloads`
case places the enrolled type, another type, another tenant, an object-denied
same-tenant row, and an allowed row in the same source history. Each subscriber
receives only the allowed snapshot row and the subsequent allowed live mutation.
Denied and out-of-scope events can advance only an opaque client checkpoint; no
key, event identity, source position, entity type, tenant, or record crosses the
application boundary.

The snapshot fixture's durable cursor advances through later other-type,
other-tenant, and object-denied rows. The visible barrier remains at LSN 100,
the authorized row's source position, while the later durable position appears
only inside the encrypted checkpoint.

The transport regression `delete_transport_never_exposes_a_record` also proves
that a delete's record is removed before protobuf encoding even if an upstream
caller accidentally supplies one. Malformed canonical byte values fail with
gRPC `DATA_LOSS` rather than being replaced with empty data.

No CI test result or skipped dependency substitutes for this evidence.
