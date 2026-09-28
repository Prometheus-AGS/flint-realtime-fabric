# Tasks

- [x] Add typed `frf.routed-observer/1` source, route, delivery, projection, and causal contracts in `crates/frf-domain/src/routed_observer.rs`; export them from `crates/frf-domain/src/lib.rs`. This is an additive public Rust API.
- [x] Validate routed candidates at Iggy publish and replay boundaries in `crates/frf-broker-iggy/src/broker.rs` without changing the `LogBroker` port or legacy envelopes.
- [x] Report unsupported runtime cancellation from `crates/frf-gateway/src/agent_grpc_service.rs` without changing frozen `proto/flint/v1/agent.proto`.
- [x] Pin SurrealDB 3.3.0 in `Cargo.toml` and align `Cargo.lock` without running a build.
- [ ] At the completed **C08** production boundary, run one local composed integration gate through authenticated gateway and Iggy with BossFang, Gate, and UAR: profile survives protobuf and restart replay, two observer cursor identities stay separate, revoked grant is withheld, repeated source/action does not cause duplicate execution, and runtime cancel remains unsupported. Record exact provider revisions and results; no CI test run.
