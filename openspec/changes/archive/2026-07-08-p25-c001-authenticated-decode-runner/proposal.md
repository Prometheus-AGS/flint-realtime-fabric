# p25-c001-authenticated-decode-runner

## Why

Phase-24's decode run failed because the runner's no-JWT fallback chained a dev override that
*replaces* the gateway `depends_on` and hides infra behind `profiles:[full]` → invalid compose
project. The operator chose the **authenticated path**: use a real flint-gate-minted JWT so the
proof exercises the c003 ADR-007 authenticated-subject authz, not a bypass. This change makes the
runner do that.

## What Changes

- Rework `scripts/run-media-decode.sh`: bring up the sovereign stack **including flint-gate** with
  a clean file set (base + `compose.sovereign.yml`, plus flint-gate's secret), obtain a real
  `E2E_JWT` (determine the mint mechanism — flint-gate's `mint_jwt` is a proxy hook on `/**`; the
  runner may mint by calling through flint-gate, or sign directly with `FLINT_GATE_JWT_SECRET`
  matching flint-gate's issuer/audience), seed `(sub=<minted subject>, view, <room>)`, and run
  `media-decode.spec.ts` with the JWT.
- Remove the broken no-JWT `compose.override.example.yml` fallback path.

## Impact

- `scripts/run-media-decode.sh` (reworked); possibly a small `scripts/mint-e2e-jwt.sh` helper.
- No production code; no gate flip. Makes the authenticated live decode run (c002) reachable.
