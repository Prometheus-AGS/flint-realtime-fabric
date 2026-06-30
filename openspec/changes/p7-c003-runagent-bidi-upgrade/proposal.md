# p7-c003 — `RunAgent` Proto Bidi Upgrade + Handler

## Summary

Upgrade `proto/flint/v1/agent.proto` from server-streaming to true
bidirectional streaming for `RunAgent`. Add `AgentRunControl` message.
Update `agent_grpc_service.rs` to handle inbound client control messages.

## Motivation

The current server-streaming shape does not allow the browser to cancel or
pause a running agent. True bidi is required for agent lifecycle control.

## Changes

### `proto/flint/v1/agent.proto`

Change:
```proto
rpc RunAgent(AgentRunRequest) returns (stream AgentEvent);
```
To:
```proto
rpc RunAgent(stream AgentRunRequest) returns (stream AgentEvent);
```

Add message:
```proto
message AgentRunControl {
  bool cancel = 1;
  bool pause  = 2;
  bool resume = 3;
}
```

Update `AgentRunRequest`:
```proto
message AgentRunRequest {
  oneof payload {
    AgentRunStart  start   = 1;
    AgentRunControl control = 2;
  }
}

message AgentRunStart {
  string agent_id   = 1;
  string tenant_id  = 2;
  string session_id = 3;
  google.protobuf.Struct metadata = 4;
}
```

### `crates/frf-gateway/src/agent_grpc_service.rs`

Update `run_agent` handler signature from:
```rust
async fn run_agent(&self, request: Request<ProtoAgentRunRequest>) -> Result<Response<Self::RunAgentStream>, Status>
```
To:
```rust
async fn run_agent(&self, request: Request<Streaming<ProtoAgentRunRequest>>) -> Result<Response<Self::RunAgentStream>, Status>
```

Handler logic:
1. Extract first message from incoming stream → must be `AgentRunStart` variant; fail with `InvalidArgument` if not
2. Extract JWT from metadata, verify via `IdentityVerifier`, perform subscribe-time Keto check (same as current)
3. Open subscriber on `agent_bus.subscribe(tenant_id)` filtered by `agent_id` + `session_id`
4. Spawn background task reading incoming stream: on `AgentRunControl { cancel: true }` → drop subscriber, close stream
5. Return merged stream: events from subscriber, terminated on cancel or bus close

### `crates/frf-agentproto/src/convert.rs`

Add:
```rust
pub fn start_from_proto(req: ProtoAgentRunStart) -> Result<AgentRunRequest, AgentProtoError> { ... }
```

## Quality Gates

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
- Proto regeneration via `cargo build -p frf-proto` produces updated tonic server trait
