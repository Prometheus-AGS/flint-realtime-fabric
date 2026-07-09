# Phase-31 signoff — sovereign SFU decode: prebuilt image + harness fixes

> Date: 2026-07-09. Phase-31 decoupled the gateway image build (c001, fixing the phase-30 OOM) and
> cleared four more harness/infra layers (c002) so the decode now starts — but it stops at a
> **secure-context limit** (`getUserMedia` on an insecure in-network origin). **`SFU_MODE=sovereign`
> stays gated OFF** — no decoded frame; a harness/environment detail, not a media-path defect.

## Gate decision: OFF (honest gate held; live-iteration loop stopped)

No `framesDecoded > 0`; no session negotiated. Per the phase-16→30 discipline: **the gate does not
flip until a real receiver observes a decoded frame.** `crates/frf-gateway/src/main.rs` untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p31-c001 | Prebuilt-image runner (presence-check + fail-fast; no in-run build) | none (harness); OOM fixed |
| p31-c002 | Harness fixes (workspace mount, image v1.61, `DECODE_ONLY` skips `webServer`, secure-context flags) + decode run | **held OFF** |

## Evidence & progress

- **c001 decouple proven:** the gateway image built cleanly **run alone** (`BUILD_EXIT=0`, no OOM) —
  the phase-30 crash was contention, not a memory ceiling. Runner now uses the pre-built image.
- **c002 fixed a real cascade** (each surfaced only by running): pnpm-workspace mount → Playwright
  image/CLI version → `webServer: pnpm dev` exit 127 → secure context. The run now boots the full
  stack and starts the decode spec.
- **Unresolved blocker:** the in-network browser (`http://gateway:8080`, insecure) has
  `navigator.mediaDevices === undefined`; `getUserMedia` throws before any offer. The
  `--unsafely-treat-insecure-origin-as-secure-origin` flags did not register `mediaDevices` in the
  headless Playwright container. **Harness/environment, not str0m.**

## Iteration discipline

c002 cleared four harness layers; the fifth (secure context in a headless container) is Chromium-flag
arcana with no media-path signal, on a VM that crashed twice this phase. Per "avoid rabbit holes,"
the live-iteration loop was **stopped and recorded** rather than thrashing on flags.

## Carried to next phase — a real secure context for the in-network browser

1. **Serve the gateway over HTTPS** in the decode stack (self-signed + `--ignore-certificate-errors`)
   → `https://gateway:8443` is a genuine secure context; `getUserMedia` works with no unsafe flags.
2. **Or** a `*.localhost` origin alias (Chromium treats `*.localhost` as trustworthy).
3. **Or** repair the insecure-origin flag propagation through `launchPersistentContext`.

Then re-run and flip only on `framesDecoded > 0`. The SFU engine, shared-socket demux, `.local` skip,
STUN, and in-network topology are all proven/in-place; the remaining gap is a secure-context harness
detail upstream of the media path.

## Verification

- Host `cargo test -p frf-media-str0m` (31), `cargo fmt --check`, admin-ui eslint — green (no engine
  change; the media-path Rust is untouched).
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the secure-context
  blocker.
