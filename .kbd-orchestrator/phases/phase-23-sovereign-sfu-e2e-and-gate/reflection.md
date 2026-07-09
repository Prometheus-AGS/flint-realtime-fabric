# Reflection — phase-23-sovereign-sfu-e2e-and-gate

> Generated 2026-07-08. The phase that was to flip `SFU_MODE=sovereign` on. It reached the
> honest decision instead: **re-affirm the gate off** — the media plane is composed, authz-gated,
> browser-drivable, and documented, but the one thing that can't be faked (a real receiver
> observing decoded media) needs infra this environment lacks.

## Delta — movement against phase goals

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — browser E2E harness** | ✅ MET | `media-webrtc.spec.ts` + `webrtc-client.ts`: a real Chromium `RTCPeerConnection` negotiates over `/ws/v1/signal` and reaches `Connected`; integration-gated. |
| **G2 — decoded-media proof** | ◐ CAPABLE, NOT PROVEN IN-ENV | `media-decode.spec.ts` + `decode-probe.ts` assert `getStats().framesDecoded > 0` (correct metric). str0m is sans-codec → decode is browser-side → needs live gateway + Chromium fake-media, absent here. `test.skip`-gated, did not run. |
| **G3 — media security boundary** | ✅ MET | ADR-007 per-participant Keto `view` at room-join (fail-closed, p23-c002); `(TenantId,room)` isolation; documented SECURITY §1/§2 Layer C/§5 (p23-c005). |
| **G4 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet in-env; production `from_env` defaults to hosted; the sovereign branch keeps its gate-off warning. No code flip. |
| **G5 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC (ADR-004 + IdP). |

**3/5 MET, 1 capable-not-proven, 1 carried. The flip was withheld on evidence, not shipped on optimism.**

## Root cause — why G2/G4 did not close

Not a defect and not under-execution: a **capability boundary of the environment + the SFU
design**. str0m is deliberately sans-codec (it forwards RTP; codecs live in endpoints), so a
"decoded frame" only exists in a browser. Proving it needs a running `SFU_MODE=sovereign` gateway
plus two Chromium peers with fake media — none present in a headless CI shell. The proof is
therefore *authored and correct* but *not runnable here*. Flipping the gate anyway would assert a
plane works beyond what has been observed — the precise failure the project has refused since
phase 16.

## Delivered changes (6/6 archived)

1. **p23-c001** — ADR-007 media-path authz (Keto `view` at room-join; enforcement in the bridge).
2. **p23-c002** — enforce it (`MediaTransportBridge` + `AuthzProvider`, fail-closed; 4 tests).
3. **p23-c003** — WS inbound→bridge path (`/ws/v1/signal` reads offers, drives the bridge, relays
   answers; shared bridge via `AppState`) + browser signaling harness (4 Rust + spec/helper).
4. **p23-c004** — decoded-media harness (`framesDecoded` probe + fake-media sender), honestly gated.
5. **p23-c005** — SECURITY §1/§2 Layer C/§5/§6 media boundary.
6. **p23-c006** — re-affirm gate off + CHANGELOG + PHASE-23-SIGNOFF; release gate re-run green.

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes | 6/6 archived |
| Final QA verdict | 6/6 ALL PASS |
| Changes needing a re-run before archive | 1 (c002 — fmt BLOCK, caught) |
| Archive aborts caught + fixed | 1 (c005 — MODIFIED→ADDED delta) |
| Release gate at close | fmt ✅ · clippy --workspace ✅ · check ✅ · str0m 24 · gateway 38 (was 30) |

### Recurring / notable

- No recurring constraint violations. The +8 gateway lib tests are all this phase's authz + WS
  inbound coverage.
- **Corrective interventions (four, none papered over):**
  1. c002 QA-gate BLOCK on `R4 fmt` — read verdict, fixed, re-ran to ALL PASS before archive.
  2. c003 `/ws/v1/signal` was outbound-only — a naïve "Connected" spec would have been a fake
     pass; surfaced to operator, scope widened to add the real inbound path.
  3. c004 str0m-is-sans-codec — a Rust decode assertion is impossible; decode moved browser-side
     with the correct `framesDecoded` metric.
  4. c005 openspec archive abort (`MODIFIED` header not found) — caught in the output, changed to
     `ADDED`, re-archived. (verify PASS ≠ archive success — separate gates.)

## Technical debt introduced

- **None structural.** The un-run decode proof is a documented, gated deferral (G2/G4), not hidden
  debt. The WS inbound path + shared-bridge `AppState` field are production code, tested.

## Lessons captured

1. **Match the proof to where the property actually lives.** A "decoded frame" cannot be asserted
   in a sans-codec Rust SFU; it must be measured in the browser (`getStats().framesDecoded`).
   Authoring the harness with the *correct metric*, honestly gated, is the right move when the
   env can't run it — not a fabricated in-Rust stand-in.
2. **verify PASS and archive success are different gates.** c005 passed `verify` but the openspec
   `archive` aborted on a spec-delta header mismatch. Read the archive output too, every time.
3. **Compose new cross-transport state on `AppState`, not inline.** The WS route needed the same
   bridge the gRPC service used; building it once in `main.rs` and sharing via `AppState` kept a
   single engine and honored the dependency rule (authz in the gateway, not the adapter).
4. **Surface capability blockers as operator decisions.** Twice (c003 WS-inbound, c004 decode
   topology) the honest path was a scope/architecture choice only the operator should make — not
   something to silently work around.

## Recommended next phase

Two coherent options; recommend **phase-24-sovereign-sfu-live-decode-and-flip**:

- Stand up a real `SFU_MODE=sovereign` gateway (compose) + a Dagger job with Chromium fake-media;
  run `media-decode.spec.ts` against it and **observe `framesDecoded > 0`**.
- **Only then** flip the `main.rs` sovereign branch (remove the warning) + update SECURITY §6 +
  CHANGELOG. This is the single remaining step to make sovereign media a shipped path.
- Fold in the carried G5 live proofs (LiveKit `realtime` cross-node; admin-ui OIDC once ADR-004 +
  IdP) as the environment to run them now exists.

Alternative if live media infra is not yet available: pivot to a different plane (e.g. the carried
LiveKit/OIDC proofs, or SDK parity work) and hold the sovereign flip until infra lands — the plane
stays honestly gated in the meantime.
