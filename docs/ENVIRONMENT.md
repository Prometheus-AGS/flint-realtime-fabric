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
| `GRPC_PORT` | | | `9090` | gRPC / gRPC-web port. `0` disables the gRPC server. |
| `IGGY_CONNECTION_STRING` | ✅ | ✅ | — | Iggy broker DSN `iggy://user:pass@host:port` (contains credentials). |
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

> `FLINT_GATE_JWT_SECRET` is consumed by the **flint-gate** service (not the gateway)
> to sign minted JWTs. It is **secret** and required for the flint-gate container —
> see [`.env.example`](../.env.example) and `deploy/flint-gate`.

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
| `LIVEKIT_API_KEY` | ⚠️ | ✅ | — | Required when `SFU_MODE=hosted` (validated at boot). |
| `LIVEKIT_API_SECRET` | ⚠️ | ✅ | — | Required when `SFU_MODE=hosted`. |
| `LIVEKIT_SERVER_URL` | ⚠️ | | — | Required when `SFU_MODE=hosted`. |
| `LIVEKIT_ROOM_PREFIX` | | | `frf/` | Prefix for tenant-namespaced LiveKit rooms. |

⚠️ = required only in the applicable mode; `config.validate()` fails boot if missing.

## CDC (Postgres logical replication) (p16-c020)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `CDC_ENABLED` | | | `false` | Enable Postgres CDC → spine ingestion. |
| `CDC_REPLICATION_URL` | ⚠️ | ✅ | — | Postgres DSN (contains credentials). Required when `CDC_ENABLED=true`. |
| `CDC_SLOT_NAME` | ⚠️ | | — | Logical replication slot. Required when `CDC_ENABLED=true`. |
| `CDC_PUBLICATION_NAME` | ⚠️ | | — | Publication name. Required when `CDC_ENABLED=true`. |
| `CDC_TENANT_ID` | | | — | Tenant UUID that ingested changes are stamped with. |
| `CDC_CHANNEL_PATH` | | | — | Channel path on the spine (e.g. `entities`). |

## Federation (DEFERRED / half-implemented — off by default) (p16-c009)

| Variable | Req | Secret | Default | Purpose |
|----------|:---:|:------:|---------|---------|
| `FEDERATION_ENABLED` | | | `false` | Opt-in for Matrix/ATProto bridges (half-implemented for v1). |
| `FEDERATION_TENANT_ID` | ⚠️ | | — | Tenant for ingested federated events. Required when federation is enabled. |
| `FEDERATION_CHANNEL_ID` | | | — | Channel for ingested federated events. |
| `MATRIX_HOMESERVER_URL` | | | — | Matrix homeserver (outbound send only for v1). |
| `MATRIX_ACCESS_TOKEN` | | ✅ | — | Matrix access token (secret). |
| `MATRIX_ROOM_ID` | | | — | Matrix room to bridge. |
| `ATPROTO_JETSTREAM_URL` | | | — | ATProto Jetstream firehose (inbound only for v1). |
| `ATPROTO_COLLECTIONS` | | | _(empty)_ | Comma-separated ATProto collections to ingest. |

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

## Minimum required to boot (production)

`IGGY_CONNECTION_STRING`, `KETO_BASE_URL`, `GATEWAY_JWKS_URL`, `JWT_AUDIENCE`, and
`JWT_ISSUER` (a production build refuses to boot without it — p17-c002). Also required in
the default topology: `LIVEKIT_*` (since `SFU_MODE` defaults to `hosted`).
`config.validate()` fails fast at boot on all of these mode-specific gaps (p16-c020).
