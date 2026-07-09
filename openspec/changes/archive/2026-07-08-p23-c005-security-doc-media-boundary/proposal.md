# p23-c005-security-doc-media-boundary

## Why

ADR-007 + p23-c002 enforce a per-participant Keto `view` check on media room-join, and the
signal/media channel is already JWT-gated with `(TenantId, room)` isolation. The security model
(`docs/SECURITY.md` §1–§5) does not yet describe the media path. Per the phase discipline —
"extend §1–§5 to cover a plane's boundary before enabling it" — document it now (before any
future flip).

## What Changes

- `docs/SECURITY.md` §1: note the `/ws/v1/signal` + gRPC signal channels are JWT-gated (the WS
  route resolves tenant from the verified `token`; missing/invalid ⇒ rejected).
- §2: add the media-path read boundary — per-participant Keto `check(subject,"view",room)` at
  room-join (ADR-007), layered on `(TenantId,room)` tenant isolation; cross-tenant fan-out is
  structurally impossible.
- §5 trust-boundary table: add the media rows (unauthenticated→authenticated; cross-tenant
  media; unauthorized room-join).
- §6: reflect that the media path's boundary is now documented + enforced, while the gate stays
  off pending the in-env decoded-media proof (G2, deferred).

## Impact

- `docs/SECURITY.md` §1/§2/§5/§6. Doc-only. No gate flip.
