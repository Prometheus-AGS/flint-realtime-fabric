# p7-c002 — `frf-wasm` Browser SDK Depth + Build Script

## Summary

Expand `crates/frf-wasm` from a skeleton (`crdt_apply_delta` only) to a
functional browser SDK: add JWT Bearer token support to `publish` and
`subscribe`, add `AgentStream` module for Connect-ES server-streaming,
and write `build_wasm.sh`.

## Motivation

The admin-UI and browser clients cannot authenticate or stream agent events
via WASM today. This change wires the full client surface the browser needs.

## Changes

### `crates/frf-wasm/src/publish.rs`
- Add `token: Option<String>` parameter to `PublishClient::new()`
- Add `Authorization: Bearer {token}` header to fetch `RequestInit` when token is Some
- Add `credentials: RequestCredentials::SameOrigin` to fetch config

### `crates/frf-wasm/src/subscribe.rs`
- Add `token: Option<String>` field to `SubscribeClient`
- Pass `Authorization: Bearer {token}` as WebSocket subprotocol or URL query param
  (use query param: `?token=...` — WebSocket protocol header is not universally supported)
- Add exponential backoff retry: initial delay 500ms, max 30s, max 10 retries
- Emit `ReadyState` events to JavaScript callback on connect/disconnect/retry

### `crates/frf-wasm/src/agent.rs` (NEW)
```rust
// AgentStream — Connect-ES server-streaming fetch for RunAgent gRPC.
// Uses the standard Connect binary protocol (POST /flint.v1.AgentService/RunAgent).
// Response body is a ReadableStream of length-prefixed protobuf frames.
#[wasm_bindgen]
pub struct AgentStream { ... }

#[wasm_bindgen]
impl AgentStream {
    pub fn new(gateway_url: &str, token: &str) -> AgentStream { ... }
    // Returns JS Promise<Void>; invokes on_event callback for each AgentEvent frame.
    pub fn open(&self, request_json: &str, on_event: js_sys::Function) -> js_sys::Promise { ... }
    pub fn close(&self) { ... }
}
```

### `crates/frf-wasm/src/lib.rs`
- Add `pub mod agent;`
- Re-export `AgentStream`

### `crates/frf-wasm/Cargo.toml`
- Add web-sys features: `"Request"`, `"RequestInit"`, `"RequestCredentials"`, `"Headers"`, `"Response"`, `"ReadableStream"`, `"ReadableStreamDefaultReader"`
- Add `frf-proto = { path = "../frf-proto" }` (for frame encoding)
- Add `prost = { workspace = true }` (length-prefix framing)

### `crates/frf-wasm/build_wasm.sh` (NEW)
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
wasm-pack build --target web --out-dir "../../sdks/ts/frf-wasm" --out-name frf_wasm
echo "[frf-wasm] Build complete → sdks/ts/frf-wasm/"
```

### `admin-ui/src/types/frf-wasm.d.ts`
- Delete this file — it will be replaced by wasm-pack's generated `.d.ts`

## Quality Gates

- `cargo check -p frf-wasm --target wasm32-unknown-unknown`
- `cargo clippy -p frf-wasm --target wasm32-unknown-unknown -- -D warnings -W clippy::pedantic`
- `chmod +x crates/frf-wasm/build_wasm.sh` + verify script runs (requires wasm-pack installed)
