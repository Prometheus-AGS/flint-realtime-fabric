---
id: planes
title: The planes
sidebar_label: The planes
---

FRF carries four kinds of traffic with genuinely different characteristics. They
are called *planes* because each has its own delivery semantics, and treating
them uniformly would make every one of them worse.

Planes are Cargo features. A deployment that does not run media does not compile
the media stack.

| Plane | Status | Delivery shape |
|---|---|---|
| Entity | Live | Durable, ordered, replayable |
| Agent | Live | High-frequency, ephemeral, grouped by run |
| Media | Signaling live; sovereign SFU newly proven — see below | Real-time, loss-tolerant, never persisted |
| Federation | Functional, off by default | Projected copies of foreign event graphs |

## Entity plane

Postgres logical replication decodes the write-ahead log into the event spine,
which is filtered per subscriber by Keto and fanned out. The domain type is
`EntityChange`, carrying a `ChangeOp` of `Insert`, `Update`, `Delete`, or
`Upsert`.

Storage spans three tiers, chosen per role rather than per preference: redb for
the on-device operation log, SurrealDB for server-side CRDT checkpoints, and
Postgres as the source of truth.

**CRDT sync deliberately bypasses the spine for live traffic.** Peer-to-peer
edits flow over WebRTC; only checkpoints and announcements go on the spine. The
reasoning: the spine's value is durability and ordering, and collaborative
editing needs neither — it needs latency. Offline, a device appends to its local
operation log and applies changes optimistically; on reconnect it sends a
version vector and exchanges only the operations the other side is missing.

## Agent plane

Agent traffic is high-frequency and ephemeral. `AgentEvent` carries a protocol
tag (`AgUi`, `A2a`, `A2ui`, `Custom`) and an event kind — `RunStart`,
`TextDelta`, `ToolCall`, `ToolResult`, `StateSnapshot`, `RunEnd`, `Error` —
grouped by `run_id`. Transport is an actor bus built on ractor.

The volume is what shapes the design. [ADR-002](../decisions/overview.md)
measured 50–200 frames per second per run, which is why agent streams are
authorized **once at subscribe time** rather than per event: a 2ms Keto check at
100 frames per second is 200ms of latency per second of streaming, a 20%
slowdown, for a permission that has not changed.

The cost is stated rather than hidden: **revocation does not take effect until
the client disconnects.** ADR-009 records that this is insufficient for
protected clinical output, which is why that lane stays disabled — see the
[prior authorization case study](../case-studies/prior-auth.md).

## Media plane

`SignalEnvelope` carries an `SfuMode` of `Sovereign` or `Hosted`. Signaling is
relayed and **never persisted** — it does not go on the spine.

Two paths exist. **Hosted** uses LiveKit and is the supported v1 media path.
**Sovereign** uses str0m, an in-process Rust WebRTC implementation, so media
never leaves infrastructure you control.

:::info Sovereign SFU status — recently changed, narrowly scoped
The sovereign gate was flipped on after a local decode proof observed
`inbound-rtp.framesDecoded > 0` with a real browser pair. The scope of that
proof, quoted from the gateway source:

> it was a LOCAL run on a single Compose bridge, both peers inside the network,
> with coturn available. It demonstrates the relay decodes real media. It is NOT
> a multi-host, NAT-traversal, or scale result, and the advertised-candidate
> configuration is topology-sensitive.

`SFU_MODE` still defaults to `hosted`, and any unrecognised value falls back to
hosted. Configuration is topology-sensitive: `MEDIA_ADVERTISE_IP` must resolve
to an address the peer can actually reach. On a dual-stack bridge a hostname can
resolve IPv6-first and strand ICE in `new` with no error — a failure that looks
exactly like a broken media path and is not one.

Some pages in `docs/` still describe this gate as closed. The
[project status](../status.md) page tracks that.
:::

Two decisions shaped the sovereign path, and both came from failures worth
recording. [ADR-008](../decisions/overview.md) consolidated to **one shared UDP
socket** demultiplexed by `Rtc::accepts()`, because a fixed media port with
per-session binding is by construction single-session — the second session
failed with `EADDRINUSE`. [ADR-006](../decisions/overview.md) put RTP fan-out in
a room router rather than on the `MediaTransport` port, keeping packet
forwarding out of the port surface.

## Federation plane

Matrix (via Tuwunel) and ATProto (via the Tranquil firehose) each have working
inbound and outbound paths, but federation is **off by default**. The gateway
refuses to boot with `FEDERATION_ENABLED` set unless both
`FEDERATION_TENANT_ID` and `FEDERATION_CHANNEL_ID` are also present —
half-configured federation is a data-leak shape, so it fails loudly instead.

The model is projection, not ownership: Matrix owns its room DAG, and the spine
indexes a projected copy.

:::note Conflicting documentation
`ENVIRONMENT.md` describes federation as deferred and half-implemented, while
`SECURITY.md` marks all four paths functional. `SECURITY.md` is newer and more
specific. Tracked on the [status](../status.md) page.
:::

## The relational replication lane

[ADR-009](../decisions/overview.md) adds an authorized shape facade — a
server-side authorization boundary in front of ElectricSQL, so a local-first
client syncs only rows it may see.

:::caution Built, disabled, not certified
This lane is behind an off-by-default `shape-facade` Cargo feature. The live
Electric exchange has **never been run against a server**. Even when configured,
the gateway logs a warning saying the lane is not certified. A half-configured
deployment registers no route at all rather than an empty catalog.

Do not present this as available. See [local-first](../guides/local-first.md).
:::

## See also

- [Ports and adapters](ports.md) — the seams each plane sits behind
- [Authorization](authorization.md) — how visibility differs per plane
- [Project status](../status.md) — what is proven, built, and gated off
