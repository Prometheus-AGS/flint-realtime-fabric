# p0-c002 — frf-domain

## Affected crates
- `crates/frf-domain` (new)

## Dependency-rule impact
`frf-domain` sits at the innermost layer. Its `[dependencies]` must contain only `serde` and `serde_json` (plus `uuid`, `chrono` for ID/timestamp newtypes). No I/O, no async, no adapter crates. The compiler enforces this because no other FRF crate is listed as a dependency here. **Semver impact:** every type exported from this crate is load-bearing for `frf-ports`, `frf-proto`, `frf-app`, and all SDKs — treat every public type change as a semver minor or major.

## Phase 0 exit criterion satisfied
_Domain crate compiles; serde round-trip unit tests pass._

## What this change does
1. Creates `crates/frf-domain/Cargo.toml` (workspace member, `[dependencies]` = serde + workspace deps only).
2. Implements all domain types from RFC-FRF-002 §02:
   - Newtypes: `ChannelId`, `EventId`, `CursorId`, `EntityId`, `AgentId`, `SessionId` — all `#[repr(transparent)]` over `uuid::Uuid` or `String`.
   - `EventEnvelope` — stamped, typed envelope for every event on the spine.
   - `Channel`, `Offset`, `Cursor` — spine addressing and read-pointer.
   - `EntityChange` — CDC / CRDT delta for entity management.
   - `AgentEvent` — AG-UI / A2A / A2UI agent protocol envelope.
   - `SyncOp` — CRDT operation for offline sync.
   - `Presence` — user/device presence record.
   - `SignalEnvelope` — WebRTC signaling wrapper.
3. All types derive `Serialize`, `Deserialize`, `Debug`, `Clone`. Public enums get `#[non_exhaustive]`.
4. Module tree capped at 500 lines/file; split into submodules (`envelope.rs`, `entity.rs`, `agent.rs`, `sync.rs`, `presence.rs`, `signal.rs`, `ids.rs`).
5. Unit tests in `tests/serde_roundtrip.rs` covering JSON round-trips for every top-level type.

## Non-goals
- No I/O, tokio, or async anywhere in this crate.
- No port traits (those are p0-c003).
- No proto-generated types (those are p0-c004).
- No business logic or use-cases.
