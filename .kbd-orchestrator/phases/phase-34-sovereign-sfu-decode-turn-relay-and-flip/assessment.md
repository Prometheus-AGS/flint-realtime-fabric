# Assessment — phase-34-sovereign-sfu-decode-turn-relay-and-flip

> Date: 2026-07-09. Gap report against G1 (TURN relay on the bridge stack) + G2 (decode + flip) + G3
> (carried). Grounded in the coturn config, the harness `iceServers`, and — critically — str0m's
> relay-candidate support in the source.

## Method

Read `compose.sovereign.yml` (coturn `--stun-only`), the harness `iceServers` wiring (p29/p32), and
**str0m 0.21's SDP candidate parser** to confirm relay-candidate acceptance. No code changed.

## Critical finding: str0m accepts `typ relay` candidates (the whole approach is de-risked)

`str0m-0.21/src/sdp/parser.rs:229-232` maps the candidate `typ` token:

```
string("host").map(|_| CandidateKind::Host),
string("prflx").map(|_| CandidateKind::PeerReflexive),
string("srflx").map(|_| CandidateKind::ServerReflexive),
string("relay").map(|_| CandidateKind::Relayed),
```

A `typ relay` remote candidate **parses cleanly** (it carries the coturn **relay IP** — a real
address, unlike the `.local` names that fail IP parsing). So the gateway's `add_remote_candidate`
accepts the browser's relay candidate and can pair through it. **The str0m-side risk is resolved** —
TURN is a valid fix, no engine change needed.

## Current state (what exists)

- **Furthest-working env (phase-32):** bridge + Caddy TLS (`https://caddy:8443`) + in-network
  Playwright + coturn **STUN-only**. `getUserMedia` works, sessions negotiate, ICE reaches `checking`
  — but no routable pair (browser offers only mDNS `.local`; 0 srflx; gateway advertises `127.0.0.1`
  = the browser's own loopback in-container).
- **coturn is `--stun-only`** (`compose.sovereign.yml:108`) — no relay allocations. That is the gap.
- The harness `iceServers` currently carries only `stun:` (p29-c002 / p32). Adding `turn:` is a
  small, additive change.

## Gap analysis

### G1 — TURN relay on the bridge stack  ·  **GAP: CONFIRMED (the phase core), low-risk**

- **Defect:** STUN-only yields no usable candidate in the bridge topology; no routable pair forms.
- **Required (all on the bridge stack — NOT host-net, per phase-33):**
  1. **coturn → TURN:** drop `--stun-only`; add `--realm=frf`, a **long-term credential**
     (`--user=frf:<secret>` + `--lt-cred-mech`, or `--use-auth-secret` + `--static-auth-secret`), and
     **`--external-ip`/`--relay-ip`** = coturn's reachable bridge address so the relay candidate it
     hands out is routable by the (in-network) browser + gateway. Keep STUN available.
  2. **harness `iceServers`:** add `{ urls: 'turn:coturn:3478', username, credential }` alongside the
     existing `stun:` in both the sender and receiver PCs — so each browser gathers a **relay
     candidate**.
  3. **str0m:** accepts `typ relay` (confirmed above) — no engine change; verify the relay pair
     completes in the run.
  4. **`MEDIA_ADVERTISE_IP` = the gateway bridge IP** (not `127.0.0.1`) as a second (host) path.
- **Scope:** `compose.sovereign.yml` (coturn TURN flags), `admin-ui/e2e/media-decode.spec.ts` +
  `decode-probe.ts` (`iceServers` turn: entry + the credential threaded through args), possibly the
  runner (pass the TURN secret/`MEDIA_ADVERTISE_IP`). **No `frf-*` engine change.** File-size ≤500.
- **Risk:** low-medium. The str0m side is de-risked. Remaining risk is coturn TURN config on the
  bridge (the relay IP the browser must reach — `coturn`'s bridge IP, resolvable in-network) and
  credential wiring. Standard coturn usage.

### G2 — decoded frame + flip  ·  **GAP: OPEN (depends on G1)**

Unchanged discipline: re-run; observe `ice=connected` + `framesDecoded > 0`; flip only on a genuine
frame; else re-affirm gated. This is the run where a relay candidate should finally form the routable
pair phase-32 couldn't.

### G3 — carried proofs  ·  **GAP: CARRIED (unchanged)**

LiveKit x-node, admin-ui OIDC — integration-gated.

## Open question for plan/analyze

**None blocking.** TURN-on-bridge is the operator-chosen path (phase-33 decision), str0m relay support
is confirmed, and coturn TURN is standard. The plan should note the **CI escalation** (Target B) as
the recorded fallback if TURN-on-bridge still fails, and keep the credential out of committed source
(env/`--static-auth-secret` from the runner).

## Recommended change ordering (for plan)

1. **c001 — coturn TURN + harness iceServers** (relay path on the bridge; credential via env).
2. **c002 — decode re-run + conditional flip** (relay pair → decode; PHASE-34-DECODE-RESULT; flip or
   re-affirm + CI-pivot rec).

## Exit posture

The load-bearing risk (does str0m accept a relay candidate?) is **resolved in the source** — TURN is a
valid, no-engine-change fix on the known-good bridge stack. G2 is the honest gate. If it still fails,
the recorded next step is CI (Target B), not another local variant. Nothing here re-opens an ADR;
credentials stay out of committed source.
