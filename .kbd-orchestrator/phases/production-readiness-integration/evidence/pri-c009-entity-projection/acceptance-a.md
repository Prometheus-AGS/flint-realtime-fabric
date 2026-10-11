# c009 acceptance A — database commit to v1 entity APIs

Captured: `2026-09-16T13:28:51Z`

Status: **PASS**

## Exact local proof

```sh
./scripts/run-entity-projection-integration.sh --preflight-only
./scripts/run-entity-projection-integration.sh
```

The runner created an owned PostgreSQL 17, Iggy, and SurrealDB Compose project,
ran the ignored acceptance test by exact name, captured immutable image IDs, and
removed only that project's containers and volumes.

Runtime receipt: `target/entity-projection-receipts/20260916132836-97391/receipt.json`

- Exit status: `0`
- Candidate artifact-set SHA-256:
  `bf36b30267b69ac6f60f433ce7e69ff157bb0047161dc5ff12b44627993a38de`
- Test-log SHA-256:
  `61e705fc0dc1bb7bd3286e973f10e6a35268153036c1ae7ad405ede34a0e5031`
- Cleanup: `owned-resources-removed`

## Observed contract

The test committed an insert through PostgreSQL, consumed the transaction only
after its COMMIT boundary, published the canonical mutation through real Iggy,
stored the entity and cursor transactionally in real SurrealDB, and read the
result through the existing v1 `GetEntity` and `WatchEntity` service methods.

It then installed a snapshot at the same broker checkpoint and restarted the
projector inclusively. The overlapping mutation was classified as a duplicate;
the projected state and cursor did not regress and no duplicate watch change was
emitted. The acceptance marker was:

```text
ENTITY_PROJECTION_PASS get=true watch=true backlog_ready=true overlap_dedup=true restart=true delete=true per_event_auth=true transaction_cancel=true
```

This is runtime evidence from real adapters. No CI result or compile-only check
is used as acceptance evidence.
