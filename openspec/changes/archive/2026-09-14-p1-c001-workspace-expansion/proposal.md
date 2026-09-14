# p1-c001 — Workspace Expansion

## Affected files
- Root `Cargo.toml`

## What this change does

1. Adds five new crates to `[workspace.members]`:
   - `crates/frf-app`
   - `crates/frf-broker-iggy`
   - `crates/frf-authz-keto`
   - `crates/frf-identity-ory`
   - `crates/frf-postgres-cdc`

2. Adds new shared dependencies to `[workspace.dependencies]`:
   ```toml
   async-stream    = "0.3"
   futures-util    = "0.3"
   reqwest         = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
   jsonwebtoken    = "9"
   tokio-postgres  = { version = "0.7", features = ["with-serde_json-1"] }
   async-trait     = "0.1"
   mockall         = "0.13"
   httpmock        = "0.7"
   ```

3. Creates stub `Cargo.toml` for each new crate (minimal — just enough for `cargo check --workspace` to pass). Actual `src/lib.rs` implementations are created by p1-c002 through p1-c007.

## Dependency-rule impact
This is a scaffolding change. No business logic is touched. All new crates will have empty `src/lib.rs` files after this change — they are added to the workspace manifest only so subsequent changes can fill them in and `cargo check --workspace` can always be verified.

## Phase 1 exit criterion satisfied
_Workspace compiles with 5 new members_ — `cargo check --workspace` passes after this change with stub crates.

## Non-goals
- Does not implement any port traits.
- Does not add any use-case logic.
- Does not modify `frf-gateway`.
