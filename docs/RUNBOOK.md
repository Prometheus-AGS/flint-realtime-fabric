# Deployment & Operations Runbook — flint-realtime-fabric

Operating guide for the gateway and its dependencies in production. Companion docs:
[`ENVIRONMENT.md`](ENVIRONMENT.md) (env-var reference), [`deploy/keto/README.md`](../deploy/keto/README.md)
(Keto migration), and the security model ([`SECURITY.md`](SECURITY.md), p16-c024).

---

## 1. Topology & service dependencies

The stack (see `compose.yml` for the reference deployment):

```
                       ┌──────────────┐
   browser / SDKs ───▶ │  frf-gateway │ ── gRPC/gRPC-web :9090 ─┐
   (JWT bearer)        │  HTTP :8080  │ ── WS /ws/v1/*          │
                       └──────┬───────┘                          │
          verifies JWT via    │ authz (per-event view checks)    │ media signaling
      ┌─────────────┐         │         ┌──────────┐             │
      │  flint-gate │◀────────┘         │   Keto   │◀────────────┘
      │  JWKS :4457 │                   │  :4466   │
      └─────────────┘                   └────┬─────┘  (Zanzibar authz)
                                             │ depends_on: keto-migrate (completed)
   ┌────────────┐   ┌──────────────┐   ┌─────┴─────┐   ┌───────────┐
   │ iggy-server│   │  postgres    │   │ keto-migr │   │ surrealdb │
   │  :8090     │   │  :5432 (WAL) │   │ (one-shot)│   │  :8000    │
   │ event spine│   │  CDC source  │   └───────────┘   │ server store│
   └────────────┘   └──────────────┘                   └───────────┘
```

**Boot dependency order** (compose `depends_on` conditions):

1. `keto-migrate` runs `keto migrate up` and exits.
2. `keto` starts only after `keto-migrate` **completed successfully**, then becomes
   healthy on `/health/ready`.
3. `iggy`, `postgres`, `surreal`, and `flint-gate` become healthy.
4. `gateway` starts only after `iggy` (healthy), `keto` (**healthy**), and
   `postgres` and `surreal` (healthy) — it never boots against an
   unmigrated/unready authz or projection store.

**Ports:** the reference profile publishes only the TLS edge. Gateway, Gate,
Keto, Iggy, Postgres, and SurrealDB remain on internal Compose networks.

### Health & readiness

- **`GET /healthz`** — liveness (process up). Use for the RESTART decision.
- **`GET /readyz`** — readiness: probes Keto, JWKS, and Iggy; returns 503 with
  per-dependency detail when any is down. Use for the load-balancer ROTATION decision
  (K8s `readinessProbe`). The compose gateway healthcheck gates on `/readyz`.
- **`GET /metrics`** — Prometheus exposition (`frf_publish_total`,
  `frf_events_delivered_total`, `frf_subscriptions_opened_total`).

### Graceful shutdown

On **SIGTERM** or **SIGINT** the gateway stops accepting new connections and drains
in-flight HTTP requests and WebSocket streams before exiting. Set the orchestrator's
termination grace period ≥ the longest expected request (K8s
`terminationGracePeriodSeconds`, default 30s is usually fine).

---

## 2. Secret provisioning

Secrets must come from a secret manager or a gitignored `.env` — never the repo.
See [`ENVIRONMENT.md`](ENVIRONMENT.md) for the full list. The secrets are:

| Secret | Used by | Notes |
|--------|---------|-------|
| `FLINT_GATE_JWT_SECRET` | flint-gate | HMAC signing secret for minted JWTs. Compose `compose.yml` fails to start if unset. **Rotate on any exposure.** |
| `IGGY_CONNECTION_STRING` | gateway | Contains broker credentials. |
| `CDC_REPLICATION_URL` | gateway (CDC) | Contains Postgres replication credentials. |
| `ENTITY_PROJECTION_PASSWORD` | gateway, SurrealDB | Protects the durable entity projection. |
| `LIVEKIT_API_KEY` / `LIVEKIT_API_SECRET` | gateway (hosted SFU) | LiveKit server credentials. |
| `MATRIX_ACCESS_TOKEN` | gateway (federation) | Only if federation is enabled. |

