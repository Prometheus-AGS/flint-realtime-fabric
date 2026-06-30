# Tasks — p4-c004 frf-wasm

- [ ] **T1** Scaffold `crates/frf-wasm/` crate
  - Files: `Cargo.toml`, `src/lib.rs`, `src/subscribe.rs`, `src/publish.rs`, `src/crdt.rs`
  - `Cargo.toml`:
    ```toml
    [lib]
    crate-type = ["cdylib", "rlib"]

    [dependencies]
    wasm-bindgen = "0.2.100"
    wasm-bindgen-futures = "0.4"
    js-sys = "0.3"
    web-sys = { version = "0.3", features = ["WebSocket", "console"] }
    frf-crdt = { path = "../frf-crdt" }
    serde = { workspace = true }
    serde_json = { workspace = true }
    ```
  - Add to workspace `Cargo.toml` members: `"crates/frf-wasm"`
  - Verification: `cargo check -p frf-wasm --target wasm32-unknown-unknown` exits 0

- [ ] **T2** Implement `crdt_apply_delta` WASM binding
  - File: `crates/frf-wasm/src/crdt.rs`
  - ```rust
    #[wasm_bindgen]
    pub fn crdt_apply_delta(snapshot: &[u8], delta: &[u8]) -> Vec<u8> {
        frf_crdt::apply_delta(snapshot, delta)
    }
    ```
  - Re-export from `lib.rs`
  - Verification: `cargo check -p frf-wasm --target wasm32-unknown-unknown` exits 0

- [ ] **T3** Implement `publish` WASM binding
  - File: `crates/frf-wasm/src/publish.rs`
  - `#[wasm_bindgen] pub async fn publish(endpoint: &str, channel_path: &str, payload: JsValue) -> Result<(), JsValue>`
  - Serialize `payload` → JSON → send via `web_sys::WebSocket` or fetch (Connect-ES unary)
  - Return `Err(JsValue)` on network failure (no panic, no unwrap)
  - Verification: `cargo check -p frf-wasm --target wasm32-unknown-unknown` exits 0

- [ ] **T4** Implement `subscribe` WASM binding
  - File: `crates/frf-wasm/src/subscribe.rs`
  - `#[wasm_bindgen] pub async fn subscribe(endpoint: &str, channel_path: &str, callback: js_sys::Function) -> Result<(), JsValue>`
  - Open `WebSocket` to `{endpoint}/flint.v1.SubscribeService/Subscribe`
  - Deserialize each inbound frame, call `callback.call1(&JsValue::NULL, &frame_jsvalue)`
  - Verification: `cargo check -p frf-wasm --target wasm32-unknown-unknown` exits 0

- [ ] **T5** Add wasm-pack build script
  - File: `crates/frf-wasm/build_wasm.sh`
  - `wasm-pack build --target web --out-dir ../../../sdks/ts/frf-wasm crates/frf-wasm`
  - Verification: script is executable (`chmod +x`)

- [ ] **T6** Gitignore wasm-pack output
  - File: `sdks/ts/.gitignore` (create if absent)
  - Ignore: `frf-wasm/`, `*.wasm`, `package-lock.json`
  - Verification: file exists

- [ ] **T7** Add wasm32 smoke test (native)
  - File: `crates/frf-wasm/tests/crdt_binding.rs`
  - Test `crdt_apply_delta_roundtrips`: create a Loro snapshot via `frf-crdt`
    helpers, apply a delta, assert the merged value is correct (same test logic
    as p3-c013 but called through the WASM binding function)
  - NOTE: Run with `cargo test -p frf-wasm` (not wasm target) to avoid needing
    a browser; the binding function itself is pure Rust with no JS deps
  - Verification: `cargo test -p frf-wasm` exits 0
