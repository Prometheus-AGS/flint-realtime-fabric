# Assessment — phase-36-sovereign-sfu-ice-linux-fix

> Date: 2026-07-09. Phase-35 CI run 29057452278 proved the sovereign SFU stack builds and runs
> end-to-end on `ubuntu-latest`. The decode reached `ice=checking remoteCandidates=1` then stalled
> at `framesDecoded=0`. The gateway str0m log was empty in the artifact. This assessment maps the
> two concrete root causes and the minimal fixes that unblock both.

---

## Current State vs Goals

### G1 — Fix gateway-log capture in CI

**Status: GAP — fixable in one line.**

The script `scripts/run-media-decode.sh` correctly captures the gateway log to `/tmp/p29-gateway.log`
before `exit $harness_rc` on failure (lines 177–179). However:

1. The workflow step 72–74 (`Collect gateway logs`) then runs
   `docker compose logs gateway > gateway.log`, but by this point the EXIT trap has already fired
   `docker compose down -v` — the gateway container is gone, so the step produces an empty file.
2. The script writes to `/tmp/p29-gateway.log`, but the workflow artifact upload reads `./gateway.log`
   (workspace root) — a different path.

**Fix:** In `scripts/run-media-decode.sh`, after writing to `/tmp/p29-gateway.log`, also copy it to
`${GITHUB_WORKSPACE:-/tmp}/gateway.log`. The workflow's artifact upload step reads `gateway.log` at
workspace root — this copy lands there before the EXIT trap fires. No workflow change needed.

Alternatively: skip the in-script `down -v` when `CI=true` and let the workflow's step 73 read a
live container. Either approach works; writing to `$GITHUB_WORKSPACE/gateway.log` directly is
simpler and doesn't require restructuring the workflow.

---

### G2 — Read the Linux str0m candidate log; diagnose ICE stall

**Status: BLOCKED on G1 fix.** The gateway log was empty in run 29057452278, so the candidate
exchange is not yet visible. The diagnosis below is based on Docker network topology analysis.

**Root cause diagnosed from topology:**

- All Compose services (gateway, playwright, caddy, coturn) share the **default Compose bridge
  network** (`flint-realtime-fabric_default`, subnet ~`172.18.0.0/16`).
- `MEDIA_ADVERTISE_IP=172.17.0.1` is set in `run-media-decode.sh` when `uname -s = Linux`. This is
  the **docker0 host bridge IP** — the host side of Docker's default bridge. It is **NOT** the same
  bridge as the Compose project network.
- The Playwright container (on the Compose bridge `172.18.0.0/16`) **cannot reach `172.17.0.1`**
  across the bridge boundary. `172.17.0.1` is only reachable from containers on the legacy
  `docker0` bridge or from the host itself.
- Result: the gateway's ICE host candidate (`172.17.0.1:40000`) is unreachable from the Playwright
  container → no host candidate pair completes → `ice=checking` forever.
- **Same problem for TURN relay:** `TURN_EXTERNAL_IP=172.17.0.1` → coturn advertises
  `172.17.0.1:4916x` as the relay address → Playwright cannot reach it from the Compose bridge →
  relay candidate also fails.

This explains `localCandidates=5 remoteCandidates=1 framesDecoded=0 ice=checking`: the gateway
offers candidates, the browser offers one, but no pair survives connectivity checks.

---

### G3 — Fix the Linux candidate pairing

**Status: GAP — two targeted fixes identified.**

**Fix A — MEDIA_ADVERTISE_IP (host candidate):**

`crates/frf-media-str0m/src/config.rs:50` stores `MEDIA_ADVERTISE_IP` as a raw string and resolves
it at negotiate time via `resolve_advertised_ip()` — which calls `to_socket_addrs()`, so
**hostnames are supported**. Setting `MEDIA_ADVERTISE_IP=gateway` in `compose.sovereign.yml` causes
str0m to resolve `gateway` (Docker injects the container name into `/etc/hosts` of sibling
containers) to the gateway's own Compose-bridge IP (`172.18.0.x`). The Playwright container
**can** reach `172.18.0.x` directly (same bridge). This requires only one line change in
`compose.sovereign.yml` and zero code changes.

**Fix B — TURN_EXTERNAL_IP (relay candidate):**

`coturn`'s `--external-ip` flag requires a concrete IP, not a hostname. The cleanest fix: override
the coturn container's `entrypoint` in `compose.sovereign.yml` to `/bin/sh -c` so the `command`
string can use `$(hostname -i | cut -d' ' -f1)` to get coturn's own Compose-bridge IP at startup.
This ensures the relay candidate points to coturn's actual reachable bridge address.

File: `compose.sovereign.yml` coturn service — change `command:` array to an `entrypoint: ["/bin/sh", "-c"]`
with an inline shell string embedding all existing flags plus `--external-ip=$(hostname -i | cut -d' ' -f1)`.

**No changes needed to:** `run-media-decode.sh` (beyond the log-capture fix), the spec, the
gateway Rust code, or the Playwright network config. Both playwright and caddy stay on the bridge;
all service-name DNS still works.

---

### G4 — Re-run CI decode; flip `SFU_MODE=sovereign`

**Status: PENDING G1 + G3 fixes.** No code change in `main.rs` until `framesDecoded > 0` is
observed in a CI run on `ubuntu-latest`. Gate discipline unchanged from phases 16–35.

---

### G5 — Open PR `sovereign-sfu-decode-proof` → `main`

**Status: PENDING G4.** Branch `sovereign-sfu-decode-proof` holds all SFU work (phases 0–35,
untouched `main`). PR will be opened after the flip.

---

## Change Surface

| Change | Scope | Files touched |
|---|---|---|
| p36-c001 | Fix gateway-log capture + ICE candidate addressing | `scripts/run-media-decode.sh` (1 line: copy to `$GITHUB_WORKSPACE/gateway.log`), `compose.sovereign.yml` (2 changes: `MEDIA_ADVERTISE_IP=gateway`, coturn entrypoint shell override) |
| p36-c002 | CI re-run; read gateway log; flip `SFU_MODE=sovereign` on `framesDecoded > 0` | No code change unless the flip is earned; `docs/PHASE-36-DECODE-RESULT.md`, `docs/PHASE-36-SIGNOFF.md`, `docs/SECURITY.md`, `CHANGELOG.md` |
| p36-c003 | Open PR `sovereign-sfu-decode-proof` → `main` | GitHub — after G4 is confirmed |

---

## Open Questions

1. **Will `MEDIA_ADVERTISE_IP=gateway` resolve from WITHIN the gateway container itself?**
   Docker injects all service names into sibling containers' `/etc/hosts`, but the service itself
   may not have its own name in its own `/etc/hosts` — it would have `hostname` instead. If
   `gateway` doesn't resolve from within the gateway container, the fallback is to use the gateway
   container's own hostname (via `$(hostname -i)`) — same entrypoint pattern as coturn Fix B.
   This is verifiable by reading the gateway log after Fix A is deployed.

2. **Does the CI run logs show str0m WARN/ERROR for candidate resolution?**
   After G1 fix (log capture), the gateway log will either show successful host-candidate
   resolution (`[sovereign] host candidate 172.18.0.x:40000 gathered`) or a resolution error.
   Read-the-log is the next diagnostic step.

3. **Is `framesDecoded > 0` a sufficient flip gate, or do we need sustained decoding?**
   Phase-16 discipline: flip on `framesDecoded > 0` from a real browser receiver. One decoded
   frame is sufficient — the gate has been consistent across 20 phases.
