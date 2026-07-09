# Reflection — phase-22-sovereign-sfu-npeer-pli

> Generated 2026-07-08. Phase built on phase-21's async media engine (`session/driver/room`)
> + `RoomRouter` 1-to-1 forwarding + ADR-005/006. Closes the *in-process* sovereign media
> plane; the external browser proof + gate flip carry to phase-23.

## Goal achievement

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — N-peer per-room fan-out** | ✅ MET | `RoomRouter::forward` fans a frame to all *other* members; a 3-session test proves N-peer delivery (sender excluded, both other members receive). p22-c001. |
| **G2 — PLI / keyframe forwarding** | ✅ MET | A receiver's `Event::KeyframeRequest` is routed receiver→sender via `ForwardedFrame::KeyframeRequest` and applied with `Writer::request_keyframe`; routing test confirms reverse-direction delivery. p22-c002. |
| **G3 — gateway sovereign composition** | ✅ MET | `MediaTransportBridge` drives `MediaTransport` from the signal path (`Offer`→`create_session`+answer, `RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`); `main.rs` composes `StrOmTransport` + bridge for `SFU_MODE=sovereign`; 3 bridge tests with a real transport. p22-c003. |
| **G4 — end-to-end browser media proof** | ⏳ DEFERRED (honest) | Browser-gated; a real peer exchanging *decoded* media through the sovereign gateway is not provable in this harness. Carried to phase-23 as its G1/G2. **This is why `SFU_MODE=sovereign` stays gated off.** |
| **G5 — flip `SFU_MODE=sovereign`** | ⏳ DEFERRED (honest) | Gated on G4 + the media-path security boundary (per-event Keto RLS + tenant isolation). Carried to phase-23 (G3/G4). |

**Goal completion: 3/5 MET (60%), 2/5 honestly deferred with fresh rationale.** The deferred
two are the *external* proof + the production flip — exactly the "no healthy but does nothing"
line the phase set for itself. Nothing was advertised beyond what it does.

## Delivered changes

1. **p22-c001** — N-peer per-room fan-out proof (3-session test).
2. **p22-c002** — PLI/keyframe forwarding (receiver→sender via `RoomRouter`).
3. **p22-c003** — gateway sovereign composition (`MediaTransportBridge`, `main.rs`/`signal_service.rs` wiring).
4. **p22-c004** — carry E2E + close: seeded phase-23, updated SECURITY §6 + CHANGELOG, wrote `docs/PHASE-22-SIGNOFF.md`, re-ran the release gate.

All 4 archived (`openspec/changes/archive/2026-07-08-p22-c00{1..4}-*`).

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes with QA (code changes) | 3/3 |
| Final-pass QA verdict | 3/3 ALL PASS |
| Changes requiring a re-run before archive | 1 (c003) |
| Doc-only change (QA N/A by rule: <3 files / docs) | 1 (c004) |

### Constraint notes

- No recurring constraint *violations* across the phase — every archived change's final QA log
  is ALL PASS (R4 fmt, R5 ≤500 lines, S1 no-secret, P1 openspec validate, plus clippy/check).
- **c003 required one re-run:** the QA gate BLOCKED on a clippy `doc_markdown`
  (`Offer→create_session` needed backticks in a `signal_service.rs` doc comment). It was
  **caught before archiving** — the phase-21 c004 lesson held — fixed, and re-run to ALL PASS.

## Technical debt introduced

- **None structural.** The media plane is composed but the RTP forwarding path is proven only
  at the layer/integration level, not against a real browser — that's the *known, documented*
  gap G4/G5 carry forward, not hidden debt.
- Local test runs are per-crate lib suites (str0m 24, gateway 30); full `cargo test --workspace`
  runs in CI (str0m + httpmock exceed the local link budget). Documented in the sign-off.

## Lessons captured

1. **Read the QA-gate *verdict*, not just the verify step, before archiving.** Applied
   consistently this phase — the clean-workspace `--workspace` clippy in the gate caught c003's
   `doc_markdown` that a cached per-crate clippy would miss. This is now muscle memory across
   phases 21→22.
2. **`Writer::request_keyframe(&mut self)` self-guards** — a redundant
   `is_request_keyframe_possible` pre-check causes an E0596 borrow conflict. Use
   `let Some(mut writer) = …` and act on the returned error, don't pre-guard.
3. **A crate that only *tests* against another crate still needs it as a dev-dependency** —
   `frf-gateway`'s media_bridge test needed `str0m` added as a dev-dep (E0432/E0433 otherwise).
4. **Honest deferral scales** — carrying the browser proof + flip as an explicitly-seeded next
   phase (with SECURITY §6 naming *why* the gate is off) keeps the plane honest without stalling
   forward motion. Seven phases of this discipline; it holds.

## Recommended next phase

**`phase-23-sovereign-sfu-e2e-and-gate`** (already seeded by p22-c004). Focus:

- **G1/G2** — a Playwright + WebRTC browser harness that connects a real peer to the sovereign
  gateway, reaches `Connected`, and **decodes** a relayed frame (the honestly-gated proof).
- **G3** — cover the media path in SECURITY §1–§5: per-event Keto RLS on fan-out where required
  + verify `RoomRouter`'s `(TenantId, room)` keying admits no cross-tenant fan-out.
- **G4** — flip `SFU_MODE=sovereign` **only** once G2 proves decoded media flows and G3 covers
  the boundary; else re-affirm gated with fresh rationale.
- **G5** — carried live proofs (LiveKit cross-node `realtime` feature; admin-ui OIDC once ADR-004
  Accepted + IdP deployed).
