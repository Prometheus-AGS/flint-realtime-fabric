---
id: knowme
title: KnowMe — a hybrid-mobile local-first workspace
sidebar_label: KnowMe
---

:::caution Read this framing first
KnowMe is **not an FRF deployment**, and this page does not present it as one.
A search of the KnowMe Builder repository for `frf-`, `frf_`, and
`flint-realtime-fabric` returns **zero matches** in shipping code. The
connection is a *ratified intent* with no implementation behind it.

It is included here because the design problems it solves — a shared core with
thin platform bindings, a disciplined choice about when to use CRDTs, and a
structural privacy boundary — are the same problems FRF's ports answer, reached
independently. Where the two projects agree, the agreement is evidence. Where
they differ, the difference is instructive.
:::

## What the product is

KnowMe is a private personal-intelligence workspace. Its product promise, from
the project's UI/UX standard, is "AI that understands you."

The problem: consumer AI assistants forget you between sessions, run everything
in someone else's cloud, and stop working without a network. KnowMe combines
private memory, on-device inference, and cross-device continuity so a user's
accumulated context stays theirs.

The demo narrative in the project's own plan is the clearest statement of
intent:

> Record a voice note → on-device transcription → auto-ingest into the memory
> graph → ask chat about it → streamed, cited answer → airplane mode still works
> → open the desktop app, it synced.

:::note What the repository actually contains
`hybrid-mobile-architecture-src` is **KnowMe Builder** — a code generator that
scaffolds KnowMe-shaped applications — not the KnowMe app. The project's own
master plan says so: *"The repo is a skill package, not an application."* The
app that implements this design lives elsewhere. Everything below is either
generator output or ratified doctrine, and each item is labelled.
:::

## The invariant: one core, thin bindings

KnowMe's governing rule is a close cousin of [FRF's dependency
rule](../theory/dependency-rule.md):

> All networking, LLM interaction, inference, MCP, agent logic, and persistence
> live in the shared Rust core. Never re-implemented in Dart or TypeScript.

The core is a layered Rust workspace of roughly thirteen crates, with a frozen
types crate at Layer 0 and FFI leaves at the top — the same shape as FRF's
domain-at-the-bottom, interface-at-the-top layering. Platform code (Flutter and
Riverpod on mobile; Tauri 2, React 19 and Zustand on desktop) is presentation
only.

The load-bearing detail is that **the FFI surface is intent-level, never
data-level**:

> Dart/TS never see raw SurrealQL or SQL. The bridge exposes intent functions
> only (`chat_send`, `memory_search`, `graph_expand`, `entity_list/get/...`).

This is the same reasoning as [one port per adapter](../theory/ports.md). If a
binding could issue queries, every binding would grow its own dialect of the
data model and they would drift. Exposing intents means the merge algorithm,
the reconnection policy, and the protocol exist exactly once.

**Status: built.** `flutter_rust_bridge` is pinned at 2.12.0, with a rule that
the Rust crate and the Dart package must move together.

The project also documents its one exception — WebLLM is a TypeScript library on
the web surface, because Rust inference compiled to WASM is not viable there.
The seam holds because the intent API is unchanged; only the fulfilment differs.
That is the same discipline as FRF's [recorded
deviation](../theory/dependency-rule.md#a-recorded-deviation): name the
exception, keep the boundary, deny it precedent.

## CRDTs are a considered choice, not a default

The most transferable idea in KnowMe is its refusal to treat CRDTs as the answer
to synchronisation. Its sync doctrine splits data into three lanes:

| Lane | Authority | Merge model |
|---|---|---|
| Relational app data | Server (Postgres) | Server-authoritative last-writer-wins, plus a client upload queue |
| Collaborative or user-owned documents | CRDT (Loro) | CRDT merge |
| Append-only histories | Local, then batched | Append/event-log — no merge |

Stated directly in the doctrine:

> CRDT is NOT a default. It is reserved for data that is intrinsically
> collaborative or intrinsically user-owned.

This is worth dwelling on. CRDTs make concurrent edits converge without a
coordinator, which is genuinely hard to achieve otherwise — and they cost
metadata growth, opaque state, and debugging difficulty. Applying them to
server-authoritative rows pays that cost for a guarantee the server already
provides.

FRF reached the same conclusion from the other direction: its CRDT engine sits
behind a port and is a Phase-3 decision recorded in
[ADR-001](../decisions/overview.md), not a foundation the entity plane rests on.

**Status: doctrine is ratified; the sync engine is not built.** The transport
(PSyncV2 — WebSocket plus MessagePack) is tagged as proposed in the project's
own plan, and slices currently run on a development loopback transport. The
distinction matters: the design is settled, the implementation is pending.

## Structural privacy, not filtered privacy

KnowMe's profile vault holds preferences and agent-learned facts, and syncs
peer-to-peer only — never server-side. Data carries a privacy class of `public`,
`trusted`, or `local`, and enforcement is deliberately structural:

> Enforcement is structural and fail-closed: the write queue's enqueue function
> takes the entity's declared class and **refuses** `local` rows; vault tables
> are never registered in any SyncScope; unknown class ⇒ `local`. Server-side
> filtering is defense-in-depth, never the primary gate.

Three properties make this robust, and all three appear in the [prior
authorization case study](prior-auth.md) as well:

1. **Fail-closed default.** An unclassified entity is `local` — the safe
   outcome. A new table is private until someone deliberately publishes it.
2. **Refusal, not filtering.** The queue rejects the row rather than dropping it
   quietly. A mistake is loud.
3. **Server filtering is secondary.** The primary gate is nearest the data.

The same reasoning drives FRF's [authorized shape
facade](../guides/local-first.md): a client should not be trusted to filter what
it was sent, so the boundary is enforced where the rows are produced.

