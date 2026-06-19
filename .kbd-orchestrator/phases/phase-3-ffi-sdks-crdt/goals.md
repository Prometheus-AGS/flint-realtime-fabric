# Goals — Phase 3: CRDT Core + Offline Persistence + FFI SDK Tier

> RFC-FRF-002 · Prometheus AGS
> Source: IMPLEMENTATION-PLAN.md §314 — Phase 3

## Prerequisites (must resolve at kickoff — load-bearing decisions)

- **CRDT engine decision: Loro vs automerge-rs** — this choice propagates into every FFI binding and every offline merge path. MUST be committed before any crate implementation begins. Document the decision as an ADR in `docs/decisions/`.
- **Confirm UniFFI version + language coverage** — UniFFI version shifts; confirm Swift + Kotlin + Java coverage before FFI scaffold begins.
- **Confirm flutter_rust_bridge version** — confirm Dart/Flutter support before `frf-ffi` bindings are written.
- **Complete `frf-postgres-cdc`** — WAL logical replication consumer loop is dead code (Phase 1 + Phase 2 HIGH debt). Complete before CDC-sourced entity events can flow end-to-end.

## Primary Goals

- **CRDT engine crate (`frf-crdt`)** — implement `CrdtStore` port with the chosen engine (Loro or automerge-rs); encode/decode binary CRDT operations as `EventEnvelope.payload`; merge function that converges on reconnect
- **Offline op-log crate (`frf-store-redb`)** — implement on-device op-log using redb; persist unsynced ops; replay on reconnect using incremental offset tracking
- **FFI scaffold (`frf-ffi`)** — UniFFI scaffold exposing the Rust core (identity verify, subscribe, publish, CRDT merge) to Swift, Kotlin/Java, and Dart via flutter_rust_bridge
- **Swift SDK** (`sdks/swift/`) — UniFFI-generated Swift bindings over `frf-ffi`; demo app that edits offline and converges on reconnect
- **Kotlin SDK** (`sdks/kotlin/`) — UniFFI-generated Kotlin bindings; Java can consume via the Kotlin binding (no separate Java SDK)
- **Dart SDK** (`sdks/dart/`) — flutter_rust_bridge bindings over the Rust core; Flutter demo page
- **CI codegen pipeline** — wire `buf generate` + `pnpm -r build` + Go `go generate` into Dagger so SDK stubs are always fresh (closes Phase 2 CI gap)

## Exit Criterion (IMPLEMENTATION-PLAN §314)

> Mobile app edits offline, reconnects, converges; identical merge on all three platforms (iOS/macOS via Swift, Android/JVM via Kotlin, Flutter via Dart).

## Non-Goals for Phase 3

- WebRTC / media / SFU (Phase 4)
- Agent protocols / BossFang (Phase 5)
- Federation bridges (Phase 6)
- SDK registry publishing / release automation (Phase 7)
