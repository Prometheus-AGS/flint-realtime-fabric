# Tasks — p16-c021

- [x] Add a keto migrate init container/step to compose
- [x] Ensure gateway waits for migration completion
- [x] Document Keto migration in the runbook

## Problem (#37)

`keto serve` requires the schema to be migrated first. `keto.yml` uses `dsn: memory`
(auto-migrates in-memory), but a **persistent-DSN** deployment fails to start without an
explicit `keto migrate up` — so lifting the stack to a real database would break at boot.

## Fix

### keto-migrate init service (task 1)

Added a one-shot `keto-migrate` service to `compose.yml`: `keto migrate up -c
/etc/config/keto/keto.yml -y`, `restart: "no"`. Idempotent — a no-op for `dsn: memory`,
mandatory for a persistent DSN. Runs to completion and exits.

### Dependency chain (task 2)

- `keto` now `depends_on: keto-migrate: condition: service_completed_successfully` — so
  `keto serve` never starts against an unmigrated database.
- `gateway`'s keto dependency upgraded from `service_started` → **`service_healthy`** — so
  the gateway waits until keto (post-migration) reports ready, never booting against an
  unmigrated/unready authz store.

Verified: `docker compose -f compose.yml config` validates; the migrate service and the
`service_completed_successfully` gate are present in the resolved config;
`compose.ci.yml` still validates.

### Documentation (task 3)

Added `deploy/keto/README.md` (co-located with `keto.yml`, where an operator looks):
covers the compose auto-migration chain, running `keto migrate up` manually for non-compose
deploys (Helm/managed), idempotency, and configuring a persistent DSN. Cross-referenced to
the deployment runbook (c023), which links here for the full Keto bring-up procedure.

## Verification

- `docker compose -f compose.yml config` → valid (keto-migrate + completed_successfully +
  gateway→keto:service_healthy)
- `docker compose -f compose.ci.yml config` → valid
- `docker compose --profile full config` → valid
