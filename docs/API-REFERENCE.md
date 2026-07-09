# API Reference — flint.v1

Generated from the frozen `proto-v1` contract (`proto/flint/v1/*.proto`). The proto is
the source of truth for every non-Rust SDK; a wire-breaking change is a new proto
version, never an edit.

Transports: native gRPC (HTTP/2) on the gRPC port, and Connect / gRPC-web (HTTP/1.1)
for browsers. All calls carry `Authorization: Bearer <jwt>` (browser WebSocket routes
use a `?token=` query parameter).

**Server status** — whether the gateway currently implements the service:

| Service | Server | Notes |
|---------|:------:|-------|
| `SpineService` | ✅ live | publish / subscribe / ack — the core data plane |
| `SignalService` | ✅ live | WebRTC signaling (bidi) |
| `SyncService` | ✅ live | CRDT sync (bidi) + checkpoint |
| `AgentService` | ✅ live | agent runs (server-streaming) |
| `EntityService` | ✅ live | entity read/watch — auth-guarded (p17-c004) |
| `AuthzService` | ✅ live | relation check/write/delete via Keto (p17-c005) |

---

## SpineService (`envelope.proto`)

The event spine — publish/subscribe over channels.

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `Publish` | unary | `PublishRequest` → `PublishResponse` (assigned `Offset`) |
| `Subscribe` | server-stream | `SubscribeRequest` → stream of `EventEnvelope` |
| `Ack` | unary | `AckRequest` → `AckResponse` |

`SubscribeRequest { channel_id, consumer_id, from: Offset }`. The stream resumes from
`from`; each `EventEnvelope` carries an `offset` for replay (the Rust/TS/Go/C# SDKs use
it for reconnect-and-resume).

## SignalService (`signal.proto`)

WebRTC signaling relay.

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `Signal` | bidi-stream | stream `SignalEnvelope` ↔ stream `SignalEnvelope` |

Routing: a signal is delivered to `to_session` (unicast) or, when unset, fanned out to the
other members of `room_id` — never echoed to the sender (p18-c001). The reported
`sfu_mode` reflects the gateway's configured mode (p18-c003). The media plane itself
(str0m sovereign SFU) is still signaling-only / negotiation-spike; hosted (LiveKit) is the
live media path — see `docs/SECURITY.md` §6.

The browser admin UI also has an Axum WebSocket endpoint `/ws/v1/signal?room=&tenant=&token=`
that streams `SignalFrame` JSON (see the gateway routes).

## SyncService (`sync.proto`)

CRDT synchronization.

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `Sync` | bidi-stream | stream `SyncRequest` ↔ stream `SyncResponse` (op batches + checkpoints) |
| `GetCheckpoint` | unary | `SyncCheckpoint` → `SyncCheckpoint` |

## AgentService (`agent.proto`)

Agent execution (AG-UI / A2A).

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `RunAgent` | bidi-stream | stream `AgentRunRequest` → stream `AgentEvent` |

## EntityService (`entity.proto`)

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `GetEntity` | unary | `GetEntityRequest` → `EntityResponse` |
| `WatchEntity` | server-stream | `WatchEntityRequest` → stream `EntityChange` |

Live since p17-c004. Every read is auth-guarded: the bearer JWT is verified, its tenant
must equal the requested tenant, and the subject must hold Keto `view` on the entity.

## AuthzService (`authz.proto`)

| RPC | Kind | Request → Response |
|-----|------|--------------------|
| `Check` | unary | `CheckRequest` → `CheckResponse` |
| `WriteRelation` | unary | `WriteRelationRequest` → `WriteRelationResponse` |
| `DeleteRelation` | unary | `DeleteRelationRequest` → `DeleteRelationResponse` |

Live since p17-c005, delegating to the Keto-backed `AuthzProvider`. Each op verifies the
caller's token and enforces tenant-equality (a caller may only operate on relations in
its own tenant). The **`frf keto seed|revoke`** CLI remains available for operators.

---

## Regenerating this reference

The service/RPC list above is derived from `proto/flint/v1/*.proto`. When the proto
version advances, regenerate the per-language SDKs (`sdks/*`) from the source proto and
update this table. For full message field definitions, read the `.proto` files directly —
they are the authoritative, commented contract.
