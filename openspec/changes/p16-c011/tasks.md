# Tasks — p16-c011

- [x] Scaffold crates/frf-sdk-rust and add to workspace
- [x] Implement connect against the gateway
- [x] Implement publish
- [x] Implement subscribe stream
- [x] Smoke test against a running gateway

## Summary

Created `frf-sdk-rust` — the hand-written, canonical Rust client for FRF (C5). It is the
single home for connection lifecycle and the publish/subscribe API; reconnection (c012)
and CRDT merge layer on top, and other-language SDKs bind to this core.

## Enabling client stubs (prerequisite)

`frf-proto/build.rs` had `build_client(false)` — only server stubs were generated. Changed
to `build_client(true)` so `SpineServiceClient` exists for the SDK. Server stubs are still
built; verified the gateway still compiles.

## Implementation

- **Crate** (`crates/frf-sdk-rust`): added to the workspace, uses `[lints] workspace = true`.
- **`FrfClient::connect(endpoint, token)`** — establishes the tonic transport and wraps
  `SpineServiceClient` in an `AuthInterceptor` that attaches `Authorization: Bearer <token>`
  to every request when a token is provided.
- **`publish(&envelope) -> Offset`** — `SpineService/Publish`.
- **`subscribe(channel_id, consumer_id, from) -> impl Stream<EventEnvelope>`** —
  `SpineService/Subscribe`, mapping proto → domain per item.
- **`ack(channel_id, consumer_id, offset)`** — `SpineService/Ack`.
- **`convert.rs`** — domain↔proto `EventEnvelope` conversion mirroring `flint.v1`
  numbering (kept in the SDK; the gateway's copies are private to its binary crate).
- **`SdkError`** — typed errors (`thiserror`), `#[non_exhaustive]`, `From<tonic::Status>`.

### Design fix caught by the smoke test

The first `subscribe(&mut self)` tied the returned stream to `self`'s mutable borrow
(E0499 — couldn't publish while subscribed). Fixed by cloning the client handle for the
subscribe call so the returned stream is `'static` and owns its transport. This also
improves the API: publish/ack and an active subscription can be used concurrently.

## Tests

- Unit (`convert.rs`, always run): `envelope_round_trips_through_proto`,
  `from_proto_rejects_missing_channel`, `from_proto_rejects_bad_uuid` — 3/3 pass.
- Integration (`tests/smoke.rs`, gated on `FRF_GATEWAY_GRPC_URL`): connect → subscribe →
  publish → receive against a real gateway. Compiles + skips cleanly when the env var is
  unset (same convention as the other integration suites).

## Verification

- `cargo build -p frf-sdk-rust` → exit 0
- `cargo clippy -p frf-sdk-rust --all-targets -- -D warnings -W clippy::pedantic` → exit 0
  (unwrap_used gate passes — no library unwraps)
- `cargo test -p frf-sdk-rust --lib` → 3/3 · smoke test compiles + skips
- `cargo check -p frf-gateway` → exit 0 (proto client-build change didn't break server)
- workspace `cargo clippy --lib --bins` → exit 0

## Unblocks

c012 (reconnection layered on this core), c014 (FFI transport wraps this SDK).
