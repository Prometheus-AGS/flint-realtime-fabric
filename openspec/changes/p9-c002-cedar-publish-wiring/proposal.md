# p9-c002 — Wire Cedar into Publish Route

## Summary

Adds `POLICY_ENGINE` env var, `DynPolicyProvider` type alias, and wires
`CedarPolicyEngine` into the publish handler when `POLICY_ENGINE=cedar`.

## Dependencies

- p9-c001 must be complete (`AppState<P>` slot must exist)

## Files to Modify

- `crates/frf-ports/src/policy.rs` — add `DynPolicyProvider = Arc<dyn ActionPolicyProvider + Send + Sync>`
- `crates/frf-gateway/Cargo.toml` — add `frf-policy-cedar` dep
- `crates/frf-gateway/src/config.rs` — add `PolicyEngineMode` enum + `policy_engine` field + `POLICY_ENGINE` env var
- `crates/frf-gateway/src/main.rs` — construct Cedar or NoOp based on config; update both `AppState` sites
- `crates/frf-gateway/src/routes/publish.rs` — add `is_permitted` check before broker publish; return 403 on deny

## Exit Criteria

- `cargo check --workspace` passes
- `cargo test -p frf-gateway` passes
- `cargo test -p frf-policy-cedar` passes (3/3)
- `POLICY_ENGINE=cedar` construction path compiles
