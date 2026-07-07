# Security Model — flint-realtime-fabric

How the fabric authenticates callers, isolates tenants, and authorizes actions.
The whole premise is **sovereign auth + tenant isolation**; this document is the
authoritative description of how that is enforced. Companion docs:
[`ENVIRONMENT.md`](ENVIRONMENT.md), [`RUNBOOK.md`](RUNBOOK.md).

> To report a vulnerability, see [`../SECURITY.md`](../SECURITY.md) (disclosure policy).
> This document is the *design*; that file is the *process*.

---

## 1. Authentication — the JWT boundary

**Every request is authenticated at the gateway boundary before any domain logic
runs.** There is no trusted-network assumption.

- Callers present a **JWT bearer token**. HTTP/gRPC calls use
  `Authorization: Bearer <jwt>`; browser WebSocket endpoints (which cannot set
  headers) pass the token as a `?token=` query parameter, verified the same way.
- The gateway verifies the JWT with `OryIdentityVerifier` against the JWKS served
  by **flint-gate** (`GATEWAY_JWKS_URL`). It checks:
  - **Signature** (RS256) against the JWKS (with a one-time JWKS refresh on key
    rotation).
  - **Audience** (`JWT_AUDIENCE`).
  - **Issuer** (`JWT_ISSUER`) — when set, tokens with a missing or mismatched `iss`
    are rejected (p16-c004). When unset, the gateway logs a warning and does NOT
    validate the issuer; **set it in production** so only your IdP's tokens are
    trusted.
- **Claims are never trusted downstream unverified.** The verified `tenant_id`,
  `subject`, and `session_id` come only from the validated token.
- **flint-gate** is the identity edge: it mints and signs outbound JWTs (secret
  `FLINT_GATE_JWT_SECRET`) and publishes the JWKS the gateway verifies against.

### The DEV_NO_AUTH bypass (dev only — cannot exist in production)

A `dev-endpoints` build with `DEV_NO_AUTH=true` skips JWT verification for
publish/subscribe (for local/CI integration without minting real JWTs). This is
**compile-gated**: the production release image is built **without** the
`dev-endpoints` feature, so `dev_no_auth()` and the bypass branches do not exist in
the production binary (p16-c001). The production `compose.yml` sets neither the
feature nor the env var.

---

## 2. Tenant isolation

Tenant isolation is enforced in **two independent layers**, defense-in-depth:

### Layer A — per-event authorization (Keto)

Visibility is governed by **Ory Keto** (a Zanzibar-style relation store), not
application code. Two checks:

1. **Subscribe-time:** before a subscription opens, Keto is asked
   `check(subject, "subscribe", channel)` — a subject who may not subscribe to the
   channel is rejected.
2. **Per delivered event:** every envelope on a subscription's fan-out stream is
   filtered by `check(subject, "view", envelope.id)` scoped to the subscriber's
   `tenant_id` (`frf-app/src/subscribe.rs`). An event the subject/tenant may not
   `view` is silently dropped from that subscriber's stream — it never leaves the
   gateway.

   > Performance note: this is one Keto check per delivered event. Because event
   > ids are globally-unique v4 UUIDs, the decision is object-scoped and cannot
   > leak across tenants. At scale, cache `view` decisions at subscribe time to
   > bound per-event Keto latency (a known optimization, tracked separately).

### Layer B — app-layer tenant-equality guard (publish)

Beneath Keto, the publish use-case rejects a mismatch between the caller's verified
JWT `tenant_id` and the target channel/envelope `tenant_id`
(`frf-app/src/publish.rs`, p16-c002) — **before** the Keto check or the broker
append. So a caller authenticated for tenant A cannot forge a write into tenant B's
channel even if a stray relation tuple would permit it. The verified JWT tenant is
authoritative; the caller-supplied envelope tenant must match it.

