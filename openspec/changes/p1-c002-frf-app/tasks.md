# Tasks — p1-c002 frf-app

- [x] **T1** Create `crates/frf-app/src/error.rs`
  - `AppError`: `#[non_exhaustive]` thiserror enum
  - Variants: `Unauthorized(String)`, `Forbidden(String)`, `Broker(#[from] PortError)`, `Identity(PortError)`
  - Verification: `cargo check -p frf-app` exits 0

- [x] **T2** Create `crates/frf-app/src/subscribe.rs`
  - `SubscribeRequest { channel_id: ChannelId, bearer_token: String, from: Offset }`
  - `SubscribePipeline<L, A, I>` struct holding `Arc<L>`, `Arc<A>`, `Arc<I>`
  - `impl<L, A, I> SubscribePipeline<L, A, I>` with `pub async fn execute(&self, req: SubscribeRequest) -> Result<EventStream, AppError>`
  - Pipeline: verify → check "subscribe" relation → broker.subscribe → per-event filter on "view" relation (cache check via AuthzProvider)
  - Verification: `cargo check -p frf-app` exits 0

- [x] **T3** Create `crates/frf-app/src/publish.rs`
  - `PublishRequest { envelope: EventEnvelope, bearer_token: String }`
  - `PublishUseCase<L, I>` struct holding `Arc<L>`, `Arc<I>`
  - `pub async fn execute(&self, req: PublishRequest) -> Result<Offset, AppError>`
  - Pipeline: verify token → broker.publish(envelope) → return offset
  - Verification: `cargo check -p frf-app` exits 0

- [x] **T4** Update `crates/frf-app/src/lib.rs`
  - Export: `pub mod error; pub mod subscribe; pub mod publish;`
  - Re-export: `pub use error::AppError; pub use subscribe::{SubscribePipeline, SubscribeRequest}; pub use publish::{PublishUseCase, PublishRequest};`
  - `#![deny(warnings)]` + `#![warn(clippy::pedantic)]`
  - Verification: `cargo check -p frf-app` exits 0

- [x] **T5** Write unit tests for `SubscribePipeline`
  - File: `crates/frf-app/tests/subscribe_pipeline.rs`
  - Use `mockall::mock!` to generate `MockLogBroker`, `MockAuthzProvider`, `MockIdentityVerifier`
  - Tests:
    - `returns_stream_when_all_checks_pass`
    - `returns_forbidden_when_subscribe_check_fails`
    - `returns_unauthorized_when_token_invalid`
    - `filters_events_where_view_check_fails`
  - Verification: `cargo test -p frf-app` — all 4 tests pass

- [x] **T6** Write unit tests for `PublishUseCase`
  - File: `crates/frf-app/tests/publish_usecase.rs`
  - Tests:
    - `returns_offset_on_success`
    - `returns_unauthorized_when_token_invalid`
    - `propagates_broker_error`
  - Verification: `cargo test -p frf-app` — all 3 tests pass