**Provisioning steps:**

1. Generate a strong random `FLINT_GATE_JWT_SECRET` (e.g. `openssl rand -hex 32`).
2. Inject secrets via the orchestrator's secret store (K8s `Secret` → env, Vault,
   cloud secret manager). For compose, use a gitignored `.env`.
3. **Never** set `DEV_NO_AUTH` in production. The release image is built without the
   `dev-endpoints` feature, so the bypass cannot even exist in the production binary.
4. Set `JWT_ISSUER` so only your IdP's tokens are accepted.

**Rotation:** rotate `FLINT_GATE_JWT_SECRET` by deploying the new value to flint-gate;
since the gateway verifies via JWKS (asymmetric or fetched keys), coordinate the cutover
with your IdP/flint-gate key rollover. Rotate DB/broker/LiveKit credentials at the source
and update the secret store.

---

## 3. CDC replication-slot lifecycle & recovery

CDC (`CDC_ENABLED=true`) streams Postgres logical-replication changes onto the spine.
The reference Compose profile defaults it off until reviewed migrations add
every configured enrollment to the explicit publication.
Postgres must run with `wal_level=logical` (the reference compose sets
`-c wal_level=logical -c max_replication_slots=5 -c max_wal_senders=5`).

**Lifecycle:** a reviewed database migration creates the explicit publication
(`CDC_PUBLICATION_NAME`) and adds its tables. `FOR ALL TABLES` is rejected. The
gateway validates `CDC_ENROLLMENTS_JSON` against the live catalog before creating
or opening `CDC_SLOT_NAME`. It rejects missing keys, inadequate replica identity,
unapproved or duplicate projection columns, and unsupported source types.

The consumer buffers row events until PostgreSQL COMMIT, maps keys and values to
the canonical v2 contract, and publishes every mutation in transaction order.
It advances the slot's confirmed LSN only after the full commit is durable in
Iggy. A broker, decode, schema, or transaction error leaves that commit below the
checkpoint. A retry republishes the same stable event IDs, and Iggy deduplicates
the already-durable prefix.

`CDC_SOURCE_EPOCH` identifies the slot's logical history. Keep it stable across
normal gateway and database restarts. Rotate it when the slot is dropped and
recreated, or when PostgreSQL is restored to a history that can reuse source
positions; downstream type watches must resnapshot for the new epoch.

### ⚠️ WAL pinning — the critical failure mode

A replication slot **pins WAL**: Postgres cannot recycle WAL past the slot's confirmed
LSN. If the gateway (the only consumer) is **down or stalled**, the slot stops advancing
and WAL accumulates — eventually **filling the Postgres disk** and taking the database
down.

**Detect:**
```sql
-- Slot lag (bytes of WAL retained). Growing unbounded = danger.
SELECT slot_name, active,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), confirmed_flush_lsn)) AS retained
FROM pg_replication_slots;
```
Alert when `retained` grows past a threshold, or `active = false` while CDC is enabled.

**Recover:**
1. **Preferred — restart the consumer:** bring the gateway back up so it reconnects to
   the slot and drains the backlog; the slot advances and WAL frees.
2. **If the gateway will be down for a long time / disk is critical:** drop the slot to
   release WAL immediately (you will lose the un-consumed CDC backlog):
   ```sh
   frf cdc slot drop --slot frf_slot          # or: SELECT pg_drop_replication_slot('frf_slot');
   ```
   The gateway recreates it on next start from the current LSN, so the gap is lost.
   Rotate `CDC_SOURCE_EPOCH` and require downstream resnapshot before restoring
   service. To pre-create it explicitly: `frf cdc slot create`.
3. **Emergency disk pressure:** free space (archive/expand) first so Postgres stays up,
   then apply (1) or (2).

CLI reference:
- `frf cdc status` — resolved slot/publication config + replication URL.
- `frf cdc slot create|drop --slot <name>` — manage the logical replication slot.
- `frf broker checkpoint --channel <uuid> --consumer <id> --offset <n>` — force a cursor.
- `frf broker offsets --channel <uuid> --consumer <id>` — read a consumer's stored offset.
- `frf keto seed|revoke` — manage Keto relation tuples.

