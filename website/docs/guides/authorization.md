---
id: authorization
title: Configuring authorization
sidebar_label: Authorization
---

The [theory page](../theory/authorization.md) explains how the three mechanisms
differ. This one covers configuring them.

## Identity

| Variable | Required | Purpose |
|---|---|---|
| `GATEWAY_JWKS_URL` | yes | Public keys for verifying inbound JWTs |
| `JWT_ISSUER` | **yes in production** | Expected `iss`; the gateway refuses to boot without it |
| `JWT_AUDIENCE` | yes | Expected `aud` |

A production binary — one built without the `dev-endpoints` feature — will not
start without `JWT_ISSUER`. That is deliberate: accepting any JWKS-valid token
regardless of issuer is a silent authentication bypass, and refusing to start is
the correct response.

Browser WebSockets cannot set headers, so they pass `?token=`. Verification is
identical.

## Visibility

| Variable | Purpose |
|---|---|
| `KETO_BASE_URL` | Ory Keto read/write API |
| `KETO_NAMESPACE` | Relation-tuple namespace |

Two relations carry the model:

- `subscribe` on a channel — checked once when a subscription opens
- `view` on an object — checked per delivered event, and at media room-join

Write tuples through the `AuthzProvider` port rather than Keto's API directly,
so the cache invalidates on delete.

### Cache behaviour

Checks are cached for 60 seconds by default, keyed on
`(subject, relation, object)`, and invalidated when a tuple is deleted.

Understand what this does and does not buy you. Because the per-event `view`
object is the event id, **each event is a distinct cache key** — the cache
collapses repeated checks on the same object, not per-event checks in general.
If you are designing a high-fan-out channel, authorize at subscribe time.

The 60-second TTL is also the window in which a revoked permission may still be
honoured. If that is unacceptable for your data, the per-object check is not the
mechanism you want — see [ADR-009](../decisions/overview.md).

## Action policy

| Variable | Values | Default |
|---|---|---|
| `POLICY_ENGINE` | `cedar`, `none` | `none` |

`none` logs `action policy engine: no-op (all permitted)` at startup. Check for
that line before assuming a policy is in force.

Cedar governs mutations only. Do not attempt to express tenant visibility in
Cedar — that belongs in Keto, and splitting visibility across two systems means
neither is authoritative.

Cedar currently evaluates against an empty entity store, so attribute-based
policies produce an authorization *error* rather than a silent deny. An error is
the right failure mode for a policy that cannot be evaluated.

## Verifying it works

The checks worth performing locally, in order of how badly they fail if broken:

1. **Cross-tenant publish.** A token for tenant A publishing into a tenant-B
   channel must be refused — by the tenant-equality guard, even with a
   permissive relation tuple present.
2. **Per-event filtering.** A subscriber without `view` on an object must not
   receive it. It should be dropped silently, not error.
3. **Media room-join.** A subject without `view` on the room must not join. Then
   stop Keto and retry: the join must still be **refused**, because the check is
   fail-closed.
4. **Boot refusal.** Unset `JWT_ISSUER` in a production build and confirm the
   gateway refuses to start.

Run these against a local stack. Never in CI.
