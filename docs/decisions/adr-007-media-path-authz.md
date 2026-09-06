# ADR-007: Media-path authorization at room-join

## Status

Accepted — 2026-07-08 (p23-c001)

Gates the `SFU_MODE=sovereign` flip (phase-23 G4). No flip lands until this authorization
boundary is enforced (p23-c002) and documented in `docs/SECURITY.md` §1–§5 (p23-c005).

## Scope and reconciliation — 2026-09-06

This is the generic room-join authorization decision. For ASO protected media,
[ADR-009](adr-009-aso-runtime-integration.md) additionally requires bounded
expiry/revocation and fan-out removal. The room-lifetime cache and hot-path
non-goal below do not waive that requirement; they do not prove ASO readiness.

## Context

The sovereign SFU (`StrOmTransport`, [ADR-005](adr-005-media-transport-port.md)) is composed and
driven from the signal path by `MediaTransportBridge` (p22-c003). Before it can carry real media
in production, the media-path authorization boundary must be decided.

What already holds on the media/signal path today:

- **Authentication (§1).** The gRPC signal service rejects any request missing a Bearer token
  with `UNAUTHENTICATED` **before any domain logic** (`signal_service.rs`). Every media
  participant is an authenticated session with a verified `tenant_id`.
- **Tenant isolation (§2/§5).** `RoomRouter` keys rooms by `(TenantId, String)`
  (`room.rs`), and `MediaTransportBridge` threads the **verified** `tenant_id` from the signal
  envelope into `create_session`/`join_room`. `fan_out` can only reach members sharing the exact
  `(tenant, room)` key — **cross-tenant fan-out is structurally impossible.**

What is **missing**: there is **no per-participant authorization** on media fan-out. Any
authenticated session in the correct tenant can join any room in that tenant and receive its
media. Tenant isolation is not the same as per-room visibility — a tenant may contain rooms a
given subject must not see.

This mirrors the gap the event spine already closed: SECURITY §2 filters every delivered event
by a Keto `check(subject, "view", object)`. The media path has no equivalent.

## Decision

**Enforce a per-participant Keto `check(subject, "view", room)` at RoomJoin**, layered on top of
the existing JWT gate and `(TenantId, room)` isolation.

```
RoomJoin (authenticated signal envelope)
  → MediaTransportBridge
      → authz.check(subject, "view", room_id)     // NEW — per-participant
          deny  → reject join; session NOT added to membership; receives no fan-out
          allow → StrOmTransport.join_room(session, tenant, room)  // existing
```

- The **subject** derives from the authenticated session (the verified identity behind
  `from_session`), never from caller-supplied envelope fields.
- The check is at **room-join** (subscribe-time), not per RTP packet — one check per participant
  per room, cached for the room's lifetime. This mirrors the SECURITY §2 performance note
  (cache `view` at subscribe time to bound per-event latency) and avoids a Keto round-trip on
  the hot RTP path.
- **Deny is silent to other participants**: a rejected join simply never enters membership; no
  media leaves the gateway to that session.

### Enforcement location — the bridge, not the adapter

The check lands in **`MediaTransportBridge` in `frf-gateway`**, which already composes the
`KetoAuthzProvider` seam. It does **not** land in `frf-media-str0m`.

Rationale (the Absolute Dependency Rule + one-port-per-adapter):

- `StrOmTransport` implements `MediaTransport` as a pure media engine. The crate
  separately contains `StrOmSignaler` for signaling. Neither may import an authz
  crate or reach across to `frf-authz-keto`.
- Composition of ports (media × authz) happens **only** in `frf-gateway`. The bridge is that
  composition point.
- The str0m adapter stays testable in isolation with no authz dependency; the authz policy stays
  swappable without touching the media engine.

## Rejected alternative

**`(TenantId, room)` membership as the sole boundary** (no per-participant Keto check).

Rejected because tenant-scoping is coarser than per-room visibility: it would let any
authenticated member of a tenant join any of that tenant's rooms and receive media, even rooms
they have no relation to. That is weaker than the read boundary the event spine already enforces
(SECURITY §2 per-event `view`), and it would create an inconsistent, weaker rule for media than
for events. Defense-in-depth and parity with the spine win.

## Consequences

- **Positive:** media visibility matches the Zanzibar `view` model already used for events;
  one consistent read boundary across events and media; no authz code in the media adapter.
- **Cost:** one Keto `check` per participant per room-join (cached), plus a `AuthzProvider` field
  on `MediaTransportBridge`. Negligible vs the per-event alternative.
- **Follow-on:** p23-c002 enforces this (bridge gains `Arc<dyn AuthzProvider>`; unauthorized-join
  + cross-tenant-no-fanout tests). p23-c005 writes it into SECURITY §1–§5. p23-c006 may then flip
  the gate — **only** once decoded media is also proven (p23-c004).
- **Non-goal:** per-RTP-packet authorization. The hot path is unauthenticated *after* an
  authorized join, exactly as subscribe-time authorization works for the event spine.

## Related

- [ADR-005](adr-005-media-transport-port.md) — the `MediaTransport` port this authorizes.
- [ADR-006](adr-006-rtp-fanout.md) — the `RoomRouter` fan-out this gates entry to.
- `docs/SECURITY.md` §2 — the event-spine per-event `view` filter this achieves parity with.

## Implementation clarification — 2026-09-06

`frf-media-str0m` exports two distinct adapter types: `StrOmSignaler` implements
`MediaSignaler` (`src/sfu.rs`), and `StrOmTransport` implements `MediaTransport`
(`src/session.rs`). Neither type combines the contracts. Housing both in one
crate is an existing deviation from CLAUDE.md's crate-level one-port rule.
This clarification records that discrepancy without changing the rule or code.