Local acceptance for mapping and checkpoint behavior:

```sh
./scripts/run-cdc-integration.sh
```

The runner uses an owned PostgreSQL 17 plus Iggy Compose project, executes the
ignored test by exact name, records image and source hashes, and removes only its
project's containers and volumes.

### Entity projection lifecycle and recovery

The CDC publisher and entity projector form one ordered pipeline. The projector
stores each entity mutation and its broker cursor in one SurrealDB transaction,
then acknowledges the Iggy message. A restart resumes inclusively from the stored
cursor; replay of the cursor message is detected as a duplicate. `/readyz` remains
unready while CDC is enabled until both the logical-replication publisher and the
durable projector are active.

Keep `CDC_SOURCE_EPOCH`, `ENTITY_PROJECTION_NAMESPACE`, and
`ENTITY_PROJECTION_DATABASE` stable across ordinary restarts and rollbacks. Back
up the `surreal-data` volume with the same recovery point as the source database.
If the replication slot is recreated, the source database is restored to another
history, or the projection cursor reports a different source epoch, install a
fresh projection snapshot and resume from its checkpoint. Do not edit the cursor
or skip a poison message manually.

Detect a stalled projection by combining `/readyz`, Iggy consumer lag, and the
Postgres slot-lag query above. Restore service in this order:

1. Keep the CDC slot and Iggy data intact; repair or restart SurrealDB.
2. Restart the singleton CDC/projector gateway and allow inclusive cursor replay.
3. If the projection state or source history is irrecoverable, create a consistent
   source snapshot, install it with the matching source epoch and checkpoint, then
   restart the projector.

Local acceptance for the complete database-to-v1 API path:

```sh
./scripts/run-entity-projection-integration.sh
```

The runner owns PostgreSQL, Iggy, and SurrealDB fixtures and proves direct v1
`GetEntity` and `WatchEntity`, snapshot/WAL overlap, restart recovery, delete
handling, and per-event authorization before removing only its owned resources.

---

## 4. Scaling, upgrade & rollback

### Scaling

- **Gateway** is horizontally scalable for the HTTP/gRPC data plane — run N replicas
  behind a load balancer using `/readyz` for rotation. Per-event authz is enforced on
  every replica.
- **CDC** must run on **exactly one** gateway replica (a single logical-replication
  consumer per slot). Run CDC as a singleton: either a dedicated single-replica
  deployment with `CDC_ENABLED=true`, or a leader-elected instance; the data-plane
  replicas run with `CDC_ENABLED=false`. The durable entity projector runs with
  that singleton so only one consumer advances its projection cursor.
- **Media (LiveKit hosted):** outbound signaling is cross-node; cross-node **inbound**
  relay is a known v1 limitation (see the LiveKit adapter docs) — single-process
  signaling is unaffected.

### Upgrade (rolling)

1. Run any new **Keto migrations** first (`keto migrate up` / the `keto-migrate` step).
2. Roll gateway replicas one at a time. `/readyz` keeps a replica out of rotation until
   its dependencies are reachable; graceful shutdown drains the old replica.
3. The proto contract is frozen (`proto-v1`) — a wire-breaking change is a new proto
   version, so rolling gateway + SDKs is safe within v1.

### Rollback

- **Gateway image:** redeploy the previous image tag; `/readyz` + graceful shutdown make
  this safe mid-request.
- **Keto migrations:** Keto migrations are generally forward-only — prefer rolling
  forward with a fix. If a rollback is unavoidable, restore the Keto DB from backup
  (do NOT down-migrate a live authz store casually).
- **CDC:** retain the same enrollment and source epoch when rolling back to a
  compatible consumer. Verify the slot remains active and lag is bounded. Never
  move the slot forward manually to bypass a poison commit; repair the mapping or
  source data and restart from the unchanged confirmed LSN. Retain the matching
  SurrealDB projection and cursor; a rollback that cannot read them requires a
  source-bound resnapshot before traffic returns.
