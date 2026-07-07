# Ory Keto — deployment & schema migration

Keto is the Zanzibar-style authorization store for FRF (per-event `view` checks,
tenant isolation). This directory holds `keto.yml` (the Keto config) which the
compose stack mounts into the Keto containers.

## Schema migration (required for persistent DSNs)

Keto must have its schema migrated **before** `keto serve` starts. The compose
stack does this automatically:

- **`keto-migrate`** — a one-shot service that runs `keto migrate up -y` against
  the configured DSN and exits.
- **`keto`** — `depends_on: keto-migrate: condition: service_completed_successfully`,
  so `keto serve` never starts against an unmigrated database.
- **`gateway`** — `depends_on: keto: condition: service_healthy`, so it never
  boots against an unmigrated/unready authz store.

For the default `dsn: memory` in `keto.yml`, the migration is an in-memory no-op
(harmless to always run). For a **persistent DSN** (Postgres/MySQL/CockroachDB),
the migration is mandatory — without it, `keto serve` fails to start, which was the
production defect this step fixes (p16-c021).

## Running the migration manually

If you deploy Keto outside compose (e.g. a Helm chart or a managed instance), run
the migration as a pre-start / init step before `keto serve`:

```bash
keto migrate up -c /path/to/keto.yml -y
```

`migrate up` is idempotent — safe to run on every deploy. Run it after any Keto
version upgrade that ships new migrations.

## Configuring a persistent DSN

Set `dsn` in `keto.yml` (or the `DSN` env var) to your database URL, e.g.:

```yaml
dsn: postgres://keto:secret@postgres:5432/keto?sslmode=disable
```

Then the `keto-migrate` step creates/updates the schema on that database before
`keto serve` starts.

> Cross-reference: the deployment runbook (docs, p16-c023) links here for the
> full Keto bring-up and upgrade procedure.
