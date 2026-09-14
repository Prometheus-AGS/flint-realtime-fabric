# Tasks — p3-c006 sync-use-case

- [ ] **T1** Add `ApplyDelta` trait to `crates/frf-ports/src/crdt_store.rs` (minor port extension)
  - ```rust
    pub trait ApplyDelta: Send + Sync + 'static {
        fn apply(&self, existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, PortError>;
    }
    ```
  - This keeps `frf-app` free of `frf-crdt` imports
  - Verification: `cargo check -p frf-ports` exits 0

- [ ] **T2** Create `crates/frf-app/src/sync.rs`
  - Define `SyncUseCase<C: CrdtStore, O: OpStore>` with injected `apply_delta: Arc<dyn ApplyDelta>`
  - Implement `apply_incoming`, `flush_pending`, `confirm_synced`
  - `apply_incoming`:
    1. `crdt_store.restore(entity_id, tenant_id).await?` → get current `encoded`
    2. If None → treat as empty doc (first write)
    3. Call `apply_delta.apply(existing, delta)?` → `merged: Vec<u8>`
    4. `crdt_store.checkpoint(CrdtSnapshot { encoded: merged.into(), ... }).await?`
    5. Return merged bytes
  - `#[tracing::instrument(skip(self, delta))]` on each method
  - No `unwrap()` / `expect()`
  - Verification: `cargo check -p frf-app` exits 0

- [ ] **T3** Re-export from `crates/frf-app/src/lib.rs`
  - `pub mod sync;`
  - `pub use sync::SyncUseCase;`
  - Verification: `cargo doc -p frf-app --no-deps` exits 0

- [ ] **T4** Write unit tests in `crates/frf-app/src/sync.rs` (`#[cfg(test)]`)
  - Use in-memory mock implementations of `CrdtStore`, `OpStore`, `ApplyDelta`
  - Test `apply_incoming_empty_store`: no prior snapshot → returns delta bytes as-is
  - Test `apply_incoming_existing`: prior snapshot exists → merged bytes returned; checkpoint called
  - Test `confirm_synced_delegates`: calls `op_store.mark_synced` with correct args
  - Verification: `cargo test -p frf-app` exits 0; 3 tests pass

- [ ] **T5** Clippy + workspace check
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
