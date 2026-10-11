# Goals — production-readiness-integration

Created 2026-09-15 at the operator's request using `kbd-new-phase`.
Revised 2026-09-16 to prioritize continuously usable prototype applications.

## Outcome

Deliver the fastest continuously usable prototype path across Fabric, Forge,
PGlite, ElectricSQL and Prometheus Entity Management (PEM), while retaining a
working rollback configuration during development. Complete the user-facing
data loop before broad packaging, hardening or isolated tests. Later resume the
separate ASO, hosted, sovereign and federated production-readiness gates.

## Goals

- G1: Establish a reproducible local build and integration baseline for identified
  revisions of Fabric, Gate, Forge and PEM; remove CI test execution paths.
- G2: Make production manifests portable and bootable with explicit identity,
  media, secrets, dependency health, TLS and network-boundary configuration.
- G3: Prove real Gate-issued identity and current authorization at every exposed
  data lane, including bounded revocation for protected ASO workloads.
- G4: Close OQ-FRF-1 through a frozen, versioned table-watch contract, durable
  Fabric implementation and a working Forge adapter with per-event RLS checks.
- G5: Deliver a runnable online loop where PGlite optimistic mutations travel
  through authenticated Forge writes to PostgreSQL and canonical rows return
  through ElectricSQL plus Fabric into PGlite and PEM.
- G6: Make that loop durable with a PGlite outbox, stable mutation identity,
  restart/reconnect replay, conflict visibility and two-client convergence.
- G7: Replace vacuous integration guards with observed pass/fail evidence and
  establish restart, load, backup/restore and operational recovery criteria.
- G8: Prove hosted media and separately close sovereign decode, authorization
  lifetime and federation recovery gaps before enabling their production profiles.
- G9: Produce current security, capability and release records bound to exact
  source and package/image digests; keep implementation, evidence, certification
  and publication status distinct.

## Deliverables and lifecycle

- `plan.md`: ordered candidate work packages, dependencies, owners and exit gates.
- `feedback-review.md`: corrections to the supplied assessment and source evidence.
- `entry-state.json`: prior orchestration state and source snapshot provenance.
- c011 is the active source change. c012 and c013 are the next prototype slices.
- The first behavioral proof is one complete local integration campaign after
  the whole user flow exists; unit/component tests cannot substitute for it.
