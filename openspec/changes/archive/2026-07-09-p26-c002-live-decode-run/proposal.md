# p26-c002-live-decode-run

## Why

Phase-26 G2 — the proof, first run to reach the SFU. c001 fixed the Dockerfile so the gateway
image builds; this change executes the authenticated decode runner against the live stack and
records the actual `framesDecoded` outcome — the input that decides the flip (c003).

## What Changes

- Execute `scripts/run-media-decode.sh` (authenticated) and record the real result in
  `docs/PHASE-26-DECODE-RESULT.md`. Along the way this hardened the runner + harness against the
  concrete blockers each run surfaced (JWKS-port leak, JWT_ISSUER, flint-gate build stall, Keto
  write path, getUserMedia secure-context) — all fixed so the run reaches the media exchange.
- Recorded outcome: the full pipeline now works up to the media (build→boot→healthy→Keto view
  grant→browser harness), but the WebRTC decode did **not** complete within timeout — a genuine
  media-transport symptom, not plumbing. `framesDecoded > 0` NOT observed → c003 re-affirms gated.

## Impact

- New: `docs/PHASE-26-DECODE-RESULT.md`; hardened `scripts/{run-media-decode,seed-media-view}.sh`,
  `scripts/mint-e2e-jwt.mjs`, `compose.sovereign.yml`, `admin-ui/e2e/media-decode.spec.ts`.
- No production Rust change; no gate flip.
