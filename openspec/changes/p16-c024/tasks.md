# Tasks — p16-c024

- [x] Document the JWT auth boundary and verification
- [x] Document tenant isolation model (Keto per-event view)
- [x] Document Keto vs Cedar responsibilities

## Summary (#41 — security-model document)

Wrote `docs/SECURITY.md` — the authoritative description of how the fabric
authenticates, isolates tenants, and authorizes. Every mechanism was verified
against the actual code (not aspirational).

### §1 JWT auth boundary (task 1)

Every request authenticated at the gateway before domain logic: Bearer header (HTTP/gRPC)
or `?token=` query (browser WS). `OryIdentityVerifier` checks signature (RS256 vs
flint-gate JWKS), audience, and — when `JWT_ISSUER` set — issuer (p16-c004). Claims never
trusted unverified. Documents the DEV_NO_AUTH bypass as **compile-gated out of the
production image** (p16-c001).

### §2 Tenant isolation (task 2)

Two independent layers:
- **Keto** — subscribe-time `subscribe` check + **per delivered event** `view` check on
  fan-out (`frf-app/src/subscribe.rs`), tenant-scoped; unauthorized events never leave the
  gateway. Notes the object-UUID-uniqueness property and the subscribe-time caching
  optimization.
- **App-layer tenant-equality guard** on publish (p16-c002) — JWT tenant must equal the
  envelope's channel tenant, checked before Keto/broker. Documents why subscribe has no
  equivalent (bare ChannelId; the `view` filter is the read boundary).

### §3 Keto vs Cedar (task 3)

Side-by-side table of the distinct, non-overlapping roles: Keto = visibility/relationships
(per-object), Cedar = mutation ACTION policy (action-level). Documents Cedar honesty
(p16-c007): explicit matchable policy, `None` mode logs allow-all, attribute policies
surface an error rather than silently denying.

Plus §4 transport controls (rate-limit/body/CORS/secrets/logging) and §5 a trust-boundary
summary table.

## Verification

Cross-checked 8 claims against code: subscribe-time `subscribe` check, per-event `view`
check, publish tenant-equality guard, `set_issuer`, Cedar `diagnostics().errors()`
surfacing, NoOp "all permitted" log, `dev_no_auth` cfg-gating, `?token=` WS auth — all
present. `docs/RUNBOOK.md` cross-ref exists; root `SECURITY.md` (disclosure policy) is a
forward reference to c026.
