# Phase-25 decoded-media proof — live run result (p25-c002)

> Date: 2026-07-08. The actual, un-fabricated outcome of running the **authenticated** decode
> proof. This is the input that decides the `SFU_MODE=sovereign` flip (p25-c003).

## Result: ❌ NOT PROVEN — `framesDecoded > 0` was NOT observed

The run got **substantially further** than phase-24 — mint, JWKS, compose merge, and the admin-ui
build all succeeded — but failed on a **Docker gateway-build defect** before the stack came up.
`RUNNER_EXIT=1`. No decoded frame was observed, so **G2 is not met** and c003 re-affirms the gate
off.

## What actually happened (progress + the blocker)

```
[run-media-decode] minting an RS256 JWT + JWKS…                 ✓
[run-media-decode] serving the JWKS on :8791…                   ✓
[run-media-decode] bringing up the sovereign stack…             (docker build)
  #31 ✓ admin-ui built in 19.39s                                ✓ (vite build ran)
  #45 error: #[derive(RustEmbed)] folder
      '/build/crates/frf-gateway/../../admin-ui/dist' does not exist. cwd: '/build'
  #45 error[E0599]: no associated function `get` for `AdminUiAssets`
  #45 error: could not compile `frf-gateway` (lib) due to 3 previous errors
  target gateway: failed to solve … exit code: 101
RUNNER_EXIT=1
```

## Root cause (diagnosed) — a real Dockerfile defect

- `frf-gateway` embeds the admin UI via `#[derive(RustEmbed)]` on `admin-ui/dist`.
- The **`Dockerfile` never builds or copies `admin-ui/dist`** — its build context is only
  `Cargo.toml`/`Cargo.lock`, `crates/`, `proto/`. So inside the image `admin-ui/dist` does not
  exist → `RustEmbed` fails at compile time → the gateway lib won't build → the stack never boots.
- On the **host**, `admin-ui/dist` *does* exist (a prior local `vite build`), which is why host
  `cargo build`/tests pass — the gap is Docker-context-only.

This is **infrastructure/build plumbing**, not the media path. The proof again did not reach
ICE/DTLS/RTP — the genuine media-path unknown remains untested.

## Honest assessment

- **No decoded frame.** `framesDecoded > 0` was not observed. The proof does not pass.
- **Real progress this run:** the phase-25 authenticated path works up to the Docker build — RS256
  mint + self-served JWKS + `GATEWAY_JWKS_URL` override + sovereign compose merge + admin-ui vite
  build all succeeded. Two prior blockers (compose merge, HS256/RS256 mismatch) are behind us.
- **The remaining blocker is a Dockerfile defect** (admin-ui `dist` not built/copied before the
  Rust embed) — concrete and fixable, but its own change.

## Consequence for the flip (p25-c003)

**`SFU_MODE=sovereign` stays gated OFF.** G2 unproven. Per the operator-confirmed honest gate,
c003 re-affirms gated with this concrete blocker — no forced flip.

## To make the proof pass (next attempt)

1. **Fix the Dockerfile** so `admin-ui/dist` exists before the gateway builds: add a Node build
   stage (`pnpm --dir admin-ui build`) and `COPY admin-ui/dist` into the Rust build context, or
   build admin-ui on the host and `COPY` the artifact. (A follow-up change — likely
   phase-26-gateway-docker-adminui-embed.)
2. Re-run `scripts/run-media-decode.sh`; the stack should boot and the harness should reach the SFU.
3. Then the **real media-path test** finally runs: does Chromium complete ICE/DTLS/RTP to the
   `host.docker.internal` UDP candidate and decode a frame? That is the still-untested unknown.
4. Only if `framesDecoded > 0` is observed does the gate flip.