There is a sharp corollary in the retrieval design, and it generalises well
beyond this project:

> Vault content is embedded into a **separate local-only index** — an embedding
> of a secret is still a secret.

Any system deriving vectors from protected data should have an explicit answer
to that sentence.

## On-device inference: the engine is a device choice

The largest body of real code in the repository is local inference, and its
central rule reads as an architectural principle rather than a configuration
note:

> The engine is a per-DEVICE choice. The lane is a per-TURN choice. They are
> different things, and neither is "mobile vs desktop".

The engine matrix is per-target: `llama-cpp-2` on desktop, LiteRT-LM on Android,
MLX on iOS, MLX-C on macOS, WebLLM on web. The justification for Android is the
most useful part, because it records a failure rather than a preference:

> llama.cpp **builds** for Android, so it looks like the obvious single mobile
> engine. In practice its GPU path proved **device-specific**: it works on the
> phone you tested and fails on the next one, which is the worst possible
> failure shape for a correctness baseline.

And the rule that follows: *"There is no 'mobile engine.' A `[inference] mobile
= ...` key, or a `cfg(mobile)` branch selecting an engine, is the bug this table
exists to prevent."*

Every engine sits behind one trait, so the choice never leaks upward:

```rust
//! InferenceProvider — the local-inference engine seam. One trait, per-lane
//! implementations selected at build time in gen_ui_inference. UI layers and
//! gen_ui_agent depend on this trait only — never on an engine crate — so
//! swapping or adding an engine never ripples past gen_ui_inference.
```

That comment would sit unchanged in `frf-ports`. Two projects, no shared code,
the same seam.

Supply-chain handling is equally disciplined: the model catalog pins a revision
and a SHA-256 digest per artifact, so a mutable upstream branch cannot silently
swap a model.

**Status: built.** This is the most complete part of the repository.

## Governance in the type, not the policy document

The generated agent runtime carries its budgets in the request type:

```rust
pub trait UarRuntimeFacade {
    fn run(&mut self, request: RunRequest) -> Result<Vec<UarEvent>, RuntimeError>;
    fn cancel(&mut self, run_id: &str) -> Result<(), RuntimeError>;
    fn recover(&self, run_id: &str) -> Result<Option<PersistedProjection>, RuntimeError>;
}
```

`RunRequest` carries `max_turns`, `max_retries`, `max_duration_ms`, and
`max_output_bytes`; a zero on any of them yields `RuntimeError::BudgetExceeded`.
An agent cannot run unbounded because there is no way to express an unbounded
run — the same move as putting tenant isolation in relation tuples instead of
trusting every query to carry a `WHERE` clause.

## What FRF should take from this

**Name the lane before choosing the merge model.** KnowMe's three-lane split is
a better default than "CRDT everywhere," and it is a useful lens on
[ADR-001](../decisions/overview.md).

**Make the privacy default the safe one.** Unknown class ⇒ `local` is a pattern
FRF's shape catalog should match: an unlisted column should be excluded, not
included.

**Record the failure, not just the decision.** The Android/llama.cpp note is
more useful than a table of engines, because it tells a future reader what
*happens* if they revisit it.

## Honest status summary

| Area | Status |
|---|---|
| Builder CLI, five profiles, digest-tracked generation | **Built** |
| On-device inference (engines, catalog, download, RAM preflight) | **Built** — the largest real code body |
| Frozen FFI seams (`InferenceProvider`, `StreamEvent`) | **Built** |
| Sync doctrine, ADRs, invariants | **Built as doctrine** |
| Runnable vertical slice | **Built**, but a deterministic stub rather than a live agent |
| Local-first sync engine (PSyncV2, buckets, write queue) | **Planned** — loopback transport today |
| Profile vault peer CRDT | **Planned** — design complete, no implementation |
| FRF integration | **Planned / absent** — zero code references |

## See also

- [Prior authorization](prior-auth.md) — a PHI boundary enforced the same way
- [Local-first](../guides/local-first.md) — FRF's authorized shape facade
- [Ports and adapters](../theory/ports.md) — the seam both projects converged on
