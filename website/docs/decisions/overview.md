---
id: overview
title: Decision records
sidebar_label: Decision records
---

Nine architectural decision records live in `docs/decisions/`. Each states what
was decided, what was rejected, and — most usefully — what the decision costs.

A note on status: several ADRs are marked **Proposed** even though the code they
describe is implemented. That is deliberate. As the records put it, *"existing
implementation does not retroactively record acceptance."* Shipping something is
not the same as having agreed to it.

## ADR-001 — CRDT engine · Accepted

**Loro 1.13.1**, over automerge.

Measured: 2–9× faster encode/decode, 3.7× smaller documents (68 kB versus 250
kB), 2.7× less memory. Loro's Fugue algorithm prevents interleaving anomalies
that RGA produces. `loro-swift` is production-grade where `automerge-uniffi` was
experimental — decisive for a project binding the same core to Swift, Kotlin and
Dart.

**Cost:** a younger ecosystem, and migrating later would mean rebuilding every
FFI binding. Kotlin support was symmetric (neither is first-party), so other
axes decided it.

## ADR-002 — Agent bus tenant isolation · Accepted, partially superseded by 009

Agent streams are authorized **once at subscribe time**, not per event.

Rejected: per-event Keto checks, measured at roughly 2ms × 100 frames/second =
200ms per second of streaming — a 20% slowdown for a permission that has not
changed. Agent frames are ephemeral and have no persistent object to check
against.

**Cost, stated plainly:** revocation mid-stream does not take effect until the
client disconnects. ADR-009 later found this **insufficient** for protected
clinical output.

## ADR-003 — FFI and codegen versions · Accepted

tonic 0.14, Connect ^1.7, and **UniFFI 0.31.2 for Swift, Kotlin and Dart**.

Rejected: flutter_rust_bridge — for a concrete and disqualifying reason. FRB
cannot bridge a UniFFI crate: its parser panics on `#[uniffi::export]`, emits
non-functional stubs, and injects a conflicting module. One FFI framework
everywhere beats two that cannot coexist.

**Cost:** Dart bindings are not yet generated. `pub get` succeeds but exposes no
API.

## ADR-004 — Admin UI OIDC provider · Proposed

Recommends Kratos plus Hydra.

Rejected: using flint-gate as an authorization server — it deliberately has no
`/authorize` hook, because it is an identity edge, not an IdP. Keto is
authorization, not authentication, and cannot fill the gap either.

**Consequence:** no IdP is deployed. The interim is a hardened paste-a-JWT token
gate that is expiry-aware and logs out automatically.

## ADR-005 — MediaTransport port · Proposed

A `MediaTransport` port separate from `MediaSignaler`, with str0m implementing
both as distinct types.

Rejected: extending `MediaSignaler` (would force LiveKit to stub media methods
it does not have) and a gateway-driven engine with no port at all (breaks the
[dependency rule](../theory/dependency-rule.md)).

This is the ADR that records the [one-crate-two-ports
deviation](../theory/dependency-rule.md#a-recorded-deviation).

## ADR-006 — RTP fan-out · Proposed, socket assumptions superseded by 008

A central room registry with per-session forwarding channels and `Arc<[u8]>`
payloads.

Rejected: peer-to-peer driver channels (O(n²) rewiring) and a shared-`Rtc` room
actor (serializes an entire room).

Bounded channels **drop on overflow**, which is correct for RTP — it is
loss-tolerant, and a backed-up queue delivering stale video is worse than a gap.

## ADR-007 — Media path authorization · Accepted

A per-participant Keto `check(subject, "view", room)` at room-join, enforced in
the gateway's bridge rather than in the media adapter.

This is [ports composing without coupling](../theory/ports.md#composing-two-ports-without-coupling-their-adapters)
in practice: the media engine gains no authz dependency, and the authz provider
stays swappable.

**Non-goal:** per-RTP-packet authorization. ADR-009 later found the
room-lifetime cache insufficient for protected clinical output.

## ADR-008 — Shared media socket · Accepted

**One** shared UDP socket owned by the transport, demultiplexed to per-session
`Rtc` instances by `Rtc::accepts()` — str0m's own model.

This fixed a hard blocker rather than optimising anything: a fixed media port
with per-session binding is *by construction* single-session, and the second
session failed with `EADDRINUSE`. Because str0m is single-threaded per `Rtc`,
one owning task needs no locks.

## ADR-009 — Authorized replicas · Accepted as target, implementation not certified

An authorized HTTP shape facade between ElectricSQL and a local SQL replica.
Rows, columns and tenant scope are derived **server-side** from verified
identity, and **every continuation** is authorized — not just the first request.

This is the record that qualifies its predecessors: it partially supersedes
ADR-002's subscribe-time authorization and ADR-007's room-lifetime cache,
finding both insufficient for protected clinical output.

Its central claim generalises well beyond healthcare:

> A client predicate or tenant-scoped adapter is useful validation but cannot
> enforce access against a modified client.

**Status:** every lane it governs stays disabled. See
[local-first](../guides/local-first.md).

## Reading these well

Three patterns are worth imitating:

**Rejected options are recorded with their reasons.** ADR-003 does not say
"chose UniFFI" — it says FRB's parser panics on UniFFI exports. A future reader
asking "why not FRB?" gets an answer instead of re-running the experiment.

**Costs appear beside benefits.** ADR-002's latency argument is compelling *and*
the record states the revocation gap it creates. That is what made ADR-009 able
to find it later.

**Later records supersede earlier ones explicitly.** ADR-009 does not quietly
contradict ADR-002 and ADR-007; it names them and says which parts no longer
hold.

Full text is in
[`docs/decisions/`](https://github.com/Prometheus-AGS/flint-realtime-fabric/tree/main/docs/decisions).
