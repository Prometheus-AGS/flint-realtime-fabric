# Phase-25 Sign-off — sovereign SFU authenticated decode retry & the gate decision

> Date: 2026-07-08 · Closing verification for phase-25 (p25-c003). Made the live decode run
> *authenticated* (a real gateway-accepted JWT), ran it, and reached the honest decision:
> `SFU_MODE=sovereign` **stays off** — the run got further than phase-24 but did not pass.

## Gates (re-run at phase close — actual results)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ (exit 0) |
| `cargo check --workspace` | ✅ (exit 0) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ (exit 0) |
| `cargo test -p frf-media-str0m --lib` | ✅ 27 passed |
| `cargo test -p frf-gateway --lib` | ✅ 39 passed |
| `cargo test -p frf-domain` | ✅ 10 passed |

> Phase-25 is a scripts/harness/docs phase (no new Rust beyond phase-24's `MediaConfig`), so the
> host test counts are unchanged from the phase-24 close — the workspace stays green.

## Goal status (honest)

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — fix the runner's live path** | ✅ MET | The runner now authenticates with a real RS256 JWT the gateway accepts: `mint-e2e-jwt.mjs` mints it + emits a JWKS (**verified to decode against its own JWKS**); `run-media-decode.sh` self-serves the JWKS, overrides `GATEWAY_JWKS_URL`, brings up the sovereign stack, seeds the `view` grant. The phase-24 compose-merge fallback is gone. (p25-c001) |
| **G2 — real decoded frame** | ❌ NOT PROVEN | The authenticated run was **executed** and got further (mint ✓, JWKS ✓, compose ✓, admin-ui vite build ✓) but failed on a **Dockerfile defect**: `frf-gateway` `rust-embed`s `admin-ui/dist`, which the Dockerfile never builds/copies → the gateway image won't compile → the stack never booted → **no `framesDecoded > 0`** (`docs/PHASE-25-DECODE-RESULT.md`). |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet → no flip; `main.rs` gate-off warning intact; production defaults hosted. |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC (ADR-004 + IdP). |

## The blocker-clearing arc (why this is real progress despite the ❌)

Each attempt clears the previous blocker and surfaces the next — the failures are *specific and
converging*, not a stuck loop:

1. **Phase-24:** compose-merge in the no-JWT fallback (`undefined service keto`) → **fixed** (dropped the broken fallback).
2. **Phase-25a (c001):** the gateway verifier is **RS256/JWKS**, flint-gate signs **HS256** → a flint-gate token would be rejected → **fixed** (self-mint RS256 + self-serve JWKS, verified to decode).
3. **Phase-25b (c002):** **Dockerfile defect** — `admin-ui/dist` not built/copied before the Rust embed → **the current blocker.**

The genuine media-path unknown — whether Chromium completes ICE/DTLS/RTP over the
`host.docker.internal` UDP candidate and decodes a frame — is **still untested**, because the run
has never reached a booted gateway.

## The gate decision — stated plainly

**`SFU_MODE=sovereign` stays off.** The proof was run, got materially further, and did not observe
a decoded frame. Enabling the gate now would advertise a plane that has never moved one — the
failure this project has refused for ten phases.

**To make it pass (next phase):** fix the Dockerfile (add a Node build stage `pnpm --dir admin-ui
build` + `COPY admin-ui/dist` into the Rust build context), re-run `scripts/run-media-decode.sh`,
and — for the first time — actually test the media path: observe `framesDecoded > 0` (or diagnose
the ICE/DTLS/RTP failure). Only then does the gate flip.

## Process note (honest)

- **G1 caught a real config incompatibility** (RS256-verifier vs. HS256-signer) that would have
  silently failed any flint-gate token — solved by minting RS256 + serving a JWKS, **verified**
  before wiring the runner.
- **c002 ran the proof for real and it failed** — recorded in `PHASE-25-DECODE-RESULT.md` with a
  diagnosed Dockerfile root cause, not dressed up.
- Both carried QA lessons applied every change (verdict + archive output read); **zero re-runs,
  zero archive aborts** this phase.

## Sign-off

Phase-25 made the decode run authenticated and honest, cleared two real blockers, ran the proof,
and recorded a third concrete blocker (a Dockerfile defect) — so **`SFU_MODE=sovereign` is
re-affirmed off** with a precise next step. No new CRITICAL/HIGH; all gates green. Hosted (LiveKit)
remains the media path. No plane was advertised beyond what it does.
