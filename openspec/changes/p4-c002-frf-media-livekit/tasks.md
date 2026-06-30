# Tasks — p4-c002 frf-media-livekit

- [ ] **T1** Add `MediaSignaling` trait to `frf-ports`
  - File: `crates/frf-ports/src/media.rs` (new file)
  - Define `MediaSignaling` trait with `join_room`, `leave_room`, `relay_signal`
  - All methods `async fn`, return `Result<(), PortError>`, `#[async_trait]`
  - Re-export from `crates/frf-ports/src/lib.rs`
  - Verification: `cargo check -p frf-ports` exits 0; no new deps in frf-ports

- [ ] **T2** Create `crates/frf-media-livekit/` scaffold
  - Files: `Cargo.toml`, `src/lib.rs`, `src/config.rs`, `src/adapter.rs`
  - `Cargo.toml`: deps = `frf-ports`, `frf-domain`, `frf-proto`, `livekit`,
    `tokio`, `thiserror`, `tracing`, `async-trait`
  - `lib.rs`: `pub use adapter::LiveKitSignaling;`
  - `config.rs`: `LiveKitConfig { api_key, api_secret, server_url, room_prefix }`
    sourced from env vars `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`,
    `LIVEKIT_SERVER_URL`, `LIVEKIT_ROOM_PREFIX`
  - Verification: `cargo check -p frf-media-livekit` exits 0

- [ ] **T3** Implement `LiveKitSignaling` adapter
  - File: `crates/frf-media-livekit/src/adapter.rs`
  - `struct LiveKitSignaling { config: LiveKitConfig, client: livekit::RoomServiceClient }`
  - Implement `MediaSignaling` for `LiveKitSignaling`:
    - `join_room`: call `RoomServiceClient::create_room` with tenant-namespaced ID
    - `leave_room`: call `RoomServiceClient::delete_room` if empty
    - `relay_signal`: call `RoomServiceClient::send_data` with serialized payload
  - Error type: `#[non_exhaustive] LiveKitError` implementing `Into<PortError>`
  - No unwrap()/expect() — use `?` with thiserror
  - `#[instrument]` tracing spans on all public methods
  - Verification: `cargo clippy -p frf-media-livekit -- -D warnings -W clippy::pedantic` exits 0

- [ ] **T4** Add `frf-media-livekit` to workspace
  - File: `Cargo.toml` (workspace root)
  - Append `"crates/frf-media-livekit"` to `members` array
  - Add `livekit = "0.4"` to `[workspace.dependencies]` (confirm current version at execution)
  - Verification: `cargo check --workspace` exits 0

- [ ] **T5** Unit tests for `LiveKitSignaling`
  - File: `crates/frf-media-livekit/src/adapter.rs` (inline `#[cfg(test)]`)
  - Test `tenant_namespaced_room_id`: assert room ID contains tenant UUID prefix
  - Test `relay_signal_maps_envelope_fields`: mock `RoomServiceClient`, assert
    `send_data` called with correct room and payload bytes
  - Use `mockall` for the LiveKit client mock
  - Verification: `cargo test -p frf-media-livekit` exits 0
