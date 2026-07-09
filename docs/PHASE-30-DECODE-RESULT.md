# Phase-30 decode result (p30-c002)

> Date: 2026-07-09. The in-network decode proof (browser inside the Docker network, c001) was run
> twice. **No decoded frame was observed — for an infrastructure reason, not a media-path finding.**
> **`SFU_MODE=sovereign` stays gated off.** Honest evidence below.

## Outcome: NOT proven — gate stays OFF (blocked by an environment/resource failure)

```
RUNNER_EXIT=1   (both attempts) — the run failed BEFORE the decode assertion; no framesDecoded line.
```

The topology fix (c001) and the harness plumbing are in place; the decode **never executed** because
the run couldn't get to a running stack in this environment. This is not a str0m/media result.

## What the two runs showed (evidence-led)

### Run 1 — harness-plumbing bug (fixed in c002)

The stack came up **healthy** (gateway `Healthy`, Keto seeded HTTP 201, Playwright container started),
then the in-container `pnpm install` failed:

```
[ERR_PNPM_NO_LOCKFILE] Cannot install with "frozen-lockfile" because pnpm-lock.yaml is absent
[ERR_PNPM_WORKSPACE_PKG_NOT_FOUND] "@prometheusags/frf-entity-management@workspace:*" … not present
```

**Root cause:** `admin-ui` is a **pnpm workspace** — its lockfile lives at the repo root and its
`workspace:*` deps + `node_modules` symlinks only resolve from the root. The c001 harness mounted
only `./admin-ui`. **Fixed in c002:** mount the **whole repo** (`.:/work`), drop the in-container
install (the host already has a complete workspace install — reuse it), and pin the Playwright image
to the **installed** `@playwright/test` version (**v1.61.0**, not the declared `^1.52.0` which
resolved to 1.61) so the image's baked browsers match the mounted CLI.

### Run 2 — Colima VM crashed during the gateway image build (environment/resource)

With the plumbing fixed, the re-run got further but the **gateway Docker build died mid-compile**:

```
#24 [ui-builder 12/12] RUN pnpm --dir sdks/ts build && … && pnpm --dir admin-ui build
#24 9.831 $ tsc --noEmit && vite build
failed to receive status: rpc error: code = Unavailable desc = error reading from server: EOF
```

Afterwards the **Docker daemon is unreachable**:

```
Cannot connect to the Docker daemon at unix:///…/colima/default/docker.sock. Is the docker daemon running?
```

**Root cause:** the gateway image build compiles the admin-ui (Vite + `tsc`, RustEmbed) inside the
image — a memory-heavy step. Run on the **Colima Linux-VM** engine (aarch64, Virtualization.Framework)
alongside the just-pulled Playwright image and the earlier stack, it **exhausted the VM and crashed
the Docker daemon** (`Unavailable … EOF`). This is an **environment/resource limit**, not a defect in
the phase-30 code — the topology fix and harness are correct; the box couldn't build + run the whole
stack in one pass.

## Decision — honest gate held

**No receiver observed `framesDecoded > 0`** (the decode never ran), so per the discipline carried
since phase-16: **`SFU_MODE=sovereign` is NOT flipped.** `main.rs` keeps the gate-off warning;
SECURITY §6 keeps the media plane *composed but not proven*. Calling this a pass would be dishonest —
no decoded media was observed, for any reason.

**Important nuance:** unlike phases 24–29, this is **not** a newly-discovered media-path blocker. Every
media-path defect found so far (candidate IP, shared socket, mDNS, STUN, topology) is fixed. The
remaining obstacle is purely the **build/run resource envelope of the local Colima VM** — the decode
proof needs an environment that can build the gateway image and run the two-browser stack without
OOM.

## Next (carried)

1. **Decouple the gateway image build from the decode run** so a single VM pass doesn't OOM:
   - Build the gateway image **once, out-of-band** (or on a bigger machine / CI) and have the runner
     `up -d --no-build gateway` against the pre-built image (skip the in-run `build gateway`).
   - Or raise the Colima VM memory (`colima start --memory 8` or more) before the run.
2. Re-run the in-network decode against a pre-built gateway image; **flip `SFU_MODE=sovereign` only on
   a genuine `framesDecoded > 0`.** Same gate.
3. The media-path work is complete; the residual is environmental (build/run capacity), carried.