> Subscribe has no equivalent channel-tenant equality check because a channel's
> owning tenant is not resolved at the app layer (subscribe carries only a bare
> `ChannelId`); the per-event Keto `view` filter is the read-path tenant boundary.

---

## 3. Keto vs Cedar — separate responsibilities

The system uses **two** policy engines with **distinct, non-overlapping** roles.
Do not conflate them.

| | **Keto** (Zanzibar) | **Cedar** |
|--|--------------------|-----------|
| Governs | **Visibility / relationships** — who may `view`/`subscribe`/`publish` to which object | **Mutation ACTION policy** — whether an action (e.g. `Publish`) is permitted |
| Question | "Is subject S related to object O by relation R?" | "Is this action allowed by policy?" |
| Where | Subscribe-time + per-event `view` fan-out filter | The publish route's action check |
| Model | Relation tuples (fine-grained, per-object) | Policy set (`policy.cedar`), action-level |

**Cedar honesty (p16-c007):** the bundled `policy.cedar` explicitly permits the
mutation actions in use (`Publish`, `Subscribe`) and denies others — it is not a
blanket allow-all, and `PolicyEngineMode::None` (the default) logs "no-op (all
permitted)" explicitly rather than silently. Cedar evaluates against an empty entity
store, so **action-level** policies work; a policy that references principal/resource
**attributes** cannot resolve them and surfaces an **authorization error** (logged
and propagated), never a silent deny. Full attribute ABAC requires an entity store
(out of scope for v1).

---

## 4. Transport & request-level controls

- **Rate limiting** — a global token-bucket cap (`RATE_LIMIT_PER_SEC` /
  `RATE_LIMIT_BURST`) bounds request volume (p16-c005).
- **Body-size limit** — `MAX_BODY_BYTES` (default 1 MiB) caps per-request memory.
- **CORS** — an explicit exact-match allowlist (`CORS_ALLOWED_ORIGINS`); empty means
  no cross-origin browser access.
- **Secrets** — never committed; sourced from env / a secret manager
  (`FLINT_GATE_JWT_SECRET`, broker/DB/LiveKit creds). See `RUNBOOK.md` §2.
- **Logging** — JWT payloads, relation tuples, and tenant ids are not logged in
  debug output.

---

## 5. Trust boundaries — summary

| Boundary | Enforced by |
|----------|-------------|
| Unauthenticated → authenticated | JWT verify at the gateway (flint-gate JWKS) |
| Cross-tenant write | JWT-tenant == channel-tenant guard (publish) + Keto |
| Cross-tenant read | Per-event Keto `view` check on fan-out |
| Unauthorized subscribe | Subscribe-time Keto `subscribe` check |
| Disallowed mutation action | Cedar action policy |
| Request-volume / body abuse | rate-limit + body-size + CORS layers |

Every downstream component assumes the JWT boundary holds — which is why the
production image cannot contain the auth bypass (§1).

## 6. Deferred planes — not yet security-relevant (because not implemented)

These advertised planes are **not fully implemented** and are gated off or labeled
unimplemented rather than shipped in a half-secured state. They carry no production
security posture yet because there is no live data path to secure:

- **str0m sovereign SFU** — signaling-only; no real WebRTC media plane (no `Rtc`/SDP/ICE/
  RTP). Gated off by default (`SFU_MODE=hosted`), boots with a warning. Do not enable
  `SFU_MODE=sovereign` in production expecting media to flow or to be secured.
- **Federation** — Matrix inbound and ATProto outbound are stubs; LiveKit inbound is
  single-process only. Gated behind `FEDERATION_ENABLED`. Cross-instance event flow is
  not authenticated end-to-end because it does not yet function.
- **admin-ui login** — no interactive OIDC/flint-gate redirect flow; the operator pastes a
  JWT. The gateway still verifies that token normally, but the UI does not mint or refresh
  it. Treat the admin UI as an authenticated-operator tool behind your own access control.

When any of these is implemented, extend §1–§5 to cover its boundary before enabling it.
