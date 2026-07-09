# Assessment — phase-23-sovereign-sfu-e2e-and-gate

> Generated 2026-07-08. Gap report of the composed sovereign media plane against G1–G5.
> This is the **flip phase**: `SFU_MODE=sovereign` turns on only if G2 proves decoded media
> flows to a real browser peer *and* G3 covers the media-path security boundary; else it stays
> gated with fresh rationale. Discipline carried from phases 16–22.

## Starting state (proven / present)

- **str0m media engine** — `crates/frf-media-str0m/src/{session,driver,room}.rs`: N-peer
  per-room fan-out (`RoomRouter::forward`), PLI/keyframe forwarding, 1-to-1 RTP forwarding, two
  peers DTLS-connected in-process. 24 lib tests. (phases 21–22)
- **Gateway composition** — `crates/frf-gateway/src/media_bridge.rs` + `main.rs`/`signal_service.rs`:
  `MediaTransportBridge` drives `MediaTransport` from the signal path for `SFU_MODE=sovereign`
  (gate off). (p22-c003)
- **Signal channel is already JWT-gated** — `signal_service.rs:228–234`: a request missing a
  Bearer token is rejected `UNAUTHENTICATED` **before any domain logic**. G3's §1 auth boundary
  on the media/signal path is therefore *already enforced* (tested: `missing_bearer_token_produces_unauthenticated`).
- **Tenant identity flows into the media path** — the bridge passes the envelope's verified
  `tenant_id` into `create_session`/`join_room` (`media_bridge.rs:39,55,72,89`), and
  `RoomRouter` keys rooms by `(TenantId, String)` (`room.rs:50,54,71`). **Cross-tenant fan-out
  is structurally impossible** — `fan_out` only reaches members sharing the exact `(tenant,room)`
  key — but this is not yet asserted by a test or documented in SECURITY §1–§5.
- **E2E harness scaffolding exists** — `admin-ui/playwright.config.ts` + `admin-ui/e2e/*.spec.ts`
  (signaling, publish, subscribe, crdt). Reusable Playwright config + fixtures.
- **str0m offerer/spike harnesses** — `rtc_spike.rs`, `transport_spike.rs` — a headless str0m
  peer usable as the *other* end of a browser↔headless media proof (G2).

## Gaps (against G1–G5)

### G1 — Browser E2E test harness  ·  **GAP (partial base)**
- **No WebRTC/media spec** in `admin-ui/e2e/` — existing specs cover signaling/publish/subscribe,
  none drives `RTCPeerConnection` / `getUserMedia`. Need a new spec that connects a real browser
  peer to a running `SFU_MODE=sovereign` gateway and reaches `Connected`.
- **No served WebRTC client page** — the gateway does not yet embed a minimal WebRTC test page;
  the harness needs a page (served or Playwright-injected) that does offer/answer + ICE via the
  gateway signal channel.
- **CI gating** — must run in Dagger (Node + Chromium) behind a browser-available flag
  (`dagger/` exists; codegen.ts present).

### G2 — Decoded-media end-to-end proof  ·  **GAP (the honestly-gated proof)**
- No test yet exercises real ICE + DTLS/SRTP + RTP forwarding + PLI to the point a **receiver
  decodes a frame**. Options: browser↔browser, or browser↔headless-str0m (reuse the spike peer).
  This is the single load-bearing gap that unblocks the gate flip.

### G3 — Security boundary for the media path (before the flip)  ·  **PARTIAL GAP**
- ✅ **§1 auth**: signal/media channel already rejects missing Bearer tokens (done — document it).
- ✅ **Tenant isolation is structural**: `RoomRouter` `(TenantId, room)` keying — but **needs a
  cross-tenant-no-fanout test** and a SECURITY §2/§5 entry.
- ❌ **Per-event Keto RLS on fan-out**: there is **no `check(subject, "view", …)` on the media
  path** (confirmed: no Keto call in `media_bridge.rs`/`signal_service.rs`). Decide + document
  whether media fan-out requires a per-participant Keto `view`/`subscribe` check at room-join
  (subscribe-time, cached — mirroring the event-spine pattern in SECURITY §2) or whether
  room-membership + tenant-keying is the boundary. Must be resolved and written into §1–§5
  **before** G4.

### G4 — Flip `SFU_MODE=sovereign`  ·  **BLOCKED on G2+G3**
- Currently `main.rs:269–284` composes the plane but keeps the honest gate-off warning. The flip
  is a one-line branch change **plus** removing the warning — but only after G2 proves decoded
  media and G3 covers the boundary. Else: re-affirm gated with fresh rationale (no "healthy but
  does nothing").

### G5 — Carried live proofs  ·  **RE-AFFIRM (external-infra gated)**
- **LiveKit cross-node**: `realtime` feature exists (`frf-media-livekit/Cargo.toml:30`) but is
  off-by-default (heavy native WebRTC); cross-node relay is an integration test vs a live server.
- **admin-ui OIDC**: ADR-004 documents flint-gate has **no `/authorize`** — needs Kratos+Hydra
  in compose + a `/callback` path. Blocked on IdP deployment + ADR-004 Accepted.

## Open questions for plan/analyze

1. **G2 topology**: browser↔browser (two Chromium contexts) vs browser↔headless-str0m (reuse
   `rtc_spike`)? The headless-peer path is cheaper to make deterministic in CI and reuses proven
   code — likely the recommended approach.
2. **G3 Keto decision**: is per-participant `view`/`subscribe` at room-join the right media-path
   RLS, or is `(tenant, room)` membership sufficient? This is a **security decision that gates
   G4** — surface to the operator; likely an ADR-007 (media-path authz).
3. **CI browser availability**: does the Dagger runner have Chromium? If not, G1/G2 run locally +
   are documented as locally-proven (honest), not silently skipped.

## Recommendation

Order the plan: **G3 security decision (ADR) → G1 harness → G2 decoded-media proof → G4 flip (or
re-affirm) → G5 re-affirm**. G3's Keto/isolation decision is load-bearing for the flip and should
be settled (and its boundary documented) before or alongside the harness work — the flip must
never precede the documented boundary.
