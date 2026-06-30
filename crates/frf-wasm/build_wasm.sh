#!/usr/bin/env bash
# Build frf-wasm and output the npm package to sdks/ts/frf-wasm/.
# Requires wasm-pack: https://rustwasm.github.io/wasm-pack/installer/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUT_DIR="${WORKSPACE_ROOT}/sdks/ts/frf-wasm"

echo "[frf-wasm] Building WASM package → ${OUT_DIR}"

wasm-pack build \
  --target web \
  --out-dir "${OUT_DIR}" \
  --out-name frf_wasm \
  --release \
  "${SCRIPT_DIR}"

echo "[frf-wasm] Build complete."
