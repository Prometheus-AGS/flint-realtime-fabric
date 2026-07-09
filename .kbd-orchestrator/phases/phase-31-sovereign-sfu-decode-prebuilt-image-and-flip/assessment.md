# Assessment — phase-31-sovereign-sfu-decode-prebuilt-image-and-flip

> Date: 2026-07-09. Gap report against G1 (decouple the gateway image build from the decode run) +
> G2 (decode + flip) + G3 (carried proofs). Grounded in the runner, the Dockerfile, and the live
> Colima state.

## Method

Read `scripts/run-media-decode.sh` (the in-run build), `compose.yml` (gateway build/image),
`Dockerfile` (build weight), and the Colima/daemon state. No code changed (assess only).

## Current state (what exists)

- **The OOM culprit is `run-media-decode.sh:70` — `docker compose build gateway`**, run
  **unconditionally every decode run**. It builds the root `Dockerfile`, which is a heavy multi-stage
  build: a **`ui-builder` stage** (`node:24-slim` → `pnpm build` for sdks + admin-ui, i.e. Vite +
  `tsc` — the step that crashed in phase-30) **plus a `rust:latest builder` stage**. Doing this
  in-run, alongside the just-pulled Playwright image and the booting stack, is what exhausted the VM.
- **The gateway service has `build:` but no `image:` name** (`compose.yml:4-7`), so its image is
  **implicitly** `flint-realtime-fabric-gateway` (project `flint-realtime-fabric`). `up --no-build`
  uses whatever image is already tagged that name.
- **Colima:** `default` profile is **12 GiB / 6 CPUs** — memory is not tiny. So the phase-30 crash
  was **contention** (heavy build *concurrent with* the stack + Playwright pull), not a hard ceiling.
  This means **decoupling the build (G1.1) is the primary fix; more memory (G1.2) is secondary.**
- **Daemon is DOWN** — Colima crashed in phase-30 and has not recovered; `colima start` is required
  before any Docker work.

## Gap analysis

### G1 — decouple the gateway image build from the decode run  ·  **GAP: CONFIRMED (the phase core)**

- **Defect:** the runner rebuilds the heavy gateway image **inside** the decode run, OOM-crashing the
  VM before the decode executes (phase-30 evidence).
- **Required:**
  1. Change `run-media-decode.sh:70` from an unconditional `build gateway` to a **presence check**:
     if `flint-realtime-fabric-gateway` exists → skip the build (go straight to `up --no-build`); if
     absent → **fail fast** with a clear message pointing at the out-of-band build command
     (`docker compose … build gateway`), never silently rebuild in-run. Optionally honor a
     `PREBUILD_GATEWAY=1` escape hatch to build-then-run for a fresh checkout.
  2. Document the one-time out-of-band build step in the runner header + phase docs.
  3. (Secondary) note/raise Colima memory (`colima start --memory 8`+) as belt-and-suspenders.
- **Scope:** `scripts/run-media-decode.sh` (the build→check swap + header), docs. **No `frf-*` engine
  change, no compose change** (the gateway service already builds to a stable implicit image name).
  File-size ≤500.
- **Risk:** low-code, but the phase's exit depends on a **live run** — which needs the daemon back
  (`colima start`) and the image built once. The build itself is still heavy; it just happens **once,
  alone**, not every run.

### G2 — decoded frame + flip  ·  **GAP: OPEN (depends on G1 + a working daemon)**

Unchanged discipline: re-run the in-network decode; observe `ice=connected` + `framesDecoded > 0`;
flip `SFU_MODE=sovereign` **only** on a genuine decoded frame; else re-affirm gated. All media-path +
harness fixes are already in place (phases 24–30). G1's UNVERIFIED topology fix (phase-30) gets its
first real verification here.

### G3 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated. No new finding.

## Open question for plan/analyze

**None blocking** — G1's approach (presence-check + fail-fast + out-of-band build) is unambiguous and
low-risk. The only external dependency is operational: **the daemon must be restarted** (`colima
start`, ideally `--memory 8`+) and the gateway image built once before the decode can run. Flag this
in the plan as a manual prerequisite the operator runs via `!`.

## Recommended change ordering (for plan)

1. **c001 — prebuilt-image runner** (presence-check + fail-fast + out-of-band build docs; the
   decouple). Unblocks the environment.
2. **c002 — decode re-run + conditional flip** (against a pre-built image on a restarted daemon;
   PHASE-31-DECODE-RESULT; flip or re-affirm). First real verification of the phase-30 topology fix.

## Exit posture

The phase's load-bearing risk is **operational, not code** — the runner change (G1) is small and
safe; the hard part is that a **live run finally happens** on a healthy daemon against a pre-built
image. G1 verifies the phase-30 topology fix; G2 is the honest gate. Nothing here re-opens an ADR.
