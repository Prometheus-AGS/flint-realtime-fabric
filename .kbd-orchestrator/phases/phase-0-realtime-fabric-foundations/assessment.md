# Assessment: Phase 0 — Flint Realtime Fabric Foundations

> **RFC-FRF-002 · Prometheus AGS**
> Status: assessment complete · 2026-06-17
> Prepared by: kbd-assess workflow

---

## 1. Objective

Determine what it takes to build a realtime communications, pub/sub, and agentic
application support framework that is **demonstrably superior** to the current
field across every dimension that matters: WebRTC/media, persistent pub/sub,
CRDT/offline sync, federated protocols (Matrix, ATProto), and agentic protocols
(AG-UI, A2A, A2UI, MCP). Produce a competitive analysis matrix to anchor the
design decision gate before Phase 1.

---

## 2. Market Landscape Summary

Nine distinct competitor categories were researched:

| Category | Representatives |
|---|---|
| Postgres-native realtime | Supabase Realtime |
| Reactive database | Convex |
| BEAM/Erlang pub/sub | Elixir Phoenix Channels |
| WebRTC SFU | LiveKit |
| Enterprise managed pub/sub | Ably |
| Federated messaging | Matrix (Synapse/Dendrite) |
| Open social protocol | AT Protocol (Bluesky) |
| Enterprise event streaming | Apache Kafka, Redpanda, Apache Pulsar |
| Rust-native message spine | Apache Iggy |

---

## 3. Competitive Analysis Matrix

### 3a. Core Realtime Capabilities

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka/Redpanda | Apache Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Transport: WebSocket** | ✅ | ✅ | ✅ | ✅ signaling | ✅ | ✅ | ✅ | ❌ (pull) | ✅ | ✅ |
| **Transport: SSE** | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ (Connect-ES) |
| **Transport: MQTT** | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ⬜ future |
| **Transport: QUIC** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ via Iggy |
| **Transport: gRPC** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (tonic) |
| **Transport: Connect** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Connect-ES |
| **Named pub/sub channels** | ✅ | ❌ (implicit) | ✅ | ✅ rooms | ✅ | ✅ rooms | ❌ firehose | ✅ topics | ✅ topics | ✅ |
| **Presence** | ✅ | ❌ | ✅ (CRDT) | ✅ participants | ✅ | ⚠️ limited | ❌ | ❌ | ❌ | ✅ |
| **Message history/replay** | ❌ (ephemeral) | ❌ | ❌ | ❌ | ✅ 365d Pro | ❌ ephemeral | ✅ (repo) | ✅ | ✅ | ✅ |
| **Guaranteed delivery** | ❌ | ❌ | ❌ | ❌ | ✅ (idempotent) | ❌ | ❌ | ✅ (EOS) | ✅ | ✅ |
| **Message ordering** | ❌ | ✅ (OCC) | ❌ | ❌ | ✅ per-publisher | ✅ per-room | ✅ (repo log) | ✅ per-partition | ✅ per-partition | ✅ |
| **Durable message spine** | ❌ WAL only | ❌ | ❌ | ❌ | ✅ | ✅ replicated | ✅ CAR/MST | ✅ | ✅ | ✅ Iggy |
| **Throughput ceiling** | ~DB WAL rate | DB-bound | 2M WS/node | media-only | 10K msg/s Pro | federation-limited | firehose-limited | 1T msg/day | 5GB/s+ | ✅ Iggy-backed |
| **P99 latency** | ~25ms | OCC-latency | <1ms local | <100ms WebRTC | 6.5ms edge | variable | variable | 5ms Kafka | ~2ms | ✅ <5ms target |

