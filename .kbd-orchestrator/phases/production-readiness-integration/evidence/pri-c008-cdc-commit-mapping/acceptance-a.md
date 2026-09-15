# c008 acceptance A — typed committed CDC mapping

Status: **PASS**

Command:

```text
./scripts/run-cdc-integration.sh
```

The owned PostgreSQL 17 and Iggy fixture completed at `2026-09-15T20:23:22Z`
with exit status 0. The ignored integration test ran exactly once and printed:

```text
CDC_COMMIT_MAPPING_PASS core=6 crash_unique=2 poison_blocked=true schema_blocked=true
test result: ok. 1 passed; 0 failed; 0 ignored
```

The live assertions prove:

- INSERT, UPDATE and DELETE produce six committed events; a rolled-back row
  produces none.
- Three mutations in one transaction share the commit LSN and preserve source
  order as transaction indexes 0, 1 and 2.
- `c008.orders@summary` uses a composite `(region text, number bigint)` key even
  though those columns are not the first source columns.
- Column-derived UUID tenants and an explicitly configured fixed tenant both
  retain the accepted tenant value, and every envelope tenant matches its
  decoded source payload tenant.
- Booleans, signed integers, floats, exact decimals, bytes, UUIDs, temporal
  values and canonical JSON cross the real WAL path; unchanged TOAST is named
  explicitly on UPDATE.
- Array mapping and insufficient replica identity fail catalog enrollment
  before replication begins.
- The test compiles and executes under the gateway composition crate; the CDC
  adapter has no dependency on the Iggy adapter.

The exact candidate source hashes are in `source-binding.sha256`; the complete
machine-readable result is in `runtime-receipt.json`.
