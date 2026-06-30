# p13-c004 — Install `wasm-pack` in Stage 6 Dagger container

## Summary

Stage 6 of the Dagger pipeline (`dagger/codegen.ts`) runs
`wasm-pack build crates/frf-wasm --target web --out-dir /workspace/sdks/ts/frf-wasm`
but the Stage 6 base image (`rust:latest`) does not include `wasm-pack` or
`wasm-opt`. The stage will fail with `command not found: wasm-pack` unless
the tool is installed inside the container before the build step.

Additionally, `wasm-opt` (from the `binaryen` package) is required by
`wasm-pack` for optimisation. It must also be installed.

## File to change

- `dagger/codegen.ts` — Stage 6 container setup

## Specification

Find the Stage 6 container chain (the one that calls `wasm-pack build`)
and prepend the wasm-pack installation before the build command.

The Stage 6 container currently uses `rust:latest` as its base. Add:

```typescript
// Before .withExec(["wasm-pack", "build", ...]):
.withExec(["apt-get", "update"])
.withExec(["apt-get", "install", "-y", "--no-install-recommends",
    "binaryen",          // provides wasm-opt
    "curl",
    "ca-certificates",
])
.withExec(["sh", "-c",
    "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
])
```

The `curl | sh` pattern is the official wasm-pack installer. Pin to a specific
version by using:
```
"curl -sSL https://github.com/rustwasm/wasm-pack/releases/download/v0.13.1/wasm-pack-init.sh | sh"
```
if reproducibility is required. The current proposal uses latest to reduce
maintenance overhead.

## Acceptance criteria

1. Stage 6 runs to completion without `command not found: wasm-pack` error.
2. `sdks/ts/frf-wasm/frf_wasm_bg.wasm` is produced.
3. `sdks/ts/frf-wasm/package.json` `name` field is `frf-wasm` (existing jq
   validation in Stage 6 passes).
4. The WASM size gate emits `WASM binary size: <N> bytes` and either
   `OK` (if baseline exists) or `No baseline found` (first run).
5. Dagger layer cache is preserved — apt-get and wasm-pack install steps
   are ordered before any `COPY` of workspace source so they cache
   independently of source changes.
