---
id: ports
title: Ports and adapters
sidebar_label: Ports and adapters
---

A port is a trait in `frf-ports` describing a capability in the *application's*
vocabulary — "append to a durable log", "check whether a subject may view an
object". An adapter is a crate implementing exactly one port against exactly one
technology.

Every port is `Send + Sync + 'static`, so it can live behind an `Arc` in the
gateway's shared state.

## The thirteen ports

| Port | Abstracts | Implemented by |
|---|---|---|
| `LogBroker` | Durable event spine: publish, subscribe, seek, ack | `IggyBroker` (Apache Iggy) |
| `AuthzProvider` | Zanzibar relation-tuple check/write/delete | `KetoAuthzProvider` (Ory Keto) |
| `IdentityVerifier` | JWT/OIDC verification at the boundary | `OryIdentityVerifier` |
| `ActionPolicyProvider` | Cedar policy on **mutations** — not visibility | `CedarPolicyEngine`, `NoOpPolicyProvider` |
| `EntityStore` | Read and watch entities, tenant-scoped | `InMemoryEntityStore` |
| `CrdtStore` | Checkpoint, restore, purge CRDT snapshots | `SurrealCrdtStore`, `InMemoryCrdtStore` |
| `OpStore` | On-device durable write-ahead log for outgoing ops | `RedbOpStore` |
| `ApplyDelta` | Engine-agnostic CRDT merge function | `LoroDeltaApplier` |
| `AgentEventBus` | Actor-based agent event bus | `LibreFangBus` (ractor) |
| `MediaSignaler` | WebRTC signaling relay — never stores media | `StrOmSignaler`, `LiveKitSignaling` |
| `MediaTransport` | Per-session sovereign media engine | `StrOmTransport` (hosted has no implementation) |
| `FederationBridge` | Send and receive across Matrix and ATProto | `MatrixBridge`, `AtProtoBridge` |
| `ShapeFacade` | Authorized relational read path (ADR-009) | `ElectricShapeFacade` — **disabled by default** |

All adapters share one error type, `PortError`, marked `#[non_exhaustive]` so
adding a variant is not a breaking change for matchers.

## What a port looks like

```rust
/// Durable event spine — publish, subscribe, seek, acknowledge.
///
/// Implemented by `frf-broker-iggy`. Wired in `frf-gateway`.
/// Adapter crates MUST instrument their implementations with
/// `#[tracing::instrument(name = "port::LogBroker::<method>")]`.
#[async_trait]
pub trait LogBroker: Send + Sync + 'static {
    /// Publish an event to a channel. Returns the assigned `Offset`.
    async fn publish(&self, envelope: EventEnvelope) -> Result<Offset, PortError>;

    /// Open a streaming subscription starting from `from`.
    ///
    /// Pass `Offset::BEGINNING` to replay from the start.
    async fn subscribe(
        &self,
        channel_id: ChannelId,
        consumer_id: String,
        from: Offset,
    ) -> Result<EventStream, PortError>;
