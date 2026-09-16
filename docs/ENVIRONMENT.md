# Environment Variable Reference — frf-gateway

Every environment variable read by the gateway (and its adapters), what it does,
whether it is required, and whether it is a secret or dev-only. Copy
[`.env.example`](../.env.example) to `.env` (gitignored) to configure a local stack.

Legend: **Req** = required at boot · **Secret** = never commit / use a secret
manager · **Dev-only** = must NOT be set in production.

## Core / networking

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `BIND_ADDR` | | | `0.0.0.0:8080` | HTTP (Axum) listen address. |
| `GATEWAY_PROFILE` | | | `full` | `full` mounts the normal FRF surfaces; `shape-only` mounts only health, readiness, and the authorized Electric facade and requires the `shape-facade` build feature, `GRPC_PORT=0`, CDC off, and federation off. |
| `GRPC_PORT` | | | `9090` | gRPC / gRPC-web port. `0` disables the gRPC server. |
| `IGGY_CONNECTION_STRING` | ⚠️ | ✅ | — | Iggy broker DSN `iggy://user:pass@host:port`. Required by `full`; rejected by `shape-only`. |
| `RUST_LOG` | | | `info` | `tracing` log filter. |

## Identity & authorization (security-critical)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `GATEWAY_JWKS_URL` | ✅ | | — | flint-gate JWKS endpoint used to verify inbound JWTs. |
| `JWT_AUDIENCE` | ✅ | | — | Expected JWT `aud`. |
| `JWT_ISSUER` | ✅ (prod) | | _(dev builds only: unset → not validated)_ | Expected JWT `iss`. **Required in production:** a production (non-`dev-endpoints`) binary **fails to boot** with a config-validation error if this is unset — without it the `iss` claim is unvalidated and any JWKS-valid token from any issuer is accepted. In `dev-endpoints` builds it is relaxed to a startup warning (p16-c004, enforced p17-c002). |
| `KETO_BASE_URL` | ✅ | | — | Ory Keto read/write API base URL. |
| `KETO_NAMESPACE` | | | `default` | Keto namespace for relation tuples. |
| `POLICY_ENGINE` | | | `none` | Action policy engine: `none` (allow-all, logged) or `cedar`. |
| `DEV_NO_AUTH` | | | `false` | **DEV-ONLY BYPASS.** When `true` AND the binary is built with the `dev-endpoints` feature, JWT verification is skipped for publish/subscribe. **Never set in production** — the release image is built without `dev-endpoints`, so this has no effect there (p16-c001). |

> The production profiles require a matching asymmetric Gate signing-key pair.
> `flint-gate-db-init` seeds its public half into Gate's database before Gate
> starts, so the public JWKS endpoint cannot report ready with an empty key set.

## Security middleware (p16-c005)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `RATE_LIMIT_PER_SEC` | | | `50` | Global request rate cap (requests/sec). |
| `RATE_LIMIT_BURST` | | | `100` | Burst allowance above the sustained rate. |
| `MAX_BODY_BYTES` | | | `1048576` | Max request body size (1 MiB). |
| `CORS_ALLOWED_ORIGINS` | | | _(empty → no CORS)_ | Comma-separated exact-match allowed origins. |

## Media / SFU (p16-c010, p16-c020)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `SFU_MODE` | | | `hosted` | `hosted` (LiveKit — supported v1 path) or `sovereign` (str0m — DEFERRED, no media flows). |
| `MEDIA_ENABLED` | | | `false` | Explicit media-lane opt-in. The selected `full` data and `shape-only` release profiles set this false; LiveKit requirements apply only when true. |
| `AGENT_ENABLED` | | | `false` | Explicit agent lane opt-in. The selected data profiles leave it disabled. |
| `ADMIN_ENABLED` | | | `false` | Explicit embedded admin UI opt-in. The selected data profiles leave it disabled. |
| `LIVEKIT_API_KEY` | ⚠️ | ✅ | — | Required when `SFU_MODE=hosted` (validated at boot). |
| `LIVEKIT_API_SECRET` | ⚠️ | ✅ | — | Required when `SFU_MODE=hosted`. |
| `LIVEKIT_SERVER_URL` | ⚠️ | | — | Required when `SFU_MODE=hosted`. |
| `LIVEKIT_ROOM_PREFIX` | | | `frf/` | Prefix for tenant-namespaced LiveKit rooms. |

