# Phase-21 Sign-off — sovereign RTP forwarding

> Date: 2026-07-08 · Closing verification for phase-21 (p21-c005). Extends the phase-20
> sign-off (`docs/PHASE-20-SIGNOFF.md`) with the media path: two-peer connectivity + 1-to-1
> RTP forwarding.

## Gates (re-run at phase close)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo test -p frf-media-str0m --lib` | ✅ 22 passed, 0 ignored |

> Full `cargo test --workspace` runs in CI (str0m + httpmock exceed the local link budget).
> The per-crate lib suites for the crates this phase touched pass locally, plus the standing
> `check`/`clippy` workspace gates.

## What now functions (phase-21)

- **Two-peer DTLS-connected (in-process, real)** — an offerer + answerer complete a real ICE
  connectivity check + DTLS handshake to `Connected` over loopback (p21-c002). The phase-20
  `#[ignore]`d test is now a passing proof; the offerer is a test harness (the `MediaTransport`
  port stays answerer-only).
- **1-to-1 RTP forwarding (wired + layer-proven)** — a central `RoomRouter` + per-session
  forwarding channels (ADR-006, p21-c004): router fan-out unit-tested (to the other member,
  not the sender; deregister), the driver write path (`Event::MediaData` →
  `writer(mid).write`), and the `StrOmTransport` create/join/remove wiring integration-tested.
- **session/driver split** (p21-c001) — `driver.rs` extracted so `session.rs` has headroom.
- **ADR-005 / ADR-006** — the `MediaTransport` port and the RTP fan-out architecture.

## Re-affirmed deferred (documented, gated off — carried to phase-22)

- **End-to-end browser media proof** — two connected peers exchanging *decoded* media through
  the gateway. Browser/negotiation-gated; not done this phase. **This is why
  `SFU_MODE=sovereign` stays gated off** (defaults to `hosted`).
- **N-peer per-room fan-out, PLI/keyframe + renegotiation** — the `RoomRouter` is general, but
  phase-21 proved only the 2-peer/1-to-1 case.
- **Gateway composition** (`StrOmSignaler` + `StrOmTransport`) + the actual `SFU_MODE=sovereign`
  flip — phase-22, once media flows end-to-end.
- **LiveKit cross-node live proof / admin-ui OIDC** — carried (external-infra gated).

See `docs/SECURITY.md` §6 and
`.kbd-orchestrator/phases/phase-22-sovereign-sfu-npeer-pli/goals.md`.

## Process note (honest)

**c004 was archived on a QA-gate BLOCK, then corrected.** The QA gate flagged a clippy
`match_same_arms` in `RoomRouter::forward` (two empty match arms), which my incremental
per-crate clippy had cached past. I archived c004 before reading the gate verdict — a process
miss. I caught it, collapsed the match into an `if let Err(Full)` (identical behavior — RTP
loss-tolerant drop-on-full), and re-ran the gate to a genuine **ALL PASS** (`--workspace
--lib --bins` + `--all-targets` both clean). Lesson recorded for the reflection: read the
QA-gate *verdict*, not just the verify step, before archiving; the gate's clean-workspace
clippy catches what a cached per-crate split misses.

## Sign-off

Phase-21 completes the scoped media-path work — two-peer connectivity + 1-to-1 RTP forwarding
— with no new CRITICAL/HIGH and all gates green. The hosted deployment signed off in
phase-17/18 is unchanged. **`SFU_MODE=sovereign` moves no end-to-end media and stays gated
off**: the engine is present and proven at the unit/integration layer, but the browser
end-to-end proof and N-peer/PLI are deferred to **phase-22-sovereign-sfu-npeer-pli** and must
not be advertised as shipped until they function end-to-end. No plane was advertised beyond
what it does.
