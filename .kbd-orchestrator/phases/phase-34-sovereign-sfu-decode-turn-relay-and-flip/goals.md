# Goals — phase-34-sovereign-sfu-decode-turn-relay-and-flip

> Seeded from: phase-33 (reflection `summaryForNext`). Phase-33 was a **wrong turn, correctly
> abandoned**: Target A (host-net) was structurally implemented (`!reset` cleared merge-inherited
> `ports:`; Colima IPs derived; VM→host JWKS reachable) but the **gateway goes `unhealthy` under
> `network_mode: host`** (a Colima plumbing issue, not JWKS/media), so the decode never ran — a
> *regression* vs. phase-32, which reached ICE `checking` on the bridge+HTTPS stack. See
> `docs/PHASE-33-DECODE-RESULT.md`. **Correction (phase-33 lesson):** pivot **TOWARD** the last
> working state, not away from it — build on the phase-32 **bridge+HTTPS** stack, which is the
> furthest-working environment.
>
> The furthest-working state (phase-32): `getUserMedia` works, sessions negotiate, ICE reaches
> `checking` — but no routable pair forms (browser offers only mDNS `.local`; STUN srflx alone did
> not yield a usable candidate; gateway advertises `127.0.0.1` = the browser's own loopback). The
> standard, environment-independent answer to "no routable host/srflx pair" is a **TURN relay**: a
> relay candidate always routes regardless of host/mDNS/srflx/loopback topology.

This phase adds a **TURN relay to the phase-32 bridge+HTTPS stack** so a relay candidate always forms,
runs the decode end-to-end, and — **only** on a real `framesDecoded > 0` — flips `SFU_MODE=sovereign`.

**⚠️ Do NOT use host-net (Target A) — it is a dead end on this box** (`compose.host-net.yml` is a
recorded, superseded artifact). If TURN on the bridge *also* fails, that is the strong signal to move
the proof to **CI / a real Linux host (Target B)** — not to attempt further local network variants.

**PREREQUISITE (operational, via `!`):** the Colima daemon may need `colima start`; the gateway image
is pre-built (p31 fail-fast enforces it). Keep concurrent VM load low.

**Discipline (carried 16→33):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (phase-29). Absence of a
defect is not presence of a proof (phase-30). Pivot when the environment is the blocker (phase-32) —
**toward the last working state, not away (phase-33)**.

---

## G1 — Add a TURN relay to the bridge+HTTPS stack so a routable pair always forms

Phase-32 evidence: `ice=checking → Disconnected`, 0 srflx, `.local` skipped, no routable pair. TURN
is the standard answer — a relay candidate routes regardless of topology.

- **G1.1:** on the **phase-32 bridge stack** (compose.yml + compose.sovereign.yml, NOT host-net),
  upgrade coturn from `--stun-only` to a **TURN relay**: `--external-ip` (the coturn container's
  reachable IP on the bridge), a `--realm`, and **long-term credentials** (`--user=frf:<secret>` or
  `--lt-cred-mech`). Keep STUN available too.
- **G1.2:** the harness `iceServers` adds the **`turn:` URL** with `username`/`credential` alongside
  the existing `stun:` — so both the sender and receiver browsers gather a **relay candidate**. Keep
  the p32 Caddy TLS secure context + in-network Playwright + DECODE_ONLY.
- **G1.3:** verify **str0m accepts + pairs the relay candidate** (the SFU must accept a `typ relay`
  remote candidate and route through it). ADR only if str0m needs relay-specific handling.
- **G1.4** `MEDIA_ADVERTISE_IP` = the gateway's **bridge IP** (not `127.0.0.1`) so its host candidate
  is also browser-reachable as a second path. File-size ≤500; no library `unwrap`/`expect`.

**Exit:** the decode run forms a routable pair (relay or host) → `ice=connected` + ≥1 session
`state=Connected` + inbound `MediaData`/fan-out at the gateway, or the concrete blocker is diagnosed +
recorded (→ CI pivot if TURN-on-bridge fails).

## G2 — Observe a real decoded frame + flip

- **G2.1** Re-run `scripts/run-media-decode.sh` (bridge + TURN); observe `getStats().framesDecoded >
  0`. Capture `docs/PHASE-34-DECODE-RESULT.md` (browser assertion + gateway str0m logs).
- **G2.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-34-SIGNOFF.md`.
  Else re-affirm gated; if TURN-on-bridge failed, recommend the **CI pivot (Target B)**. Conditional
  on G2.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G3 — Carried live proofs (from phase-19…33)

- **G3.1** LiveKit cross-node inbound (`realtime` feature). **G3.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**).
- `SFU_MODE=sovereign` is enabled **only** if G2 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001…008 without a new finding.
- **host-net (Target A) — a recorded dead end.** No further same-host-VM network variants beyond TURN.
- Net-new media features; simulcast/SVC out of scope.

## Starting point (from phase-33)

- Furthest-working env = phase-32 **bridge+HTTPS** stack (getUserMedia + ICE `checking`). The full rig
  exists: JWKS mint/serve, Keto seed, coturn (STUN-only → upgrade to TURN), Caddy TLS, in-network
  Playwright, DECODE_ONLY, prebuilt-image runner.
- Blocker to clear: **a routable candidate pair** — via a **TURN relay** on the bridge (G1).
- host-net is superseded; CI (Target B) is the fallback if TURN-on-bridge fails.
