# Goals — phase-27-sovereign-sfu-media-transport-debug

> Seeded from: phase-26 (reflection). The entire pipeline now works up to the media exchange: the
> gateway image builds, the stack boots, the gateway is healthy, the ADR-007 Keto `view` is
> granted, and the browser harness runs. Phase-26 reached the media path — then the **WebRTC decode
> timed out (30s)**: offer/answer/ICE/DTLS/RTP did not complete to a decoded frame
> (`docs/PHASE-26-DECODE-RESULT.md`). **Not yet active** — `/kbd-next-phase` flips here after
> `/kbd-assess`.

This is the **focused WebRTC media-transport investigation**. Eleven phases of plumbing are behind
us; the remaining unknown is genuinely in the media path — where does the exchange stall, and why.
Find it, fix it, observe `framesDecoded > 0`, and — **only then** — flip `SFU_MODE=sovereign`.

**Discipline (carried 16–26):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite.

---

## G1 — Instrument the media path (find where it stalls)

- **G1.1** Add diagnostics to the harness (`webrtc-client.ts` / `decode-probe.ts`): log
  `iceConnectionState` / `connectionState` transitions, gathered + received ICE candidates,
  DTLS/selected-candidate-pair, `ontrack` firing, and a bounded timeout that reports the **last
  observed state** (no candidates? ICE stuck at `checking`? track never arrived?).
- **G1.2** Capture the **gateway media logs** during the run (str0m `driver`/`session` tracing):
  did it receive the offer, produce an answer, gather a host candidate on the advertised IP,
  reach `Connected`, and forward RTP?

**Exit:** a concrete diagnosis of the stall point (browser-side ICE, gateway-side, or fan-out).

## G2 — Fix the media-transport defect

- **G2.1** Address the diagnosed cause. Likely candidates (confirm before fixing):
  - **ICE reachability** — the advertised `host.docker.internal:40000/udp` candidate is not
    reachable from the host browser, or the container doesn't bind/receive on it (verify the p24
    `MediaConfig` advertise-IP + UDP mapping *in practice*, not just config).
  - **Fan-out / room join** — sender + receiver must share the room and the `RoomRouter` must relay
    the sender's RTP to the receiver **over the real network** (this path was only proven in-process).
  - **str0m negotiation** — offer/answer/DTLS specifics over a real socket vs the loopback tests.
- **G2.2** Whatever the fix, keep the `MediaConfig`/`RoomRouter`/bridge contracts intact (or ADR any
  real design change). File-size ≤500; no library `unwrap`/`expect`.

**Exit:** the diagnosed defect is fixed and re-run reaches ICE `connected` + track received.

## G3 — Observe a real decoded frame + flip

- **G3.1** Re-run `scripts/run-media-decode.sh`; observe `getStats().framesDecoded > 0` on the
  receiver. Capture as `docs/PHASE-27-DECODE-RESULT.md`.
- **G3.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the warning) +
  SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-27-SIGNOFF.md`. Else re-affirm gated
  with the fresh detail. Conditional on G3.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with rationale.

## G4 — Carried live proofs (from phase-19…26)

- **G4.1** LiveKit cross-node inbound (`realtime` feature). **G4.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply** — the phase-26 c002 lesson).
- `SFU_MODE=sovereign` is enabled **only** if G3 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001/003/004/005/006/007 without a new finding.
- Net-new media features beyond what the decode proof + flip require.

## Starting point (from phase-24/25/26)

- `scripts/{mint-e2e-jwt.mjs,run-media-decode.sh,seed-media-view.sh}` — the working authenticated
  runner (reaches the media exchange).
- `admin-ui/e2e/{media-decode.spec.ts,support/{decode-probe,webrtc-client}.ts}` — the harness (to
  instrument).
- `compose.sovereign.yml` — sovereign UDP media override + `JWT_ISSUER` + `GATEWAY_JWKS_URL`.
- `Dockerfile` — builds the gateway image with the admin UI embedded.
- `crates/frf-media-str0m/src/{config.rs,session.rs,driver.rs,room.rs}` — the media engine.
- `docs/PHASE-26-DECODE-RESULT.md` — the recorded media-transport timeout + investigation plan.
