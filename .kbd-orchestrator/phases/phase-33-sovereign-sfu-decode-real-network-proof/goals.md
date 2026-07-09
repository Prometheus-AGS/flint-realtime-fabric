# Goals — phase-33-sovereign-sfu-decode-real-network-proof

> Seeded from: phase-32 (reflection `summaryForNext`). Phase-32 was a breakthrough: the Caddy TLS
> sidecar fixed the secure context, so `getUserMedia` works, sessions negotiate, and ICE reaches
> `checking` — the furthest point in the whole 24→32 arc, and the first time the media exchange runs
> end-to-end. But `framesDecoded=0`: the browser offers **only mDNS `.local` host candidates** (no
> usable STUN `srflx`), and the gateway advertises `127.0.0.1` (the browser's own loopback
> in-container) — so no **routable candidate pair** forms and ICE stalls `checking` → `Disconnected`.
> See `docs/PHASE-32-DECODE-RESULT.md`. G1 (secure context) MET; G2 (decoded frame) NOT met.
>
> **The layered peel has reached the media layer.** Phases 24→31 peeled harness/environment (auth,
> Docker, embed, timing, socket, OOM, secure context); phase-32 exposed the actual remaining WebRTC
> problem: a routable candidate pair. The honest conclusion (recorded in phase-32): a **same-host
> Docker VM is the wrong environment** for a browser↔SFU media proof — host↔VM↔bridge split +
> mDNS-hides-host-IPs + loopback-is-the-browser mean the peers never trivially share a routable
> address (this re-surfaced in four forms across phases 28→32). The durable answer is a **real
> network + a TURN relay**, not another local variant.

This phase **pivots the environment**: run the decode where the browser and SFU share a routable
network, add a TURN relay so a pair always forms, and — **only** on a real `framesDecoded > 0` — flip
`SFU_MODE=sovereign`.

**⚠️ OPERATOR INPUT LIKELY NEEDED (surface before/at assess):** this phase depends on an environment
decision that is the operator's to make — **which target?** Options: (a) a Linux CI runner with host
networking (GitHub Actions / self-hosted); (b) a deployed staging host where the gateway has a real
reachable IP; (c) stay local but add a TURN relay + advertise the gateway bridge IP as a
belt-and-suspenders. The KBD assess/plan should ask the operator which target before committing the
change set, since the work differs materially per target.

**Discipline (carried 16→32):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (phase-29). Absence of a
defect is not presence of a proof (phase-30). When the *environment itself* is the blocker, pivot to
a real one rather than peel further local variants (phase-32).

---

## G1 — Pivot to an environment where the browser + SFU share a routable network

Phase-32 evidence: `ice=checking → Disconnected`, 0 srflx, 12 `.local` skipped, gateway advertises
`127.0.0.1` (= the browser's own loopback in-container). No routable pair on the same-host VM.

- **G1.1 (operator decision — surface at assess):** pick the target — Linux/CI host with host
  networking, deployed staging, or local-plus-TURN. Consolidate the proof rig (JWKS mint, Keto seed,
  coturn, in-network Playwright, DECODE_ONLY, Caddy TLS) into **one reproducible compose profile / CI
  job** so the run is portable to the chosen target.
- **G1.2:** on the target, the **gateway advertises its real reachable IP** (`MEDIA_ADVERTISE_IP` =
  the host/bridge IP, not `127.0.0.1`), and the browser reaches it directly — so host candidates pair.
- **G1.3** No `frf-*` engine change expected (the engine is proven); harness/compose/CI + docs. ADR
  only if the target forces a genuine engine change. File-size ≤500.

**Exit:** on the target, the decode run reaches `ice=connected` + at least one session `state=Connected`
+ inbound `MediaData`/fan-out at the gateway, or the concrete blocker is diagnosed + recorded.

## G2 — Add a TURN relay so a routable pair always forms

Phase-32 evidence: STUN srflx alone did not yield a usable candidate; relay is the standard WebRTC
answer to exactly this NAT/topology failure.

- **G2.1** Add a **TURN relay** — coturn with `--external-ip` + a realm + long-term credentials (or
  the target's managed TURN). The harness `iceServers` includes the `turn:` URL (+ username/cred)
  alongside STUN, so both peers gather a **relay candidate** that routes regardless of
  host/mDNS/srflx topology.
- **G2.2** Verify the gateway accepts + pairs the relay candidate (str0m relay candidate support);
  ADR only if str0m needs relay-specific handling.

**Exit:** the decode run forms a routable pair (relay or host) → `ice=connected` → `MediaData` flows.

## G3 — Observe a real decoded frame + flip

- **G3.1** Re-run on the target; observe `getStats().framesDecoded > 0`. Capture
  `docs/PHASE-33-DECODE-RESULT.md` (browser assertion + gateway str0m logs).
- **G3.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-33-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G3.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G4 — Carried live proofs (from phase-19…32)

- **G4.1** LiveKit cross-node inbound (`realtime` feature). **G4.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**).
- `SFU_MODE=sovereign` is enabled **only** if G3 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001…008 without a new finding.
- Net-new media features beyond the decode proof + flip. Simulcast/SVC out of scope.
- Further peeling of same-host-VM candidate-topology variants without the environment pivot
  (phase-32 conclusion).

## Starting point (from phase-32)

- Media path + secure context proven live (getUserMedia works, sessions negotiate, ICE checking); the
  full proof rig exists (JWKS, Keto, coturn STUN, in-network Playwright, DECODE_ONLY, Caddy TLS).
- Blocker to clear: **a routable candidate pair** — via a real network (G1) + a TURN relay (G2).
- The environment target is an **operator decision** to surface at assess.
