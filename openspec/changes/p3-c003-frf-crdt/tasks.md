# Tasks — p3-c003 frf-crdt

- [ ] **T1** Create `crates/frf-crdt/Cargo.toml`
  - `[package] name = "frf-crdt" edition = "2024"`
  - Dependencies: `frf-ports = { path = "../frf-ports" }`, `loro = { workspace = true }`, `bytes = { workspace = true }`, `thiserror = { workspace = true }`, `tracing = { workspace = true }`, `async-trait = { workspace = true }`
  - `[lints.clippy] pedantic = "warn"`
  - Verification: `cargo check -p frf-crdt` exits 0

- [ ] **T2** Create `crates/frf-crdt/src/error.rs`
  - Define `CrdtError` with `#[non_exhaustive]` and `#[derive(Debug, thiserror::Error)]`
  - Variants: `Decode(String)`, `Merge(String)`, `Store(#[from] PortError)`
  - Verification: file compiles as part of crate

- [ ] **T3** Create `crates/frf-crdt/src/merge.rs`
  - Implement `apply_delta(existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, CrdtError>`
  - Use `loro::LoroDoc` to load `existing`, apply `delta` as an import, export merged state
  - Add `#[tracing::instrument(skip(existing, delta))]` span
  - No `unwrap()` — map Loro errors to `CrdtError::Decode` / `CrdtError::Merge`
  - Verification: unit test `apply_delta_roundtrip` passes: create doc A, set key, export; create doc B with different key, export delta; apply_delta → merged doc has both keys

- [ ] **T4** Create `crates/frf-crdt/src/store.rs`
  - Define `LoroCrdtStore<S>` where `S: CrdtBackend` (a small internal trait for `get/put/delete` bytes by `(EntityId, TenantId)`)
  - Implement `CrdtStore` for `LoroCrdtStore<S>`:
    - `checkpoint`: serialize current Loro doc → bytes → call `inner.put()`
    - `restore`: call `inner.get()` → deserialize Loro doc → return `CrdtSnapshot`
    - `purge`: call `inner.delete()`
  - Add `tracing::instrument` span on each method
  - No `unwrap()` / `expect()`
  - Verification: `cargo check -p frf-crdt` exits 0

- [ ] **T5** Create `crates/frf-crdt/src/lib.rs`
  - `pub mod error; pub mod merge; pub mod store;`
  - `pub use error::CrdtError; pub use merge::apply_delta; pub use store::LoroCrdtStore;`
  - Verification: `cargo doc -p frf-crdt --no-deps` exits 0

- [ ] **T6** Write unit tests in `crates/frf-crdt/src/merge.rs` (inline `#[cfg(test)]`)
  - Test `apply_delta_roundtrip`: two docs diverge offline, delta applied, both keys present
  - Test `apply_empty_delta`: empty delta → original doc unchanged
  - Verification: `cargo test -p frf-crdt` exits 0; 2 tests pass

- [ ] **T7** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
