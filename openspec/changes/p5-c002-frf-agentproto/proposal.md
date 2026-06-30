# p5-c002 — frf-agentproto crate

## Summary

Create the `frf-agentproto` crate: typed `ContentBlock` variants for AG-UI,
A2A, and A2UI payloads, plus conversions between `frf-proto` generated types
and `frf-domain` types.

## Motivation

`frf-domain::AgentEvent` currently stores `content: serde_json::Value` — an
untyped blob. `frf-agentproto` introduces typed `ContentBlock` variants so
that the gateway, the admin UI, and SDK consumers can work with structured
payloads without parsing JSON manually.

## Design

```rust
// frf-agentproto/src/content_block.rs

use serde::{Deserialize, Serialize};

/// Typed content payload for an agent event.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    TextDelta { delta: String },
    ToolCall { tool_name: String, input: serde_json::Value },
    ToolResult { tool_name: String, output: serde_json::Value, is_error: bool },
    StateSnapshot { state: serde_json::Value },
    RunStart { model: Option<String> },
    RunEnd { stop_reason: Option<String> },
    Error { message: String, code: Option<String> },
    Unknown(serde_json::Value),
}
```

Conversions:
- `impl TryFrom<frf_proto::fv1::AgentEvent> for frf_domain::AgentEvent` — parse
  `content: Struct` into typed `ContentBlock`, fall back to `Unknown` on parse error.
- `impl From<ContentBlock> for serde_json::Value` — lossless round-trip.

## Files Changed

- `crates/frf-agentproto/Cargo.toml` — NEW crate, deps: `frf-domain`, `frf-proto`, `serde`, `serde_json`, `thiserror`
- `crates/frf-agentproto/src/lib.rs` — re-exports
- `crates/frf-agentproto/src/content_block.rs` — `ContentBlock` enum
- `crates/frf-agentproto/src/convert.rs` — proto ↔ domain conversions
- `Cargo.toml` (workspace) — add `frf-agentproto` member

## Acceptance Criteria

- [ ] `cargo check -p frf-agentproto` clean
- [ ] `ContentBlock` round-trips through serde (unit test)
- [ ] `Unknown` variant absorbs unrecognized content without panicking
- [ ] No `unwrap()` — `thiserror` for error type
- [ ] `clippy::pedantic` passes
