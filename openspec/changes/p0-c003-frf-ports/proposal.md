# p0-c003 — frf-ports

## Affected crates
- `crates/frf-ports` (new)
- `crates/frf-domain` (read-only dependency)

## Dependency-rule impact
`frf-ports` depends on `frf-domain` (inward only). No adapter crates appear here. `frf-app` will depend on `frf-ports`; adapter crates will implement its traits. The compiler prevents any adapter from being imported into this crate. **Semver impact:** every trait signature change is a semver major for all downstream adapter crates — treat signatures as frozen once p0-c004 is tagged.

## Phase 0 exit criterion satisfied
_frf-ports compiles; frf-app can be wired against it without adapter crates present._

## What this change does
1. Creates `crates/frf-ports/Cargo.toml` (workspace member, `[dependencies]` = `frf-domain` + `async-trait` + `thiserror` + `bytes` + `tracing`).
2. Defines six async trait seams from RFC-FRF-002 §02:
   - `LogBroker` — publish/subscribe/seek on the event spine (Iggy behind this).
   - `AuthzProvider` — Zanzibar-style permission check (Keto behind this).
   - `IdentityVerifier` — JWT/OIDC token verification (Kratos/Oathkeeper behind this).
   - `CrdtStore` — checkpoint / restore CRDT state (SurrealDB behind this).
   - `MediaSignaler` — WebRTC offer/answer/ICE relay (str0m / LiveKit behind this).
   - `FederationBridge` — send/receive federated events (Tuwunel/Tranquil behind this).
3. Each trait has an associated `Error` type bounded by `std::error::Error + Send + Sync + 'static` (thiserror-compatible).
4. `tracing::instrument` attributes on every trait method (span name = `port::<TraitName>::<method>`).
5. Port error enums defined in `crates/frf-ports/src/error.rs`.
6. Module tree: one file per port (`log_broker.rs`, `authz.rs`, `identity.rs`, `crdt_store.rs`, `media.rs`, `federation.rs`), each ≤ 500 lines.

## Non-goals
- No implementations — traits only.
- No `frf-app` use-case logic.
- No dependency on any adapter crate.
- No proto-generated types (frf-proto is a separate crate).
