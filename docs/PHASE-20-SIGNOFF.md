# Phase-20 Sign-off — sovereign SFU media loop

> Date: 2026-07-08 · Closing verification for phase-20 (p20-c006). Extends the phase-19
> sign-off (`docs/PHASE-19-SIGNOFF.md`) with the sovereign SFU media-plane work up to the
> DTLS-connected milestone.

## Gates (re-run at phase close)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo test -p frf-media-str0m --lib` | ✅ 18 passed, 1 ignored (integration-gated) |
| `cargo test -p frf-ports --lib` | ✅ (trait crate; compiles) |

> Full `cargo test --workspace` runs in CI (str0m + httpmock exceed the local link budget).
> The per-crate lib suites for the crates this phase touched pass locally, plus the standing
> `check`/`clippy` workspace gates.

## What now functions (phase-20)

- **`MediaTransport` port (ADR-005)** — a new port for the sovereign media engine, separate
  from the signaling-only `MediaSignaler`; str0m implements two distinct ports.
- **Async per-session str0m engine** (`StrOmTransport`) — a per-session tokio task drives an
  `Rtc` over a tokio `UdpSocket` (the sans-I/O loop), proven by a test.
- **Trickle ICE** — inbound candidate envelopes feed the session; local host candidate +
  state changes relay outbound via `local_signals`.
- **DTLS-connected milestone** — the engine installs a crypto provider and exposes
  `wait_for_connected`; the wait mechanic + crypto install are proven (a peerless session
  times out without a fake `Connected`).

## Re-affirmed deferred (documented, gated off — carried to phase-21)

- **RTP forwarding** — `Event::MediaData` relay, per-room fan-out, PLI, and the offerer/peer
  role. Reaching *connected* is **not** media flowing; `SFU_MODE=sovereign` stays gated off
  (defaults to `hosted`).
- **Two-peer DTLS-connected proof** — integration-gated (`#[ignore]`); needs an offerer role
  / real peer.
- **LiveKit cross-node live proof** — behind the off-by-default `realtime` feature; needs a
  live server.
- **admin-ui OIDC login** — ADR-004; needs an IdP deployed. Token gate is the interim.

See `docs/SECURITY.md` §6 for each item's posture, and
`.kbd-orchestrator/phases/phase-21-sovereign-rtp-forwarding/goals.md` for the seeded
successor phase.

## Sign-off

Phase-20 completes the scoped sovereign-media work to the DTLS-connected milestone with no
new CRITICAL/HIGH and all gates green. The hosted deployment signed off in phase-17/18 is
unchanged. RTP forwarding, the two-peer/browser proofs, and the live cross-node/OIDC proofs
are deferred to **phase-21-sovereign-rtp-forwarding** and must not be advertised as shipped
until they function end-to-end. `SFU_MODE=sovereign` moves no media and stays gated off — the
phase reaches *connected*, not *media forwarded*. No plane was advertised beyond what it does.
