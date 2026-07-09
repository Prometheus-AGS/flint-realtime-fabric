# Phase-31 decode result (p31-c002)

> Date: 2026-07-09. With the gateway image pre-built (c001) and a recovered daemon, the in-network
> decode was driven through **four** distinct harness/infra failures, each fixed in turn. The run now
> **starts the decode spec and boots the full stack**, but stops at a **secure-context limitation**:
> `navigator.mediaDevices` is undefined for the in-network browser, so `getUserMedia` throws before
> any offer. **No decoded frame. `SFU_MODE=sovereign` stays gated OFF.** This is a harness/environment
> limit, not a media-path defect.

## Outcome: NOT proven — gate stays OFF (harness/environment)

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading 'getUserMedia')
1 failed  ·  RUNNER_EXIT=1
```

The gateway boots (only the startup gate-off warning logs); the sender's `getUserMedia` fails before
any session negotiates, so there is no str0m lifecycle to read. `framesDecoded` was never reached.

## What c001 + c002 achieved (real progress this phase)

- **c001's decouple works, proven:** with the in-run build removed, the heavy gateway image built
  **cleanly when run alone** (`BUILD_EXIT=0`, no OOM) — confirming the phase-30 crash was *contention*,
  not a hard memory ceiling. The runner now presence-checks the pre-built image and fails fast.
- **c002 fixed a real cascade of harness bugs**, each a genuine blocker surfaced only by running:
  1. **pnpm-workspace mount** — `workspace:*` deps only resolve from the repo root (mount `.:/work`).
  2. **Playwright image/CLI version mismatch** — declared `^1.52` resolved to `1.61` (pin image
     `v1.61.0`).
  3. **`webServer: pnpm dev` exit 127** — the config started a Vite dev server the decode spec does
     not need, and `pnpm` isn't on the container PATH (`DECODE_ONLY=1` skips it).
  4. **secure context / `getUserMedia`** — the current, unresolved one (below).

## The unresolved blocker: secure context for `getUserMedia` (harness/environment)

The in-network browser navigates to `http://gateway:8080` — an **insecure origin**, so
`navigator.mediaDevices` is `undefined` and `getUserMedia` throws. The standard mitigation —
`--unsafely-treat-insecure-origin-as-secure-origin=http://gateway:8080` plus
`--disable-site-isolation-trials` and `--disable-features=IsolateOrigins,site-per-process` — was
applied at both browser launch sites but **did not** register `mediaDevices` in this headless
`mcr.microsoft.com/playwright` container. On the host run this never appeared because
`http://localhost:28080` is a secure context by Chromium's rule; in-network `gateway:8080` is not.

This is an **environment/harness** limitation of driving `getUserMedia` over a non-HTTPS in-network
origin, not a str0m/media defect. It is a *different* obstacle than the phase-29 candidate topology —
that fix (browser-in-network) is in place; this is upstream of it (the sender can't even acquire a
track).

## Decision — honest gate held (and the live-iteration loop stopped)

No `framesDecoded > 0`; no session negotiated. Per the discipline carried since phase-16:
**`SFU_MODE=sovereign` is NOT flipped.** `main.rs` untouched; SECURITY §6 keeps the media plane
*composed but not proven*.

**Iteration discipline:** c002 cleared four harness/infra layers, but this fifth (secure context in a
headless container) is Chromium-flag arcana with no media-path signal and a VM that crashed twice
this phase. Continuing to guess at flags live is a rabbit hole — stopped here and recorded, rather
than thrashing. The media-path work is done; what remains is a **harness that can give the in-network
browser a secure context**.

## Next (carried) — give the browser a real secure context

The clean fixes, in order of robustness:

1. **Serve the gateway over HTTPS in the decode stack** (self-signed cert + the browser launched with
   `--ignore-certificate-errors`), so `https://gateway:8443` is a genuine secure context and
   `getUserMedia` works with no unsafe flags.
2. **Or** reach the gateway as a `*.localhost` origin (Chromium treats `*.localhost` as
   potentially-trustworthy) via a compose network alias `gateway.localhost` + a matching in-network
   resolvable name.
3. **Or** verify/repair the `--unsafely-treat-insecure-origin-as-secure-origin` propagation through
   `launchPersistentContext` (it may need `ignoreDefaultArgs` or a different launch path in this
   Playwright/Chromium build).

Then re-run and flip only on `framesDecoded > 0`. The SFU engine, shared-socket demux, STUN, and
in-network topology are all proven/in-place; the remaining gap is a secure-context harness detail.
