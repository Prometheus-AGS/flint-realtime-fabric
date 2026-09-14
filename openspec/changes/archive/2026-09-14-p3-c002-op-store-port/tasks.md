# Tasks — p3-c002 op-store-port

- [ ] **T1** Add `bytes` to `frf-ports` Cargo.toml if not already present
  - Check `crates/frf-ports/Cargo.toml` for existing `bytes` dep
  - If missing: add `bytes = { workspace = true }` (already in workspace deps from p0)
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T2** Create `crates/frf-ports/src/op_store.rs`
  - Define `PendingOp` struct with `#[non_exhaustive]`
  - Define `OpStore` trait with `queue_op`, `drain_pending`, `mark_synced`
  - All methods are `async` using `async_trait` macro (already in frf-ports)
  - Return type is `Result<_, PortError>` — use the existing `PortError` from `crates/frf-ports/src/lib.rs`
  - No `unwrap()` or `expect()` anywhere
  - Verification: file compiles (`cargo check -p frf-ports`)

- [ ] **T3** Re-export from `crates/frf-ports/src/lib.rs`
  - Add `pub mod op_store;`
  - Add `pub use op_store::{OpStore, PendingOp};` to public surface
  - Verification: `cargo check -p frf-ports` exits 0; `cargo doc -p frf-ports --no-deps` exits 0

- [ ] **T4** Verify no downstream breakage
  - `cargo check --workspace` exits 0
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
