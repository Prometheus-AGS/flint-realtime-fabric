# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project is pre-1.0 and
does not yet follow strict SemVer for the workspace (the frozen `proto-v1` contract is
the stable interface).

## [Unreleased]

### Phase 36 — sovereign SFU: ICE solved on Linux; media path outstanding

Fixed the phase-35 ICE stall: str0m was not running **ICE-lite**. With that corrected, **ICE
completes on `ubuntu-latest`** (`ice=connected`) and ~1.8 MB of RTP reaches the receiver. The decode
still reports `framesDecoded=0`, so **`SFU_MODE=sovereign` stays off** — but the failure has moved
one layer downstream, from connectivity to media flow, and the remaining defect is now identified.

#### Deliverables

- **ICE fix + gateway-log capture** (p36-c001): enabled ICE-lite on the str0m session
  (`crates/frf-media-str0m/src/session.rs:229`, `Rtc::builder().set_ice_lite(true)`) — the root cause
  of the phase-32→35 `ice=checking` stall. Also addressed the phase-35 log-capture harness bug,
  **partially**: a `gateway-capture.log` now captures container output, but the intended
  `gateway.log` artifact still uploads as 0 bytes.
- **Decode evidence + gate decision** (p36-c002, partial): recorded CI run 29112243615 in
  `docs/PHASE-36-DECODE-RESULT.md` — `ice=connected`, `localCandidates=1 remoteCandidates=1`,
  `bytes≈1,800,000`, `framesDecoded=0`, `reason=timeout`, consistent across all three attempts.
  Gate **held OFF**. Gateway log shows 1,997 `inbound MediaData → fan-out` events (audio + video,
  MIDs correct — the `p36-c002h` MID fix works) but **zero PLI/FIR/keyframe requests**, `room=` empty
  on every event, and no room-join events: the receiver never registers as a room member, so the
  `p36-c002g` proactive-PLI path never fires and the receiver joins mid-GOP with no I-frame. Same
  defect class as the phase-27 `RoomJoin` finding. Tasks T1–T4 (live CI run + flip) remain open
  pending authorization to run CI.
