# Security Model — flint-realtime-fabric

How the fabric authenticates callers, isolates tenants, and authorizes actions.
The whole premise is **sovereign auth + tenant isolation**; this document is the
authoritative description of how that is enforced. Companion docs:
[`ENVIRONMENT.md`](ENVIRONMENT.md), [`RUNBOOK.md`](RUNBOOK.md).

> To report a vulnerability, see [`../SECURITY.md`](../SECURITY.md) (disclosure policy).
> This document is the *design*; that file is the *process*.

---

## 1. Authentication — the JWT boundary

**Every request is authenticated at the gateway boundary before any domain logic
runs.** There is no trusted-network assumption.

- Callers present a **JWT bearer token**. HTTP/gRPC calls use
  `Authorization: Bearer <jwt>`; browser WebSocket endpoints (which cannot set
  headers) pass the token as a `?token=` query parameter, verified the same way.
- The gateway verifies the JWT with `OryIdentityVerifier` against the JWKS served
  by **flint-gate** (`GATEWAY_JWKS_URL`). It checks:
  - **Signature** (RS256) against the JWKS (with a one-time JWKS refresh on key
    rotation).
  - **Audience** (`JWT_AUDIENCE`).
  - **Issuer** (`JWT_ISSUER`) — when set, tokens with a missing or mismatched `iss`
    are rejected (p16-c004). When unset, the gateway logs a warning and does NOT
    validate the issuer; **set it in production** so only your IdP's tokens are
    trusted.
- **Claims are never trusted downstream unverified.** The verified `tenant_id`,
  `subject`, and `session_id` come only from the validated token.
- **flint-gate** is the identity edge: it mints and signs outbound JWTs (secret
  `FLINT_GATE_JWT_SECRET`) and publishes the JWKS the gateway verifies against.

### The DEV_NO_AUTH bypass (dev only — cannot exist in production)

A `dev-endpoints` build with `DEV_NO_AUTH=true` skips JWT verification for
publish/subscribe (for local/CI integration without minting real JWTs). This is
**compile-gated**: the production release image is built **without** the
`dev-endpoints` feature, so `dev_no_auth()` and the bypass branches do not exist in
the production binary (p16-c001). The production `compose.yml` sets neither the
feature nor the env var.

---

## 2. Tenant isolation

Tenant isolation is enforced in **two independent layers**, defense-in-depth:

### Layer A — per-event authorization (Keto)

Visibility is governed by **Ory Keto** (a Zanzibar-style relation store), not
application code. Two checks:

1. **Subscribe-time:** before a subscription opens, Keto is asked
   `check(subject, "subscribe", channel)` — a subject who may not subscribe to the
   channel is rejected.
2. **Per delivered event:** every envelope on a subscription's fan-out stream is
   filtered by `check(subject, "view", envelope.id)` scoped to the subscriber's
   `tenant_id` (`frf-app/src/subscribe.rs`). An event the subject/tenant may not
   `view` is silently dropped from that subscriber's stream — it never leaves the
   gateway.

   > Performance note: this is one Keto check per delivered event. Because event
   > ids are globally-unique v4 UUIDs, the decision is object-scoped and cannot
   > leak across tenants. At scale, cache `view` decisions at subscribe time to
   > bound per-event Keto latency (a known optimization, tracked separately).

### Layer B — app-layer tenant-equality guard (publish)

Beneath Keto, the publish use-case rejects a mismatch between the caller's verified
JWT `tenant_id` and the target channel/envelope `tenant_id`
(`frf-app/src/publish.rs`, p16-c002) — **before** the Keto check or the broker
append. So a caller authenticated for tenant A cannot forge a write into tenant B's
channel even if a stray relation tuple would permit it. The verified JWT tenant is
authoritative; the caller-supplied envelope tenant must match it.

> Subscribe has no equivalent channel-tenant equality check because a channel's
> owning tenant is not resolved at the app layer (subscribe carries only a bare
> `ChannelId`); the per-event Keto `view` filter is the read-path tenant boundary.

### Layer C — media path (sovereign SFU)