### 3b. Database / State Synchronization

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Postgres CDC** | ✅ WAL | ❌ | ❌ | ❌ | ✅ LiveSync | ❌ | ❌ | ✅ (Debezium) | ✅ connector | ✅ frf-postgres-cdc |
| **Row-Level Security** | ✅ Postgres RLS | ❌ code-only | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Keto + RLS |
| **CRDT / offline sync** | ❌ | ❌ | ✅ Presence CRDT | ❌ | ⚠️ LiveObjects | ❌ | ❌ | ❌ | ❌ | ✅ Loro/automerge |
| **Local-first / offline** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ redb + reconnect |
| **Multi-DB CDC** | ❌ | ❌ | ❌ | ❌ | ✅ PG+Mongo | ❌ | ❌ | ✅ | ❌ | ⬜ Phase 6+ |
| **Reactive queries** | ❌ CDC only | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⬜ via CDC |
| **On-device store** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ redb (native) + IndexedDB (WASM) |

### 3c. WebRTC / Media

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **1:1 voice/video** | ❌ | ❌ | ❌ | ✅ SFU | ❌ | ✅ VoIP | ❌ | ❌ | ❌ | ✅ str0m/LiveKit |
| **Group calls / SFU** | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ MatrixRTC | ❌ | ❌ | ❌ | ✅ |
| **E2E encrypted media** | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ str0m |
| **STUN/TURN bundled** | ❌ | ❌ | ❌ | ✅ | ❌ | ⚠️ external | ❌ | ❌ | ❌ | ✅ |
| **SIP/PSTN telephony** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ⬜ Phase 4+ |
| **Recording / egress** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ⬜ Phase 4+ |
| **AI voice agent** | ❌ | ❌ | ❌ | ✅ Agents | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ via str0m + spine |
| **Sovereign SFU** | ❌ | ❌ | ❌ | ✅ (Apache 2) | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ str0m (sovereign) |

### 3d. Agent / AI Protocol Support

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **AG-UI streaming** | ❌ | ❌ | ❌ | ❌ | ✅ AI Transport | ❌ | ❌ | ❌ | ❌ | ✅ frf-agentproto |
| **A2A (agent-to-agent)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ frf-agentproto |
| **A2UI (agent-to-UI)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ frf-agentproto |
| **MCP server** | ❌ | ❌ | ❌ | ✅ docs MCP | ❌ | ❌ | ❌ | ❌ | ✅ 40+ tools | ✅ frf-gateway |
| **LLM token streaming** | ❌ | ❌ | ❌ | ❌ | ✅ AI Transport | ❌ | ❌ | ❌ | ❌ | ✅ via AG-UI events |
| **Durable agent sessions** | ❌ | ❌ | ❌ | ❌ | ✅ AI Transport | ❌ | ❌ | ❌ | ❌ | ✅ Iggy durability |
| **BossFang / ractor actors** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ frf-librefang |
| **Human-in-the-loop** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ AG-UI interrupt |
| **Multimodal events** | ❌ | ❌ | ❌ | ✅ (tracks) | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

### 3e. Federation / Open Protocol Support

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Matrix federation** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ native | ❌ | ❌ | ❌ | ✅ Tuwunel bridge |
| **ATProto firehose** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ native | ❌ | ❌ | ✅ Tranquil bridge |
| **ActivityPub** | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ bridge | ❌ | ❌ | ❌ | ⬜ future |
| **IRC / XMPP** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ bridges | ❌ | ❌ | ❌ | ⬜ via Matrix |
| **Kafka outbound** | ❌ | ❌ | ❌ | ❌ | ✅ Firehose | ❌ | ❌ | native | ✅ connector | ✅ via Iggy |
| **Custom AppView** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ ATProto |
| **Self-hostable** | ✅ | ✅ FSL | ✅ | ✅ Apache 2 | ❌ | ✅ | ✅ | ✅ | ✅ Apache | ✅ |
| **Open spec / OSS** | ✅ Elixir | ✅ FSL | ✅ Apache 2 | ✅ Apache 2 | ❌ proprietary | ✅ Apache 2 | ✅ (OSS) | ✅ Apache 2 | ✅ Apache | ✅ MIT |

