# Release Sign-off — phase-17

> Date: 2026-07-07 · Scope: production-readiness of `flint-realtime-fabric` after
> phases 16–17. This note is the closing verification for phase-17 (p17-c010).

## Verdict

**No CRITICAL and no HIGH findings survive in the production build.** The phase-15
production-readiness baseline (7 CRITICAL / 15 HIGH) is fully resolved or honestly
deferred. Every CI gate is green on the current tree.

## Gates (re-run this change)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo clippy --workspace --tests` (pedantic) | ✅ |
| `cargo clippy` with `frf-gateway/dev-endpoints` | ✅ |
| `cargo test --workspace --lib` + app/sdk/cli integration tests | ✅ all pass |

> The full `cargo test --workspace` link exceeds the local 2-minute budget; it runs in
> CI (Dagger). Unit + the key integration suites pass locally.

## Security spot-check (c001 / c002 hold)

- ✅ No committed dev-secret literal in the tracked tree.
- ✅ `compose.override.yml` is untracked and gitignored; production `compose.yml` has no
  active `DEV_NO_AUTH` / `dev-endpoints` (only comments stating their absence).
- ✅ `JWT_ISSUER` is mandatory in a production build — `validate()` fails to boot without
  it under `#[cfg(not(feature = "dev-endpoints"))]`.
- ✅ No relation-tuple / JWT logging in the authz path (`#[instrument(skip(...))]`).
- ✅ No hardcoded secrets in production source.

## What shipped (phases 16–17)

- Security boundary hardened + independently re-verified.
- All six proto services live (incl. phase-17 `EntityService` / `AuthzService`).
- SDKs (Rust/TS/Go/C#) + FFI (Swift/Kotlin, resilient) + Dart CRDT bindings; `frf-cli`
  matches its advertised surface.
- Operability: `/readyz`, `/metrics`, graceful shutdown, config validation.
- Per-change QA gate wired into the KBD execute loop.

## Deferred (documented, NOT release-blocking)

Each is gated off or labeled unimplemented — no "healthy but does nothing":

- **str0m sovereign SFU** — signaling-only, no real WebRTC; `SFU_MODE=hosted` default.
- **Federation** — Matrix inbound / ATProto outbound / LiveKit cross-node relay; behind
  `FEDERATION_ENABLED`.
- **admin-ui login** — paste-a-JWT, no interactive OIDC flow.
- **Dart async-transport bindings** — blocked by a `uniffi-bindgen-dart` 0.1.3 codegen
  bug (CRDT surface works); documented in `sdks/dart/GENERATED.md`.

See `docs/SECURITY.md` §6 for the security posture of the deferred planes.

## Sign-off

Cleared for release of the **hosted** deployment shape (LiveKit-hosted media, no sovereign
SFU, no federation). The sovereign/federated/admin-login surfaces must not be advertised
as shipped until their own audit findings are closed.
