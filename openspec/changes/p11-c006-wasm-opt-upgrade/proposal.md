# p11-c006 — wasm-opt Upgrade (Remove --no-opt)

## Phase
phase-11-layer3-e2e-wasm-opt-cdc

## Summary

Replace the `--no-opt` workaround in Dagger Stage 6 with a `binaryen` >= 116
install step that supports the `bulk-memory` proposal used by Loro CRDT. After
installing a current `wasm-opt`, remove `--no-opt` from the `wasm-pack build`
command to re-enable wasm optimization.

## Files to Create/Modify

- `dagger/codegen.ts` — Stage 6 (wasm-pack build):
  1. Add a `binaryen` install step BEFORE the `wasm-pack build` step:
     ```typescript
     .withExec(["sh", "-c",
         "apt-get update -qq && " +
         "apt-get install -y --no-install-recommends wget && " +
         "BINARYEN_VER=version_116 && " +
         "wget -q https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VER}/binaryen-${BINARYEN_VER}-x86_64-linux.tar.gz -O /tmp/binaryen.tar.gz && " +
         "tar -xzf /tmp/binaryen.tar.gz -C /usr/local --strip-components=1 && " +
         "wasm-opt --version"
     ])
     ```
  2. Remove `"--no-opt"` from the `wasm-pack build` args array.

## Design Notes

The `--no-opt` flag was introduced in p10-c001 as a workaround for a
`wasm-opt: unexpected end of section or function` error that occurs when
`wasm-bindgen-cli 0.13.1` bundles `wasm-opt` 105 (which pre-dates the
`bulk-memory` proposal). Loro CRDT uses bulk-memory WASM instructions; the
bundled `wasm-opt` cannot validate them.

Binaryen 116 (released 2023-11) fully supports the `bulk-memory` proposal
(spec merged 2022). Installing it to `/usr/local/bin/wasm-opt` shadows the
`wasm-pack`-bundled copy, which `wasm-pack` invokes via `PATH` lookup.

The binaryen GitHub release tarball structure places binaries under
`binaryen-<ver>/bin/` — `--strip-components=1` extracts to `/usr/local/bin/`
and `/usr/local/lib/` directly.

**Version pinning**: pin to `version_116` (exact GitHub tag). Do not use
`latest` — wasm-opt ABI changes may require `wasm-bindgen` version bumps.

## Exit Criteria

- `wasm-opt --version` inside the Dagger Stage 6 container outputs `version_116`
- `wasm-pack build --target web` completes without `--no-opt` (exit 0)
- `sdks/ts/frf-wasm/frf_wasm_bg.wasm` is smaller than the `--no-opt` variant
  (optimization reduces size by ~15-30% for typical Loro WASM)
- Dagger Stage 6 verification step confirms `frf_wasm.js` and
  `frf_wasm_bg.wasm` both exist and `package.json` name is `"frf-wasm"`
