# Tasks — p19-c005-livekit-inbound-relay-capability

- [x] 1. adapter.rs: extract session fan-out into a reusable `fan_out(&SignalEnvelope)` method; call it from `send_signal`
- [x] 2. inbound.rs (new): `LiveKitDataSource` trait + `spawn_inbound_relay` loop (payload → deserialize → fan_out; skip+warn on malformed); `LiveKitSignaling::start_inbound_relay`
- [x] 3. inbound.rs: unit tests with a mock data source — valid payload forwards to a subscribed session; malformed payload is skipped without panic
- [x] 4. Cargo.toml: add off-by-default `realtime` feature (seam for the libwebrtc-backed source); lib.rs re-exports; update adapter.rs limitation note
- [x] 5. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green for frf-media-livekit
