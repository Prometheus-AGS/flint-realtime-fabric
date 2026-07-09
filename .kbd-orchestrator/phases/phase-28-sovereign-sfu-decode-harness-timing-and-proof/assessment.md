# Assessment — phase-28-sovereign-sfu-decode-harness-timing-and-proof

> Generated 2026-07-09. Narrow gap report. The three media-negotiation fixes landed (phase-27) but
> the decode run couldn't report *whether they worked* — a harness-timing bug aborts the test before
> the diagnostics print. Fix the visibility first, then read the evidence.

## The harness-timing bug — fully characterized (G1)

- `media-decode.spec.ts` runs the receiver's `connectToSovereignSfu` (`timeoutMs: 15_000`) **then**
  `probeDecodedMedia` (`timeoutMs: 20_000`) **sequentially** = up to **35s** in one test.
- Playwright's **per-test timeout defaults to 30s**. The spec sets **no** `test.setTimeout` override.
  (`playwright.config.ts:24`'s `timeout: 30_000` is the **webServer** timeout, not the test timeout —
  a red herring; the test uses the 30s default.)
- So the test is killed at 30s, **mid-probe**, before the diagnostic `expect(...)` message
  (`ice=… localCandidates=… remoteCandidates=…`) runs → the phase-27 instrumentation never surfaces.

### Additional gap — gateway logs are destroyed before they can be read (G1.3)
`run-media-decode.sh` dumps `docker compose logs gateway` **only on the health-fail path** (line 83).
On a **decode failure**, the cleanup trap runs `docker compose down -v` (line 37) and tears the stack
down before the gateway's str0m lifecycle logs can be captured. The runner must dump gateway logs
**before teardown** whenever the harness fails, so both browser + gateway evidence survive.

## Gaps (against G1–G4)

| Goal | Readiness / gap |
|------|-----------------|
| **G1** surface diagnostics | ◐ Precise: (a) drop the redundant `connectToSovereignSfu` pre-check (separate recvonly PC, no media, 15s wasted); (b) `test.setTimeout(60_000)` in the spec; (c) capture gateway logs before teardown on harness failure. |
| **G2** read + fix | ⛔ Blocked on G1 producing readable evidence. Once visible: interpret `ice`/`remoteCandidates` + gateway `session negotiated`/`connection state`/`inbound MediaData` logs, then fix the revealed media-path issue (candidate reachability, DTLS, fan-out over real socket). |
| **G3** decode + flip | 🔒 Conditional on a real `framesDecoded > 0`. |
| **G4** LiveKit/OIDC | ⏳ Carried. |

## Open questions for plan/analyze

1. **Does the receiver even need the sender?** The spec's structure is: a fake-media **sender**
   Chromium context publishes into the room, then a **receiver** context probes for a decoded frame.
   Confirm the probe's recvonly PC actually gets the sender's track via the SFU fan-out — the c001
   gateway `inbound MediaData` log on the *sender's* session + a forward to the *receiver's* session
   is the proof. If the sender's RTP never reaches the SFU, that's the real bug (not ICE).
2. **Settle-with-diagnostics vs. raise-timeout:** both. Raising `test.setTimeout` prevents the abort;
   ensuring the probe *resolves* (with `ice`/candidate counts) at its own 20s deadline guarantees the
   assertion+message run regardless. The probe already settles on its deadline — the only reason it
   didn't surface is the outer 30s killed it first (connect+probe > 30s).
3. **Is dropping the connect pre-check safe?** Yes — the probe self-connects (offer/answer/ICE/
   RoomJoin) and asserts decode; the pre-check is a redundant separate session. Removing it also
   frees 15s of budget.

## Recommendation

Plan order: **G1 (drop connect pre-check + `test.setTimeout(60s)` + capture gateway logs before
teardown)** → **G2 re-run, read the surfaced `ice`/`remoteCandidates` + gateway str0m logs, fix the
revealed media-path issue** → **G3 decode + flip-or-reaffirm** → **G4 carried**. This is the tightest
possible phase — one visibility fix, then the evidence dictates the rest. If the evidence shows a
deep media-path issue (e.g. str0m ICE over mapped UDP), ADR it and hold the gate.
