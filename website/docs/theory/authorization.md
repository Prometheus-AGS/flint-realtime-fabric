---
id: authorization
title: Authorization
sidebar_label: Authorization
---

FRF uses three security mechanisms that are routinely confused with one another.
The project's own `SECURITY.md` puts it bluntly: **do not conflate them.**

| Mechanism | Question it answers | Scope |
|---|---|---|
| **flint-gate + JWT** | Who are you? | Authentication, at the boundary |
| **Ory Keto (Zanzibar)** | May this subject *see* this object? | Visibility, per object |
| **Cedar** | Is this *mutation* permitted? | Action policy, never visibility |

Getting these backwards produces systems that look secure and are not. A Cedar
policy saying "clinicians may read cases" does not stop a clinician from reading
*another practice's* case — that is a visibility question, and it belongs in
Keto.

## Authentication: flint-gate and JWT

Every request is authenticated at the gateway boundary before any domain logic
runs. There is **no trusted-network assumption**: an internal caller is
authenticated exactly like an external one.

flint-gate is the identity edge. It mints and signs outbound JWTs and publishes
a JWKS; the gateway verifies RS256 signatures against `GATEWAY_JWKS_URL`, plus
`JWT_AUDIENCE` and `JWT_ISSUER`.

:::danger Never use Ory Oathkeeper
flint-gate replaces Oathkeeper in this architecture. Do not introduce
Oathkeeper, and do not name environment variables after it.
:::

Two details carry real weight:

**`JWT_ISSUER` is mandatory in production.** A binary built without the
`dev-endpoints` feature *refuses to boot* without it. The reasoning is worth
internalising: without an issuer check, any token that validates against the
JWKS is accepted — including one minted by a different issuer that happens to
share a key source. Failing to start is the correct response to a configuration
that would silently accept forged identity.

**The auth bypass does not exist in production builds.** `DEV_NO_AUTH` is
compile-gated behind the `dev-endpoints` Cargo feature. The production image is
built without it, so the bypass branches are not merely disabled — they are not
in the binary.

Browser WebSockets cannot set headers, so they pass `?token=`. The token is
verified identically. **Claims are never trusted downstream unverified**:
`tenant_id`, `subject`, and `session_id` come only from the validated token,
never from the request body.

## Visibility: Keto relation tuples

Tenant isolation is enforced at the Keto layer, **not in application code**.
This is the single most important structural decision in the security model,
because it means a use-case cannot forget a check it never performs.

There are three enforcement points:

**1 — Subscribe time.** `check(subject, "subscribe", channel)` before the
subscription opens.

**2 — Per delivered event.** Every envelope is filtered by
`check(subject, "view", envelope.id)`. A denied event is dropped silently and
never leaves the gateway:

```rust
let filtered = raw_stream.filter_map(move |item| {
    let authz = Arc::clone(&authz);
    let subject = subject.clone();
    async move {
        match item {
            Err(e) => Some(Err(e)),
            Ok(envelope) => {
                if envelope.channel.tenant_id != tenant_id {
                    return None;
                }
                let view_tuple = RelationTuple {
                    tenant_id,
                    subject,
                    relation: "view".to_owned(),
                    object: envelope.id.to_string(),
                };
                match authz.check(&view_tuple).await {
                    Ok(true) => Some(Ok(envelope)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e)),
                }
            }
        }
    }
});
```

Note that this lives in `frf-app` and is written against the `AuthzProvider`
port — the rule is expressed in application terms and tested without Keto.

**3 — Media room join.** `check(subject, "view", room)` before a session enters
fan-out, and it is **fail-closed**: a check *error*, not merely a denial, denies
the join. A permission service that is down must not become a permission service
that permits.

Cross-tenant media fan-out is additionally impossible by construction, because
the room router keys rooms by `(TenantId, room)` rather than by room name.

### Does a per-event check scale?

This is the obvious objection, and the answer is a real mechanism rather than a
hope. `KetoAuthzProvider` holds a TTL cache keyed on
`(subject, relation, object)`, with a 60-second default, invalidated when a
tuple is deleted.

One honest caveat: because the per-event `view` object is `envelope.id`, each
event is a *distinct* cache key. The cache collapses repeated checks on the same
object — it does not make per-event checks free in general. High-fan-out designs
should cache authorization at subscribe time, which is exactly what
[ADR-002](../decisions/overview.md) does for the agent plane.

### Defense in depth beneath Keto

The publish path carries a tenant-equality guard *underneath* the authorization
provider:

```rust
// Tenant-equality assertion (defense in depth beneath the configured
// authorization adapter): a caller
// authenticated for tenant A must not publish into a channel owned by
// tenant B, even if a stray relation tuple would allow it. The verified
// JWT tenant is authoritative; the envelope's channel tenant is
// caller-supplied and must match it.
if claims.tenant_id != req.envelope.channel.tenant_id {
    return Err(AppError::Forbidden(format!(
        "subject {} (tenant {}) may not publish into a channel owned by tenant {}",
        claims.subject, claims.tenant_id, req.envelope.channel.tenant_id
    )));
}
```

"Even if a stray relation tuple would allow it" is the point. Relation tuples
are data, and data can be wrong. A cross-tenant publish is severe enough to
warrant a second, independent check that does not depend on that data being
correct.

## Action policy: Cedar

Cedar governs **mutations** — `Publish`, `Subscribe`, `Delete` — and never
visibility. It is applied at exactly one place, the publish route, and
deliberately **not** in `frf-app`.

The default is `POLICY_ENGINE=none`, which logs `action policy engine: no-op
(all permitted)` rather than staying silent. A permissive default that announces
itself can be caught in a log review; one that says nothing cannot.

Current limitation, stated plainly: Cedar evaluates against an empty entity
store, so action-level policies work while attribute-based policies surface an
authorization *error* rather than a silent deny. Full ABAC is out of scope for
v1. An error is the right failure mode — a policy that cannot be evaluated must
not quietly become "allow".

## How they compose

```text
authN  — at connect     client JWT → OryIdentityVerifier (GATEWAY_JWKS_URL)
                                   → Subject + verified claims
authZ  — at subscribe   Keto.check(subject, "subscribe", channel)   [coarse, cached]
RLS    — at fan-out     Keto.check(subject, "view", envelope.id)    [per object]
action — orthogonal     Cedar.is_permitted(principal, action, res)  [mutations only]
```

## Operational rule

**Never log** JWT payloads, relation tuples, or tenant identifiers. A debug log
that dumps a token is a credential disclosure with a long tail, and relation
tuples describe who may see what — which is itself sensitive.

## Known limits

Documented here rather than discovered later:

- **Agent-plane revocation is not immediate.** [ADR-002](../decisions/overview.md)
  authorizes agent streams at subscribe time, not per event, so a revocation
  mid-stream does not take effect until the client disconnects. ADR-009 records
  that this is **insufficient** for protected clinical output.
- **The media check is per room-join, not per packet.** Per-RTP-packet
  authorization is an explicit non-goal.
- **Cedar ABAC is unavailable**, as described above.

## See also

- [The dependency rule](dependency-rule.md) — why authorization lives outside application code
- [Authorization guide](../guides/authorization.md) — configuring it
- [Prior authorization case study](../case-studies/prior-auth.md) — these limits under clinical constraints