The sovereign SFU media plane applies the same read-boundary model as the event spine
(ADR-007, p23-c002):

1. **Authenticated channel.** Both media/signaling transports are JWT-gated at the boundary.
   The gRPC signal service rejects a request with no Bearer token as `UNAUTHENTICATED` before
   any domain logic; the browser `/ws/v1/signal` WebSocket (which cannot set headers) verifies
   the `token` query param and takes the tenant from the verified claims — an
   absent/invalid token is refused.
2. **Per-participant Keto `view` at room-join.** For `SFU_MODE=sovereign`, before a session
   enters a room's fan-out, `MediaTransportBridge` calls
   `check(subject, "view", room)` (ADR-007). A negative result **or a check error** denies the
   join **fail-closed** — the session is never added to room membership and receives no media.
   The subject is the authenticated, server-assigned session id, never a caller-supplied field.
   This is the subscribe-time analogue of the event spine's `view` filter; it is one check per
   participant per room (cached for the room lifetime), not per RTP packet.
3. **Tenant isolation is structural.** `RoomRouter` keys rooms by `(TenantId, room)` and the
   bridge threads the verified `tenant_id` in; fan-out can only reach members sharing the exact
   `(tenant, room)` key, so **cross-tenant media fan-out is impossible** by construction.

> Enforcement lives in the gateway (`frf-gateway/src/media_bridge.rs`), composed over the same
> `AuthzProvider` seam publish/subscribe use. The str0m adapter carries no authz dependency
> (one-port-per-adapter + the absolute dependency rule). See ADR-005/006/007.

---

## 3. Keto vs Cedar — separate responsibilities

The system uses **two** policy engines with **distinct, non-overlapping** roles.
Do not conflate them.

| | **Keto** (Zanzibar) | **Cedar** |
|--|--------------------|-----------|
| Governs | **Visibility / relationships** — who may `view`/`subscribe`/`publish` to which object | **Mutation ACTION policy** — whether an action (e.g. `Publish`) is permitted |
| Question | "Is subject S related to object O by relation R?" | "Is this action allowed by policy?" |
| Where | Subscribe-time + per-event `view` fan-out filter | The publish route's action check |
| Model | Relation tuples (fine-grained, per-object) | Policy set (`policy.cedar`), action-level |

**Cedar honesty (p16-c007):** the bundled `policy.cedar` explicitly permits the
mutation actions in use (`Publish`, `Subscribe`) and denies others — it is not a
blanket allow-all, and `PolicyEngineMode::None` (the default) logs "no-op (all
permitted)" explicitly rather than silently. Cedar evaluates against an empty entity
store, so **action-level** policies work; a policy that references principal/resource
**attributes** cannot resolve them and surfaces an **authorization error** (logged
and propagated), never a silent deny. Full attribute ABAC requires an entity store
(out of scope for v1).

---

## 4. Transport & request-level controls

- **Rate limiting** — a global token-bucket cap (`RATE_LIMIT_PER_SEC` /
  `RATE_LIMIT_BURST`) bounds request volume (p16-c005).
- **Body-size limit** — `MAX_BODY_BYTES` (default 1 MiB) caps per-request memory.
- **CORS** — an explicit exact-match allowlist (`CORS_ALLOWED_ORIGINS`); empty means
  no cross-origin browser access.
- **Secrets** — never committed; sourced from env / a secret manager
  (`FLINT_GATE_JWT_SECRET`, broker/DB/LiveKit creds). See `RUNBOOK.md` §2.
- **Logging** — JWT payloads, relation tuples, and tenant ids are not logged in
  debug output.

---

## 5. Trust boundaries — summary

| Boundary | Enforced by |
|----------|-------------|
| Unauthenticated → authenticated | JWT verify at the gateway (flint-gate JWKS) |
| Cross-tenant write | JWT-tenant == channel-tenant guard (publish) + Keto |
| Cross-tenant read | Per-event Keto `view` check on fan-out |
| Unauthorized subscribe | Subscribe-time Keto `subscribe` check |
| Unauthorized media room-join | Room-join Keto `view` check (ADR-007), fail-closed |
| Cross-tenant media fan-out | `RoomRouter` `(TenantId, room)` keying — structurally impossible |
| Unauthenticated media/signal | JWT verify on gRPC signal + `/ws/v1/signal` (`token` param) |
| Disallowed mutation action | Cedar action policy |
| Request-volume / body abuse | rate-limit + body-size + CORS layers |