- **Retired the proof-branch workflow** (p36-c003): all sovereign SFU work is already merged into
  `main` (PRs #3, #4; `9ba04ae` is an ancestor), and `sovereign-sfu-decode-proof` has been deleted —
  so `decode-proof.yml`'s `push` trigger scoped to that branch was dead and **no push could fire the
  decode proof**. Removed it, leaving `workflow_dispatch`
  (`gh workflow run decode-proof.yml --ref main`). Retired goal G5 (the PR it called for would be
  empty) and recorded the phase in `docs/PHASE-36-SIGNOFF.md`.

#### Gate

`SFU_MODE=sovereign` remains **OFF** (defaults to `hosted`). `crates/frf-gateway/src/main.rs` is
unchanged and still warns that end-to-end media is not proven.

### Phase 35 — sovereign SFU decode: CI/Linux proof; environment unblocked

Escalated the decode proof to GitHub Actions `ubuntu-latest` — and it **worked**: the whole stack now
builds + boots + runs the two-browser decode end-to-end on real Linux, removing the six-phase
same-host environment blocker. The decode itself is `framesDecoded=0` at `ice=checking`, so
**`SFU_MODE=sovereign` stays off** — but this is now a normal debugging loop on a working harness, not
an environmental dead-end.

#### Deliverables

- **CI decode job + Linux-portable runner** (p35-c001): `.github/workflows/decode-proof.yml`
  (`workflow_dispatch` + branch push) builds the gateway image on `ubuntu-latest`, boots the sovereign
  stack, runs the Playwright decode, asserts `framesDecoded > 0`, uploads artifacts. The runner
  auto-detects Linux and uses `172.17.0.1` (docker0 gateway) for host addressing (replaces
  `host.docker.internal`); `MEDIA_ADVERTISE_IP`/`TURN_EXTERNAL_IP` follow suit. New `media-e2e` spec.
- **CI decode run + gate decision** (p35-c002): run 29057452278 (branch `sovereign-sfu-decode-proof`,
  `main` untouched) — after two CI interpolation fixes (`FLINT_GATE_JWT_SECRET`, `TURN_SECRET`
  generated job-wide, S1-clean) — built the image, booted the stack, and ran the decode:
  `getUserMedia` works, ICE reaches `checking` (`remoteCandidates=1`), `framesDecoded=0`. Recorded in
  `docs/PHASE-35-DECODE-RESULT.md`. Found a harness bug: the script's `down -v` EXIT trap tears down
  the gateway before the workflow captures its str0m log (empty `gateway.log`).

#### Gate decision — held OFF (environment unblocked; normal debugging loop remains)

No `framesDecoded > 0`, so `SFU_MODE=sovereign` is **NOT** flipped (`main.rs` untouched). Unlike phases
28→34, the residual is **not** environmental — the proof runs end-to-end on Linux CI. Next: fix the
gateway-log capture (copy the in-script log to an artifact / skip in-script `down -v` in CI), read the
Linux candidate exchange, adjust the advertised/relay addressing, re-run, flip on a genuine decoded
frame.

### Phase 34 — sovereign SFU decode: TURN relay; gate decision + CI escalation

Added a TURN relay on the bridge and confirmed str0m accepts `typ relay`, but **0 relay candidates
reached the gateway** (coturn's relay address was the browser's own in-container loopback — the same
bridge address-confusion in a TURN variant). **`SFU_MODE=sovereign` stays off.** Across phases 28→34
the media stack is proven correct in pieces; the only recurring failure is the same-host Docker
candidate topology. **Decision: escalate the proof to CI / a real Linux host (Target B)** — no more
local same-host variants.

#### Deliverables

- **coturn TURN relay** (p34-c001): coturn upgraded from STUN-only to a TURN relay (`--realm`,
  `--lt-cred-mech`, `--user=frf:${TURN_SECRET}` with the secret **env-required, never committed**,
  `--external-ip`, relay port range). Harness `iceServers` gains a `turn:` entry with credentials in
  **both** the sender + receiver PCs (threaded via env). str0m accepts `typ relay` (`parser.rs:232`)
  — no engine change. New `media-e2e` spec requirement.
- **Decode run + gate decision** (p34-c002): sessions negotiate, `getUserMedia` works, ICE reaches
  `checking` — but 0 `typ relay` candidates reach the gateway (only mDNS `.local`, all skipped),
  `Connecting → Disconnected`, `framesDecoded=0`. coturn's `--external-ip=127.0.0.1` is the browser's
  own container loopback. `docs/PHASE-34-DECODE-RESULT.md`.

#### Gate decision — held OFF + CI escalation (recorded)

No `framesDecoded > 0`, no relay pair, so `SFU_MODE=sovereign` is **NOT** flipped (`main.rs`
untouched). This was the 11th blocker and the sixth candidate-topology form to fail on the same-host
Colima/Docker-bridge environment. Per the phase-32/33/34 escalation rule: **containerize the proof rig
and run it on `ubuntu-latest` / a Linux runner** (native host networking) — the environment is the
blocker, and a real Linux host is the answer. The media path + engine are proven; the residual is
purely the proof environment.

### Phase 33 — sovereign SFU decode: host networking attempt; gate decision + TURN pivot

Tried host networking (Target A) to give the browser + SFU one routable stack, but the gateway goes
`unhealthy` under `network_mode: host` (a Colima host-net plumbing issue) before the decode runs.
**`SFU_MODE=sovereign` stays off.** The recorded conclusion: stop peeling same-host-VM plumbing and
take the standard answer — a **TURN relay** (Target C), with CI/a real Linux host as the durable home.

#### Deliverables

- **Host-net decode stack** (p33-c001): a `compose.host-net.yml` override puts gateway + playwright +
  caddy + coturn on `network_mode: host` (using `!reset` to clear merge-inherited `ports:`), with all
  addresses on the VM's `localhost` and `MEDIA_ADVERTISE_IP=127.0.0.1`; the runner gains a `HOST_NET=1`
  mode that derives the Colima VM/host-NAT IPs. New `media-e2e` spec requirement.
- **Host-net decode run + gate decision** (p33-c002): the stack boots most services and VM→host JWKS
  reachability is confirmed, but the **gateway container is `unhealthy` under host-net** (a `/readyz` /
  `8080` host-net interaction, not JWKS, not media), cascading to the browser via `depends_on`. The
  decode never runs → `framesDecoded=0`. Recorded in `docs/PHASE-33-DECODE-RESULT.md`.

#### Gate decision — held OFF + TURN pivot recommended

No decoded frame, gateway never healthy, so `SFU_MODE=sovereign` is **NOT** flipped (`main.rs`
untouched). This was the 10th distinct blocker and the third *environment* variant to fail its own
way. Per the phase-32/33 plan's fallback rule and the "pivot when the environment is the blocker"
lesson: **next is a TURN relay (Target C)** — a relay candidate always routes regardless of topology,
on the bridge stack that boots healthy — or **CI (Target B)** as the durable home. The media path +
secure context are proven; the residual is purely a routable candidate pair.

### Phase 32 — sovereign SFU decode: HTTPS secure context; gate decision + pivot

Fixed the secure context (Caddy TLS sidecar) — `getUserMedia` now works and ICE reaches `checking`,
the furthest point in the whole decode sequence. But `framesDecoded=0`: the browser produces only mDNS
`.local` candidates (no usable STUN srflx), so no routable pair forms. **`SFU_MODE=sovereign` stays
off** — and we recommend **pivoting the proof to a real Linux/CI host** rather than peel more
candidate-topology on the local VM.

#### Deliverables

- **TLS sidecar + secure context** (p32-c001): a Caddy service fronts the plain-HTTP gateway with an
  auto self-signed internal cert (`https://caddy:8443`, transparent HTTP + WS-upgrade proxy). The
  harness `GATEWAY_URL=https://caddy:8443` gives the browser a genuine secure context; Chromium
  launches with `--ignore-certificate-errors` and the **p31 unsafe-origin flags are removed**. New
  `media-e2e` spec requirement.
- **Decode run + gate decision** (p32-c002): `getUserMedia` succeeds, sessions negotiate, and ICE
  advances to `checking` (`remoteCandidates=1`) — the shared-socket demux + `.local` skip hold under a
  real browser. But 0 srflx candidates, 0 `Connected`, 0 `MediaData`: the browser only offers mDNS
  `.local` host candidates and the gateway advertises `127.0.0.1` (the browser's own loopback
  in-container), so no routable pair forms. `docs/PHASE-32-DECODE-RESULT.md`.

#### Gate decision — held OFF + environment-pivot recommended

No `framesDecoded > 0`, no `Connected`, so `SFU_MODE=sovereign` is **NOT** flipped (`main.rs`
untouched). This was the 9th decode attempt on the same-host Colima VM. The media-path code is proven
correct (negotiate, shared-socket demux, `.local` skip, secure context all work live); the residual is
a routable-candidate-pair/networking problem the local setup keeps re-surfacing. **Recommended: run
the proof on a Linux/CI host with host networking (or add a TURN relay + advertise the gateway bridge
IP)** rather than continue peeling candidate topology locally.

### Phase 31 — sovereign SFU decode: prebuilt image + harness fixes; gate decision

Decoupled the gateway image build from the decode run (fixing the phase-30 OOM) and cleared four more
harness/infra layers so the decode now starts — but it stops at a secure-context limit
(`getUserMedia` on an insecure in-network origin). **`SFU_MODE=sovereign` stays off.** A
harness/environment detail, not a media-path defect.

#### Deliverables

- **Prebuilt-image runner** (p31-c001): `scripts/run-media-decode.sh` presence-checks the pre-built
  `flint-realtime-fabric-gateway` image and fails fast if absent (with the one-time out-of-band build
  command), instead of an unconditional heavy in-run build. Proven: the image builds cleanly run alone
  (`BUILD_EXIT=0`) — the phase-30 crash was contention, not a memory ceiling. New `media-e2e` spec req.
- **Harness fixes + decode run** (p31-c002): fixed the pnpm-workspace mount, the Playwright image/CLI
  version mismatch, and a stray `webServer: pnpm dev` (exit 127; `DECODE_ONLY=1` skips it). The run
  now boots the full stack and starts the decode spec, but the in-network browser (`http://gateway:8080`,
  insecure) has `navigator.mediaDevices === undefined`, so `getUserMedia` throws before any offer.
  The `--unsafely-treat-insecure-origin-as-secure-origin` flags did not register `mediaDevices` in the
  headless container. Recorded in `docs/PHASE-31-DECODE-RESULT.md`.

#### Gate decision — held OFF (operator-confirmed discipline)

No `framesDecoded > 0`, no session negotiated, so `SFU_MODE=sovereign` is **NOT** flipped (`main.rs`
untouched; SECURITY §6 keeps the media plane *composed but not proven*). This is a **secure-context
harness limit** (a browser needs HTTPS or a `*.localhost` origin for `getUserMedia`), not a str0m
defect — and it is upstream of the phase-29 topology fix. Carried: serve the gateway over HTTPS (or a
`*.localhost` alias) in the decode stack, then re-run.

### Phase 30 — sovereign SFU decode topology (browser-in-Docker); gate decision

Implemented the browser-in-Docker topology fix so the media-path engineering is complete — but the
decode did not execute because the Colima VM crashed building the gateway image. **`SFU_MODE=sovereign`
stays off.** The residual is environmental (build/run capacity), not a media-path defect.

#### Deliverables

- **Browser-in-Docker harness** (p30-c001): a Playwright/Chromium service on the compose network
  (in-network `gateway:8080`, `--unsafely-treat-insecure-origin-as-secure-origin`,
  `stun:coturn:3478`) so the browser and SFU share one address space on the Colima VM bridge —
  fixing the phase-29 host-vs-container candidate mismatch. New `media-e2e` spec requirement.
- **Harness plumbing fix + decode run** (p30-c002): mount the whole pnpm workspace (`workspace:*`
  deps + `node_modules` symlinks only resolve from the repo root), reuse the host install (no
  in-container install), pin the image to the installed `@playwright/test` (v1.61.0). The run then
  reached the gateway image build, where the **Colima Linux-VM crashed** (Vite/`tsc` admin-ui
  compile → `rpc error: Unavailable … EOF`, Docker daemon down) — an OOM/resource limit, not a
  media defect. Recorded in `docs/PHASE-30-DECODE-RESULT.md`.

#### Gate decision — held OFF (operator-confirmed discipline)

No receiver observed `framesDecoded > 0` (the decode never ran), so `SFU_MODE=sovereign` is **NOT**
flipped (`main.rs` gate untouched; SECURITY §6 keeps the media plane *composed but not proven*).
Notably this is **not** a new media-path blocker — every media-path defect (candidate IP, shared
socket, mDNS, STUN, topology) is fixed; the remaining obstacle is the local Colima VM's build/run
capacity. Carried: build the gateway image out-of-band / in CI (or raise VM memory), then re-run.

### Phase 29 — sovereign SFU shared socket + STUN; gate decision

Cleared both phase-28 blockers (shared demuxing socket + STUN srflx) and got ICE actually attempting
connectivity for the first time — but the decode still yields no frame, so **`SFU_MODE=sovereign`
stays off**. The gap narrowed to a single same-host networking item.

#### Deliverables

- **Shared demuxing socket** (p29-c001, **ADR-008**): `StrOmTransport` owns **one** shared `UdpSocket`
  demultiplexed to every session's `Rtc` by `Rtc::accepts()` in a single owning task (`demux.rs`),
  replacing the per-session bind that collided on the fixed media port (phase-28 B2). New test:
  two sessions on one fixed port, no `EADDRINUSE`.
- **STUN srflx path** (p29-c002): a hermetic coturn STUN service in `compose.sovereign.yml` +
  harness `iceServers` (`STUN_URL`) so Chrome gathers a routable `srflx` candidate; the `.local`
  mDNS-candidate skip is made explicit and unit-tested (phase-28 B1).
- **Decode re-run + honest gate decision** (p29-c003): two sessions negotiate on the one shared
  socket; `remoteCandidates` 0→1, `ice` `new`→`disconnected` (real progress), but neither reaches
  `Connected` and no `MediaData` flows — the browser's bridge `srflx` and the gateway's `127.0.0.1`
  host candidate are mismatched network views. `framesDecoded=0`. Recorded in
  `docs/PHASE-29-DECODE-RESULT.md`.

#### Gate decision — held OFF (operator-confirmed discipline)

`SFU_MODE=sovereign` is **NOT** flipped (`main.rs` gate untouched; SECURITY §6 keeps the media plane
*composed but not proven*). The SFU engine is proven correct (negotiate, shared-socket demux,
`.local` skip); the remaining gap is the same-host ICE candidate topology (browser-in-Docker or a
bridge-reachable advertised IP), carried forward. No decoded frame, no flip.

### Phase 28 — sovereign SFU decode-harness timing & proof; gate decision

Restored the harness diagnostics, fixed the ICE candidate-IP bug they revealed, and re-ran the
decode proof — it now reaches the media exchange itself but still yields no decoded frame, so
**`SFU_MODE=sovereign` stays off**. Two precise, well-understood str0m-SFU blockers remain.

#### Deliverables

- **Harness diagnostics restored** (p28-c001): dropped the redundant 15s connect pre-check and raised
  the Playwright timeout to 60s so the ICE-state / candidate-count assertion prints; the runner dumps
  gateway logs on failure before teardown.
- **ICE candidate-IP fix** (p28-c002): `MediaConfig.advertise_ip: Option<IpAddr>` →
  `advertise_host: Option<String>` with `resolve_advertised_ip` (literal IP passthrough; hostname →
  DNS at negotiate time; hard error, no silent `0.0.0.0` fallback). Compose advertises `127.0.0.1`.
  The gateway now creates a **valid** host candidate instead of the rejected `0.0.0.0`.
- **Decode re-run + honest gate decision** (p28-c003): `framesDecoded=0 ice=new remoteCandidates=0`;
  gateway logs show `session negotiated … advertised=… 127.0.0.1 40000 typ host`, then two new
  blockers — **B1** Chrome mDNS `<uuid>.local` host candidates rejected by str0m, **B2** fixed single
  UDP port can't bind per-session (`EADDRINUSE`). Recorded in `docs/PHASE-28-DECODE-RESULT.md`.

#### Gate decision — held OFF (operator-confirmed discipline)

`SFU_MODE=sovereign` is **NOT** flipped (`main.rs` gate untouched; SECURITY §6 keeps the media plane
*composed but not proven*). The candidate is now valid; the remaining work is two transport-design
items (shared demuxing UDP socket; STUN srflx / mDNS), carried forward. No decoded frame, no flip.

### Phase 27 — sovereign SFU media-transport debug & gate decision

Diagnosed the WebRTC media-negotiation defects, fixed three of them, and re-ran the proof — the
decode still times out, so **`SFU_MODE=sovereign` stays off**. Real progress on the media path plus
a precise next step.

#### Deliverables

- **Media instrumentation** (p27-c001): str0m driver lifecycle `info!` logs (offer/host-candidate/
  ICE-state/first-MediaData) + harness ICE-state + candidate-count diagnostics.
- **Bidirectional trickle ICE** (p27-c002): the `/ws/v1/signal` route relays the sovereign engine's
  `local_signals` (trickle candidates + connection-state) to the browser as `ice-candidate` frames;
  the harness sends its candidates up and applies inbound ones. Fixes the ICE-exchange gap that left
  the answerer gateway unable to complete ICE.
- **RoomJoin fan-out** (p27-c003): both peers `RoomJoin` the shared room so the `RoomRouter` relays
  the sender's RTP to the receiver.
- **Decode re-run** (p27-c004): **executed** — still a 30s timeout, no `framesDecoded`
  (`docs/PHASE-27-DECODE-RESULT.md`). A harness-timing bug (15s connect pre-check + 20s probe >
  Playwright's 30s test timeout) masked the new ICE diagnostics; next attempt must fix the harness
  timing so the diagnostics surface.

#### Gate decision (honest)

`SFU_MODE=sovereign` is **re-affirmed off** — decode not observed. No forced flip. Next: fix the
harness timing, read the diagnostics, fix the revealed media-path issue, observe
`framesDecoded > 0`. See `docs/PHASE-27-SIGNOFF.md`.

#### Carried (integration-gated)

- LiveKit cross-node inbound (`realtime` feature); admin-ui OIDC (ADR-004 + IdP).

### Phase 26 — gateway Docker admin-ui embed, live media path & gate decision

Fixed the gateway image build and drove the authenticated decode run all the way to the media
exchange. The honest decision holds: **`SFU_MODE=sovereign` stays off** — the WebRTC decode does
not complete yet, but the failure is finally at the media transport, not plumbing.

#### Deliverables

- **Dockerfile admin-ui embed** (p26-c001): a Node 24 build stage builds the admin UI (pnpm
  workspace, `frf-wasm` stubbed) and `COPY`s `admin-ui/dist` into the Rust build context before
  `cargo build`, so the compile-time `rust-embed` resolves. **The gateway image builds** — fixing
  the defect that blocked phases 24–25.
- **Live decode run** (p26-c002): **executed** the authenticated runner; hardened it + the harness
  against every concrete blocker each run surfaced (JWKS-port leak, `JWT_ISSUER` boot requirement,
  flint-gate `--build` stall, Keto `/admin/relation-tuples` write path, `getUserMedia`
  secure-context). Result: build → boot → **gateway healthy** → **Keto `view` granted (201)** →
  **browser harness runs**, then the WebRTC decode **times out (30s)** — no `framesDecoded`
  (`docs/PHASE-26-DECODE-RESULT.md`). The media path is reached; the exchange doesn't complete.

#### Gate decision (honest)

`SFU_MODE=sovereign` is **re-affirmed off** — the proof reaches the media exchange but no decoded
frame is observed. No forced flip. Next: a focused WebRTC media-transport investigation (ICE
state, `host.docker.internal` UDP candidate reachability, SFU RTP fan-out). See
`docs/PHASE-26-SIGNOFF.md`.

#### Carried (integration-gated)

- LiveKit cross-node inbound (`realtime` feature); admin-ui OIDC (ADR-004 + IdP).

### Phase 25 — sovereign SFU authenticated decode retry & gate decision

Made the live decode run *authenticated* (a real gateway-accepted JWT), ran it, and reached the
honest decision again: **`SFU_MODE=sovereign` stays off** — the run cleared two prior blockers but
hit a Dockerfile defect before any media flowed.

#### Deliverables

- **Authenticated decode runner** (p25-c001): the gateway verifier is RS256/JWKS, not HS256 (so a
  flint-gate token is rejected). `scripts/mint-e2e-jwt.mjs` (zero-dep node) mints a real RS256 JWT
  with the required `FrfClaims` and emits its JWKS — **verified to decode against its own JWKS**.
  `run-media-decode.sh` reworked: mint → self-serve JWKS → `GATEWAY_JWKS_URL` override → sovereign
  stack → seed `view` → run the harness with the JWT. The broken no-JWT fallback is removed.
- **Live decode run** (p25-c002): **executed** the authenticated runner. It failed
  (`docs/PHASE-25-DECODE-RESULT.md`) on a **Dockerfile defect** — `frf-gateway` `rust-embed`s
  `admin-ui/dist`, which the Dockerfile never builds/copies → the gateway image won't compile → the
  stack never came up → **no `framesDecoded` observed**. Two prior blockers (compose merge,
  HS256/RS256) are behind us; the media path itself is still untested.

#### Gate decision (honest)

`SFU_MODE=sovereign` is **re-affirmed off** — the proof was run, got further, and did not pass. No
forced flip. Next: fix the Dockerfile (build + `COPY admin-ui/dist`), re-run, observe
`framesDecoded > 0`. See `docs/PHASE-25-SIGNOFF.md`.

#### Carried (integration-gated)

- LiveKit cross-node inbound (`realtime` feature); admin-ui OIDC (ADR-004 + IdP).

### Phase 24 — sovereign SFU live-decode path & gate decision

Built the full infrastructure for a real browser↔gateway decoded-media proof, ran it, and
reached the honest decision: **`SFU_MODE=sovereign` stays off** — the live decode did not pass.

#### Deliverables

- **str0m bind/advertise config** (p24-c001): `MediaConfig` + `StrOmTransport::with_config` — bind
  `0.0.0.0` + advertise a host-reachable candidate IP on a fixed UDP port (`MEDIA_*` env);
  loopback default kept. Dissolves the loopback-bind blocker. (`session.rs` split 517→314 lines.)
- **Sovereign compose override** (p24-c002): `compose.sovereign.yml` maps the UDP media port +
  advertise IP; the hosted `compose.yml` default is untouched. Verified via `docker compose config`.
- **Authenticated-subject authz + seed** (p24-c003): the ADR-007 `view` check now authorizes the
  verified **JWT subject** (stable/seedable), threaded WS-token→envelope→bridge with a
  `from_session` fallback; `scripts/seed-media-view.sh` grants it.
- **Decode runner + honest result** (p24-c004): `scripts/run-media-decode.sh` boots the stack,
  seeds the grant, runs the browser `framesDecoded` harness. **Ran it — it FAILED**
  (`docs/PHASE-24-DECODE-RESULT.md`): a compose-merge blocker in the no-JWT fallback; the stack
  never came up; **no decoded frame observed.**

#### Gate decision (honest)

`SFU_MODE=sovereign` is **re-affirmed off** — the un-fakeable proof (a real `framesDecoded > 0`)
was attempted and did not pass. No forced flip, no relaxed proof. To flip: mint a real `E2E_JWT`
via flint-gate (or a purpose-built no-auth sovereign override), re-run the harness, observe
`framesDecoded > 0`. See `docs/PHASE-24-SIGNOFF.md`.

#### Carried (integration-gated)

- LiveKit cross-node inbound (`realtime` feature); admin-ui OIDC (ADR-004 + IdP).

### Phase 23 — sovereign SFU browser E2E, media authz & gate decision

Made the sovereign media plane authz-gated and browser-drivable, and reached the honest gate
decision: **`SFU_MODE=sovereign` stays off.** The one gate on the flip — an *in-environment*
decoded-media proof — is not satisfiable here (str0m is sans-codec; the decode is browser-side
and needs a live gateway + Chromium fake-media). Everything around it is done; the flip is not.

#### Deliverables

- **ADR-007 — media-path authz** (p23-c001): per-participant Keto `check(subject,"view",room)`
  at room-join, layered on the JWT gate + `(TenantId,room)` isolation; enforcement in the
  gateway bridge, never in the str0m adapter (rejected `(tenant,room)`-only alternative).
- **Keto view-check enforced** (p23-c002): `MediaTransportBridge` gains an `AuthzProvider`;
  `RoomJoin` is authorized **fail-closed** (deny *or* check error ⇒ no membership, no fan-out).
- **Browser → SFU WS inbound path** (p23-c003): `/ws/v1/signal` now reads inbound offers,
  carries SDP, and drives the sovereign bridge, relaying the Answer back — a browser drives the
  SFU over its natural transport. A single bridge is shared by the gRPC + WS paths via
  `AppState`. + Playwright signaling harness (`media-webrtc.spec.ts`), honestly gated.
- **Decoded-media harness** (p23-c004): a receiver `getStats().framesDecoded>0` probe +
  sender fake-media harness (`media-decode.spec.ts`). Correct metric; `test.skip`-gated — not
  run in this headless env. **Proven-capable, not proven-here.**
- **SECURITY media boundary** (p23-c005): §1 (JWT on both signal transports), §2 Layer C (Keto
  `view` at room-join + `(TenantId,room)` isolation), §5 (media trust-boundary rows), §6 status.

#### Gate decision (honest)

`SFU_MODE=sovereign` is **re-affirmed off** (production `from_env` defaults to hosted). It is
enabled only once a real receiver observes decoded media relayed by a live sovereign gateway.
Hosted (LiveKit) remains the media path. See `docs/PHASE-23-SIGNOFF.md`.

#### Carried (integration-gated)

- LiveKit cross-node inbound (`realtime` feature); admin-ui OIDC (ADR-004 + IdP).

### Phase 22 — sovereign SFU N-peer fan-out, PLI & gateway composition

Composed the sovereign media plane end-to-end in code and layer-proved it; the browser E2E
proof + `SFU_MODE=sovereign` flip are carried to phase-23 (the gate stays off).

#### Deliverables

- **N-peer fan-out proven** — `RoomRouter::forward` (already general) delivers a frame to all
  other room members; a 3-peer test confirms it.
- **PLI/keyframe forwarding** — a receiver's `Event::KeyframeRequest` is relayed
  receiver→sender via the router (a `ForwardedFrame` enum on the per-session channel) and
  applied with `Writer::request_keyframe`.
- **Gateway sovereign composition** — for `SFU_MODE=sovereign` the gateway composes
  `StrOmSignaler` (signaling) + `StrOmTransport` (media) and drives `MediaTransport` from the
  signal path via a `MediaTransportBridge` (`Offer`→`create_session` + relay the answer,
  `RoomJoin`→`join_room`, `IceCandidate`→`add_remote_candidate`). Unit-tested; **the gate is
  not flipped** — the media plane is present but end-to-end media is unproven.

#### Deferred to phase-23 (documented, gated off)

- **Browser end-to-end proof** — a Playwright + WebRTC harness proving two real peers exchange
  *decoded* media through the sovereign gateway; and the **`SFU_MODE=sovereign` flip** (with
  the media-path boundary covered in SECURITY §1–§5). `SFU_MODE=sovereign` stays gated off
  until then.
- **Live proofs (carried)** — LiveKit cross-node inbound; admin-ui OIDC once ADR-004 is
  Accepted + an IdP exists.

### Phase 21 — sovereign RTP forwarding

Built the media path on the phase-20 connected engine: two peers connect in-process, and
1-to-1 RTP forwarding is wired + layer-proven. `SFU_MODE=sovereign` stays gated off until
the end-to-end browser proof (phase-22).

#### Deliverables

- **session/driver split** — extracted the async driver loop into `driver.rs`, giving
  `session.rs` headroom (was 498/500) for the forwarding engine.
- **Two-peer DTLS-connected proof** — an offerer + answerer complete a real ICE connectivity
  check + DTLS handshake to `Connected` in-process over loopback (no browser); the phase-20
  `#[ignore]`d test is now a passing proof. The offerer is a test harness — the
  `MediaTransport` port stays answerer-only.
- **ADR-006** — the RTP fan-out architecture: a central `RoomRouter` in `StrOmTransport` +
  per-session forwarding `mpsc` (chosen over direct peer channels / a shared-`Rtc` actor),
  since each session's `Rtc` runs in an isolated driver task.
- **1-to-1 RTP forwarding** — `RoomRouter` (rooms + forwarders, bounded drop-on-full) + a
  driver `forward` arm (`Event::MediaData` → `writer(mid).write`) + `join_room`. Router
  fan-out unit-tested; the `StrOmTransport` create/join/remove wiring integration-tested.

#### Deferred to phase-22 (documented, gated off)

- **N-peer per-room fan-out, PLI/keyframe + renegotiation**, the gateway composition
  (`StrOmSignaler` + `StrOmTransport`), and the **end-to-end browser media proof**. Reaching
  connected + 1-to-1 forwarding is not the full SFU; `SFU_MODE=sovereign` stays gated off
  until media flows end-to-end.
- **Live proofs (carried)** — LiveKit cross-node inbound; admin-ui OIDC once ADR-004 is
  Accepted + an IdP exists.

### Phase 20 — sovereign SFU media loop

Built the sovereign media plane up to its scoped milestone (DTLS-connected) behind a new
port, and split the RTP forwarding media loop into phase-21.

#### Deliverables

- **ADR-005** — the sovereign media plane gets a new **`MediaTransport` port** (separate
  from the signaling-only `MediaSignaler`), so str0m implements two distinct ports
  (one-port-per-adapter). RTP forwarding explicitly deferred to phase-21.
- **`MediaTransport` port** — `create_session`/`add_remote_candidate`/`local_signals`/
  `connection_state`/`remove_session` + `ConnectionState`, defined in `frf-ports` (no impl).
- **Async per-session str0m engine** (`StrOmTransport`) — a per-session tokio task owns an
  `Rtc` + a tokio `UdpSocket` and drives str0m's sans-I/O loop (`select!` over the
  `poll_output` timeout, `recv_from`, and a command channel). Promotes the p19-c006 blocking
  transport spike into the real engine.
- **Trickle ICE wiring** — inbound `IceCandidate` envelopes → `add_remote_candidate`; the
  session's local host candidate + connection-state changes relay outbound via
  `local_signals`. (str0m has no per-local-candidate event; srflx/connectivity is STUN/
  browser-gated.)
- **DTLS-connected milestone** — the engine installs a crypto provider so DTLS can key, and
  exposes `wait_for_connected`. The two-peer connected proof is integration-gated (needs an
  offerer role / real peer), honestly `#[ignore]`d rather than faked.

#### Deferred to phase-21 (documented, gated off)

- **RTP forwarding** — `Event::MediaData` relay between peers, per-room fan-out, PLI, and the
  offerer/peer role. Reaching *connected* is not media flowing; `SFU_MODE=sovereign` stays
  gated off until it is.
- **Live proofs (carried)** — LiveKit cross-node inbound against a live server; admin-ui OIDC
  once ADR-004 is Accepted + an IdP is stood up.

### Phase 19 — sovereign media & federation completion

Landed the achievable deferred work honestly, took the two big planes as far as they can go
without a live server/browser, and split the full sovereign SFU media build into phase-20.

#### Deliverables

- **admin-ui lint clean** — fixed the 2 pre-existing errors; the `p7-smoke` env guard now
  uses `=== "true"` (the old `!!process.env[…]` treated `"false"` as truthy and would have
  run integration tests meant to be skipped).
- **ATProto outbound wired into the gateway** — the tested PDS write capability (p18-c008)
  is now reachable: the gateway builds the bridge with an outbound writer when
  `ATPROTO_PDS_URL` / `ATPROTO_PDS_IDENTIFIER` / `ATPROTO_PDS_APP_PASSWORD` are set
  (**all-or-none**, enforced at boot; the app-password is a secret, never logged).
- **LiveKit inbound-relay capability** — a `LiveKitDataSource` trait seam + relay loop
  forward server-originated payloads into the per-session streams; unit-tested. The
  libwebrtc-backed data source is behind an off-by-default `realtime` feature so the default
  gateway build stays light; the cross-node proof is integration-gated on a live server.
- **str0m live-UDP transport loop proven** — a `TransportLoop` binds a real UDP socket and
  turns the sans-I/O event loop (`poll_output`→`send_to`, `handle_input`), de-risking the
  load-bearing unknown from the phase-18 spike. Tested; no browser required.
- **ADR-004** — the admin-ui OIDC IdP decision: recommends Ory Kratos + Hydra in compose
  (flint-gate has no `authorize` endpoint; no IdP is deployed). The hardened token gate is
  the interim; no OIDC frontend code ships until the ADR is Accepted and an IdP exists.
- **Dart async transport** — deferral re-affirmed with a dated (2026-07-07) upstream check:
  `uniffi-bindgen-dart 0.1.3` remains the latest and still emits broken async codegen.

#### Deferred to phase-20 (documented, gated off)

- **Full str0m sovereign SFU media loop** — trickle ICE, DTLS/SRTP, RTP fan-out, per-session
  async tasks. `SFU_MODE=sovereign` stays gated off (no media). Built on the c006 proof.
- **Live cross-node proofs** — LiveKit inbound against a live server; admin-ui OIDC once
  ADR-004 is Accepted + an IdP is stood up.

### Phase 18 — media, federation & auth-flow

Fixed the cheap correctness bugs, then completed the scoped deferred planes (the two XL
items — full str0m SFU and LiveKit cross-node inbound — pushed to a future phase).

#### Correctness fixes

- **str0m signaling routing bug fixed** — `send_signal` now delivers to `to_session`
  (unicast) or fans out to the room via `room_id`, instead of echoing to the sender. Even
  plain signaling was broken before; a same-session test had masked it.
- Federation config now requires `FEDERATION_CHANNEL_ID` when enabled (was silently
  falling back to a per-boot random channel).
- Signal envelopes report the actually-configured `sfu_mode` instead of a hardcoded
  `Hosted`.
- Dart SDK doc drift fixed (pubspec/GENERATED.md no longer credit flutter_rust_bridge).

#### Planes & deliverables

- **Matrix inbound** — real `/sync` long-poll loop (bearer auth, since-token, backoff),
  no Tuwunel dependency. **ATProto outbound** — authenticated PDS write capability
  (`createSession` → `createRecord`), tested against a mocked PDS.
- **str0m spike** — proved the WebRTC negotiation round-trip on str0m 0.21 (offer→answer);
  the live UDP/ICE/DTLS media loop is documented as deferred (`SPIKE-FINDINGS.md`).
- **admin-ui token hardening** — JWT `exp` decode, auto-logout on expiry, token cleared on
  gateway `Unauthenticated` (not a full OIDC flow — deferred; no IdP deployed).
- **Dart transport shim** — a stable `FrfTransport` API + working `FrfCrdt` surface; the
  generated file was patched (documented) so the package compiles.

#### Deferred to a future phase (documented, gated off)

- Full str0m sovereign SFU (live media loop + RTP fan-out); LiveKit cross-node inbound
  relay; full admin-ui OIDC login (blocked on an IdP + flint-gate login endpoint); ATProto
  outbound gateway wiring (bridge capability exists; needs PDS config in `main.rs`); Dart
  async-transport bindings (blocked on an upstream `uniffi-bindgen-dart` fix).

### Phase 17 — plane completion & release audit

Independently re-audited the release claim, wired the QA gate, and completed the
pure-Rust deferred planes.

#### Verification & process

- Independent production-readiness re-audit against real code: **zero CRITICAL, zero
  HIGH** survive in the production build; `cargo check` / `clippy` (pedantic +
  `unwrap_used`) / `fmt` green.
- Removed the committed `compose.override.yml` (auto-merged dev-secret + auth-bypass
  footgun); ship `compose.override.example.yml` with placeholders instead.
- `JWT_ISSUER` is now **mandatory in production** — a release build fails to boot without
  it (dev builds keep the warning).
- Per-change QA gate wired into execute (`.kbd-orchestrator/constraints.md` +
  `qa-gate.sh`), closing the phase-16 process gap where QA never ran.

#### Deliverables

- **`EntityService`** gateway server (read/watch, auth-guarded: identity + tenant-equality
  + Keto `view`).
- **`AuthzService`** gateway server (relation check/write/delete via Keto, tenant-scoped).
  **All six proto services now have server implementations.**
- `frf-sdk-rust` binds all five non-Spine service clients (incl. Entity/Authz); FFI mobile
  subscribe is now resilient (reconnect/replay) and exposes `ack`; TS/Go/C# wrappers add
  Entity/Authz.
- `frf-cli`: `broker offsets` (read stored consumer offset) and `cdc slot create|drop`
  (manage the replication slot) — the CLI now matches its advertised surface.
- **Dart SDK** bindings generated via `uniffi-bindgen-dart` and committed (CRDT surface
  usable; async transport surface blocked by a generator bug — documented in
  `sdks/dart/GENERATED.md`, not hidden).

#### Deferred to a future phase (documented, not silently missing)

- str0m sovereign SFU real WebRTC (still signaling-only, gated off); Matrix inbound /
  ATProto outbound / LiveKit cross-node inbound relay; admin-ui interactive OIDC login;
  Dart async-transport bindings (pending an upstream `uniffi-bindgen-dart` fix).

### Phase 16 — production hardening

Closes the release-blocking gaps found in the production-readiness audit.

#### Security

- Production `compose.yml` no longer compiles in or activates the dev auth bypass; the
  release image is built without the `dev-endpoints` feature, so `DEV_NO_AUTH` cannot
  take effect there.
- App-layer tenant-equality guard on publish (JWT tenant must match the channel tenant).
- JWT issuer (`iss`) verification via `JWT_ISSUER`.
- `clippy::unwrap_used` / `expect_used` enforced workspace-wide in CI.
- Rate-limiting, request body-size limit, and CORS middleware on the gateway.
- flint-gate signing secret externalized out of the repo.
- Cedar surfaces policy-evaluation errors instead of silently denying.

#### Deliverables

- **`frf-sdk-rust`** — hand-written Rust client with reconnection/backoff and
  replay-from-offset; reconnection surfaced in the TS/Go/C# SDKs.
- **`frf-cli`** (`frf`) — operator CLI: Keto tuple seed/revoke, broker checkpoint,
  CDC status.
- FFI transport (connect/auth/publish/subscribe) for Swift and Kotlin via UniFFI.
- Gateway registers Spine, Signal, Sync, and Agent gRPC services (gRPC-web enabled);
  Sync/Agent/Signal clients bound in the SDKs.
- All SDKs generate from the frozen `proto-v1` (C# proto fork removed).

#### Operability

- `/readyz` readiness probe (Keto/JWKS/Iggy); `/metrics` (Prometheus); graceful
  shutdown on SIGTERM/SIGINT (drains in-flight requests + WS streams).
- Semantic config validation at boot (fails fast on invalid config).
- Keto schema-migration step in compose.

#### Docs

- `docs/ENVIRONMENT.md` (env-var reference), `docs/RUNBOOK.md` (operations),
  `docs/SECURITY.md` (security model), `.env.example`.
- `LICENSE` (MIT), `CONTRIBUTING.md`, `SECURITY.md` (disclosure policy), this changelog,
  and `docs/decisions/adr-003-ffi-codegen-versions.md`.

#### Deferred at the time (see Phase 17 for what shipped since)

- str0m sovereign SFU (real WebRTC); Matrix/ATProto federation protocol impls;
  `EntityService`/`AuthzService` gateway servers; Dart transport bindings; LiveKit
  cross-node inbound relay. Phase 17 delivered the `EntityService`/`AuthzService` servers
  and the Dart CRDT bindings; the rest remain deferred.

### Earlier phases (0–15)

Foundations through live Layer-3 E2E validation — see
`.kbd-orchestrator/phases/` for per-phase plans, assessments, and reflections, and
`docs/IMPLEMENTATION-PLAN.md` (RFC-FRF-002) for the phase-by-phase build plan.
