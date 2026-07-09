# p35-c001-ci-decode-job

## Why

Phases 28→34 proved every media-path layer correct in pieces but hit a fresh candidate-address
confusion on the same-host macOS + Colima-VM + Docker-bridge box (six forms). On a **real Linux host**
(GitHub Actions `ubuntu-latest`) host networking is native and containers reach the host at the docker0
bridge gateway `172.17.0.1` — the confusions simply don't arise. This change makes the runner
Linux-portable and adds a `decode-proof` CI job to run the whole proof there.

## What Changes

- **`scripts/run-media-decode.sh`:** default (non-HOST_NET) path uses **`172.17.0.1`** for the
  gateway→JWKS URL on Linux (replaces `host.docker.internal`, which is macOS/Docker-Desktop-only);
  detect Linux (`uname`/`CI`) to pick the host address, keep `host.docker.internal` for macOS. The
  colima/HOST_NET branch stays gated behind `HOST_NET=1` (unused on Linux).
- **`.github/workflows/decode-proof.yml`:** a `workflow_dispatch` job on `ubuntu-latest` — checkout →
  toolchains (rust + protoc + node/pnpm) → build the gateway image → run the decode entrypoint
  (compose up + Playwright decode) → assert `framesDecoded > 0` → upload gateway logs + the Playwright
  report as artifacts. Secrets (TURN/JWT) generated in-job — never committed (S1).

## Impact

- `scripts/run-media-decode.sh` (Linux host address), `.github/workflows/decode-proof.yml` (new). No
  `frf-*` engine change; no same-host macOS/Colima variant.