### 3f. Security & Identity

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **JWT auth** | ✅ JWKS | ❌ code | ✅ tokens | ✅ | ✅ | ✅ access tokens | ✅ OAuth 2.0 | ⚠️ external | ✅ PAT | ✅ Kratos + Oathkeeper |
| **SSO / OIDC** | ✅ | ❌ | ❌ | ❌ | ✅ Enterprise | ✅ Matrix 2.0 | ✅ OAuth | ❌ | ❌ | ✅ Oathkeeper |
| **ReBAC (Zanzibar)** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Ory Keto |
| **Attribute-based policy** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Cedar (PAUX-1) |
| **Per-event RLS** | ✅ Postgres RLS | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Keto check pipeline |
| **E2EE** | ❌ | ❌ | ❌ | ✅ | ✅ channel | ✅ Olm/Megolm | ⬜ MLS roadmap | ❌ | ✅ AES-256-GCM | ✅ |
| **Tenant isolation** | ✅ per-project | ✅ | ❌ | ✅ rooms | ✅ | ✅ per-server | ✅ per-PDS | ⚠️ | ✅ per-stream | ✅ Keto tenant |
| **Audit trail** | ❌ | ❌ | ❌ | ❌ | ✅ Enterprise | ❌ | ✅ (signed) | ❌ | ❌ | ✅ tracing spans |

### 3g. Developer Experience & SDK Coverage

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Rust SDK** | ❌ | ✅ (backend) | ❌ | ❌ | ❌ | Ruma lib | ❌ | ❌ | ✅ native | ✅ hand-written |
| **TypeScript/JS SDK** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ generated |
| **Go SDK** | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ generated |
| **Swift SDK** | ✅ | ❌ | ⚠️ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ UniFFI |
| **Kotlin/Android SDK** | ✅ | ❌ | ⚠️ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ UniFFI |
| **Flutter/Dart SDK** | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ flutter_rust_bridge |
| **C# SDK** | ❌ | ❌ | ⚠️ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ generated |
| **Proto-generated SDKs** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ tonic codegen |
| **WASM / browser CRDT** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ frf-wasm |
| **Admin UI** | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ⚠️ | ✅ SvelteKit | ✅ React 19 + shadcn |
| **Single binary deploy** | ❌ | ❌ | ❌ | ✅ | ❌ (SaaS) | ⚠️ | ✅ PDS | ⚠️ | ✅ | ✅ target |

### 3h. Operational Scale & SLA

| Feature | Supabase RT | Convex | Phoenix | LiveKit | Ably | Matrix | ATProto | Kafka | Iggy | **FRF Target** |
|---|---|---|---|---|---|---|---|---|---|---|
| **Concurrent connections** | 500 free / ∞ paid | serverless | 2M+ per node | cluster | 200–∞ | homeserver-limited | relay-limited | consumer-group | single-node | cluster (Phase 7) |
| **Horizontal clustering** | ✅ managed | ✅ managed | ✅ distributed Erlang | ✅ Redis-backed | ✅ managed | ✅ homeserver mesh | ✅ relay mesh | ✅ | ❌ (roadmap) | ✅ (Phase 7) |
| **Message throughput** | DB-limited | DB-limited | 100Ks/s/node | media-only | 10K/s Pro | room-limited | firehose ~5M/day | 1T+/day | 5GB/s+ | Iggy-backed |
| **Managed SaaS option** | ✅ | ✅ | ❌ | ✅ Cloud | ✅ | ❌ | ❌ (Bluesky) | ✅ Confluent | ❌ | ⬜ future |
| **On-prem / self-host** | ✅ | ✅ FSL | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ primary model |
| **99.999% SLA** | ❌ | ❌ | ❌ | ✅ Cloud | ✅ Enterprise | ❌ | ❌ | ✅ Confluent | ❌ | ⬜ Phase 7 |

---

## 4. Key Insights by Competitor

### Supabase Realtime
**Strengths:** Postgres-native CDC with RLS enforcement is the single best database-change fan-out story in the market. JWKS JWT support. Open source (Elixir).
**Critical gaps:** No WebRTC. No message persistence for broadcast. No guaranteed delivery. No agent protocols. No federation. No offline/CRDT. Free tier hard cap at 500 connections.
**FRF edge:** `frf-postgres-cdc` replicates the CDC+RLS strength. FRF adds durability (Iggy spine), WebRTC, CRDT, federation, and agent protocols on top. Supabase Realtime is strictly additive within a single Postgres project — FRF is a sovereign substrate across any data source.

