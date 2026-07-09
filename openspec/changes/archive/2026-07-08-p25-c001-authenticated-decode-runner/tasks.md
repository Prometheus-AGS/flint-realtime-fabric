# Tasks — p25-c001-authenticated-decode-runner

- [x] 1. Determine + implement the JWT mint mechanism (flint-gate mint_jwt proxy hook, or direct HS256 with FLINT_GATE_JWT_SECRET matching issuer=flint-gate-dev/audience) as `scripts/mint-e2e-jwt.sh` (or inline). Verify a minted token is accepted by the gateway's verifier.
- [x] 2. Rework `scripts/run-media-decode.sh`: base + compose.sovereign.yml (+ flint-gate), mint E2E_JWT, seed (sub, view, room), run the harness with the JWT. Drop the broken override fallback. shellcheck-clean. QA gate.
