# Goals — phase-28-sovereign-sfu-decode-harness-timing-and-proof

> Seeded from: phase-27 (reflection). Phase-27 fixed three WebRTC media-negotiation defects
> (bidirectional trickle ICE, RoomJoin fan-out, lifecycle instrumentation) but the decode re-run
> still timed out — **and a harness-timing bug masked the new ICE diagnostics** (receiver runs a 15s
> connect pre-check + a 20s probe = 35s > Playwright's 30s test timeout, so the test aborts before
> the diagnostic assertion prints). See `docs/PHASE-27-DECODE-RESULT.md`. **Not yet active** —
> `/kbd-next-phase` flips here after `/kbd-assess`.

This is the **narrow finish that finally surfaces the truth**: make the harness report its
diagnostics, read whether ICE actually completes (the c002 fix) and whether RTP fans out (c003), fix
whatever the now-visible evidence reveals, observe a real `framesDecoded > 0`, and — **only then** —
flip `SFU_MODE=sovereign`.

**Discipline (carried 16–27):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite.

---

## G1 — Fix the harness so diagnostics surface

- **G1.1** Drop the redundant `connectToSovereignSfu` pre-check in `media-decode.spec.ts` (a
  separate recvonly PC with no media source — it burns 15s and adds no decode value; the probe
  self-connects + RoomJoins).
- **G1.2** `test.setTimeout(60_000)` (or set it in the spec) so the probe's own 20s window + the
  diagnostic `expect(...)` message run inside the Playwright budget. Ensure the probe
  **settles-with-diagnostics** (resolves with `ice`/candidate counts) before any framework timeout
  can fire — a diagnostic that can't print is useless.
- **G1.3** Optionally surface the gateway's str0m lifecycle logs (`docker compose logs gateway`)
  into the run record so both sides are visible.

**Exit:** a failed run prints `reason=… ice=<state> localCandidates=N remoteCandidates=M`, and the
gateway str0m logs are captured.

## G2 — Read the diagnostics + fix the revealed media-path issue

- **G2.1** Re-run `scripts/run-media-decode.sh` and **read** the surfaced diagnostics + gateway logs.
  Interpret: did ICE complete (`remoteCandidates>0`, `ice=connected`)? Did the SFU accept the offer,
  reach `Connected`, receive inbound `MediaData`, and fan it out?
- **G2.2** Fix whatever the evidence reveals — likely one of: candidate reachability over
  `host.docker.internal:40000/udp`, DTLS, the sender↔receiver room/track wiring, or the
  `RoomRouter` fan-out over a real socket. Keep `MediaConfig`/`RoomRouter`/bridge contracts intact
  (ADR any real design change). File-size ≤500; no library `unwrap`/`expect`.

**Exit:** the run reaches ICE `connected` + the receiver gets a track (or the concrete blocker is
diagnosed and recorded, gate held).

## G3 — Observe a real decoded frame + flip

- **G3.1** Re-run; observe `getStats().framesDecoded > 0`. Capture `docs/PHASE-28-DECODE-RESULT.md`.
- **G3.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the warning) +
  SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-28-SIGNOFF.md`. Else re-affirm gated.
  Conditional on G3.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with rationale.

## G4 — Carried live proofs (from phase-19…27)

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

- Re-opening ADR-001/003/004/005/006/007 without a new finding.
- Net-new media features beyond what the decode proof + flip require.

## Starting point (from phase-27)

- `admin-ui/e2e/{media-decode.spec.ts,support/{decode-probe,webrtc-client}.ts}` — the harness
  (trickle ICE + RoomJoin wired; timing needs fixing so diagnostics surface).
- `crates/frf-gateway/src/routes/signal.rs` — the WS trickle-relay (`spawn_local_signals_relay`).
- `crates/frf-gateway/src/media_bridge.rs` — `local_signals` method.
- `crates/frf-media-str0m/src/{driver.rs,session.rs}` — lifecycle `info!` logs.
- `scripts/{mint-e2e-jwt.mjs,run-media-decode.sh,seed-media-view.sh}` — the working runner.
- `docs/PHASE-27-DECODE-RESULT.md` — the timeout + masked-diagnostics finding + the precise fix.