### Convex
**Strengths:** Reactive query model (read-set subscriptions) is elegant. Full ACID + OCC. TypeScript-first. Rust backend (open-sourced April 2024). FSL license allows self-hosting.
**Critical gaps:** No pub/sub channels. No WebRTC. No federation. No agent protocols. No offline. JavaScript/TypeScript only for server functions. Auth depends on Clerk/Auth0.
**FRF edge:** FRF provides the durability and fan-out model Convex deliberately does not. Convex is a reactive database; FRF is a realtime communications substrate. They solve different layers.

### Elixir Phoenix Channels
**Strengths:** Proven at 2M WebSocket connections per node. BEAM process model is the best connection-per-process story. Presence CRDT is built in and cluster-aware. Hot reload without dropping connections.
**Critical gaps:** At-most-once delivery (no message persistence). No WebRTC. No agent protocols. No federation. Custom protocol (not interoperable). No offline. Requires Erlang cluster for horizontal scale.
**FRF edge:** Phoenix is the closest architectural ancestor to FRF's WebSocket mux layer. FRF improves on it by adding a durable spine (Iggy), proto-generated multi-SDK surface, WebRTC, CRDT, and federation — while Rust provides comparable connection density without the JVM/BEAM constraint.

### LiveKit
**Strengths:** The best WebRTC SFU available. Agents framework for AI voice/video is 12–18 months ahead of any competitor. Apache 2 license. Single binary with bundled STUN/TURN. Horizontal scale via Redis. Physical AI / robotics positioning.
**Critical gaps:** No pub/sub channels (room model only). No Postgres CDC. No CRDT/offline. No federation. No A2A/AG-UI/A2UI protocol (room pub/sub is the only agent coordination model). Operational complexity for self-hosted distributed deployments (Redis required). Single-room isolation means private 1:1 agent sessions require one room per user pair.
**FRF edge:** FRF wraps LiveKit (hosted) or str0m (sovereign) behind `frf-media-*` adapters. Media is one of seven planes; FRF adds the durable event spine, CDC, CRDT, federation, and structured agent protocols that LiveKit deliberately excludes. LiveKit Agents join rooms as participants — FRF agents publish structured AG-UI/A2A/A2UI events to the spine, which is durable, replayable, and auditable.

### Ably
**Strengths:** Best-in-class managed reliability (8x9s message survivability, 700+ PoPs). Multi-protocol (WebSocket + SSE + MQTT + AMQP). Message history, rewind, exactly-once delivery. AI Transport product for LLM token streaming is the only first-party managed agentic transport in the market (ahead of all except FRF target). LiveSync for Postgres/MongoDB CDC. LiveObjects (CRDT-like state).
**Critical gaps:** SaaS-only (no self-hosting). No WebRTC. No native federation. No sovereign agent compute. Presence cap (20K max). Several Chat/Spaces features still "coming soon." High implementation complexity for collaboration features relative to higher-level platforms.
**FRF edge:** FRF is sovereign (self-hostable, open source) where Ably is proprietary managed SaaS. FRF's AI Transport equivalent (frf-agentproto) runs on a durable spine with replay semantics. FRF adds WebRTC, CRDT with local-first offline, federation (Matrix + ATProto), and a typed multi-SDK surface that Ably lacks. For enterprise users who cannot send data to a third-party managed service, FRF is the only viable option.

### Matrix Protocol
**Strengths:** The only open, federated, E2EE messaging protocol at scale. Room state replication across homeservers is a unique distributed design. Element Call (WebRTC group calls via MatrixRTC) is a first-class feature. Bridges to IRC, XMPP, Slack, Discord, Telegram. MLS-based E2EE (Matrix 2.0).
**Critical gaps:** Metadata exposure (not GPA-resistant). Synapse performance issues (O(all-state) state resolution, concurrent federation send storms). No native agent/AI protocols. Presence is limited. Account portability not fully implemented. No native pub/sub beyond room timelines.
**FRF edge:** FRF consumes Matrix via `frf-bridge-matrix` (Tuwunel projection) — taking its federation surface as an adapter, not competing on the room-sync layer. FRF adds a high-throughput durable spine, structured agent events, and per-event RLS that Matrix's DAG model cannot provide.

