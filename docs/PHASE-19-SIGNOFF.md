# Phase-19 Sign-off — sovereign media & federation completion

> Date: 2026-07-08 · Closing verification for phase-19 (p19-c007). Extends the phase-18
> sign-off (`docs/PHASE-18-SIGNOFF.md`) with the deferred XL/external-dependency work.

## Gates (re-run at phase close)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ |
| `cargo check --workspace` | ✅ |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ |
| `cargo test -p frf-media-str0m --lib` | ✅ 9 passed |
| `cargo test -p frf-media-livekit --lib` | ✅ 5 passed |
| `cargo test -p frf-bridge-atproto --lib` | ✅ 9 passed |
| `cargo test -p frf-gateway --lib config::` | ✅ 11 passed |
| admin-ui `pnpm lint` / `pnpm typecheck` | ✅ / ✅ |

> Full `cargo test --workspace` exceeds the local link budget (str0m + httpmock pull large
> test-binary trees) — it runs in CI. The per-crate lib suites above (the crates this phase
> touched) pass locally, plus the standing `check`/`clippy` workspace gates.

## What now functions end-to-end (phase-19)

- **admin-ui lint** — `pnpm lint` clean; the `p7-smoke` env guard uses strict `=== "true"`
  (fixing a latent skip bug), and a stale `react-hooks` disable directive was removed.
- **ATProto outbound (gateway-wired)** — the p18-c008 PDS write path is now reachable: the
  gateway builds an outbound-capable bridge when `ATPROTO_PDS_URL` / `ATPROTO_PDS_IDENTIFIER`
  / `ATPROTO_PDS_APP_PASSWORD` are set (all-or-none, enforced at boot; app-password secret).
- **LiveKit inbound-relay capability** — the `LiveKitDataSource` seam + relay loop forward
  server payloads into the per-session streams; unit-tested.
- **str0m live-UDP transport loop** — a `TransportLoop` binds a real socket and turns the
  sans-I/O event loop (`poll_output`→`send_to`, `handle_input`); tested, no browser.

## Re-affirmed deferred (documented, gated off — carried to phase-20)

- **Full str0m sovereign SFU media loop** — trickle ICE, DTLS/SRTP, RTP fan-out, per-session
  async tasks. `SFU_MODE` defaults to `hosted`; **sovereign stays gated off (no media)**.
  Built on the c006 transport-loop proof; scoped in `SPIKE-FINDINGS.md`.
- **LiveKit cross-node live proof** — the libwebrtc-backed data source is behind the
  off-by-default `realtime` feature; the cross-node proof is integration-gated on a live
  LiveKit server (so the default gateway build stays light).
- **admin-ui OIDC login** — decided by **ADR-004** (recommend Kratos + Hydra in compose;
  flint-gate has no `authorize` endpoint). The hardened token gate is the interim; no OIDC
  code until the ADR is Accepted + an IdP is deployed.
- **Dart async transport** — re-affirmed deferred (dated 2026-07-07): `uniffi-bindgen-dart
  0.1.3` still emits broken async codegen; the shim + documented patch remain the interim.

See `docs/SECURITY.md` §6 for the security posture of each item, and
`.kbd-orchestrator/phases/phase-20-sovereign-sfu-media-loop/goals.md` for the seeded
successor phase.

## Sign-off

Phase-19 completes the scoped deferred work with no new CRITICAL/HIGH and all gates green.
The hosted deployment signed off in phase-17/18 is unchanged. The full sovereign SFU media
loop and the live cross-node proofs are deferred to **phase-20-sovereign-sfu-media-loop**
and must not be advertised as shipped until they function end-to-end. No plane was advertised
beyond what it does: `SFU_MODE=sovereign` moves no media and stays gated off.
