# p0-c004 — frf-proto (Contract Freeze)

## Affected crates
- `crates/frf-proto` (new)
- `proto/flint/v1/` (new directory + 6 proto files)

## Dependency-rule impact
`frf-proto` depends on `frf-domain` for shared ID/timestamp types used in generated code. It does not depend on `frf-ports` or any adapter. `frf-gateway` and SDK generator pipelines depend on this crate. The git tag `proto-v1` is a hard contract freeze — no edits to v1 files after tagging; breaking changes require a new proto version. **Semver impact:** the tag itself is the semver boundary; frf-proto crate version must stay 0.1.x until v1 is stable and then lock at 1.0.0.

## Phase 0 exit criterion satisfied
_frf-proto compiles; `proto-v1` git tag created._

## What this change does
1. Creates `proto/flint/v1/` with six `.proto` files per RFC-FRF-002 §03:
   - `envelope.proto` — `EventEnvelope`, `Channel`, `Offset`, `Cursor`
   - `entity.proto` — `EntityChange`, entity CRUD events
   - `agent.proto` — `AgentEvent`, AG-UI/A2A/A2UI wire format
   - `signal.proto` — `SignalEnvelope`, WebRTC offer/answer/ICE
   - `sync.proto` — `SyncOp`, CRDT delta wire format
   - `authz.proto` — permission check request/response
2. Creates `crates/frf-proto/Cargo.toml` (workspace member, `[build-dependencies]` = tonic-build; `[dependencies]` = prost + tonic + frf-domain).
3. Writes `crates/frf-proto/build.rs` running `tonic_build::compile_protos` over all six proto files with `protoc_include` paths set.
4. Verifies `cargo build -p frf-proto` compiles cleanly.
5. Creates `git tag proto-v1` after successful compile.

## Non-goals
- Does not generate Go, C#, or TypeScript SDKs (Phase 2).
- Does not generate Swift/Kotlin UniFFI bindings (Phase 3).
- Does not implement any gRPC service handlers (those are in frf-gateway, Phase 1).
- Does not modify `proto/flint/v1/` after tagging.