### AT Protocol
**Strengths:** The only open social data protocol with true account portability (DID-based), cryptographic data integrity (signed MST repositories), and a growing ecosystem of independent AppViews and Feed Generators. OAuth 2.0 with fine-grained auth scopes (2025). IETF standardization in progress. Jetstream provides accessible JSON firehose.
**Critical gaps:** No native private data or E2EE (roadmap). No filtered server-side subscriptions (consume full firehose). Relay resource-intensive (~$150/month bare metal for full-network mirror). PLC directory governance still centralizing. No agent/AI protocol primitives. No WebRTC. Full CBOR/CAR validation is complex.
**FRF edge:** FRF consumes ATProto via `frf-bridge-atproto` (Tranquil firehose → spine → custom AppView). FRF provides the private/secured channel layer (Keto RLS) that ATProto's public firehose model lacks. FRF agents publish to the Iggy spine; the bridge projects relevant facts into ATProto-compatible record types for the open social graph.

### Apache Kafka / Redpanda / Pulsar
**Strengths:** Proven at internet scale (trillions of messages/day). Kafka is the industry standard for event streaming. Redpanda matches Kafka throughput with lower operational overhead (no JVM, no ZooKeeper). Pulsar offers native multi-tenancy and geo-replication.
**Critical gaps:** Not designed for per-client WebSocket fan-out. No presence. No CRDT. No federation. No agent protocols. Require bridge infrastructure to reach browser/mobile clients. Minimum operational footprint is large (Kafka cluster + ZooKeeper or KRaft, or Redpanda cluster).
**FRF edge:** FRF uses Iggy (not Kafka) as its spine but exposes Kafka-compatible connectors through `frf-broker-iggy`. Kafka/Redpanda are the right comparison point for the enterprise event streaming tier — FRF's Iggy adapter is designed to be swappable to NATS/Redpanda behind the `LogBroker` port if throughput requirements demand it.

### Apache Iggy
**Strengths:** The only Rust-native, io_uring-backed persistent message streaming platform. 5GB/s+ throughput. Sub-millisecond P99 latency. TCP + QUIC + WebSocket + HTTP transports. AES-256-GCM encryption. Consumer groups. S3 archiving. MCP server (40+ tools). Apache Incubator (February 2025). Single binary. Thoughtworks Technology Radar inclusion.
**Critical gaps:** No clustering yet (single node; VSR-based clustering on roadmap). io_uring is Linux-specific (degrades on macOS/Windows via Docker). Not Kafka-compatible (no wire protocol parity). Young project (pre-1.0).
**FRF edge:** Iggy IS the FRF spine (via `frf-broker-iggy`, GQAdonis fork). FRF's `LogBroker` port means Iggy is swappable to NATS/Redpanda without application changes. The clustering gap is mitigated in Phase 0–1 by single-node deployment with the `LogBroker` abstraction making migration safe when clustering lands.

---

## 5. Feature Gaps in the Entire Market (FRF Opportunities)

The following capabilities do not exist in **any** single framework today. FRF is the first to target all of them in one sovereign, self-hostable system:

1. **Unified durable event spine + per-client WebSocket fan-out + per-event Zanzibar RLS.** Supabase has CDC+RLS but no durability. Ably has durability but no RLS. Kafka has durability but no WebSocket fan-out. No one has all three.

2. **WebRTC SFU + durable spine integration with media-never-on-log separation.** LiveKit has a great SFU but its event model is ephemeral and room-scoped. Matrix has WebRTC calls but no durable spine. No framework has a sovereign SFU whose signaling events are durable, replayable, and auditable facts on a separate spine.