Every downstream component assumes the JWT boundary holds — which is why the
production image cannot contain the auth bypass (§1).

## 6. Plane status — what functions vs. what is still deferred

Updated after phase-22. Every plane below is either functional (and its boundary is
covered by §1–§5) or gated off / labeled unimplemented — no half-secured live path.

### Federation (gated behind `FEDERATION_ENABLED`; requires `FEDERATION_TENANT_ID` **and**
`FEDERATION_CHANNEL_ID` — the gateway refuses to boot without both)

- **Matrix inbound** — ✅ functional (phase-18): long-polls the Client-Server `/sync`
  endpoint, bearer-authenticated, and projects room events onto the spine.
- **Matrix outbound** — ✅ functional: authenticated HTTP PUT to the room send endpoint.
- **ATProto inbound** — ✅ functional: authenticated Jetstream WebSocket subscribe.
- **ATProto outbound** — ✅ functional (phase-18 capability, wired in phase-19 p19-c002):
  authenticated PDS write (`createSession` → `createRecord`). The gateway builds the
  ATProto bridge with an outbound writer when `ATPROTO_PDS_URL`, `ATPROTO_PDS_IDENTIFIER`,
  and `ATPROTO_PDS_APP_PASSWORD` are set — **all-or-none**, enforced at boot; the
  app-password is a secret (env/secret manager only, never logged). Without those vars the
  bridge stays inbound-only.
- **LiveKit inbound cross-node relay** — ◐ **capability present, live proof
  integration-gated (phase-19 p19-c005).** The inbound-relay plumbing exists behind a
  `LiveKitDataSource` seam and is unit-tested; the libwebrtc-backed data source that
  subscribes to the LiveKit server data channel is behind the off-by-default `realtime`
  cargo feature (so the default gateway build stays light) and its cross-node proof is an
  integration test against a live LiveKit server. Until that feature is enabled + deployed,
  cross-node inbound signals do not relay.

### Media (str0m sovereign SFU)