```

Two details are worth noticing. The trait mentions no broker product — a
different implementation could be Kafka, NATS, or an in-memory vector. And the
doc comment *mandates* a `tracing` span name, so every port crossing is
observable with a consistent naming scheme rather than whatever each adapter
author chose.

## `ApplyDelta`: a port that exists to protect a boundary

Most ports abstract infrastructure. `ApplyDelta` exists for a subtler reason: to
keep `frf-app` from importing `frf-crdt`.

CRDT merge is a pure function — given a document and a delta, produce a new
document. It needs no network and no database, so it is tempting to call the
CRDT library directly from a use-case. That import would make the Loro decision
visible to the application layer, and reversing it would then touch use-case
code.

Instead, merge is a port, and the domain's `SyncOp` carries an **engine-agnostic**
payload:

```rust
pub struct SyncOp {
    // ...
    /// Opaque, engine-specific encoded operation bytes.
    pub payload: Vec<u8>,
}
```

`Vec<u8>` rather than a Loro type. The engine choice stops at the
`frf-crdt` crate boundary, and every FFI binding gets the same merge behaviour
because there is only one implementation of it.

## Type erasure where runtime choice is needed

Some adapters are selected at runtime from configuration. `SFU_MODE` picks
between the sovereign and hosted media paths; `POLICY_ENGINE` picks between
Cedar and a no-op. Rather than making the whole gateway generic over these
choices, the ports provide erased wrappers — `DynMediaSignaler`,
`DynMediaTransport`, `BoxedPolicyProvider`.

Swapping an adapter is then a match arm rather than a refactor:

```rust
fn build_policy_provider(config: &GatewayConfig) -> Result<BoxedPolicyProvider> {
    match config.policy_engine {
        PolicyEngineMode::Cedar => {
            tracing::info!("action policy engine: Cedar");
            let engine = CedarPolicyEngine::new()
                .map_err(|e| anyhow::anyhow!("failed to load Cedar policy: {e}"))?;
            Ok(BoxedPolicyProvider(Arc::new(engine) as DynPolicyProvider))
        }
        PolicyEngineMode::None => {
            tracing::info!("action policy engine: no-op (all permitted)");
            Ok(BoxedPolicyProvider(
                Arc::new(NoOpPolicyProvider) as DynPolicyProvider
            ))
        }
    }
}
```

The `None` branch logs that it is permitting everything. A permissive default
that says nothing is how a staging configuration reaches production unnoticed.

## The composition root

`AppState` is generic over one parameter per port. Only `main.rs` names concrete
adapters:

```rust
pub struct AppState<L, A, I, M, B, P> {
    pub subscribe_pipeline: Arc<SubscribePipeline<L, A, I>>,
    pub publish_usecase: Arc<PublishUseCase<L, A, I>>,
    pub media_signaler: Arc<M>,
    pub agent_bus: Arc<B>,
    /// Identity verifier — used at every gateway boundary to verify JWTs.
    pub identity: Arc<I>,
    /// Authorization provider used for subscribe-time visibility checks.
    pub authz: Arc<A>,
```

The abstract becomes concrete in exactly one signature:

```rust
fn spawn_grpc_server(
    state: Arc<
        AppState<
            IggyBroker,
            ConfiguredAuthzProvider,
            OryIdentityVerifier,
            DynMediaSignaler,
            LibreFangBus,
            BoxedPolicyProvider,
        >,
    >,
) -> Result<Option<tokio::task::JoinHandle<()>>> {
```

If you want to know what this deployment actually runs, that list is the answer,
and it is the only place you need to look.

## Composing two ports without coupling their adapters

Media room-join requires an authorization check ([ADR-007](../decisions/overview.md)).
The obvious implementation puts a Keto call inside the str0m adapter — and that
would violate [one port per adapter](dependency-rule.md), give the media engine
an authz dependency, and make it untestable without a permission service.

Instead the composition happens in the gateway:

```rust
let media_bridge = if config.sfu_mode == frf_gateway::SfuMode::Sovereign {
    let transport = Arc::new(frf_media_str0m::StrOmTransport::with_config(
        frf_media_str0m::MediaConfig::from_env(),
    ));
    Some(Arc::new(
        frf_gateway::media_bridge::MediaTransportBridge::new(transport)
            .with_authz(Arc::clone(&authz) as Arc<dyn frf_ports::AuthzProvider>),
    ))
} else {
    None
};
```

The media engine knows nothing about authorization. The authorization provider
knows nothing about media. The bridge knows about both, and lives in the layer
whose job that is.

## See also

- [The dependency rule](dependency-rule.md) — why the seams are where they are
- [Writing an adapter](../guides/writing-an-adapter.md) — the practical walkthrough
- [Authorization](authorization.md) — how the three security mechanisms compose
