# Tasks — p16-c023

- [x] Document production topology + service dependencies
- [x] Document secret provisioning
- [x] Document CDC replication-slot lifecycle + recovery (WAL pinning)
- [x] Document scaling + upgrade/rollback

## Summary (H15 — deployment/operations runbook)

Wrote `docs/RUNBOOK.md`, a single operations guide synthesizing the whole phase.
Every claim was verified against the actual implementation (not aspirational).

### 1. Topology & service dependencies

ASCII topology (gateway ↔ flint-gate/Keto/Iggy/Postgres/SurrealDB) + the exact compose
boot order: `keto-migrate` (completed) → `keto` (healthy) → deps healthy → `gateway`
(waits on keto **healthy**). Port map. Health/readiness/metrics endpoints and their
operational meaning (`/healthz` = restart, `/readyz` = rotation, `/metrics` = scrape).
Graceful-shutdown behavior + termination grace period guidance.

### 2. Secret provisioning

Table of every secret (`FLINT_GATE_JWT_SECRET`, `IGGY_CONNECTION_STRING`,
`CDC_REPLICATION_URL`, `LIVEKIT_*`, `MATRIX_ACCESS_TOKEN`), where it's used, and
provisioning + rotation steps. Explicit "never set DEV_NO_AUTH in prod" and "set
JWT_ISSUER" guidance.

### 3. CDC slot lifecycle & recovery (the WAL-pinning trap)

Slot creation by the consumer; the critical **WAL-pinning** failure mode (a stalled
consumer pins WAL → fills the Postgres disk). Includes the detection SQL
(`pg_replication_slots` lag query) and the recovery playbook: restart the consumer;
drop the slot to release WAL (accepting backlog loss); emergency disk-pressure order.

### 4. Scaling, upgrade, rollback

Gateway scales horizontally behind `/readyz`; **CDC must be a singleton** (one
consumer per slot) while data-plane replicas run `CDC_ENABLED=false`; LiveKit
cross-node-inbound limitation noted. Rolling upgrade (migrate first, drain via graceful
shutdown, frozen proto makes it safe within v1) and rollback guidance (image redeploy;
Keto migrations forward-only; CDC slot check).

## Verification

Cross-checked runbook claims against code/config: `/readyz` + the 3 `/metrics` series +
`keto-migrate` + `wal_level=logical` + `shutdown_signal` all present. Referenced docs
(`ENVIRONMENT.md`, `deploy/keto/README.md`) exist; `SECURITY.md` is a forward reference
to c024.