⚠️ = required only in the applicable mode; `config.validate()` fails boot if missing.

## Restricted shape facade

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `SHAPE_ELECTRIC_URL` | ⚠️ | | — | Electric origin. Required by and exclusive to `shape-only`. Readiness performs a real `/v1/health` request. |
| `SHAPE_CATALOG_PATH` | ⚠️ | | — | Read-only JSON catalog path. Required with `SHAPE_ELECTRIC_URL` and exclusive to `shape-only`. |
| `SHAPE_TIMEOUT_SECS` | | | `30` | Upstream Electric request timeout. |

## CDC (Postgres logical replication)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `CDC_ENABLED` | | | `false` | Enable Postgres CDC → spine ingestion. |
| `CDC_REPLICATION_URL` | ⚠️ | ✅ | — | Normal Postgres DSN (contains credentials). Required when `CDC_ENABLED=true`; the adapter adds replication mode only to its WAL connection. |
| `CDC_SLOT_NAME` | ⚠️ | | — | Logical replication slot. Required when `CDC_ENABLED=true`. |
| `CDC_PUBLICATION_NAME` | ⚠️ | | — | Publication name. Required when `CDC_ENABLED=true`. |
| `CDC_TENANT_ID` | ⚠️ | | — | Effective tenant UUID for `fixed` tenant enrollments and the CDC channel. Required when CDC is enabled. |
| `CDC_CHANNEL_PATH` | | | — | Channel path on the spine (e.g. `entities`). |
| `CDC_SOURCE_EPOCH` | ⚠️ | | — | Stable identity for this slot's logical history. Keep it across ordinary restarts; rotate it after slot/history recreation or source restore. |
| `CDC_ENROLLMENTS_JSON` | ⚠️ | | — | Non-empty server-owned JSON allowlist of schema, table, projection, projected columns, tenant mode and optional unsigned-domain columns. Catalog validation fails startup for an unsafe mapping. |

### Durable entity projection

These values are required when `CDC_ENABLED=true`. The gateway consumes its
committed CDC stream into this SurrealDB store before reporting CDC readiness.
The reference full Compose profile always provisions that store so its topology
does not change when CDC is enabled; its renderer therefore also requires the
corresponding `FRF_SURREAL_*` image, account, namespace, database, and password
inputs while CDC remains disabled.

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `ENTITY_PROJECTION_URL` | ⚠️ | | — | SurrealDB WebSocket endpoint, such as `ws://surreal:8000`. |
| `ENTITY_PROJECTION_USERNAME` | ⚠️ | | — | SurrealDB account used by the projection adapter. |
| `ENTITY_PROJECTION_PASSWORD` | ⚠️ | ✅ | — | Password for the projection account. |
| `ENTITY_PROJECTION_NAMESPACE` | ⚠️ | | — | SurrealDB namespace containing the entity projection. |
| `ENTITY_PROJECTION_DATABASE` | ⚠️ | | — | SurrealDB database containing projected entities and the CDC cursor. |

The configured publication must exist, must not use `FOR ALL TABLES`, and must
already contain every enrolled table. Add tables through reviewed database
migrations. Each table needs a primary key and replica identity that retains its
primary key plus a column-based tenant key. `fixed` tenant tables need the primary
key in replica identity. Unsupported types stop enrollment.

