---
id: dependency-rule
title: The dependency rule
sidebar_label: The dependency rule
---

Everything else in this architecture is a consequence of one constraint:

> **Nothing in `frf-domain` or `frf-app` may import an adapter crate.**

The project's own `CLAUDE.md` states the enforcement mechanism in the same
breath as the rule, and the second sentence is the one that matters:

> The compiler must make violations impossible — keep adapter crates out of
> `frf-domain` and `frf-app` `[dependencies]`.

## Why the compiler, and not a linter

Most codebases express layering as a convention: a diagram in an onboarding doc,
a naming pattern, a lint rule someone can suppress. Conventions decay under
deadline pressure, and they decay silently — the violating import compiles, ships,
and becomes load-bearing before anyone notices.

Cargo makes a stronger enforcement available for free. A crate can only `use`
what its `Cargo.toml` declares. If `frf-domain` does not list
`frf-broker-iggy` as a dependency, then no file in `frf-domain` can import it —
not by accident, not under deadline, not because a reviewer was tired.

This turns an architectural aspiration into a build error. The cost of a
violation drops from "discovered in production six months later" to "does not
compile."

## The layers

```text
Domain (frf-domain)
  ↑ imported by
Application (frf-app, frf-ports)
  ↑ imported by
Infrastructure adapters (frf-broker-*, frf-authz-*, ...)
  ↑ wired by
Interface (frf-gateway)
```

**`frf-domain` — Layer 0.** Pure types with `serde` and nothing else. Entity
identifiers, event envelopes, tenant identifiers. No async runtime, no HTTP, no
database. It should compile for a target that has no operating system.

**`frf-ports` — Layer 1.** Trait definitions and no implementations. Each port
describes a capability the application needs in the application's own
vocabulary: "append to a log", "check whether a subject may view an object".

**`frf-app` — Layer 1.** Use-cases written against ports. This is where the
rules live, and it is the layer that must remain testable without
infrastructure.

**Adapters — Layer 2.** One crate per port implementation. `frf-broker-iggy`
implements `LogBroker` over Apache Iggy. `frf-authz-keto` implements
`AuthzProvider` over Ory Keto. Each depends on domain and ports; none depends on
another adapter.

**`frf-gateway` — Layer 3.** The composition root, and the *only* place that
knows which concrete adapters exist. Deployments select planes through Cargo
features, so a gateway that does not run media never links the media stack.

## The rule that keeps adapters honest

A second constraint prevents adapters from quietly becoming a layer of their
own:

> Each `frf-*` adapter crate implements **exactly one** port trait.

An adapter may not implement two ports, reach into another adapter, or import
application-adjacent crates. Without this, adapters accrete: the broker adapter
grows an authorization check "while it is already there", and the seam that made
the broker swappable is gone.

### A recorded deviation

`CLAUDE.md` documents one place the codebase departs from this, and the way it
is recorded is worth imitating:

> **Known deviation — `frf-media-str0m`.** This crate houses two adapter types:
> `StrOmSignaler` (implements `MediaSignaler`) and `StrOmTransport` (implements
> `MediaTransport`). Neither *type* implements two ports — the two concerns stay
> separate — but housing both in one crate departs from the crate-level rule
> above. This is recorded, not endorsed. Do not cite it as precedent for a new
> adapter; new adapters follow the rule.

"Recorded, not endorsed" is the useful phrase. An architecture document that
describes only the intended state is a work of fiction; one that names its own
exceptions, and refuses them precedent, stays trustworthy.

## What this makes possible

**Substitution without archaeology.** Replacing the event spine means writing
one crate and editing the composition root. No use-case moves, because no
use-case ever named Iggy.

**Tests without machinery.** A use-case depends on traits, so a test provides an
in-memory implementation. Rules are verified in milliseconds without Docker.

**Deployments that link only what they run.** Because composition is centralised
and feature-gated, a deployment running only the entity plane does not compile
the media stack.

## What this costs

Stated plainly, because a rule defended only by its benefits is a rule nobody
can evaluate:

- **A change touching persistence touches several files** — the domain type, the
  port signature, and every adapter implementing it.
- **Following control flow requires knowing the composition.** A reader tracing
  a call arrives at a trait method and must consult `frf-gateway` to learn which
  implementation runs.
- **A badly drawn port is expensive.** Once several adapters implement a trait,
  changing its shape is a coordinated edit. Getting the seam right matters more
  than getting it early.

## See also

- [Ports and adapters](ports.md) — the actual traits and their implementations
- [Writing an adapter](../guides/writing-an-adapter.md) — the practical walkthrough
- [Decision records](../decisions/overview.md) — where seams were drawn, and reconsidered
