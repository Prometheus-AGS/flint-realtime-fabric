# Tasks — p5-c002-frf-agentproto

- [ ] Add `frf-agentproto` to workspace members in root `Cargo.toml`
- [ ] Create `crates/frf-agentproto/Cargo.toml` (lib crate, deps: frf-domain, frf-proto, serde, serde_json, thiserror)
- [ ] Create `crates/frf-agentproto/src/lib.rs` with module declarations and re-exports
- [ ] Create `crates/frf-agentproto/src/content_block.rs` with `ContentBlock` enum (7 named variants + Unknown)
- [ ] Create `crates/frf-agentproto/src/convert.rs` with `TryFrom<frf_proto::fv1::AgentEvent>` for domain type and `From<ContentBlock>` for `serde_json::Value`
- [ ] Create `crates/frf-agentproto/src/error.rs` with `AgentProtoError` using thiserror
- [ ] Unit test: `ContentBlock::TextDelta` round-trips through serde
- [ ] Unit test: unknown `{"type":"future_field"}` deserializes to `Unknown(Value)` without error
- [ ] `cargo check -p frf-agentproto` clean
- [ ] `cargo clippy -p frf-agentproto -- -D warnings -W clippy::pedantic` clean
