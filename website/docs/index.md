---
id: index
title: Flint Realtime Fabric
sidebar_label: Overview
slug: /
---

Flint Realtime Fabric (FRF) is a Rust realtime backend for multi-tenant,
multi-plane applications: entities that sync, agents that converse, media that
flows, and federation that projects outward — over one event spine, with
authorization enforced outside application code.

This site is written to explain **why** the system is shaped the way it is. The
API reference tells you what a function takes; these pages tell you which
problem the shape solves and what breaks without it.

## Start here

| If you want to… | Read |
|---|---|
| Understand the problem being solved | [Why this exists](theory/why.md) |
| Understand the one rule everything rests on | [The dependency rule](theory/dependency-rule.md) |
| See the seams the system is cut along | [Ports and adapters](theory/ports.md) |
| Understand tenant isolation | [Authorization](theory/authorization.md) |
| Build something | [Quickstart](guides/quickstart.md) |
| See it in a shipped product | [Prior authorization](case-studies/prior-auth.md) |

## A note on how this documentation talks about status

Software documentation routinely blurs three different claims: code exists, code
compiles, and behaviour has been demonstrated. This project keeps them apart,
because conflating them is how a team convinces itself something works.

Throughout this site:

- **Built** — the code exists and compiles under the project's lint gates.
- **Proven** — a test was *executed* and observed to pass. Not "a test was
  written."
- **Gated off** — the code exists and is deliberately not enabled.

Where a feature is built but unproven, the page says so. The
[project status](status.md) page collects these in one place, including the parts
that are not finished.

:::note What this is not
FRF is not a framework you install and configure. It is a workspace of Rust
crates with explicit seams, intended to be composed into a deployment that runs
only the planes it needs. If you want a batteries-included realtime service,
this is the wrong shape.
:::
