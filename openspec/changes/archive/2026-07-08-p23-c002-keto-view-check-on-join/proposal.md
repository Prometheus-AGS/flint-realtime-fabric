# p23-c002-keto-view-check-on-join

## Why

ADR-007 (p23-c001) decided the media-path authorization boundary: a per-participant Keto
`check(subject, "view", room)` at RoomJoin, enforced in `MediaTransportBridge` (gateway), never
in the str0m adapter. This change enforces it.

## What Changes

- `MediaTransportBridge` gains an optional `Arc<dyn AuthzProvider>`. When present, on a
  `RoomJoin` envelope it builds a `RelationTuple { tenant_id, subject, relation: "view",
  object: room_id }` and calls `authz.check(&tuple)` **before** `join_room`. Deny (or a check
  error) ⇒ the session is NOT added to room membership and receives no fan-out; the join is
  dropped (logged, not relayed as media).
- The subject is the authenticated, server-assigned `from_session` id (not a caller-supplied
  field) — consistent with ADR-007's "subject derives from the authenticated session."
- The str0m adapter (`frf-media-str0m`) is untouched — no authz dependency (dependency rule +
  one-port-per-adapter).
- `main.rs` wires the existing `KetoAuthzProvider` into the bridge for `SFU_MODE=sovereign`.

## Impact

- `crates/frf-gateway/src/media_bridge.rs` — `authz` field + view-check on RoomJoin.
- `crates/frf-gateway/src/main.rs` — pass authz into the bridge.
- Tests: authorized-join admitted, unauthorized-join rejected (no membership), cross-tenant
  join yields no fan-out.
- No gate flip. `SFU_MODE=sovereign` stays off (media proof is p23-c004).
