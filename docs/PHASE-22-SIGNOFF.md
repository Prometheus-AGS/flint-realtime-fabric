# Phase-22 Sign-off — sovereign SFU N-peer fan-out, PLI & gateway composition

> Date: 2026-07-08 · Closing verification for phase-22 (p22-c004). Extends the phase-21
> sign-off (`docs/PHASE-21-SIGNOFF.md`) with N-peer fan-out, PLI forwarding, and the gateway
> sovereign composition.

## Gates (re-run at phase close)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo test -p frf-media-str0m --lib` | ✅ 24 passed, 0 ignored |
| `cargo test -p frf-gateway --lib` | ✅ 30 passed, 0 ignored |

> Full `cargo test --workspace` runs in CI (str0m + httpmock exceed the local link budget).
> The per-crate lib suites for the crates this phase touched pass locally, plus the standing
> `check`/`clippy` workspace gates.

## What now functions (phase-22)

- **N-peer per-room fan-out (proven)** — `RoomRouter::forward` delivers a frame to all other
  room members; a 3-peer test confirms it (p22-c001).
- **PLI/keyframe forwarding** — a receiver's `Event::KeyframeRequest` is relayed
  receiver→sender via the router (`ForwardedFrame` enum) and applied with
  `Writer::request_keyframe` (p22-c002).
- **Gateway sovereign composition** — for `SFU_MODE=sovereign` the gateway composes
  `StrOmSignaler` + `StrOmTransport` and drives `MediaTransport` from the signal path via
  `MediaTransportBridge` (`Offer`→`create_session`/answer, `RoomJoin`→`join_room`,
  `IceCandidate`→`add_remote_candidate`); unit-tested (p22-c003).

## Re-affirmed deferred (documented, gated off — carried to phase-23)

- **End-to-end browser proof** — two real peers exchanging *decoded* media through the
  sovereign gateway (a Playwright + WebRTC harness). Browser-gated; not done this phase.
  **This is why `SFU_MODE=sovereign` stays gated off** (defaults to `hosted`).
- **Media-path security boundary** — per-event Keto RLS + tenant isolation on media fan-out
  in SECURITY §1–§5 — precondition for the flip (phase-23).
- **`SFU_MODE=sovereign` flip** — phase-23, only once decoded media provably flows.
- **LiveKit cross-node live proof / admin-ui OIDC** — carried (external-infra gated).

See `docs/SECURITY.md` §6 and
`.kbd-orchestrator/phases/phase-23-sovereign-sfu-e2e-and-gate/goals.md`.

## Process note (honest)

**c003 was caught before archiving on a QA-gate BLOCK, then fixed.** The QA gate flagged a
clippy `doc_markdown` in `signal_service.rs` (`Offer→create_session` needed backticks). I read
the *actual* clippy output (exit 101) rather than the task's outer exit code, saw the gate had
**BLOCKED**, and did **not** archive — the phase-21 c004 lesson applied correctly. Fixed the
backticks and re-ran the gate to a genuine **ALL PASS** before archiving. (Two other
compiler-caught issues this phase — an E0596 borrow on `request_keyframe(&mut self)` and a
missing `str0m` gateway dev-dep — were likewise fixed from the actual build output.)

## Sign-off

Phase-22 completes the scoped in-process media work — N-peer fan-out, PLI forwarding, and the
gateway sovereign composition — with no new CRITICAL/HIGH and all gates green. The hosted
deployment signed off in phase-17/18 is unchanged. **`SFU_MODE=sovereign` composes the media
plane but moves no end-to-end media and stays gated off**: the browser E2E proof + the gate
flip are deferred to **phase-23-sovereign-sfu-e2e-and-gate** and must not be advertised as
shipped until they function end-to-end. No plane was advertised beyond what it does.
