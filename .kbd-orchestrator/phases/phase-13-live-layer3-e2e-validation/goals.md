# Goals — phase-13-live-layer3-e2e-validation

## G1 — Run Stage 10 live in a DinD-capable environment

Execute `ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts`
and capture the full Stage 10 output. Triage all failures by category
(iggy startup race, CDC timing, Tuwunel dependency, auth, federation).

## G2 — Commit `.wasm-size-baseline`

After the first successful Stage 6 run, measure the WASM binary size and
commit `.wasm-size-baseline` to arm the regression guard in Stage 6.

## G3 — Fix Stage 10 failures

Resolve the concrete failures found in G1. Expected categories based on Phase 12
reflection:
- Iggy startup race → retry logic or healthcheck ordering
- CDC timing → longer poll timeout in smoke-cdc.sh or compose healthcheck
- Tuwunel/federation dependency → skip or stub federation tests in Stage 10 until
  Tuwunel is available in CI
- Auth → ensure Kratos/Oathkeeper is reachable or mocked at the gateway level

## G4 — Add `make baseline-wasm` target

Add a `Makefile` (or `justfile`) with a `baseline-wasm` target that runs Stage 6
and commits the resulting `.wasm-size-baseline`. Document in
`docs/DEVELOPMENT.md`.

## G5 — ESLint rule for env var boolean coercion

Add an ESLint custom rule (or eslint-plugin-unicorn equivalent) to the
`admin-ui/` workspace that flags `!!process.env["X"]` and requires the
`=== "true"` pattern instead. Wire it into the lint step.
