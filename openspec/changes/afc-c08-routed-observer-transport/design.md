# Design

## Context

The frozen `proto/flint/v1/envelope.proto` carries a custom kind only as enum value 6; the string attached to Rust's `EventKind::Custom` is lost by both gateway and Rust SDK conversion. Iggy stores the full JSON `EventEnvelope`. The existing `SubscribePipeline` checks tenant and view permission on each event, while `LogBroker` itself is a transport port. C08's BossFang source adapter and Gate grant profile remain separate contracts.

## Decisions

1. **Companion profile, no proto-v1 edit.** `RoutedObserverEnvelopeV1` is a typed JSON payload with the discriminator `frf.routed-observer/1`. `EventEnvelope::id` and `Offset` remain transport identities; `source.occurrence_id` and `delivery.subscriber_cursor_id` are different semantic identities. Gateway/SDK custom-kind loss cannot erase the payload discriminator. An unknown version is explicitly unsupported.
2. **Validate at both transport edges.** `frf-domain` validates required fields, source/channel tenant, source/correlation equality, and version-one causal ceilings. `frf-broker-iggy` checks candidate records before publish and after decode from Iggy. Ordinary envelopes keep their old behavior. The source adapter must generate stable occurrence/delivery/action IDs; Fabric never mints replacements.
3. **Preserve the authorization placement.** BossFang/Gate decide disclosure and recipient admission before publication and again before queue release. Fabric's existing gateway publish and subscribe use cases retain verified identity, channel permission, tenant equality, and per-event view checks. Direct access to `LogBroker` is a trusted adapter seam and is not itself a public authorization API. No profile field is a grant.
4. **Keep observer state out of the Iggy consumer offset.** A profile carries subscriber and cursor identities, but the observer owner persists and advances its own cursor. Iggy offsets remain replay locations and cannot acknowledge another observer's delivery.
5. **Truthful control.** `AgentRunControl.cancel` has no runtime-control port in Fabric. Its observed stream reports `UNIMPLEMENTED` and detaches. Actual cancellation must be addressed to the owning UAR/BossFang runtime; frozen proto-v1 is unchanged.

## Compatibility and release boundary

Legacy `EventEnvelope` JSON remains readable; only explicit routed candidates enter strict decoding. The public Rust type is additive, and the profile is not treated as generally deployable until Gate disclosure/delivery grants, UAR channel-source cursors, and BossFang route affinity are pinned and the final C08 real-boundary gate passes. SurrealDB SDK 3.3.0 is an independently authorized dependency alignment; it does not change this transport schema.

## Risks

The payload classification describes a projection but cannot prove the source filtered sensitive content; Gate and the source adapter must enforce that boundary. Native message identity and durable route decisions are upstream responsibilities. A transport replay without current disclosure grant must be withheld rather than treated as executable work.
