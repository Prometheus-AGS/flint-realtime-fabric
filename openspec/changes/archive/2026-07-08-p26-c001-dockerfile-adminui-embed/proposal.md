# p26-c001-dockerfile-adminui-embed

## Why

The gateway Docker image fails to build: `frf-gateway` `#[derive(RustEmbed)]`s `admin-ui/dist` at
compile time (release), but the Dockerfile never builds/copies it → `RustEmbed` fails → the image
won't compile → the sovereign stack never boots (blocked the phase-25 decode proof;
`docs/PHASE-25-DECODE-RESULT.md`). Host builds pass only because a stale local `dist/` exists.

## What Changes

- Add a **Node 24 build stage** to `Dockerfile`: copy the root pnpm workspace manifests
  (`package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`) + `admin-ui/` + the workspace member
  sources it imports (`sdks/entity-management`, `sdks/ts`), `corepack enable && pnpm install
  --frozen-lockfile`, then `pnpm --dir admin-ui build`. `vite.config.ts` falls back to the
  `frf-wasm` stub when the wasm artifact is absent, so **no Rust→wasm step is needed** and `dist`
  is produced.
- In the Rust builder stage, `COPY --from=<node> /build/admin-ui/dist ./admin-ui/dist` **before**
  `cargo build`, so the compile-time embed resolves.

## Impact

- `Dockerfile` (+ possibly a small `.dockerignore` adjustment if a needed source is excluded — the
  current ignores are `node_modules`/`dist`, both produced in-stage, so likely none).
- No production code; no gate flip. Unblocks the live decode run (c002).
