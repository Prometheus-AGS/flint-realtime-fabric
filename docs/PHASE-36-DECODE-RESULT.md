# Phase-36 decode result — run 29112243615

> Date recorded: 2026-09-06. Evidence from GitHub Actions run
> [29112243615](https://github.com/Prometheus-AGS/flint-realtime-fabric/actions/runs/29112243615),
> head `9ba04ae` ("fix: p36-c002h — MID kind-based mapping"), started 2026-07-10T17:47:45Z,
> conclusion **failure** at step 10 "Run the decoded-media proof".
>
> **Gate: OFF.** `framesDecoded=0`. `SFU_MODE=sovereign` is not flipped.
>
> No new CI run was performed for this record — the operator directed that CI/CD workflows are not
> used for testing until all code is written. This documents the most recent existing run.

## Receiver probe (Playwright, `admin-ui/e2e/media-decode.spec.ts:202`)

All three attempts (initial + 2 retries) reported the same shape:

| Attempt | framesDecoded | bytesReceived | reason | ice | localCandidates | remoteCandidates |
|---|---|---|---|---|---|---|
| initial | 0 | 1,798,747 | timeout | connected | 1 | 1 |
| retry 1 | 0 | 1,800,403 | timeout | connected | 1 | 1 |
| retry 2 | 0 | 1,805,801 | timeout | connected | 1 | 1 |

### What this rules out

- **Not ICE.** `ice=connected` on every attempt — the phase-35 `ice=checking` stall is resolved by
  the ICE-lite fix at `crates/frf-media-str0m/src/session.rs:229`.
- **Not connectivity or routing.** ~1.8 MB of RTP reaches the receiver, consistently, across all
  three attempts. Packets arrive.
- **Not a silent/empty stream.** The byte count grows between retries, so media is genuinely flowing.

The failure is therefore **downstream of transport**: bytes arrive and the decoder produces no frame.

## Gateway evidence (`gateway-capture.log`, 642 KB)

> **Note on artifacts:** `gateway.log` in the artifact bundle is **0 bytes** — the c001 log-capture
> fix did not fully take. A second file, `gateway-capture.log`, did capture the container output and
> is the source for everything below. The c001 fix should be treated as **partially effective**, not
> complete; a follow-up should make `gateway.log` itself populate.

### Fan-out is running

1,997 `sovereign: inbound MediaData → fan-out` events from `frf_media_str0m::driver`, with both
kinds present and MIDs mapping correctly (the `p36-c002h` MID fix works):

```
sovereign: inbound MediaData → fan-out session=8d09ef8c-… room= mid=Mid(1) kind=Some(Video) bytes=2775
sovereign: inbound MediaData → fan-out session=8d09ef8c-… room= mid=Mid(0) kind=Some(Audio) bytes=40
```

### Root cause candidates — two concrete defects

**1. No keyframe is ever requested.** Across 1,997 fan-out events there are **zero** PLI, FIR, or
keyframe log lines. (`grep -icE 'pli|keyframe|fir'` returns 1, and that single match is an unrelated
`cdc::run` Postgres standby-status line.) The receiver joins mid-GOP, has no I-frame to start from,
and therefore cannot decode a single frame no matter how many bytes arrive — which is exactly the
`framesDecoded=0` / `bytes≈1.8 MB` signature observed.

This means the **`p36-c002g` "proactive PLI on room-join" fix is not firing.** The commit is on
`main` (`eb15c58`), but its trigger path is never reached.

**2. `room` is empty on every fan-out event, and there are no room-join events.** All 1,997 lines
carry `room=` with no value, while `mid` and `kind` populate correctly — so this is a genuinely empty
field, not a log-formatting artifact. Searching for room-join activity
(`grep -icE 'room.?join|join.?room'`) returns **0**.

These two findings are almost certainly the same defect: if the receiver never registers as a room
member, the room-join hook that would send the proactive PLI never runs. Media still reaches the
browser because the fan-out path does not itself depend on room membership — which is why bytes flow
while the keyframe request does not.

### Unrelated noise (not causal)

- coturn config warnings (`turnserver.conf` absent, empty `cli-password`, `STUN CHANGE_REQUEST not
  supported`) — coturn starts and relays regardless; ICE connects.
- 6 × `error 401: Unauthorized` — flint-gate, which the decode path does not exercise (the runner
  self-serves an RS256 JWKS).

## Recommended next step (G4)

Investigate why the receiver never registers a room, in this order:

1. Trace the room-join signaling path from the browser's join through `frf-gateway`'s signal service
   into `frf_media_str0m`. Establish where the room ID is dropped or the join event is not emitted.
2. Confirm whether `p36-c002g`'s PLI is gated on room-join specifically; if so, whether a
   subscriber-attach hook is the more reliable trigger.
3. Once a room is populated and a PLI is observed in the gateway log, re-run the decode proof — the
   keyframe should let the ~1.8 MB of already-arriving RTP decode.

Verification of any fix requires a live `gh workflow run decode-proof.yml --ref main`, which is
deferred under the current no-testing directive.

## Gate decision

**OFF — held honestly.** `framesDecoded=0`, so per the phase-16→35 discipline `SFU_MODE=sovereign`
is not flipped. `crates/frf-gateway/src/main.rs` is unchanged; `SfuMode` has no `Default` impl and
the "media is NOT yet proven" warning at `main.rs:298` still stands.
