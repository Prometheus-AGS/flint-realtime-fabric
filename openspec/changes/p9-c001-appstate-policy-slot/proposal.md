# p9-c001 — `NoOpPolicyProvider` + `AppState<P>` Generic Slot

## Summary

Add `NoOpPolicyProvider` to `frf-ports` and extend `AppState<L,A,I,M,B>` to
`AppState<L,A,I,M,B,P>` with a 6th `P: ActionPolicyProvider` generic slot. Wire
`NoOpPolicyProvider` as the default concrete type at all existing instantiation sites.

This is the structural prerequisite for p9-c002 (Cedar publish wiring).

## Files to Modify

- `crates/frf-ports/src/policy.rs` — add `NoOpPolicyProvider` struct + impl
- `crates/frf-ports/src/lib.rs` — re-export `NoOpPolicyProvider`
- `crates/frf-gateway/src/lib.rs` — `AppState<L,A,I,M,B,P>`, `AppStateArc`, `build_router`
- `crates/frf-gateway/src/routes/publish.rs` — add `P` bound
- `crates/frf-gateway/src/routes/subscribe.rs` — add `P` bound
- `crates/frf-gateway/src/routes/agents.rs` — add `P` bound
- `crates/frf-gateway/src/routes/dev/inject.rs` — add `P` bound
- `crates/frf-gateway/src/grpc_service.rs` — add `P` bound if `AppState` referenced
- `crates/frf-gateway/src/main.rs` — two `AppState` construction sites → add `action_policy`
- `crates/frf-gateway/tests/signal_mux.rs` — `AppState` smoke test → add `P` = `NoOpPolicyProvider`

## Exit Criteria

- `cargo check --workspace` passes
- `cargo test -p frf-gateway -- signal_mux` passes (3/3)
