---
id: why
title: Why this exists
sidebar_label: Why this exists
---

Most realtime backends begin the same way. A WebSocket handler, a broadcast
channel, a Postgres table. It works. Then the requirements arrive:

- The same events must reach a browser, a phone, and a server-to-server peer.
- Two tenants must never see each other's rows — and "must never" has to survive
  a junior developer's first pull request.
- An AI agent needs to participate in the same conversation as a human.
- A call needs to run over the same identity and permission model as the chat.
- The whole thing needs to work on a plane, offline, and reconcile on landing.

Each requirement is tractable alone. Together they produce a specific failure
mode, and it is worth naming precisely, because the architecture is a direct
response to it.

## The failure mode: infrastructure leaks into meaning

The natural way to add each capability is to reach for the tool that provides it
from wherever you happen to be standing. The WebSocket handler calls Redis
directly. The permission check inlines a SQL query. The agent code imports the
message broker's client. Each is locally reasonable.

The result is that **business rules and infrastructure choices become the same
code**. Three things follow, and they compound:

**You cannot change infrastructure.** Swapping the broker means touching every
file that publishes an event. The migration is not a refactor, it is an
archaeology project, so it does not happen and the wrong tool stays.

**You cannot test meaning without machinery.** Verifying "a member of tenant A
cannot read tenant B's entity" requires standing up the broker, the database,
and the authorization service, because the rule is expressed in terms of them.
Slow tests get skipped. Skipped tests stop constraining anything.

**Security becomes a code-review problem.** If the permission check is a
function anyone can forget to call, then tenant isolation is enforced by
whoever reviews the pull request. That is not a security boundary. It is a
hope, and it fails silently — the bug is a query returning *more* rows than it
should, which no test asserting the happy path will ever catch.

The third one is why this matters more here than in a typical service. FRF is
built for multi-tenant systems handling regulated data. A cross-tenant read is
not a bug report; it is a disclosure.

## The response

FRF applies hexagonal architecture — ports and adapters — with one rule taken
more seriously than usual: **the dependency direction is enforced by the
compiler, not by convention.**

```text
        Domain  (frf-domain)          pure types, serde only, zero infra deps
           ▲ imported by
     Application  (frf-app, frf-ports)  use-cases + trait seams
           ▲ imported by
   Infrastructure  (frf-broker-iggy, frf-authz-keto, …)  one adapter per port
           ▲ wired by
      Interface  (frf-gateway)          composition happens here, and only here
```

`frf-domain` and `frf-app` do not list adapter crates in their `[dependencies]`.
An import that violates the layering is not a review comment — it does not
compile. See [the dependency rule](dependency-rule.md) for what this buys and
what it costs.

Three consequences follow directly:

**Infrastructure becomes swappable.** The event spine sits behind a `LogBroker`
port. Replacing Apache Iggy means writing one crate that implements one trait
and changing one line in the composition root. No use-case code moves.

**Meaning becomes testable in isolation.** A use-case depends on traits, so a
test substitutes an in-memory fake. The rule "a subscriber only receives events
it may view" is verified without a broker, a database, or a network.

**Authorization stops being application code.** Tenant isolation is expressed as
Zanzibar relation tuples in [Ory Keto](authorization.md), not as `WHERE
tenant_id = ?` scattered across handlers. Application code cannot forget a check
it never performs.

## What this costs

An architecture page that lists only benefits is marketing. The costs here are
real and worth stating.

**Indirection.** Following a request from HTTP handler to storage crosses a
trait boundary where a direct call would do. Reading the code requires knowing
which adapter is composed in.

**Ceremony for small changes.** Adding a field that touches persistence means
editing a domain type, a port signature, and every adapter implementing it. In a
layered monolith it is one file.

**The seams must be right.** A port drawn at the wrong boundary is worse than no
port — it fossilises a bad abstraction behind a trait that is now hard to
change. Several of the [decision records](../decisions/overview.md) exist
because a seam was drawn, used, and reconsidered.

The trade is deliberate: more friction per change, in exchange for changes that
stay local and security properties that hold structurally. For a single-tenant
CRUD service that trade is bad. For multi-tenant regulated data with four
protocol planes, it is the reason the system is still modifiable.

## Where to go next

- [The dependency rule](dependency-rule.md) — the single constraint the rest depends on
- [Ports and adapters](ports.md) — the actual seams and what implements them
- [The planes](planes.md) — entity, agent, media, federation
- [Prior authorization](../case-studies/prior-auth.md) — the architecture under real clinical constraints
