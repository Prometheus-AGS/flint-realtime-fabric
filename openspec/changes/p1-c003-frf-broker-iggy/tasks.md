# Tasks — p1-c003 frf-broker-iggy

- [x] **T1** Audit GQAdonis Iggy fork vs Apache upstream
  - Check `https://github.com/GQAdonis/iggy` — is the fork tracking Apache `iggy v0.9.0`?
  - Confirm `IggyClient::from_connection_string` is available in the fork
  - If fork is stale: either update `Cargo.toml` to point to Apache upstream or pin to a known-good commit SHA
  - Verification: `cargo check -p frf-broker-iggy` with the Iggy dep resolves without errors

- [x] **T2** Create `crates/frf-broker-iggy/src/error.rs`
  - `IggyBrokerError`: thiserror enum wrapping Iggy client errors
  - `impl From<IggyBrokerError> for PortError` — maps to `PortError::Transport` or `PortError::NotFound`
  - Verification: `cargo check -p frf-broker-iggy` exits 0

- [x] **T3** Create `crates/frf-broker-iggy/src/channel.rs`
  - `stream_name(tenant_id: &TenantId) -> String`
  - `topic_name(path: &str) -> String` — normalizes the channel path for Iggy topic naming rules (no `/` in topic names; replace with `_` or use a canonical encoding)
  - `partition_id(consumer_id: &str) -> u32` — consistent hash (`fnv` or `std::hash`)
  - Unit tests: canonical round-trip, no panics on empty string
  - Verification: `cargo test -p frf-broker-iggy -- channel` passes

- [x] **T4** Create `crates/frf-broker-iggy/src/broker.rs`
  - `pub struct IggyBroker { client: Arc<IggyClient> }`
  - `impl IggyBroker { pub async fn new(connection_string: &str) -> anyhow::Result<Self> }`
  - `#[async_trait] impl LogBroker for IggyBroker` — all 5 methods
  - `subscribe`: spawn `tokio::task` with bounded `mpsc::channel(256)`; task polls `consumer.next()` in loop; returns `ReceiverStream` wrapped in `EventStream`
  - `ensure_channel`: call create_stream + create_topic; treat "already exists" as Ok
  - `#[tracing::instrument(name = "port::LogBroker::<method>")]` on each impl method
  - Verification: `cargo check -p frf-broker-iggy` exits 0

- [x] **T5** Update `crates/frf-broker-iggy/src/lib.rs`
  - `pub mod broker; pub mod channel; pub mod error;`
  - `pub use broker::IggyBroker;`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-broker-iggy` exits 0

- [x] **T6** Write unit tests for channel mapping
  - File: `crates/frf-broker-iggy/tests/channel_mapping.rs`
  - Tests: `tenant_maps_to_stream`, `path_encodes_without_slash`, `consumer_partition_is_stable`
  - Verification: `cargo test -p frf-broker-iggy -- channel_mapping` passes

- [x] **T7** Write integration test (requires local Iggy instance)
  - File: `crates/frf-broker-iggy/tests/publish_subscribe.rs`
  - Mark with `#[ignore]` so CI doesn't require a live Iggy server by default
  - Test: `publish_then_subscribe_receives_message` — publishes one `EventEnvelope`, subscribes from `Offset::BEGINNING`, asserts the received message matches
  - Comment: `// Run with: cargo test -p frf-broker-iggy -- --ignored`
  - Verification: test compiles; manually runnable against local Iggy
