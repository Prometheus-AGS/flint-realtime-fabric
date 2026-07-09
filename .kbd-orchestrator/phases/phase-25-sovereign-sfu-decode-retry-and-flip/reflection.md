# Reflection — phase-25-sovereign-sfu-decode-retry-and-flip

> Generated 2026-07-08. Made the live decode run *authenticated*, ran it, and honestly failed it
> again — but further, on a precise Dockerfile defect. `SFU_MODE=sovereign` re-affirmed off. The
> blocker sequence is converging, not stuck.

## Delta — movement against phase goals

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — fix runner's live path** | ✅ MET | Authenticated runner: `mint-e2e-jwt.mjs` mints an RS256 JWT + JWKS (**verified to decode against its own JWKS**), `run-media-decode.sh` self-serves the JWKS + overrides `GATEWAY_JWKS_URL` + boots sovereign + seeds `view`. The phase-24 compose-merge fallback is gone. (c001) |
| **G2 — real decoded frame** | ❌ NOT PROVEN | Authenticated run executed; got further (mint/JWKS/compose/admin-ui-build ✓) but failed on a **Dockerfile defect** (`rust-embed` of `admin-ui/dist`, never built/copied) → gateway image won't compile → stack never booted → no `framesDecoded` (`PHASE-25-DECODE-RESULT.md`). (c002) |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet → no flip; `main.rs` warning intact; production defaults hosted. (c003) |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC. |

**1/4 MET (G1); G2/G3 did not close — the proof was run, got closer, and did not pass.**

## Root cause — why G2/G3 did not close

A real, specific **Dockerfile defect**: `frf-gateway` uses `#[derive(RustEmbed)]` on `admin-ui/dist`,
but the Dockerfile's build context is only `Cargo.*`/`crates/`/`proto/` — it never builds or copies
`admin-ui/dist`. So the gateway lib fails to compile *in Docker* (host builds pass because a prior
`vite build` left `dist/` on disk). The stack never booted; the media path was never reached.

## The blocker-clearing arc (why this is convergent progress)

Three attempts, three *distinct, narrowing* blockers — each fixed unlocks the next:
1. **P24:** compose-merge fallback (`undefined service keto`) → fixed.
2. **P25a (c001):** verifier is RS256/JWKS, flint-gate signs HS256 → any flint-gate token rejected
   → fixed (self-mint RS256 + self-serve JWKS, verified to decode).
3. **P25b (c002):** Dockerfile `rust-embed` of unbuilt `admin-ui/dist` → the current blocker.

The genuine media-path unknown (ICE/DTLS/RTP over the `host.docker.internal` UDP candidate) is
**still untested** — the run has never reached a booted gateway.

## Delivered changes (3/3 archived)

1. **c001** — authenticated decode runner (RS256 mint + JWKS + reworked script; HS256/RS256
   incompatibility caught and solved before wiring).
2. **c002** — executed the proof; recorded the real Dockerfile failure with a diagnosed root cause.
3. **c003** — re-affirm gate off + SECURITY §6 + CHANGELOG + PHASE-25-SIGNOFF.

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes | 3/3 archived |
| Final QA verdict | 3/3 ALL PASS |
| Re-runs / archive aborts | 0 |
| Release gate at close | fmt ✅ · clippy --workspace ✅ · check ✅ · str0m 27 · gateway 39 · domain 10 |

### Notable

- **G1's RS256/HS256 catch** — verifying the minted token decodes against its JWKS *before* wiring
  the runner is what turned "authenticated path" from a plan into a working reality; without it the
  run would have failed opaquely at JWT verification instead of the clear Dockerfile error.
- **c002's real negative** — the most valuable artifact: a specific, fixable Dockerfile blocker
  that no code review of the media plane would have found.

## Technical debt introduced

- **None structural.** The Dockerfile defect is *pre-existing* (surfaced, not introduced) and now
  documented with a precise fix. The un-observed decode is a gated deferral.

## Lessons captured

1. **Verify the token before the run.** Minting is cheap; a token the gateway silently rejects
   wastes a full stack boot. Decode-against-own-JWKS caught the RS256/HS256 mismatch immediately.
2. **The Dockerfile build context is a boundary that hides host-only successes.** Host `cargo
   build` passed because `admin-ui/dist` existed locally; Docker didn't — the embed is only exercised
   in a clean context. Assume nothing carries into the image that isn't COPY'd.
3. **Convergent failures are progress.** Three phases, three narrowing blockers, each fixed —
   recording them plainly (not as "still blocked") shows the path is closing, and tells the next
   phase exactly where to start.

## Recommended next phase

**`phase-26-gateway-docker-adminui-embed-and-decode`** — the narrowest remaining finish:

- **Fix the Dockerfile:** add a Node build stage (`pnpm --dir admin-ui install && build`) and
  `COPY --from=<node-stage> /admin-ui/dist ./admin-ui/dist` into the Rust build context (or build on
  host + COPY), so `rust-embed` resolves. Confirm the gateway image builds clean.
- **Re-run `scripts/run-media-decode.sh`** — for the **first time** the stack should boot and the
  harness reach the SFU. Then the real media-path test runs: `framesDecoded > 0` or a diagnosed
  ICE/DTLS/RTP failure.
- **Flip `SFU_MODE=sovereign` only on a real pass**; else re-affirm with the (now media-path) detail.
- Fold in G4 (LiveKit `realtime`; admin-ui OIDC).
