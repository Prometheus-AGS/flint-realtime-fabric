# p12-c003 — Stage 8: Expand E2E to All Specs with WASM_AVAILABLE

## Phase
phase-12-layer3-e2e-runtime-wasm-gate-cdc-smoke

## Summary

Expand Dagger Stage 8 (e2e-smoke) from running only `p7-smoke.spec.ts` to
running all `e2e/*.spec.ts` files with `WASM_AVAILABLE=1` set. The WASM
package is already built by Stage 6 and mounted into the workspace, so the
CRDT Layer 2 tests in `layer2-crdt.spec.ts` can run against the built WASM.

## Files to Modify

- `dagger/codegen.ts` — Stage 8 block:
  1. Add `.withEnvVariable("WASM_AVAILABLE", "1")` before the `playwright test` call
  2. Change `"e2e/p7-smoke.spec.ts"` to `"e2e/"` to run all specs in the directory
  3. Remove `"--reporter=list"` only if needed, or keep it for CI legibility

Before:
```typescript
const e2eSmoke = pnpmBuild
    .withExec(["pnpm", "--filter", "@prometheusags/frf-admin-ui",
        "exec", "playwright", "install", "--with-deps", "chromium",
    ])
    .withWorkdir("/workspace/admin-ui")
    .withExec(["pnpm", "exec", "playwright", "test",
        "e2e/p7-smoke.spec.ts",
        "--reporter=list",
    ]);
```

After:
```typescript
const e2eSmoke = pnpmBuild
    .withExec(["pnpm", "--filter", "@prometheusags/frf-admin-ui",
        "exec", "playwright", "install", "--with-deps", "chromium",
    ])
    .withEnvVariable("WASM_AVAILABLE", "1")
    .withWorkdir("/workspace/admin-ui")
    .withExec(["pnpm", "exec", "playwright", "test",
        "e2e/",
        "--reporter=list",
    ]);
```

## Design Notes

Stage 8 runs Playwright against the admin-UI dev server (started by
`playwright.config.ts`'s `webServer.command = "pnpm dev"`). The `WASM_AVAILABLE`
env var enables Layer 2 CRDT tests. The admin-UI dev server serves the WASM
package from the workspace-linked `sdks/ts/frf-wasm/` directory (already
populated by Stage 6's `wasmOut` mount).

Integration tests in phase4/5/6 specs remain gated by `!process.env["GATEWAY_URL"]`
which is not set in Stage 8, so they skip correctly — only Layer 1 and
WASM-gated Layer 2 tests execute.

## Exit Criteria

- Dagger Stage 8 runs `e2e/` directory (all specs), not just `p7-smoke.spec.ts`
- `WASM_AVAILABLE=1` is in the Stage 8 env block
- `layer2-crdt.spec.ts` Layer 1 tests pass in Stage 8
- `layer2-crdt.spec.ts` Layer 2 tests execute (not skipped) in Stage 8 when WASM is built
- Phase4/5/6 integration tests still skip in Stage 8 (no `GATEWAY_URL`)
