# Tasks — p16-c004

- [x] Add expected issuer to gateway config (env var)
- [x] Validate iss in verifier.rs
- [x] Reject missing/mismatched iss
- [x] Test: correct iss passes; wrong iss rejected

## Notes

### Implementation

`OryIdentityVerifier` gained an `issuer: Option<String>` field and a
`with_issuer(jwks_url, audience, issuer)` constructor alongside the existing
`new(...)` (which leaves issuer unvalidated). In `decode_token`, when an issuer
is configured:

```rust
validation.set_issuer(&[issuer]);
validation.set_required_spec_claims(&["exp", "aud", "iss"]);
```

`set_issuer` alone only validates `iss` **when present**; requiring the claim also
rejects tokens that omit it — the missing-iss test caught this (RED→GREEN).

Gateway config: new optional `jwt_issuer` field, read from `JWT_ISSUER`
(empty string treated as unset). `main.rs` uses `with_issuer` when set, otherwise
constructs the issuer-less verifier and logs a startup `warn!` that iss is NOT being
validated — so an unsecured config is loud, not silent. Optional (not `context()?`
-required) for backward compatibility with existing deployments; G1.4's intent is met
(issuer CAN and by default should be validated), while a hard-require would break
current envs that have no `JWT_ISSUER` set yet.

### Tests (`crates/frf-identity-ory/tests/verifier.rs`)

- `verify_accepts_token_with_matching_issuer` — `with_issuer` + matching `iss` passes.
- `verify_rejects_token_with_wrong_issuer` — mismatched `iss` rejected.
- `verify_rejects_token_with_missing_issuer_when_issuer_required` — absent `iss`
  rejected (this drove the `set_required_spec_claims` fix).

The 4 pre-existing tests still pass because they use `new()` (no issuer required).

### Verification

- `cargo clippy -p frf-identity-ory -p frf-gateway --lib --bins` (default) → exit 0
- `cargo clippy -p frf-gateway --lib --bins --features dev-endpoints` → exit 0
- `cargo test -p frf-identity-ory --test verifier` → 7/7 pass
- `cargo fmt --check` → clean

### Follow-up for docs (G5)

`JWT_ISSUER` must be added to the env-var reference (c022) and flagged as
"recommended in production".
