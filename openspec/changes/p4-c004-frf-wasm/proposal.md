# p4-c004 — frf-wasm: wasm-bindgen browser SDK

## Phase
phase-4-webrtc-wasm-browser

## Depends on
p3-c003-frf-crdt (CRDT apply_delta function available)
p2-c005-sdk-ts (TypeScript SDK exists as base for Connect-ES client)

## Crates affected
`frf-wasm` (new crate), `Cargo.toml` workspace

## Dependency-rule impact
`frf-wasm` is a new WASM-only interface crate (analogous to `frf-gateway` for the
browser target). It imports `frf-crdt` and generates JS/TS bindings via wasm-bindgen.
It does NOT import adapter crates — it binds to a Connect-ES browser transport for
gRPC calls. Dependency rule: safe (interface layer, no adapter imports).

## Version pins (verify at execution)
- `wasm-bindgen = "0.2.100"`
- `wasm-bindgen-futures = "0.4"`
- `js-sys = "0.3"`
- `web-sys = "0.3"` (features: `WebSocket`, `console`)
- wasm-pack `0.13.1`
- `@connectrpc/connect = "1.6.1"` (npm)
- `@bufbuild/protobuf = "2.3.0"` (npm)

## What this change does

Creates `crates/frf-wasm/` exposing three functions to TypeScript via wasm-bindgen:

1. `subscribe(channel_path: &str, callback: js_sys::Function)` — opens a
   Connect-ES `createBidiStreamingClient` to `SubscribeService`, calls the JS
   callback for each inbound `EventEnvelope`
2. `publish(channel_path: &str, payload: JsValue)` — sends a Connect-ES unary
   call to `PublishService`
3. `crdt_apply_delta(snapshot: &[u8], delta: &[u8]) -> Vec<u8>` — delegates to
   `frf_crdt::apply_delta` (pure Rust, no JS interop needed in the call itself)

Also creates `sdks/ts/frf-wasm/` as the wasm-pack output directory with generated
`.js`, `.d.ts`, and `.wasm` artifacts (gitignored; built by CI).

## Non-goals
- Does not implement WebRTC in-browser media (LiveKit's JS SDK handles that)
- Does not bundle frf-wasm into the admin-ui Vite build (admin-ui consumes from `sdks/ts/`)
- Does not add session reconnection logic (Phase 7)