- ◐ **Media plane composed, layer-proven, authz-gated & browser-drivable; in-env decoded-media
  proof still deferred.** The sovereign SFU is fully composed: two peers reach DTLS-connected
  in-process (p21-c002); **N-peer per-room fan-out** (p22-c001) + **PLI/keyframe forwarding**
  (p22-c002) + **1-to-1 RTP forwarding** (ADR-006; p21-c004); the **gateway drives
  `MediaTransport` from the signal path** (p22-c003); a **browser peer can drive the SFU over
  its natural `/ws/v1/signal` WebSocket** (inbound offer → bridge → answer; p23-c003); and the
  **media-path security boundary is now documented + enforced** — JWT on both signal transports
  (§1), a per-participant Keto `check(subject,"view",room)` at room-join (ADR-007, p23-c002,
  fail-closed), and `(TenantId,room)` tenant isolation (§2 Layer C, §5).
  - **Phases 24–26 built the live path and ran the proof repeatedly; it has NOT passed, but the
    run now reaches the media exchange itself — each attempt clears the prior blocker.** The
    infrastructure exists: bindable + advertised media socket on a fixed UDP port (`MediaConfig`,
    p24-c001); sovereign compose UDP override (p24-c002); authenticated-JWT-subject `view` authz +
    Keto seed (p24-c003/p23-c002); a decode harness + runner (p24-c004); a **real RS256 JWT the
    gateway accepts** (self-mint + self-served JWKS, since the verifier is RS256/JWKS while
    flint-gate signs HS256; p25-c001); and (p26-c001) the **gateway Docker image builds** — a Node
    stage builds the admin UI so the compile-time `rust-embed` of `admin-ui/dist` resolves.
    **Blockers cleared:** compose-merge fallback; HS256/RS256 mismatch; Dockerfile admin-ui embed;
    JWKS-port leak; `JWT_ISSUER` boot requirement; flint-gate build stall; Keto write path
    (`/admin/relation-tuples`); `getUserMedia` secure-context.
  - **Phase-27 diagnosed the media-negotiation defects and fixed three of them; the decode still
    times out.** Static analysis found: no ICE candidate exchange in the harness; the gateway's
    trickle candidates were never relayed to the browser over WS; and no `RoomJoin` so the
    `RoomRouter` never fanned out. Fixed: **bidirectional trickle ICE** (p27-c002 — gateway
    `local_signals`→WS relay + harness `onicecandidate`/`addIceCandidate`), **RoomJoin fan-out**
    (p27-c003), and **lifecycle instrumentation** (p27-c001, str0m + harness). **But the p27-c004
    re-run still timed out** (`docs/PHASE-27-DECODE-RESULT.md`) — and a **harness-timing bug** (the
    receiver runs a 15s connect pre-check + a 20s probe = 35s > Playwright's 30s test timeout)
    masked the new ICE-state diagnostics before they could print. So whether ICE now completes is
    still unconfirmed.
  - **Phase-28 surfaced the diagnostics and found the exact media-path defects.** Fixing the harness
    timing (p28-c001) + the `0.0.0.0` ICE candidate-IP bug (p28-c002 — the gateway now advertises a
    valid `127.0.0.1:40000` host candidate) let the p28-c003 re-run reach the media exchange itself:
    the gateway session negotiates and advertises a valid candidate (`ice=new`,
    `localCandidates=2`, `framesDecoded=0`; `docs/PHASE-28-DECODE-RESULT.md`). Two further, concrete
    str0m-SFU engineering blockers are now visible: **(B1)** Chrome sends its host candidates as mDNS
    `<uuid>.local` hostnames, which str0m rejects (`invalid IP address syntax`) — a real SFU needs a
    STUN srflx path or mDNS resolution, not the browser host candidate; **(B2)** the transport binds
    a **single fixed UDP port per session**, so a second negotiate fails `EADDRINUSE` and its trickle
    candidates hit "unknown session" — a real SFU uses one shared demuxing socket (route by ICE
    ufrag / remote 5-tuple).
  - **Phase-29 cleared both B1 and B2 and got ICE actually checking — still no decoded frame.**
    **B2 fixed (c001, ADR-008):** `StrOmTransport` now owns **one shared `UdpSocket`** demultiplexed
    to every session's `Rtc` by `Rtc::accepts()` (str0m's `chat.rs` model); the p29-c003 run shows
    **two sessions negotiating on the one `0.0.0.0:40000` — no `EADDRINUSE`**, the two-peer room that
    was impossible before. **B1 addressed (c002):** a hermetic coturn STUN server + harness
    `iceServers` gives the browser a routable `srflx` candidate, and the `.local` mDNS skip is now
    explicit + tested. Result: `remoteCandidates` went 0→1 and ICE advanced `new`→`disconnected` —
    real motion (`docs/PHASE-29-DECODE-RESULT.md`). **But** both sessions stop at `Connecting`, never
    reach `Connected`, and no `MediaData` flows: the browser's bridge-network `srflx` and the
    gateway's `127.0.0.1` host candidate are **mismatched network views** (same-host loopback vs.
    Docker bridge), so the one pair that forms can't sustain connectivity.
  - **Phase-30 implemented the topology fix (browser inside the Docker network) — the media-path
    engineering is now complete; the residual is purely environmental.** c001 added a Playwright
    Chromium service on the compose network (in-network `gateway:8080`, `--unsafely-treat-insecure-
    origin-as-secure-origin`, `stun:coturn:3478`) so the browser and SFU share one address space;
    c002 fixed the harness plumbing (mount the whole pnpm workspace, reuse the host install, pin the
    image to the installed Playwright version). But the decode **did not execute**: the **Colima
    Linux-VM crashed during the gateway image build** (Vite/`tsc` admin-ui compile → `rpc error:
    Unavailable … EOF`, Docker daemon down) — an OOM/resource limit, **not** a media-path defect
    (`docs/PHASE-30-DECODE-RESULT.md`). Every media-path blocker found across phases 24–29 (candidate
    IP, shared socket, mDNS, STUN, topology) is fixed.
  - **Phase-31 decoupled the gateway image build (c001) and cleared four more harness/infra layers
    (c002) — the decode now starts, but stops at a secure-context limit.** c001 replaced the OOM-prone
    in-run build with a presence-check on a pre-built image (proven: the image builds cleanly when run
    alone — the phase-30 crash was contention, not a memory ceiling). c002 then fixed, in turn: the
    pnpm-workspace mount, the Playwright image/CLI version mismatch, and a stray `webServer: pnpm dev`
    (exit 127). The run now boots the full stack and starts the decode spec — but the in-network
    browser navigates to the **insecure origin** `http://gateway:8080`, so `navigator.mediaDevices` is
    undefined and `getUserMedia` throws before any offer (`docs/PHASE-31-DECODE-RESULT.md`). The
    standard `--unsafely-treat-insecure-origin-as-secure-origin` flags did not register `mediaDevices`
    in the headless Playwright container. This is a **harness/environment** limit (a secure context
    for `getUserMedia` over non-HTTPS in-network), **not** a str0m/media defect — and it is *upstream*
    of the phase-29 topology fix (the sender can't acquire a track).
  - **Phase-32 fixed the secure context (Caddy TLS sidecar) — `getUserMedia` now works and ICE reaches
    `checking`, the furthest point yet; the residual is a routable-candidate-pair problem.** c001
    fronted the gateway with a Caddy TLS sidecar (`https://caddy:8443` → `wss://`), so the in-network
    browser has a genuine secure context, `navigator.mediaDevices`/`getUserMedia` work with no unsafe
    flags, sessions negotiate, and ICE advances to `checking` (`docs/PHASE-32-DECODE-RESULT.md`). But
    `framesDecoded=0`: the browser produces **only mDNS `.local` host candidates** (no usable STUN
    `srflx`), which str0m rejects, and the gateway's advertised `127.0.0.1` is the browser's own
    loopback in-container — so no routable pair forms and ICE stalls `checking` → `Disconnected`. This
    is the phase-29 B1 candidate-topology class, now cleanly isolated.
  - **Phase-33 tried host networking (Target A) to give both peers one routable stack — it hit a
    Colima host-net plumbing failure (gateway `unhealthy`) before the media exchange ran.** The
    host-net override is structurally correct (`!reset` cleared the merge-inherited `ports:`; Colima
    addresses derived; VM→host JWKS reachability confirmed), but the gateway container goes
    `unhealthy` under `network_mode: host` (a `/readyz` / `8080` host-net interaction, not JWKS, not
    media), cascading to the browser via `depends_on`. So `framesDecoded=0`
    (`docs/PHASE-33-DECODE-RESULT.md`). This is the 10th distinct blocker and the third *environment*
    variant (bridge → secure-context → host-net) to fail in its own new way.
  - **The recorded conclusion + recommended next step:** the media path, shared-socket demux, `.local`
    skip, and secure context are all **proven live**; the residual is purely a routable candidate
    pair. **Do not keep peeling same-host-VM plumbing.** Take the standard, environment-independent
    answer: **a TURN relay** (Target C) — coturn `--external-ip` + realm + long-term creds; harness
    `iceServers` adds the `turn:` URL — so both peers always get a **relay candidate that routes**
    regardless of host/mDNS/srflx/loopback topology, on the bridge stack that already boots healthy.
    The durable home is **CI / a real Linux host** (Target B) where host networking is native. str0m
    is **sans-codec** — the decode is browser-side.
  - **Phase-34 added the TURN relay and confirmed str0m accepts `typ relay` (`parser.rs:232`) — but 0
    relay candidates reached the gateway, so the decode proof is now ESCALATED to CI / a real Linux
    host (Target B).** coturn's relay candidate address (`--external-ip=127.0.0.1`) is the browser's
    own loopback in-container — the **same bridge address-confusion in a TURN variant**
    (`docs/PHASE-34-DECODE-RESULT.md`). Across phases 28→34 the media path, shared-socket demux,
    `.local` skip, secure context, and TURN acceptance are all **proven correct in pieces**; the only
    recurring failure is the **same-host Docker candidate topology** (host-loopback / bridge / mDNS /
    srflx / relay-external-ip — six forms). The macOS + Colima-VM + Docker-bridge environment is
    fundamentally unsuited to a browser↔SFU media proof; each local fix trades one address confusion
    for another. **Decision (recorded):** containerize the proof rig (JWKS + Keto seed) and run the
    decode as **one job on `ubuntu-latest` / a Linux runner** where host networking is native — the
    browser + SFU share one real stack, host candidates pair, TURN (already wired) is
    belt-and-suspenders, and the gateway advertises a real reachable IP. No more local same-host
    variants.
  - **Phase-35 escalated the proof to GitHub Actions `ubuntu-latest` — and it WORKED: the whole stack
    now builds + boots + runs the two-browser decode end-to-end on real Linux.** After two trivial CI
    interpolation fixes (`FLINT_GATE_JWT_SECRET`, `TURN_SECRET` must exist at `docker compose build`
    time — now generated job-wide, S1-clean), CI run 29057452278 built the gateway image (no OOM),
    booted the sovereign stack, ran the Playwright decode: `getUserMedia` works, signaling + offer
    happen, ICE reaches `checking` (`remoteCandidates=1`) — but `framesDecoded=0`
    (`docs/PHASE-35-DECODE-RESULT.md`). **The six-phase environment blocker is removed** — the proof
    runs on a real Linux host. Two follow-ups remain: a harness bug tears down the gateway (`down -v`
    on the script's EXIT trap) before the workflow captures its str0m log, so the Linux candidate
    detail wasn't visible this run; and the Linux candidate addressing (`MEDIA_ADVERTISE_IP`/coturn
    `--external-ip`) needs verifying once the log is captured. This is a normal debugging loop on a
    **working** proof harness, not an environmental dead-end.
  - Therefore **`SFU_MODE=sovereign` stays gated off** (defaults to `hosted`, boots with a
    warning). Do not enable it in production expecting media to flow until the decode proof passes
    against real infra; hosted (LiveKit) remains the media path. See ADR-005/006/007,
    `docs/PHASE-24-DECODE-RESULT.md`, and `crates/frf-media-str0m/SPIKE-FINDINGS.md`.

### admin-ui login

- **Token gate, hardened (phase-18)** — still not an interactive OIDC flow (flint-gate has
  no login endpoint and no IdP is deployed). The operator supplies a JWT; the gateway
  verifies it normally. The UI now decodes `exp`, logs out on expiry, and clears the token
  on a gateway `Unauthenticated` response — so a dead credential no longer lingers. Treat
  the admin UI as an authenticated-operator tool behind your own access control. A full
  OIDC login remains deferred until an IdP is deployed — the path is the subject of
  **ADR-004** (`docs/decisions/adr-004-admin-ui-oidc-idp.md`), which recommends standing up
  Ory Kratos + Hydra in compose (flint-gate is a token-metering proxy with no `authorize`
  endpoint and cannot drive an interactive login). No OIDC frontend code ships until that
  ADR is Accepted and the IdP exists.

### Dart SDK async transport

- ⏳ **Still deferred (re-affirmed 2026-07-07, p19-c003).** The Dart async transport
  (`FrfTransport.connect` / `subscribe`) does not function: `uniffi-bindgen-dart 0.1.3`
  (still the latest as of the 2026-07-07 check) emits broken async/callback Dart. The
  package compiles via a documented post-generation patch and the shim throws
  `FrfTransportUnavailable` — no misleading "works" path. The working CRDT surface
  (`FrfCrdt`) is unaffected. Dated record + removal trigger: `sdks/dart/GENERATED.md`
  → "Deferral re-affirmed."

When any deferred item is implemented, extend §1–§5 to cover its boundary before enabling
it. Do not advertise a plane as shipped until it functions end-to-end.
