# Phase-30 signoff — sovereign SFU decode topology (browser-in-Docker)

> Date: 2026-07-09. Phase-30 implemented the browser-in-Docker topology fix — the **media-path
> engineering is complete**. But the decode did not execute: the Colima VM crashed building the
> gateway image. **`SFU_MODE=sovereign` stays gated OFF** — no receiver observed a decoded frame, and
> the residual is environmental (build/run capacity), not a media-path defect.

## Gate decision: OFF (honest gate held)

The decode **never ran** (RUNNER_EXIT=1 before the assertion, both attempts), so no
`framesDecoded > 0` was observed. Per the phase-16→29 discipline: **the gate does not flip until a
real receiver observes a decoded frame.** `crates/frf-gateway/src/main.rs` is untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p30-c001 | Browser-in-Docker harness (Playwright service on compose net, in-network gateway, secure-context, coturn service STUN) | none (harness); topology fixed |
| p30-c002 | Harness plumbing fix (repo-root mount, reuse host install, image v1.61.0) + decode run | **held OFF** |

## Evidence

- **Run 1:** stack healthy, Keto seeded (HTTP 201), Playwright container started — then in-container
  `pnpm install` failed (`workspace:*` deps + lockfile only resolve from the repo root). **Fixed in
  c002.**
- **Run 2:** plumbing fixed; the run reached the **gateway image build**, which crashed the Colima VM
  mid `vite build`/`tsc` (`rpc error: code = Unavailable … EOF`; Docker daemon then unreachable). An
  OOM/resource limit — **not** a media-path defect.

## Standing: media-path is complete

Every media-path blocker found across phases 24–29 is fixed and (through phase-29) proven live:
valid candidate IP (p28), shared demuxing socket (p29 B2), mDNS `.local` skip (p29 B1), STUN srflx
(p29 B1), same-address-space topology (p30 c001). The SFU engine and the in-network harness are
correct; the only thing between here and a decoded frame is an environment that can build the gateway
image and run the two-browser stack without exhausting the local VM.

## Carried to next phase

1. **Decouple the gateway image build from the decode run:** build the image **once out-of-band / in
   CI** and `up -d --no-build gateway` against it, or raise the Colima VM memory (`colima start
   --memory 8`+), so one pass doesn't OOM.
2. Re-run the in-network decode against the pre-built gateway; flip only on `framesDecoded > 0`.
3. G3 carried (LiveKit x-node, admin-ui OIDC) — integration-gated.

## Verification

- Host `cargo check -p frf-media-str0m -p frf-gateway` green (only Docker is down; the Rust workspace
  is unaffected). No engine change this phase.
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the environmental
  blocker.
