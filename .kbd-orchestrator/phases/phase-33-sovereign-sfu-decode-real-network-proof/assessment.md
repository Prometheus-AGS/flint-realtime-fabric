# Assessment — phase-33-sovereign-sfu-decode-real-network-proof

> Date: 2026-07-09. Gap report against G1 (routable-network environment) + G2 (TURN relay) + G3
> (decode + flip) + G4 (carried). Grounded in the proof rig, existing CI, and a re-examination of the
> phase-32 topology conclusion.

## Method

Read `scripts/run-media-decode.sh` (host-side rig pieces), `compose.sovereign.yml` (coturn STUN-only),
`.github/workflows/ci.yml` (existing CI), and re-reasoned the Colima network model. No code changed.

## Current state (what exists)

- **The media path + secure context are proven live** (phase-32: `getUserMedia` works, sessions
  negotiate, ICE `checking`). The full proof rig exists: RS256 JWT mint + self-served JWKS, Keto
  `view` seed, coturn **STUN-only**, in-network Playwright (v1.61), `DECODE_ONLY`, Caddy TLS sidecar.
- **The rig has host-side pieces** (`run-media-decode.sh`): the JWKS is served on a **host** port
  (`http.server` + `host.docker.internal`), the JWT is minted with host `node`, the Keto seed hits
  host `localhost:4467`. These must be containerized to run the whole proof on a clean runner.
- **CI already exists:** `.github/workflows/ci.yml` runs multiple `ubuntu-latest` jobs — a real Linux
  environment is available with no new infrastructure.
- **coturn is `--stun-only`** — no TURN relay yet (G2's work).

## Re-examination of the phase-32 conclusion (important)

Phase-32 said "same-host VM is the wrong environment." That is right for the *bridge* setup used —
but **Colima is itself a Linux VM**, and a container with `network_mode: host` shares the **VM's**
network stack. If **both** the gateway and the Playwright browser run in host-net containers, they
share one real network — so host candidates pair directly, with **no bridge split, no TURN, no CI**.
This local host-net path was under-explored at phase-32 (the macOS host was conflated with the VM
host). It is the **lowest-effort** way to get a routable pair and should be the first option offered.

## Gap analysis

### G1/G2 — a routable candidate pair  ·  **GAP: CONFIRMED — but the *how* is an operator decision**

Three viable targets, materially different change sets:

| Target | How | Effort | Trade-off |
|---|---|---|---|
| **A — Local host-net (both containers)** | gateway + playwright both `network_mode: host` on the Colima VM; `MEDIA_ADVERTISE_IP` = the VM host IP; STUN may even be unnecessary (host candidates pair). | **Low** | Fastest to try, no CI/TURN; still the same local VM (but on one network stack, which was the actual gap). Host-net + published-port conflicts to reconcile. |
| **B — GitHub Actions CI** (`ubuntu-latest`) | containerize the host-side rig (JWKS/mint/seed) so the whole proof runs in one CI job on a clean Linux runner; host networking is native on Linux CI. | **High** | Most reproducible + durable + shareable; slowest iteration; most rig-consolidation work. |
| **C — Local + TURN relay** | keep bridge; add coturn TURN (`--external-ip` + realm + long-term creds); harness `iceServers` adds `turn:`; advertise gateway bridge IP. | **Medium** | Robust (relay always routes) + closest to production reality; heavier coturn config; still local VM. |

**These are not equivalent** — A is a compose/`MEDIA_ADVERTISE_IP` change, B is a rig-containerization
+ CI-job change, C is a coturn-TURN + harness change. The phase cannot plan a change set without the
target. **This is the operator decision the phase goals flagged.**

### G3 — decoded frame + flip  ·  **GAP: OPEN (depends on the chosen target)**

Unchanged discipline: re-run on the target; flip only on `framesDecoded > 0`; else re-affirm gated.

### G4 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated.

## OPEN QUESTION FOR THE OPERATOR (blocking — must resolve before plan)

**Which environment target for the decode proof — A (local host-net), B (GitHub CI), or C (local +
TURN)?** The recommendation is to **try A first** (lowest effort; directly addresses the routable-pair
gap by putting both peers on one network stack; it's the option under-explored at phase-32), and fall
back to **C (TURN)** if host-net has port/topology conflicts, with **B (CI)** as the durable home once
it passes locally. But the target is the operator's call — it determines the entire change set and may
carry infra preferences (CI minutes, self-hosted runners, staging hosts) the assessment can't see.

## Recommended change ordering (conditional on the target)

- **If A:** c001 host-net gateway + browser (compose + `MEDIA_ADVERTISE_IP`) → c002 decode + flip.
- **If C:** c001 coturn TURN + harness `iceServers` + bridge-IP advertise → c002 decode + flip.
- **If B:** c001 containerize the rig (JWKS/mint/seed) → c002 CI job + decode → c003 flip.

## Exit posture

The load-bearing item is **not** code — it is the **environment-target decision**, which is the
operator's. The assessment recommends A-first (host-net) with C (TURN) as fallback and B (CI) as the
durable home. No `frf-*` engine change is expected in any path. The honest gate on G3 is unchanged.
