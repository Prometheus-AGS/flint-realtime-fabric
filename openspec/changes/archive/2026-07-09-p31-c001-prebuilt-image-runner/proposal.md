# p31-c001-prebuilt-image-runner

## Why

Phase-30: the decode runner does an **unconditional in-run `docker compose build gateway`**
(`run-media-decode.sh:70`) — a heavy multi-stage build (node ui-builder Vite/`tsc` + rust builder)
that OOM-crashed the Colima VM before the decode could run. The build must be decoupled: run it once
out-of-band, and have the runner use the pre-built image (fail fast if it is absent).

## What Changes

- `scripts/run-media-decode.sh`: replace the unconditional `build gateway` with a **presence check**
  on the implicit image `flint-realtime-fabric-gateway` (`docker image inspect`). Present → skip the
  build and `up -d --no-build`. Absent → **exit non-zero** with a clear message naming the one-time
  out-of-band build command. A `PREBUILD_GATEWAY=1` escape hatch builds-then-runs for a fresh
  checkout.
- Runner header documents the one-time build + `colima start --memory 8` prerequisite.

## Impact

- `scripts/run-media-decode.sh` only. No compose change (the gateway builds to a stable implicit
  image name), no `frf-*` engine change. Unblocks the decode run (c002).
