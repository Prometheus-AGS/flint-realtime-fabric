# p2-c001 — Add Keto write-authz to `PublishUseCase`

## Phase
phase-2-generated-sdks

## Affected crates
- `crates/frf-app` (primary)
- `crates/frf-gateway` (wiring update)

## Dependency-rule impact
Application layer. `PublishUseCase` already imports `AuthzProvider` from
`frf-ports`; this change adds the field and check. No new crate dependencies.

## Security rationale
`PublishUseCase::execute` currently verifies the JWT identity but does NOT
check Keto before calling `broker.publish()`. Any authenticated tenant can
publish to any channel without authorization. This violates the CLAUDE.md
security constraint: *"Per-event RLS: Keto check before every fan-out
delivery."* The publish path needs a symmetric write-permission check.

## What this change does

Adds `AuthzProvider` to `PublishUseCase<L, A, I>` (currently `<L, I>`).
After JWT verification, checks:
```
authz.check(RelationTuple {
    tenant_id: claims.tenant_id,
    subject:   claims.subject,
    relation:  "publish",
    object:    channel_id_from_envelope,
})
```
Returns `AppError::Forbidden` if denied.

Mirrors the pattern in `SubscribePipeline` (frf-app/src/subscribe.rs:51–68).

## Files changed

| File | Change |
|---|---|
| `crates/frf-app/src/publish.rs` | Add `authz: Arc<A>`, `A: AuthzProvider` generic; add Keto check after identity verify |
| `crates/frf-gateway/src/lib.rs` | `AppState<L,A,I>` already has authz; pass to `PublishUseCase::new()` |
| `crates/frf-gateway/src/main.rs` | Pass `Arc::clone(&authz)` to `PublishUseCase::new()` |

## Test changes

- Update 2 existing `PublishUseCase` unit tests (add mock `AuthzProvider` arg)
- Add 1 test: `publish_returns_forbidden_when_authz_denies`

## Exit criteria

- `cargo test -p frf-app` passes (all tests including new forbidden test)
- `cargo check --workspace` exits 0
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic` exits 0
