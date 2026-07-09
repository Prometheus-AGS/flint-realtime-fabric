# p23-c001-media-authz-adr

## Why

The sovereign media path is already JWT-gated at the signal boundary and `RoomRouter` keys rooms
by `(TenantId, room)`, so cross-tenant fan-out is structurally impossible. But there is **no
per-participant authorization** on media fan-out: any authenticated session in the right tenant
can join any room and receive its media. Before `SFU_MODE=sovereign` can flip on (phase-23 G4),
the media-path authorization boundary must be **decided and recorded**.

## What Changes

- Add **ADR-007 (media-path authz)** documenting the operator decision: a per-participant Keto
  `check(subject, "view", room)` at **RoomJoin**, layered on top of the existing JWT gate and
  `(TenantId, room)` isolation. Deny ⇒ the session is not added to room membership and receives
  no fan-out.
- Record the rejected alternative (`(tenant,room)` membership as the sole boundary) and why the
  stronger per-participant check was chosen (parity with the event-spine per-event `view` filter
  in SECURITY §2; defense-in-depth).
- Record **where** enforcement lands: `MediaTransportBridge` in `frf-gateway` (which already
  holds the `KetoAuthzProvider` seam) — **not** in `frf-media-str0m`, preserving the dependency
  rule and one-port-per-adapter.

Doc-only. Enforcement is p23-c002.

## Impact

- New: `docs/decisions/adr-007-media-path-authz.md`.
- No code change. Unblocks c002 (enforcement) and c005 (SECURITY docs).
