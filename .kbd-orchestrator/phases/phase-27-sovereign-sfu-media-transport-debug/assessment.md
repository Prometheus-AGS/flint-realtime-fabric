# Assessment — phase-27-sovereign-sfu-media-transport-debug

> Generated 2026-07-09. Debug-scoping gap report. The pipeline reaches the media exchange; the
> WebRTC decode times out. Static analysis already surfaces **three concrete, load-bearing
> defects** in the media-negotiation path — likely the actual causes, not just missing logs.

## Findings (static — strong candidates for the stall)

### FIND-1 — No ICE candidate exchange in the harness  ·  **BLOCKER (very likely THE cause)**
`decode-probe.ts` and `webrtc-client.ts` send only the **SDP offer** and apply the answer. They have
**no `pc.onicecandidate` handler and never call `pc.addIceCandidate`**. So:
- The browser **never sends its host/srflx candidates to the gateway.**
- The browser **never processes the gateway's trickle candidates.**
The gateway is the **answerer** — without the browser's candidate it does not know where to send
media, so ICE stays at `checking` → the exact 30s timeout observed. The gateway's own host
candidate *is* bundled in the answer SDP (`session.rs:173` `add_local_candidate` before
`accept_offer`), so the browser may know the gateway's address; the missing direction is
browser→gateway.

### FIND-2 — Gateway trickle candidates are never relayed to the browser  ·  **BLOCKER**
`media_bridge.rs` documents that "candidates + connection-state changes flow out of `local_signals`
(relayed **separately**)" — but the `/ws/v1/signal` route only streams
`signaler.subscribe_signals` (the **MediaSignaler**), **not** `StrOmTransport.local_signals`. So the
sovereign engine's outbound trickle ICE + `ConnectionState` **never reach the browser over WS**. The
"relayed separately" path was never wired for the WS transport.

### FIND-3 — No `RoomJoin`, so `RoomRouter` never fans out  ·  **BLOCKER for decode**
Both sender and receiver send an **`offer`** → each becomes its own SFU session registered in its
**own default room = its session id** (`session.rs` registers under `session_id.to_string()`).
Neither sends a **`RoomJoin`** to co-locate them in `e2e-decode-room`, so even if ICE completed the
`RoomRouter` would never relay the sender's RTP to the receiver — they are in different rooms. The
`roomId` field in the offer payload is not a join.

> Net: **even the connection can't complete (FIND-1/2), and even if it did, no media would fan out
> (FIND-3).** All three must be fixed for a decoded frame.

## Instrumentation gaps (needed to confirm + verify the fixes)

- **str0m driver** (`driver.rs`) updates `ConnectionState` on `Event::Connected`/ICE change but does
  **not `info!`-log** them, nor log `MediaData` receipt/forward. The gateway side is a black box
  during a run — add lifecycle tracing (offer accepted, host candidate, ICE state, first
  `MediaData`, forward to room).
- **Harness** reports only a terminal `reason` string; it should log the **ICE-state trajectory**
  and candidate counts so a failure says *where* it stalled.

## Gaps (against G1–G4)

| Goal | Readiness / gap |
|------|-----------------|
| **G1** instrument + diagnose | ◐ Three defects already identified statically; instrumentation confirms + guards them. |
| **G2** fix the media transport | ⛔ Real work: wire browser↔gateway ICE trickle (harness `onicecandidate`/`addIceCandidate` + FIND-2 gateway `local_signals`→WS relay), and add the `RoomJoin` so sender+receiver share the room. Keep `MediaConfig`/`RoomRouter`/bridge contracts intact. |
| **G3** decode + flip | 🔒 Conditional on G2 producing `framesDecoded > 0`. |
| **G4** LiveKit/OIDC | ⏳ Carried. |

## Open questions for plan/analyze

1. **Trickle vs bundled-only**: is host-candidate bundling in the answer SDP enough (no trickle) if
   the *browser* also bundles its candidate in the offer (non-trickle / `iceGatheringState:
   complete` before send)? A non-trickle harness (wait for gathering, then send offer with
   candidates inline) may be simpler than wiring bidirectional trickle over WS. Decide in plan.
2. **Sender/receiver topology**: does the receiver need its own offer *and* a `RoomJoin`, and the
   sender likewise? Confirm the `create_session`→`join_room` sequence the bridge expects (offer
   creates the session; a subsequent `RoomJoin` regroups it) and have the harness send both.
3. **Fan-out over a real socket**: `RoomRouter` fan-out is only proven in-process — G2's fix must be
   validated over the real UDP path (the gateway logs from G1 confirm the sender's `MediaData`
   reaches the receiver's session).

## Recommendation

Plan order: **G1 instrument (harness ICE-state + str0m driver lifecycle logs)** → **G2 fix the
three defects** (ICE candidate exchange — likely non-trickle bundled offer is simplest; the
`local_signals`→WS relay if trickle is needed; the `RoomJoin` for fan-out) → **G3 re-run → observe
`framesDecoded > 0` → flip-or-reaffirm** → **G4 carried**. The static diagnosis is strong enough
that G2 has concrete targets; confirm each with the G1 instrumentation before/after. If a fix needs
a real design change (e.g. str0m trickle-over-WS is substantial), ADR it and keep the gate off until
a decoded frame is observed.
