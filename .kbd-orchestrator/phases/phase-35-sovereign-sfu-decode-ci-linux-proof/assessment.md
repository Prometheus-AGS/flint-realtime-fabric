# Assessment — phase-35-sovereign-sfu-decode-ci-linux-proof

> Date: 2026-07-09. Gap report against G1 (containerize/portablize the rig) + G2 (run on real Linux) +
> G3 (decode + flip) + G4 (carried). Grounded in the runner's host-bound pieces, the existing
> `ci.yml`, and Linux Docker networking.

## Method

Read `scripts/run-media-decode.sh` + `seed-media-view.sh` (host-bound steps), `.github/workflows/ci.yml`
(existing CI), `Dockerfile` (build weight), and Linux Docker host-addressing. No code changed.

## Key finding: most of the rig is ALREADY Linux-portable — the macOS/Colima bits just fall away

The pieces that looked macOS-specific are **not** deep dependencies:

- **`colima ssh` / `HOST_NET` derivation** (`run-media-decode.sh:55`) — only used in the host-net path
  (phase-33, now a dead-end non-goal). **Dropped on Linux** — native docker needs none of it.
- **`host.docker.internal`** for the gateway→JWKS URL — on Linux, containers reach the host at the
  docker0 bridge gateway **`172.17.0.1`**. So `GATEWAY_JWKS_URL=http://172.17.0.1:${JWKS_PORT}/jwks.json`
  (or serve the JWKS from a container). One-line adjustment.
- **JWT mint (`node`), JWKS serve (`python3 http.server`), Keto seed (`curl localhost:4467`)** — all
  present/native on a Linux runner; `localhost:4467` works because the runner **is** the host and the
  compose port is published. **No containerization strictly required** — they run as job steps.
- **Gateway image build** — heavy (rust + admin-ui vite) but well within **ubuntu-latest's 16 GB /
  4 vCPU / 14 GB disk**; the phase-30 OOM was the **12 GB Colima VM under contention**, not a hard
  ceiling. CI can build it (or pull a pushed image).

So G1 is **smaller than seeded**: it is mostly *Linux address-adjustment + a run entrypoint*, not a
from-scratch containerization. The full media stack (28→34) is proven; on Linux the candidate-address
confusions simply do not arise (native host networking; `172.17.0.1` host addressing).

## Gap analysis

### G1/G2 — run the decode on real Linux  ·  **GAP: CONFIRMED — but the *runner* is an operator decision**

Two targets, **materially different change sets**:

| Target | How | Effort | Trade-off |
|---|---|---|---|
| **A — GitHub Actions `ubuntu-latest`** | A **net-new workflow job** in `ci.yml` (no compose job exists today): checkout → build/pull the gateway image → `docker compose up` the sovereign stack → run the decode spec (Playwright browsers via the `mcr` image or `setup` action) → assert `framesDecoded>0`. Address bits: `172.17.0.1` JWKS, drop `colima ssh`. | **medium** | Fully reproducible + shareable + the durable home; each run costs CI minutes + a gateway build (cacheable); slowest iteration. |
| **B — Self-hosted Linux box** | Linux-adjust `run-media-decode.sh` (drop `colima ssh`; `172.17.0.1` JWKS; no HOST_NET) and the **operator runs it directly** on a Linux host they provide. | **low-medium** | Fastest iteration; but the operator must provide + maintain the box, and it is not self-contained/shareable. |

**These differ**: A is a CI-workflow authoring change; B is a runner-script portability change + an
operator-run. The phase cannot plan the change set without the target. **This is the blocking operator
decision the goals flagged.** Recommendation: **A (GitHub Actions)** — it is the durable, reproducible
home, the existing `ci.yml` is the natural place, and it removes "my box" from the equation entirely
(the whole point of escalating off the local environment). B only if the operator specifically wants
faster local-Linux iteration.

### G3 — decoded frame + flip  ·  **GAP: OPEN (depends on the target running)**

Unchanged discipline: on the runner, observe `ice=connected` + `framesDecoded > 0`; flip only on a
genuine frame; else re-affirm gated. This is where the proven-in-pieces stack finally runs end-to-end.

### G4 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated.

## OPEN QUESTION FOR THE OPERATOR (blocking — resolve before plan)

**Which Linux runner — A (GitHub Actions `ubuntu-latest`) or B (a self-hosted Linux box)?**
Recommendation **A** (durable/reproducible; existing `ci.yml`; removes the local box). The plan will
ask via AskUserQuestion before committing the change set. (For A, a sub-note: is pushing a prebuilt
gateway image to a registry acceptable, or should CI build it each run with layer caching?)

## Recommended change ordering (conditional on the target)

- **If A:** c001 CI decode workflow (job in `ci.yml` + Linux address bits) → c002 run + flip.
- **If B:** c001 Linux-portable runner (drop colima/host.docker.internal; `172.17.0.1`) → c002 run
  (operator-executed) + flip.

## Exit posture

The load-bearing item is **not** code volume — the rig is largely portable and the media stack is
proven. It is the **runner-target decision** (operator's) and then a live run on real Linux where the
six local candidate-topology confusions simply don't exist. No `frf-*` engine change. Honest gate on
G3 unchanged. No same-host macOS/Colima variants (hard non-goal).
