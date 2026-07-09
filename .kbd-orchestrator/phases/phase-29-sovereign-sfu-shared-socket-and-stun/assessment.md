# Assessment — phase-29-sovereign-sfu-shared-socket-and-stun

> Date: 2026-07-09. Gap report against the two phase-28 blockers (B2 shared socket, B1 routable
> browser candidate) plus the carried live proofs. Grounded in the actual `frf-media-str0m` source
> and the str0m 0.21 reference implementation.

## Method

Read `crates/frf-media-str0m/src/{session,driver,sfu,config,room}.rs`, the `StrOmTransport` +
`SessionHandle` ownership model, and the str0m 0.21 crate (`chat.rs` example + `lib.rs` API) to
confirm the canonical shared-socket pattern and the candidate API. No code changed (assess only).

## Current state (what exists)

- **Per-session socket + driver.** `session.rs:154` — `negotiate()` calls
  `UdpSocket::bind((self.config.bind_addr, self.config.udp_port))` **once per session**, then moves
  the socket into a per-session `tokio::spawn(run_session(...))` driver (`session.rs:238`). Each
  session owns its own socket + its own `Rtc` + its own `recv_from` loop (`driver.rs`).
- **Transport registry.** `StrOmTransport` holds `sessions: DashMap<SessionId, SessionHandle>` +
  a shared `RoomRouter` + `MediaConfig` (`session.rs:55`). Sessions talk to their drivers over
  `mpsc<SessionCommand>` (`AddRemoteCandidate`, `Shutdown`).
- **Valid host candidate (p28-c002).** `negotiate` resolves `advertise_host` → concrete IP and
  advertises `candidate:… 127.0.0.1 40000 typ host` — this part is correct and proven.
- **Remote-candidate path.** `driver.rs` receives `SessionCommand::AddRemoteCandidate(String)` and
  feeds it to `Rtc::add_remote_candidate(Candidate)` — which requires a **parseable** candidate.
- **Reference pattern available.** str0m 0.21 `examples/chat.rs:152-160` demuxes **one** shared
  socket to many `Rtc` via `clients.iter_mut().find(|c| c.accepts(&input))`. This is the exact B2
  fix, with a proven port target. `Rtc::accepts(&Input)` is public (`lib.rs:1885`).

## Gap analysis

### G1 — B2: shared demuxing UDP socket  ·  **GAP: CONFIRMED (blocker)**

- **Defect:** one `UdpSocket::bind(fixed_port)` per session. The **second** `create_session` on the
  fixed port fails `udp bind: Address already in use (os error 98)` (phase-28 evidence). A single
  fixed media port is a single-session design; the two-peer decode room cannot be hosted.
- **Required:** invert socket ownership.
  1. `StrOmTransport` binds **one** shared `UdpSocket` at construction (from `MediaConfig`), not
     per session. `negotiate` stops binding; it takes the shared socket's `local_addr` for the
     advertised candidate.
  2. Replace N per-session driver tasks with **one demux loop** owning the shared socket: on each
     inbound datagram, find the owning session by `rtc.accepts(&Input)` (port str0m's
     `chat.rs:154`), feed it, and drain that `Rtc`'s outputs back to the shared socket. Per-session
     `Rtc` + state/forward channels stay; only the socket + recv loop consolidate.
- **Scope:** `session.rs` (drop per-session bind; hold `Rtc`s in a shared structure), `driver.rs`
  (single demux loop instead of per-session `run_session`), possibly a new `demux.rs` to keep files
  ≤500 lines. **ADR required** — this changes socket ownership under ADR-005/006 (a refinement, not
  a re-open). The in-process multi-session test (two sessions, one port, no `EADDRINUSE`) is the
  proof and needs no browser.
- **Risk:** highest-effort change of the phase; concurrency around the shared `Rtc` set (the demux
  loop must own or lock the `Rtc`s). str0m is sans-I/O + single-threaded per `Rtc`, so a single
  owning task (chat.rs model) is simplest and avoids locks.

