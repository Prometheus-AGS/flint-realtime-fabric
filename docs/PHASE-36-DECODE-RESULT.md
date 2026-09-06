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

### Root cause — the proactive PLI targets the wrong MID

> **Correction (2026-09-06).** An earlier revision of this document inferred from the captured log
> that the receiver never registers a room, because the log showed no room-join events and `room=`
> empty on every fan-out line. **That inference was wrong.** The capture covers only 29 seconds
> (17:58:20–17:58:49) and contains *only* `frf_media_str0m` lines — it begins after WS setup, the
> offer/answer exchange, and room-join have already happened, so their absence is a truncation
> artifact. The empty `room=` is likewise benign: `SessionMeta.room_id` is deliberately left blank
> (`session.rs:278` — "the room is unknown to the transport port"), so it is a cosmetic logging gap,
> not routing state. Room registration demonstrably *does* work: fan-out delivered ~1.8 MB to the
> peer, which is only possible once both sessions share a room.

**The real defect: the proactive PLI asks for a keyframe on the sender's audio track.**

`join_room` (`session.rs`) sends its proactive PLI with a hardcoded `Mid::from("0")`. On the
receiving side, `apply_forwarded` (`driver.rs`) resolved the writer with `rtc.writer(req.mid)` —
honouring the *requester's* MID number against the *sender's* `Rtc`.

The run's own log establishes the MID topology for this browser:

```
1426  mid=Mid(0) kind=Some(Audio)
 571  mid=Mid(1) kind=Some(Video)
```

MID 0 is **audio**. So the proactive PLI landed on the sender's audio track, `request_keyframe`
rejected it as not applicable, and the failure was logged at `debug` — invisible at the run's INFO
level. No video keyframe was ever produced, the receiver stayed on P-frames, and `framesDecoded`
remained 0 while bytes accumulated. That is exactly the observed signature.

This is the **same MID-crossing class as `p36-c002h`**, which fixed it for *media* forwarding
(`write_forwarded` resolves the destination MID by kind) but left the *keyframe* path trusting the
requester's number.

### Secondary defect — room membership leaked on re-registration

`RoomRouter::register` inserted into the new room without removing the session from its previous
one. `create_session` registers each session under its own id as a room and `join_room` then
re-registers it into the shared room, so the stale entry persisted in the old room's member set.
Not the cause of `framesDecoded=0` (`membership` is overwritten, so `fan_out` resolves the correct
room), but it would misroute media for any session that changes rooms.

### Unrelated noise (not causal)

- coturn config warnings (`turnserver.conf` absent, empty `cli-password`, `STUN CHANGE_REQUEST not
  supported`) — coturn starts and relays regardless; ICE connects.
- 6 × `error 401: Unauthorized` — flint-gate, which the decode path does not exercise (the runner
  self-serves an RS256 JWKS).

## Fix applied (2026-09-06)

1. **`driver.rs` — resolve the keyframe target by kind.** New `keyframe_target_mid` helper picks
   *this* session's video MID instead of trusting `req.mid`, mirroring what `write_forwarded`
   already does for media. The requester's MID is used only as a fallback when the session has no
   video track registered yet. Unit-tested against the run's real MID topology (0=audio, 1=video).
2. **`driver.rs` — promote the applied-PLI log to INFO.** Whether the PLI actually reached a sender
   is the decisive diagnostic for this failure, and it was previously only visible at `debug`.
3. **`session.rs` — log `join_room`.** A join now logs at INFO on success and **warns** when the
   session is unknown (previously a silent no-op, which made "no PLI" indistinguishable from "PLI
   fired but was rejected"). The placeholder `Mid("0")` is retained but documented as ignored
   downstream.
4. **`room.rs` — fix the membership leak.** `register` now removes the session from its previous
   room before inserting it into the new one.

## Verification

**Unit tests: 35 passed, 0 failed** (`cargo test -p frf-media-str0m`), including 4 new ones.

The regression tests were confirmed to be genuine by reverting both fixes and re-running: **3 of the
4 fail without them** (`test result: FAILED. 32 passed; 3 failed`), and all pass with them. The
fourth — the no-video-track fallback — passes either way by design, since it exercises the path the
fix leaves unchanged.

**The end-to-end claim remains unverified.** Confirming `framesDecoded > 0` requires
`gh workflow run decode-proof.yml --ref main`, deferred under the current no-testing directive.

### Falsifiable prediction for the next run

The next decode run should log, in order:

```
sovereign: room joined → proactive PLI to co-room senders
sovereign: keyframe request applied → PLI to sender
```

- **Both appear and `framesDecoded > 0`** → diagnosis confirmed; flip the gate.
- **Both appear but `framesDecoded=0`** → the PLI now reaches the sender and the diagnosis is
  *incomplete*. Next suspect: PT/codec translation in `write_forwarded` (`match_params` falling back
  to the sender's PT when the receiver negotiated a different one).
- **Neither appears** → the join itself is not reaching the bridge; check for the new
  `join_room for unknown session` warning, which now makes that case visible.

## Gate decision

**OFF — held honestly.** `framesDecoded=0`, so per the phase-16→35 discipline `SFU_MODE=sovereign`
is not flipped. `crates/frf-gateway/src/main.rs` is unchanged; `SfuMode` has no `Default` impl and
the "media is NOT yet proven" warning at `main.rs:298` still stands.
