# p10-c001 — WASM Package Workspace Wiring

## Phase
phase-10-e2e-layer2-wasm-federation

## Summary

Build `frf-wasm` via `wasm-pack`; wire `sdks/ts/frf-wasm/` as a named pnpm
workspace package `"frf-wasm"`; fix the hardcoded relative import in
`agentGrpcStream.ts` to use the workspace alias.

## Files to Create/Modify

- `sdks/ts/frf-wasm/` — built by `wasm-pack build` (gitignored, built in CI / locally)
- `admin-ui/package.json` — add `"frf-wasm": "workspace:../sdks/ts/frf-wasm"` to devDependencies
- `admin-ui/src/features/agents/services/agentGrpcStream.ts` — replace hardcoded `../../../../../sdks/ts/frf-wasm/frf_wasm.js` with `"frf-wasm"`
- `sdks/ts/.gitignore` — confirm `frf-wasm/` is ignored (already present; no change needed if correct)
- `pnpm-lock.yaml` — updated by `pnpm install`

## Design Notes

`wasm-pack build` generates `package.json` with `"name": "frf-wasm"` and
`"module": "frf_wasm.js"`. The pnpm workspace dep `"frf-wasm": "workspace:../sdks/ts/frf-wasm"`
causes `import("frf-wasm")` to resolve to `sdks/ts/frf-wasm/frf_wasm.js`.
`CrdtDemoButton.tsx` already imports `"frf-wasm"` via the Vite stub — no
change needed there; the real package takes over once installed.

The `frfWasmStubPlugin()` in `vite.config.ts` short-circuits when the package
is resolvable, so the stub becomes a no-op after wiring.

## Exit Criteria

- `wasm-pack build --target web --out-dir ../../sdks/ts/frf-wasm --out-name frf_wasm --release` exits 0 from `crates/frf-wasm/`
- `sdks/ts/frf-wasm/frf_wasm.js` and `frf_wasm_bg.wasm` exist
- `pnpm install` exits 0 (workspace dep resolved)
- `pnpm --filter @prometheusags/frf-admin-ui typecheck` exits 0
- `agentGrpcStream.ts` no longer contains the hardcoded relative path