### G2 — B1: routable browser candidate (STUN srflx / mDNS)  ·  **GAP: CONFIRMED**

- **Defect:** Chrome emits **host** candidates as mDNS `<uuid>.local`; `Rtc::add_remote_candidate`
  needs a parseable `Candidate`, and `.local` has no IP → phase-28 `bad address: invalid IP address
  syntax`, every browser host candidate dropped, `remoteCandidates=0`.
- **Required (preferred):** add a **STUN server** so Chrome gathers a **server-reflexive** candidate
  (a real IP that parses + routes). Wire a STUN URL into the harness `RTCPeerConnection` config; for
  the same-host loopback run, the browser's srflx to a STUN server resolves to a reachable address.
  Alternative: resolve `.local` mDNS at the gateway before `add_remote_candidate` (more moving
  parts; mDNS in a container is fiddly). **Lean STUN.**
- **Scope:** harness/compose (STUN server in the run stack + `iceServers` in the probe's PC config);
  possibly a candidate-filter in `driver.rs`/`ice.rs` to skip unparseable `.local` gracefully
  (defensive — don't error the whole session on one bad candidate). No `MediaConfig` contract change.
- **Dependency:** **G2 is only testable once G1 lands** — until the shared socket exists, the second
  session dies at bind and no candidate exchange completes regardless of B1. **Order: G1 → G2.**
- **Risk:** medium. For a pure loopback same-host run, host candidates *should* work if both ends use
  real loopback IPs — but the browser hides them behind mDNS. STUN is the standard escape hatch; a
  local coturn/STUN in compose is the likely addition.

### G3 — decoded frame + flip  ·  **GAP: OPEN (depends on G1+G2)**

- Unchanged discipline: re-run `scripts/run-media-decode.sh`; flip `SFU_MODE=sovereign` **only** on
  `framesDecoded > 0`; else re-affirm gated. `main.rs` gate + SECURITY §6 + CHANGELOG +
  PHASE-29-SIGNOFF. Harness + runner already exist (phase-24…28).

### G4 — carried live proofs  ·  **GAP: CARRIED (unchanged)**

- G4.1 LiveKit x-node inbound, G4.2 admin-ui OIDC — integration-gated as in phase-19…28. No new
  finding; re-affirm deferred unless the operator prioritizes.

## Open questions for plan/analyze

1. **Demux ownership model:** single owning task holding all `Rtc`s (chat.rs model, no locks) vs.
   keep per-session tasks but share only the socket via an actor that routes datagrams? The former
   is simpler and matches str0m's guidance — recommend it, but it's the load-bearing design call.
2. **STUN in the run stack:** add a local STUN (coturn) service to `compose.sovereign.yml`, or point
   the harness at a public STUN (`stun.l.google.com:19302`) for the same-host run? Public STUN is
   zero-infra but needs outbound network from the browser container; local coturn is hermetic.
3. **File-size:** the shared-socket demux likely pushes `session.rs`/`driver.rs` over 500 — plan a
   `demux.rs` split up front.

## Recommended change ordering (for plan)

1. **c001 — B2 shared demuxing socket** (ADR + transport refactor + multi-session in-process test).
   Blocks everything; do first.
2. **c002 — B1 STUN srflx path** (compose STUN + harness `iceServers` + defensive `.local` skip).
3. **c003 — decode re-run + conditional flip** (PHASE-29-DECODE-RESULT; flip or re-affirm).
4. **c004 (optional) — G4 re-affirm** integration-gated, only if not folded into c003's SECURITY §6.

## Exit posture

The phase's load-bearing risk is **G1 (B2)** — a genuine transport refactor with a proven str0m
reference (`chat.rs`). G2 is smaller but blocked behind G1. The honest gate (G3) is unchanged and
non-negotiable. Nothing here re-opens a settled ADR; the shared-socket ADR is a refinement of
ADR-005/006 socket ownership.
