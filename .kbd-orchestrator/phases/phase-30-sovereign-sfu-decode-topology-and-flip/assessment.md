# Assessment — phase-30-sovereign-sfu-decode-topology-and-flip

> Date: 2026-07-09. Gap report against G1 (fix the same-host candidate topology) + G2 (decode +
> flip) + G3 (carried proofs). Grounded in the actual harness/compose + a decisive platform finding.

## Method

Read `admin-ui/e2e/media-decode.spec.ts` (how Chromium launches), `compose.sovereign.yml` (media +
coturn), `crates/frf-media-str0m/src/config.rs` (`resolve_advertised_ip`), and the Docker engine
context. No code changed (assess only).

## Current state (what exists)

- **Host-side Chromium.** The harness launches `chromium.launchPersistentContext(...)` via **host**
  Playwright (`media-decode.spec.ts:48,135`) — two host browsers (sender + receiver). They navigate
  to `GATEWAY_URL` (`http://localhost:28080`, a **secure context** — required for
  `getUserMedia`/`RTCPeerConnection`). **This host-vs-container split is the root of the phase-29
  topology gap.**
- **Media path.** Gateway binds `0.0.0.0:40000` in-container, publishes `40000:40000/udp` to the
  host, advertises `MEDIA_ADVERTISE_IP` (default `127.0.0.1`). coturn publishes `3478/udp`.
- **`resolve_advertised_ip` already resolves a hostname** (`host.docker.internal` → an IP at
  negotiate time) — so advertising a bridge/host-gateway name needs **no engine change**.
- **Engine is proven correct (phase-29):** negotiate → shared-socket demux (`Rtc::accepts()`) →
  `.local` skip → two-peer room, all live. The gap is purely candidate topology.

## Decisive platform finding

The Docker engine is **Colima** (a Linux VM on macOS — `docker context show` → `colima`,
`OperatingSystem` → Ubuntu 24.04 in the VM; host is Darwin). This reshapes G1's options:

- **Host networking** binds the **Colima VM's** host stack, **not** the macOS host. The host-side
  (macOS) Chromium still can't reach VM-host-network addresses directly → **fragile on this engine.
  Not recommended.**
- The reliable fix must put the **browser and the SFU on the same network** so their ICE candidates
  share one address space.

## Gap analysis

### G1 — same-host candidate topology  ·  **GAP: CONFIRMED (the phase's core)**

The phase-29 evidence: `ice=disconnected`, no `Connected`, zero `MediaData` — the browser's
bridge-`srflx` (via coturn, VM-network view) and the gateway's host-loopback `127.0.0.1` never form a
routable pair. Three candidate fixes, assessed against the Colima reality:

| Option | How | On Colima/macOS | Verdict |
|---|---|---|---|
| **A — Browser inside the Docker network** | A Playwright/Chromium **service on the compose network**; it reaches the gateway by service name; its host candidates and the gateway's `0.0.0.0:40000` share the VM bridge → a pair completes. | Most robust — both endpoints in one network, no host↔VM split. Needs: a Chromium-in-container service (e.g. `mcr.microsoft.com/playwright`), the spec pointed at the in-network gateway URL, and **secure-context** handling (serve over the gateway origin, or `--unsafely-treat-insecure-origin-as-secure-origin` for the in-network host). | **RECOMMENDED** |
| **B — bridge-reachable `MEDIA_ADVERTISE_IP`** | Advertise the gateway container's bridge / host-gateway IP so the browser's srflx can pair with it. `resolve_advertised_ip` already supports a hostname. | Host-side (macOS) browser still reaches the VM only via published ports; the srflx it gathers is the **VM's** view of the host browser — the mismatch persists unless the browser is also in-network. Helps only if combined with A. | Weak alone |
| **C — gateway host networking** | `network_mode: host` on the gateway so `127.0.0.1` is literally shared. | Shares the **VM's** loopback, not macOS's → the host browser still can't reach it. | **Rejected on this engine** |

**Recommendation: Option A (browser-in-Docker).** It is the only option that removes the host↔VM
split on Colima, and it is also the most faithful to how a real browser reaches a real SFU. Cost: a
new Chromium-in-container harness path + secure-context handling.

**Scope (A):** `compose.sovereign.yml` (a Playwright/Chromium service on the compose network),
`scripts/run-media-decode.sh` (run the spec inside that container, or `docker compose run` it), the
spec's `GATEWAY_URL` → in-network name, and secure-context flags. Likely **no `frf-*` engine change**
(the engine is proven). If a genuine engine issue surfaces once a pair completes, ADR it then.

### G2 — decoded frame + flip  ·  **GAP: OPEN (depends on G1)**

Unchanged discipline: re-run; flip `SFU_MODE=sovereign` **only** on `framesDecoded > 0`; else
re-affirm gated. Harness + runner exist; only the topology fix (G1) gates it.

### G3 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated as in phase-19…29. No new finding.

## Open question for plan/analyze (operator decision)

**Which G1 topology fix?** The assessment recommends **A (browser-in-Docker)** given the Colima
platform reality (host networking and host-side srflx both fail the host↔VM split). B and C are
weaker on this engine. The plan should confirm A (or an explicit override) before c001, because it
determines the whole harness restructure. **This is the load-bearing decision of the phase.**

## Recommended change ordering (for plan)

1. **c001 — browser-in-Docker decode harness** (Chromium-in-container service + in-network
   `GATEWAY_URL` + secure-context; the topology fix). Blocks G2.
2. **c002 — decode re-run + conditional flip** (PHASE-30-DECODE-RESULT; flip or re-affirm).
3. **c003 (optional) — G3 re-affirm** integration-gated, if not folded into c002's SECURITY §6.

## Exit posture

The phase's load-bearing risk is **G1**, and the Colima finding makes **browser-in-Docker** the clear
fix — a harness/compose restructure, not engine code. The honest gate (G2) is unchanged and
non-negotiable. Nothing here re-opens a settled ADR.
