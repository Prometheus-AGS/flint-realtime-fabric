# Plan — phase-22-sovereign-sfu-npeer-pli

> Stage: Plan · 2026-07-08 · Backend: **OpenSpec** (`change_backend: openspec`)
> Source: `assessment.md`. Operator decisions locked (below).
> Ordering: **re-scope what's done → codeable in-process work → honest close** — the 16–21
> discipline; `SFU_MODE=sovereign` flips only if media provably flows (it won't this phase).

## Operator decisions (locked this stage)

1. **Browser E2E (G3.2) + gate flip (G4):** **carried.** This phase lands only the codeable
   in-process work; standing up a real-browser WebRTC/DTLS/SRTP + Playwright harness is a large
   separate effort. **`SFU_MODE=sovereign` stays gated off** and G3.2/G4 are re-affirmed
   deferred to a follow-on.
2. **N-peer fan-out (G1):** **test-only** — `RoomRouter::forward` already fans to all co-room
   peers; add a 3+-peer delivery test. Defer RID/simulcast (YAGNI until a real stream needs it).

## Ordered changes

Each is one OpenSpec change, walked one task/turn via `/kbd-apply` from repo root. Spec delta
before the QA gate. QA gate per code change; **read the gate verdict before archiving** (the
c004/phase-21 lesson). Heavy gates run in the background (cold-rebuild + link budget).

| # | Change ID | Goal | Type | QA | Summary |
|---|-----------|------|------|:--:|---------|
| c001 | `p22-c001-npeer-fanout-proof` | G1 | test | yes | N-peer test: 3+ sessions in a room, one sends, all others receive via `RoomRouter`. |
| c002 | `p22-c002-pli-keyframe-forwarding` | G2 | feat | yes | Forward inbound `Event::KeyframeRequest` sender↔receiver via the router; `request_keyframe` on the sender; unit-test the routing. |
| c003 | `p22-c003-gateway-sovereign-composition` | G3.1 | feat | yes | Compose gateway `SFU_MODE=sovereign` = `StrOmSignaler` + `StrOmTransport`; drive `MediaTransport` from the signal path (Offer→create_session, RoomJoin→join_room, IceCandidate→add_remote_candidate, relay local_signals). **Gate stays off (media present, unproven E2E).** |
| c004 | `p22-c004-carry-e2e-and-close` | G3.2/G4/G5/close | docs | skip | Re-affirm G3.2 (browser proof) + G4 (gate flip) + G5 carried; seed a follow-on phase; SECURITY §6 / CHANGELOG / sign-off + gate re-run. |

**4 changes.** c001 is the cheap N-peer proof; c002 wires PLI; c003 is the real gateway
composition (the biggest codeable step); c004 closes honestly and carries the browser proof.

---

## Change detail

### c001 — N-peer fan-out proof (G1) · test · QA:yes
- **Files:** `crates/frf-media-str0m/src/room.rs` (test).
- **Tasks:** an N-peer test — register 3+ sessions in one room with test channels; `forward`
  from one; assert **all others** receive and the sender does not. Confirms the existing
  general fan-out. No fan-out code change.
- **Exit:** N-peer delivery proven; clean gate.

### c002 — PLI / keyframe forwarding (G2) · feat · QA:yes
- **Files:** `crates/frf-media-str0m/src/{room.rs,driver.rs}`.
- **Tasks:** (1) a `ForwardedKeyframeRequest { mid, rid, kind }` + a router path
  `forward_keyframe_request(from, ..)` routing a receiver's request to the room's sender(s).
  (2) driver: on `Event::KeyframeRequest`, route it; on a forwarded request, call
  `writer(mid).request_keyframe(kind)` guarded by `is_request_keyframe_possible`. (3) Unit-test
  the request routing (receiver → sender). Renegotiation stays deferred (browser-gated).
- **Exit:** a keyframe request routes receiver↔sender; unit-tested. No lib `unwrap`/`expect`;
  files ≤500 (split if `room.rs`/`driver.rs` approach it).

### c003 — Gateway sovereign composition (G3.1) · feat · QA:yes
- **Files:** `crates/frf-gateway/src/{main.rs,signal_service.rs}` (+ app state).
- **Tasks:** (1) In `build_media_signaler`/app wiring, for `SFU_MODE=sovereign` build **both**
  `StrOmSignaler` (signaling) and `StrOmTransport` (media) and hold both. (2) Extend the signal
  path (`signal_service.rs`) to drive `MediaTransport`: `SignalKind::Offer`→`create_session`
  (relay the answer), `RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`, and pump
  `local_signals` back out as signal envelopes. `RoomLeave`/disconnect→`remove_session`.
  (3) **Keep the gate OFF**: `SFU_MODE=sovereign` now composes the media plane but the log still
  says "not proven end-to-end — use hosted for production"; do NOT advertise it live.
  (4) Tests: the signal→MediaTransport routing (Offer→create_session called, etc.) with a mock
  or the real `StrOmTransport`.
- **Exit:** the gateway composes signaling + media for sovereign mode and drives the port from
  the signal path; unit-tested; gate stays off. Clean gate. **Security:** JWT verification +
  tenant isolation on the signal path are unchanged (media rides the already-authed channel).

### c004 — carry E2E + close · docs · QA:skip
- **Files:** `.kbd-orchestrator/phases/<follow-on>/` (seed), `docs/SECURITY.md` §6,
  `CHANGELOG.md`, `docs/PHASE-22-SIGNOFF.md` (new).
- **Tasks:** (1) Re-affirm G3.2 (browser E2E proof) + G4 (gate flip) + G5 carried; **seed a
  follow-on phase** for the browser harness + gate flip + live proofs. (2) SECURITY §6: N-peer
  fan-out + PLI forwarding + gateway composition present; **end-to-end browser proof deferred**;
  `SFU_MODE=sovereign` still gated off. (3) CHANGELOG + sign-off; re-run the gate suite.
- **Exit:** phase-22 signed off; follow-on seeded; gate honestly off.

---

## Sequencing & rationale

- **c001 first** — the cheap N-peer proof (code already fans out); fast confidence.
- **c002** — PLI forwarding on the router, mirroring the c004 (phase-21) media path.
- **c003** — the real gateway composition (biggest codeable step); the media plane is now
  *drivable* end-to-end, but the gate stays off pending the browser proof.
- **c004 last** — closes honestly, carries G3.2/G4/G5, seeds the follow-on.

## Phase exit criteria

- N-peer delivery proven (c001); PLI forwarding wired + unit-tested (c002); the gateway composes
  `StrOmSignaler` + `StrOmTransport` and drives the media port from the signal path (c003).
- Release gate suite green; each code change passes the QA gate (**verdict read before archive**).
- **No "healthy but does nothing":** `SFU_MODE=sovereign` composes the media plane but stays
  **gated off** — the browser end-to-end proof + gate flip are honestly carried, not faked.

## First change to apply

`p22-c001-npeer-fanout-proof` — the cheap N-peer delivery proof on the existing router.
