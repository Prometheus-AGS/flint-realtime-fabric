---
id: local-first
title: Local-first replicas
sidebar_label: Local-first
---

:::caution This lane is built, disabled, and not certified
The shape facade sits behind an off-by-default `shape-facade` Cargo feature. The
live ElectricSQL exchange has **never been run against a server**. Even when
configured, the gateway logs a warning saying the lane is not certified.

This page documents a design and its reasoning. Do not deploy it expecting a
supported feature.
:::

## The problem

A local-first client keeps a queryable replica on the device, so the UI reads
local state and stays useful offline. ElectricSQL syncs Postgres rows into such
a replica through *shapes* — a table, a column projection, and a `WHERE` clause.

The security question is who decides the shape. If the client sends the
predicate, then the client decides which rows it receives, and the boundary is
enforced by code the user controls. [ADR-009](../decisions/overview.md) states
the consequence directly:

> A client predicate or tenant-scoped adapter is useful validation but cannot
> enforce access against a modified client.

This is not hypothetical. The [prior authorization case
study](../case-studies/prior-auth.md) documents a team building a careful
client-side column boundary — testing it, proving the test could fail — and then
formally downgrading it from "the control" to "useful validation" for exactly
this reason.

## The design

The facade puts the authorization decision server-side:

```text
client                gateway                          Electric        Postgres
  │  GET /v1/shape ──────►│
  │   (no predicate)      │ verify JWT → subject, tenant
  │                       │ look up shape in catalog
  │                       │ Keto: may this subject read it?
  │                       │ compose WHERE from verified claims
  │                       │                    ──────────►│ ─────────►│
  │  ◄──────────────────  │  ◄─────────────────────────────           │
  │   only permitted rows │
```

Four properties matter:

**The client sends no predicate.** It names a shape. The gateway composes the
`WHERE` clause from *verified* claims.

**Columns come from a catalog, not a request.** A column not in the catalog
entry cannot be requested. Following the prior-auth lesson, an unlisted column
should be excluded by decision rather than by omission.

**Every continuation is authorized.** Shape streams resume with a handle and
offset. Authorizing only the first request means a stream outlives the
permission that opened it — the same defect [ADR-002](../decisions/overview.md)
accepted for the agent plane and ADR-009 found insufficient here.

**Revocation is bounded.** Not instantaneous — bounded, and the bound is
documented rather than implied.

## Configuration

Both are required together:

| Variable | Purpose |
|---|---|
| `SHAPE_ELECTRIC_URL` | Upstream Electric endpoint |
| `SHAPE_CATALOG_PATH` | Shape catalog defining tables, columns, and scope |

Plus the feature at build time:

```bash
cargo build -p frf-gateway --features shape-facade
```

A half-configured deployment registers **no route at all** rather than serving
an empty catalog. An endpoint that exists and returns nothing invites a caller
to conclude there is no data, rather than that there is no configuration.

## An honest limitation

One claim in the port documentation deserves qualification. `AuthorizedShapeRequest`
is described as impossible to construct outside the authorization path. In
practice its fields are public and the type is re-exported, so a test constructs
one directly.

The *practical* guarantee holds — the gateway injects the authorization seam,
and `resolve()` is the only production construction site. But it is a
disciplined seam, not a type-system proof, and this documentation would rather
say so than let a reader trust a guarantee the compiler is not making.

## See also

- [Prior authorization](../case-studies/prior-auth.md) — why client-side is not enough
- [Authorization](../theory/authorization.md) — the enforcement model
- [ADR-009](../decisions/overview.md) — the full decision