## Federation (DEFERRED / half-implemented — off by default) (p16-c009)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `FEDERATION_ENABLED` | | | `false` | Opt-in for Matrix/ATProto bridges (half-implemented for v1). |
| `FEDERATION_TENANT_ID` | ⚠️ | | — | Tenant for ingested federated events. Required when federation is enabled. |
| `FEDERATION_CHANNEL_ID` | ⚠️ | | — | Channel for ingested federated events. **Required when federation is enabled** — the gateway refuses to boot without it (else events land on a random per-boot channel; p18-c002). |
| `MATRIX_HOMESERVER_URL` | | | — | Matrix homeserver (outbound send only for v1). |
| `MATRIX_ACCESS_TOKEN` | | ✅ | — | Matrix access token (secret). |
| `MATRIX_ROOM_ID` | | | — | Matrix room to bridge. |
| `ATPROTO_JETSTREAM_URL` | | | — | ATProto Jetstream firehose (inbound). |
| `ATPROTO_COLLECTIONS` | | | _(empty)_ | Comma-separated ATProto collections to ingest. |
| `ATPROTO_PDS_URL` | ⚠️ | | — | PDS base URL (e.g. `https://bsky.social`) for **outbound** writes. **All-or-none** with the two below — set all three to enable outbound, or none (inbound-only). The gateway refuses to boot if only some are set (p19-c002). |
| `ATPROTO_PDS_IDENTIFIER` | ⚠️ | | — | PDS account identifier (handle or DID) for outbound writes. Required with the PDS-writer group. |
| `ATPROTO_PDS_APP_PASSWORD` | ⚠️ | ✅ | — | PDS **app-password** (NOT the account password) for outbound writes. **Secret** — from a secret manager, never committed or logged. Required with the PDS-writer group. |
| `ATPROTO_WRITE_COLLECTION` | | | `app.bsky.feed.post` | Lexicon collection outbound records are written into. Optional. |

## Actor registry / eviction

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `REGISTRY_IDLE_SECS` | | | `300` | Idle time before a tenant actor is evicted. |
| `REGISTRY_SWEEP_INTERVAL_SECS` | | | `60` | Eviction sweep interval. |

## Observability

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | | | _(unset → stdout logs only)_ | OTLP endpoint; when set, spans export via OpenTelemetry. |
| `OTEL_SERVICE_NAME` | | | `frf-gateway` | OTEL `service.name` resource attribute. |

The `/metrics` endpoint (Prometheus) is always available and needs no env config (p16-c018).

---

## Deployment renderer and TLS boundary

Run `scripts/render-deployment-profile.sh full` or
`scripts/render-deployment-profile.sh shape-only` after supplying the variables
in `.env.example`. The renderer verifies every image is an immutable digest,
validates the certificate/key/CA chain and Gate signing-key pair, requires HTTPS
issuer and public origins, and rejects anonymous or symmetric Gate configurations.
The one-shot Gate database initializer must seed the public key successfully
before the Gate runtime starts.

`k8s/overlays/ssr/render-profile.sh` also requires the TLS files and
`FRF_TLS_SECRET_NAME`. It emits the bound `kubernetes.io/tls` Secret with the
rendered SSR resources after checking trust and key correspondence. Treat the
rendered output as secret material and store or transmit it accordingly.

Both Compose profiles publish only the TLS edge. The full profile exposes HTTPS
on `FRF_HTTPS_PORT` and TLS gRPC on `FRF_GRPC_HTTPS_PORT`; its edge admits only
Spine, Sync, and Entity gRPC services. Gateway, Gate, Iggy, Keto,
Electric and Postgres ports use internal Compose networks and cannot be reached
through a host-published backend port. The full edge forwards all supported data
routes. The shape-only edge forwards only `/healthz`, `/readyz`, and `/v1/shape`;
all other paths return `404` at the TLS boundary.

## Minimum required to boot (production)

The `full` profile requires `IGGY_CONNECTION_STRING`, `KETO_BASE_URL`,
`GATEWAY_JWKS_URL`, `JWT_AUDIENCE`, and `JWT_ISSUER`. When its reference profile
enables CDC, it also requires the CDC URL, slot, publication, tenant, source epoch,
enrollment allowlist, and durable entity-projection values described above. The
`shape-only` profile
requires the identity inputs plus both shape inputs and rejects Iggy, gRPC, CDC,
federation and media. `LIVEKIT_*` is required only when `MEDIA_ENABLED=true` and
`SFU_MODE=hosted`. `config.validate()` fails before binding a public socket when
these profile rules are violated.
