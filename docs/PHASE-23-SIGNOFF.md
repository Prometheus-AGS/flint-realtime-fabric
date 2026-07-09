# Phase-23 Sign-off — sovereign SFU browser E2E, media authz & the gate decision

> Date: 2026-07-08 · Closing verification for phase-23 (p23-c006). This is the phase that was
> to flip `SFU_MODE=sovereign` on — **if** decoded media provably flows end-to-end. It does not
> (in this environment), so the gate is **re-affirmed off**, honestly.

## Gates (re-run at phase close — actual results)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ (exit 0) |
| `cargo check --workspace` | ✅ (exit 0) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ (exit 0) |
| `cargo test -p frf-media-str0m --lib` | ✅ 24 passed, 0 failed, 0 ignored |
| `cargo test -p frf-gateway --lib` | ✅ 38 passed, 0 failed, 0 ignored |

> Gateway lib tests rose 30 → 38: the +8 are ADR-007 bridge authz tests (p23-c002) and the
> `/ws/v1/signal` inbound-path tests (p23-c003). Full `cargo test --workspace` runs in CI.

## Goal status (honest)

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — browser E2E harness** | ✅ MET (authored) | `admin-ui/e2e/media-webrtc.spec.ts` + `support/webrtc-client.ts`; a real Chromium `RTCPeerConnection` negotiates over `/ws/v1/signal` and reaches `Connected`. Integration-gated (`SKIP_INTEGRATION`/`GATEWAY_URL`). |
| **G2 — decoded-media proof** | ◐ **CAPABLE, NOT PROVEN HERE** | `media-decode.spec.ts` + `support/decode-probe.ts` assert `getStats().framesDecoded > 0` — the correct metric. **str0m is sans-codec** (forwards RTP; the decode is browser-side), so this needs a live sovereign gateway + Chromium fake-media. This headless CI env has neither → the test is `test.skip`-gated and **did not run here**. |
| **G3 — media security boundary** | ✅ MET | JWT on both signal transports (§1); ADR-007 per-participant Keto `check(subject,"view",room)` at room-join, fail-closed (p23-c002); `(TenantId,room)` structural isolation. Documented in SECURITY §1/§2 Layer C/§5 (p23-c005). |
| **G4 — flip `SFU_MODE=sovereign`** | ⛔ **RE-AFFIRMED OFF** | The flip's one precondition (an observed decoded frame, G2) is not met in-env. Production `config::from_env` defaults to hosted (`"sovereign"`→Sovereign, else→Hosted); the sovereign branch keeps its honest gate-off warning. **No code flip.** |
| **G5 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit cross-node `realtime` feature; admin-ui OIDC (ADR-004 + IdP). External-infra gated. |

## What now functions (phase-23)

- **Media path is authz-gated** — a per-participant Keto `view` check gates room-join,
  fail-closed, composed in the gateway bridge (str0m stays authz-free). ADR-007 + p23-c002.
- **A browser can drive the sovereign SFU over its natural transport** — `/ws/v1/signal` reads
  inbound offers, drives `MediaTransportBridge`, and relays the SDP answer back; one bridge is
  shared by the gRPC + WS paths via `AppState`. p23-c003.
- **The decode proof is authored with the correct metric** and honestly gated. p23-c004.
- **The media boundary is documented** in the security model before any flip. p23-c005.

## The gate decision — stated plainly

**`SFU_MODE=sovereign` stays off.** Everything *around* the flip is done — composed, layer-proven,
authz-gated, browser-drivable, documented. The flip itself waits on the one thing that cannot be
faked: a real receiver observing decoded media relayed by a live sovereign gateway. That proof is
browser/infra-gated and has not run in this environment. Enabling the gate now would be advertising
a plane beyond what has been observed to work — the exact failure this project has refused for
eight phases. Hosted (LiveKit) remains the media path.

**To flip in a real environment:** stand up a gateway with `SFU_MODE=sovereign`, set `GATEWAY_URL`
+ `SKIP_INTEGRATION=false`, run `admin-ui/e2e/media-decode.spec.ts` with Chromium fake-media, and
confirm `framesDecoded > 0`. Then flip the `main.rs` sovereign branch and update SECURITY §6.

## Process note (honest)

Three moments this phase where the discipline held rather than papered over:

1. **c002 QA-gate BLOCK** on `R4 fmt` — caught by reading the verdict, fixed, re-run to ALL PASS
   before archive (the phase-21 lesson, fourth application).
2. **c003 truthfulness blocker** — the `/ws/v1/signal` route was outbound-only, so a naïve
   "reaches Connected" spec would have been a fabricated pass. Surfaced to the operator, scope
   widened to add the real inbound path.
3. **c004 fundamental finding** — str0m is sans-codec, so a Rust "decoded frame" assertion is
   impossible; surfaced, and the proof was placed browser-side with the correct `framesDecoded`
   metric, honestly gated.
4. **c005 archive abort** — a spec delta used `MODIFIED` on a non-existent header; openspec
   correctly aborted, caught by reading the output, fixed to `ADDED`, re-archived.

## Sign-off

Phase-23 completes the sovereign media plane's authorization, browser transport, and security
documentation, and reaches the honest gate decision: **`SFU_MODE=sovereign` is re-affirmed off**
pending an in-environment decoded-media proof. No new CRITICAL/HIGH; all gates green. The hosted
deployment (phase-17/18) is unchanged. No plane was advertised beyond what it does.
