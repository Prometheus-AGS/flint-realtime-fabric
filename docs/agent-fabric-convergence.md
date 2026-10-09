# Agent Fabric Convergence: Flint Realtime Fabric planning notes

Date: 2026-09-25

Status: planning input; wire changes require a repository-scoped child and contract review

Source baseline: `3043ca53239cc74a7f94a9e12484e2fd17726adb`

## C08 provider implementation checkpoint (2026-09-28)

The `codex/afc-c08-routed-envelope` worktree now carries a proposed provider
profile, `frf.routed-observer/1`, in
[`frf-domain`](../crates/frf-domain/src/routed_observer.rs). It is an additive
JSON companion inside the existing `EventEnvelope`; frozen protobuf v1 remains
unchanged. The profile records source occurrence, native provider identity,
selected handler and revisions, original principal, observer delivery and
subscriber cursor identities, projection class, and causal limits. The Iggy
adapter validates candidate records on publication and replay. Fabric still
neither chooses the handler nor owns the subscriber cursor.

The payload discriminator is required because protobuf v1 maps any Rust
`EventKind::Custom` string to enum value 6. A consumer must decode the payload
with `RoutedObserverEnvelopeV1::from_event_envelope` and reject legacy or unknown
profiles; matching a custom kind alone is insufficient. The public gateway
publish/subscribe path continues to apply its existing identity and event-view
checks. The provider does **not** include Gate's new disclosure/delivery grant,
BossFang's durable route store, or UAR's channel-source observer cursor; cross-host
routing stays disabled until those checkpoints and C08's composed gate exist.

`AgentRunControl.cancel` on Fabric now returns an unsupported runtime-control
status and closes only that observation stream. The owning runtime must receive
an actual cancellation request. The repository change is
[`afc-c08-routed-observer-transport`](../openspec/changes/afc-c08-routed-observer-transport/)
and its production gate remains pending.

## Role in the convergence

Flint Realtime Fabric is the durable transport and projection plane. It can carry committed agent, domain, observer, and connector facts between hosts, retain ordered positions, and expose replay. It must not execute agents, choose a workflow step, merge permissions, or become the authoritative task board.

The convergence plan's C05 keeps the BossFang workflow while delegating a complete run to UAR, and C06/C09 place runtime instances and team state in UAR. C01 must reconcile those statements with the accepted P1 contracts. When C10 evaluates durable waits or joins, it must choose one workflow owner for each object. Fabric carries that owner's facts and commands; it does not create a second loop.

## Current source facts

- `EventEnvelope` has a stable event id, tenant-qualified channel, monotonic offset, event kind, JSON payload, timestamp, and optional correlation id (`crates/frf-domain/src/envelope.rs`).
- `LogBroker` exposes publish, subscribe-from-offset, seek, acknowledge, and ensure-channel operations (`crates/frf-ports/src/log_broker.rs`). These are useful delivery primitives, but they do not by themselves define semantic occurrence identity, chosen recipient, authorization revision, or effect identity.
- `AgentEvent` identifies agent, tenant, session, protocol, event kind, run, content, and timestamp (`crates/frf-domain/src/agent.rs`; `proto/flint/v1/agent.proto`). It has no explicit source occurrence, recipient, workspace/room/thread address, causal depth, or per-observer delivery identity at this baseline.
- `AgentEventBus::subscribe` is tenant-scoped. `LibreFangBus` routes through lazily created tenant actors with idle eviction (`crates/frf-ports/src/agent_bus.rs`, `crates/frf-librefang/src/bus.rs`). That bus is a local adapter, not evidence of durable addressed cross-host delivery.
- The Cedar adapter currently evaluates a tenant principal, string action, and string resource against an in-memory policy set and an empty entity store (`crates/frf-policy-cedar/src/lib.rs`). Attribute policies are explicitly unsupported there. The repository contract separately assigns visibility to Keto and action policy to Cedar.
- The frozen protobuf currently mirrors the existing envelope and agent-event shapes. Any new required field therefore needs a versioned compatibility decision; it cannot be slipped into v1 without contract review (`proto/flint/v1/envelope.proto`, `proto/flint/v1/agent.proto`).

## Future routing contract

C08 should add only the minimum addressing and provenance needed to deliver an already admitted message to the declared handler and authorized observer copies. The contract needs to distinguish the source occurrence from each delivery, preserve provider/account/workspace/room/thread/sender identity, identify the chosen recipient, and keep worker acknowledgement separate from each observer's cursor. Exact field names and whether they extend an envelope or use a versioned companion message are C01/C08 decisions.

Gate and UAR remain the authorization/execution boundary. Fabric may require a current authorization receipt or projection grant before accepting or delivering a protected message, but transport possession is not authority. Revocation must stop queued disclosure or execution through the resource-side enforcement point; replay defaults to observation and must not repeat a protected effect.

The local BossFang adapter should translate between BossFang-owned channel objects and the Fabric contract while preserving the selected handler. It should not broaden a tenant broadcast into an executable command, and it should not create a second BossFang or UAR run.

For Codex CLI, Fabric carries durable run observations, handoff facts, and addressed commands after admission. Codex's Rust process remains under UAR execution ownership; dropping a Fabric subscription, losing a UI stream, or detaching an observer must not implicitly cancel the run.

## Later business-data responsibilities

C11 may use Fabric to distribute Forge-committed outbox/CDC facts and rebuild authorized read models. Forge remains the source of truth for the transaction and command result. C12 may use an embeddable peer profile for consented personal documents, but organization/runtime authority must remain outside CRDT writes.

## Dependencies and next repository-scoped KBD child

Recommended next child: **C08 Fabric durable addressed agent routing**.

It requires the D-FRF checkpoint plus accepted C05 full-run delegation and C07 observer contracts. Before implementation, the child must reconcile the frozen protobuf/versioning policy and claim exact files in `frf-domain`, `frf-ports`, the selected broker/gateway adapter, and `frf-librefang`. It must keep C11 domain replication and C12 peer synchronization in later children.

Acceptance should restart both ends, replay deliveries, exercise two matching handlers and two independent observers, revoke a queued subscription, and generate an A-B-A reaction chain. Evidence must show the selected recipient stays stable, observer cursors remain independent, executor control differs from stream detach, and no duplicate protected action occurs.

## Evidence to preserve in the child plan

- Baseline commit, frozen proto revision, and selected compatibility mechanism.
- Mapping between source occurrence, transport event, delivery, run, attempt, and effect identifiers.
- Handler precedence and conflict behavior.
- Gate authorization/projection receipt consumed at publish and delivery boundaries.
- Broker retention, acknowledgement, replay, and retention-gap behavior.
- Clear separation of the durable broker path from the current tenant-local actor bus.
- A kickoff recheck of the C05/C07 dependency receipts and the single executor/workflow-owner assignments.