3. **AG-UI + A2A + A2UI on a durable spine with replay.** Ably AI Transport is the closest competitor but is SaaS-only and proprietary. No open-source framework supports the full AG-UI event model (including interrupts, state deltas, HITL, tool streaming) on a durable, self-hosted spine.

4. **CRDT offline sync with identical merge logic across Web/iOS/Android/Flutter via FFI.** All CRDT solutions (Yjs, Automerge, Loro) are language-specific. No realtime framework provides a single Rust CRDT engine bound to all platforms via FFI with on-device persistence (redb on native, IndexedDB on WASM).

5. **Matrix bridge + ATProto bridge + Postgres CDC + WebRTC + agent events — in one deployable.** No competitor bridges all three open protocol families (federated messaging, open social graph, database CDC) and wraps them with agent event protocols and WebRTC in a single gateway.

6. **9-language SDK surface from a single protobuf contract + Rust FFI core.** LiveKit comes closest (6 SDKs), but all are hand-written. FRF's approach — Rust hand-written, Go/C#/TS generated, Swift/Kotlin/Dart FFI-bound — means business logic, CRDT merge, and reconnection live in exactly one place.

7. **Sovereign, self-hostable, open-source full-stack realtime.** Ably is the most feature-complete single platform but is proprietary SaaS. LiveKit is open source but media-only. FRF targets the same feature surface as Ably (pub/sub, presence, CDC, AI transport, chat) plus WebRTC, CRDT, and federation, as a sovereign open-source deployment.

---

## 6. Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| Iggy clustering not yet available (single node) | HIGH | `LogBroker` port makes swap to NATS/Redpanda safe; accept single-node for Phase 0–2 |
| CRDT engine decision (Loro vs automerge-rs) | HIGH | Decide before Phase 3; both are mature; affects all FFI bindings |
| Per-event Keto check latency at fan-out scale | HIGH | Subscribe-time scoping + topic partitioning + check-cache with tuple-delete invalidation (design in Phase 1) |
| UniFFI / flutter_rust_bridge version currency | MEDIUM | Confirm at Phase 0 kickoff; verify language coverage |
| Nine-language SDK surface | MEDIUM | Three patterns only (hand-write Rust, generate Go/C#/TS, FFI-bind Swift/Kotlin/Dart) — never hand-write non-Rust |
| str0m maturity (sans-I/O WebRTC) | MEDIUM | LiveKit adapter as hosted fallback; str0m for sovereign deployment |
| Proto stability after freeze | LOW | Tag proto-v1 before any SDK generation; breaking changes are a new proto version |
| Tuwunel (Matrix) operational complexity | LOW | Phase 6; Matrix bridge is last because it integrates a system already running elsewhere |

---

## 7. Verdict: Build

**Decision: Proceed to planning (Phase 1 plan).**

The market analysis confirms:
- No single framework covers the full capability surface.
- The seven unique differentiators (§5) are not addressable by combining existing frameworks without a sovereign substrate.
- The technical approach (Iggy spine + Axum gateway + Keto RLS + port-based hexagonal architecture) is validated by the competitive landscape.
- The three-pattern SDK strategy (hand-write/generate/FFI) is the only tractable path for nine-language coverage.
- Ably AI Transport is the only direct competitor on agentic realtime — it is proprietary SaaS; FRF is the open-source sovereign answer.

**Phase 0 exit criteria to verify at gate:**
- [ ] Workspace compiles with `frf-domain`, `frf-ports`, `frf-proto`, `frf-gateway` crates
- [ ] `proto-v1` tagged and frozen
- [ ] `frf-gateway` serves `/healthz`
- [ ] Dagger CI green: `fmt --check`, `clippy --all-targets -- -D warnings -W clippy::pedantic`, `test`, MSRV
- [ ] CRDT engine decision recorded before Phase 3 planning begins
- [ ] Version currency confirmed: Iggy GQAdonis fork, UniFFI, flutter_rust_bridge, Connect-ES, tonic

---

## 8. Next Step

```
/kbd-plan phase-0-realtime-fabric-foundations
```
