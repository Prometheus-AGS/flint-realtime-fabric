# Plan — phase-27-sovereign-sfu-media-transport-debug

> Backend: **OpenSpec**. Generated 2026-07-09 from `assessment.md` + two operator decisions:
> **full trickle ICE over WS** (production-faithful — wire the gateway `local_signals`→WS relay +
> harness `onicecandidate`/`addIceCandidate`), and **the flip stays strictly conditional on a
> genuine `framesDecoded > 0`** (re-confirmed; the 11-phase discipline holds — no forced flip).
>
> Ordering: instrument first (so every fix is confirmed by evidence), fix the three defects, then
> re-run → decode → flip-or-reaffirm.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p27-c001-media-instrumentation` | **G1.** Add lifecycle tracing to the str0m driver (`driver.rs`): `info!` on offer-accepted/host-candidate/ICE-state-change/first-`MediaData`/forward-to-room. Add ICE-state + candidate-count logging to the harness helpers (`webrtc-client.ts`/`decode-probe.ts`) + a bounded timeout that reports the **last observed state**. So the next run says *where* it stalls. | low |
| c002 | `p27-c002-trickle-ice-over-ws` | **G2a — FIND-1/2.** Wire bidirectional trickle: (Rust) the `/ws/v1/signal` route subscribes to `MediaTransport::local_signals(session)` for the sovereign session and streams the gateway's trickle candidates + connection-state out over the WS as `ice-candidate` frames; (harness) `pc.onicecandidate` → send `ice-candidate` frames up; inbound `ice-candidate` → `pc.addIceCandidate`. Rust tests for the relay; keep bridge/one-port contracts intact. | **high** (the real Rust + harness fix) |
| c003 | `p27-c003-room-join-fanout` | **G2b — FIND-3.** The harness sends a `RoomJoin` after the offer so sender + receiver share `e2e-decode-room` and the `RoomRouter` fans the sender's RTP to the receiver. Confirm the `create_session`→`join_room` sequence via the c001 gateway logs (sender `MediaData` reaches the receiver's session over the real socket). | med |
| c004 | `p27-c004-decode-run-and-flip` | **G3.** Re-run `scripts/run-media-decode.sh`; observe `getStats().framesDecoded > 0`; record `docs/PHASE-27-DECODE-RESULT.md`. **If genuine pass:** flip `main.rs` sovereign branch + SECURITY §6 (media → functional) + CHANGELOG + PHASE-27-SIGNOFF. **Else:** re-affirm gated with the fresh detail. + **G4 carried**. | med (honest branch) |

## Dependencies

- c002/c003 depend on c001 (instrumentation confirms the stall + verifies each fix).
- c004 depends on c002 + c003 (ICE must complete *and* fan-out must occur for a decoded frame).
- **c004 flips only on a genuine `framesDecoded > 0`** — never a relaxed/worked-around proof
  (operator-confirmed, carried).

## Notes

- **FIND-1/2 are one change (c002)** — trickle is bidirectional; the harness side and the gateway
  `local_signals`→WS side are the same fix. The gateway already bundles its host candidate in the
  answer, but full trickle is the chosen (production-faithful) path per the operator decision.
- **Dependency rule:** the WS-relay wiring lives in `frf-gateway` (`routes/signal.rs` +
  `media_bridge`/`AppState` access to the transport's `local_signals`); the str0m adapter is
  untouched beyond added tracing. One-port-per-adapter intact.
- **`local_signals` is on the `MediaTransport` port** (`media_transport.rs:66`) — the route can
  subscribe to it for the sovereign session; the bridge holds the `StrOmTransport`.
- **Honest guard (operator-confirmed):** if a genuine decoded frame can't be reached cleanly this
  phase, land c001–c003 + the diagnosis and **re-affirm the gate off** — no forced flip.
- Each change passes the QA gate; **read the verdict AND the archive output**; **seed the openspec
  change dir at the start of every apply** (phase-26 c002 lesson).
- First change to apply: **`p27-c001-media-instrumentation`**.
