# p7-c006 — Dagger CI: Node 24 + WASM Build + E2E Stage

## Summary

Update `dagger/codegen.ts` to use Node 24, add a `wasm-build` stage before
`pnpm-build`, and add an `e2e-smoke` stage running Playwright Layer 1 tests.

## Changes

### `dagger/codegen.ts`

1. **Node version**: Change all occurrences of `"node:20-slim"` → `"node:24-slim"`

2. **Add `wasm-build` stage** (between current Stage 5 `buf-generate` and Stage 6 `pnpm-build`):
```typescript
// Stage 5b: wasm-build — build frf-wasm and output to sdks/ts/frf-wasm/
const wasmBuild = client
  .container()
  .from("rust:1.85-slim")
  .withExec(["apt-get", "update"])
  .withExec(["apt-get", "install", "-y", "curl", "wasm-pack"])
  .withMountedDirectory("/src", src)
  .withWorkdir("/src")
  .withExec(["bash", "crates/frf-wasm/build_wasm.sh"])
  .directory("/src/sdks/ts/frf-wasm");
```

3. **Update Stage 6 `pnpm-build`** to mount the WASM output from `wasmBuild`:
```typescript
const pnpmBuild = client
  .container()
  .from("node:24-slim")
  // ... existing mounts ...
  .withMountedDirectory("/src/sdks/ts/frf-wasm", wasmBuild)
  .withExec(["pnpm", "-r", "build"])
```

4. **Add Stage 7 `typecheck`** — separate TypeScript check:
```typescript
const typecheck = pnpmBuild
  .withWorkdir("/src/admin-ui")
  .withExec(["pnpm", "typecheck"]);
await typecheck.sync();
```

5. **Add Stage 8 `e2e-smoke`** — Playwright Layer 1 tests only:
```typescript
const e2eSmoke = client
  .container()
  .from("node:24-slim")
  .withExec(["npx", "playwright", "install", "--with-deps", "chromium"])
  // mount built output
  .withMountedDirectory("/src", src)
  .withWorkdir("/src/admin-ui")
  .withEnvVariable("SKIP_INTEGRATION", "true")
  .withExec(["pnpm", "exec", "playwright", "test",
    "--grep", "UI layer",
    "--project", "chromium"])
```

## Quality Gates

- `dagger/codegen.ts` parses and lints cleanly
- `fnm exec --using=24 pnpm typecheck` in `admin-ui/` passes (validates TS config)
