# Tasks — p2-c001 publish-authz-fix

- [ ] **T1** Update `crates/frf-app/src/publish.rs`
  - Add `authz: Arc<A>` field to `PublishUseCase<L, A, I>`
  - Add `A: AuthzProvider` bound to `impl` block
  - Update `new(broker, authz, identity)` constructor
  - After `identity.verify()`, extract `channel_id` from `req.envelope.channel.id`
  - Call `authz.check(RelationTuple { tenant_id, subject, relation: "publish", object: channel_id })`
  - Return `AppError::Forbidden(...)` if `!allowed`
  - Verification: `cargo check -p frf-app` exits 0

- [ ] **T2** Update `PublishUseCase` unit tests in `crates/frf-app/src/publish.rs`
  - Update `#[cfg(test)]` mock setup: add `MockAuthzProvider` arg to `PublishUseCase::new()`
  - Ensure existing 2 tests pass with `authz.expect_check().returning(|_| Ok(true))`
  - Add test `publish_returns_forbidden_when_authz_denies`: mock returns `Ok(false)`, assert `Err(AppError::Forbidden(_))`
  - Verification: `cargo test -p frf-app` — all 3 tests pass

- [ ] **T3** Update `crates/frf-gateway/src/lib.rs`
  - `PublishUseCase::new(Arc::clone(&broker), Arc::clone(&authz), Arc::clone(&identity))` in `build_router`
  - Verification: `cargo check -p frf-gateway` exits 0

- [ ] **T4** Update `crates/frf-gateway/src/main.rs`
  - Pass `Arc::clone(&authz)` as second arg to `PublishUseCase::new()`
  - Verification: `cargo check --workspace` exits 0

- [ ] **T5** Full CI gate
  - `cargo fmt --check --all` ✅
  - `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` ✅
  - `cargo test --workspace` ✅ (all non-ignored tests pass)
