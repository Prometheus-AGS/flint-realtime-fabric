# Plan — phase-29-sovereign-sfu-shared-socket-and-stun

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. Two operator
> decisions locked this turn: **(D1)** demux = single owning task (str0m `chat.rs` model, lock-free);
> **(D2)** STUN = local coturn in `compose.sovereign.yml`.

## Ordering rationale

Strict serial order — **G1 → G2 → G3** — because B1 (STUN) is untestable until the shared socket
exists (a second session dies at `EADDRINUSE` before any candidate exchange completes). B2 is the
load-bearing transport refactor; it goes first and alone. Each change ends at a QA gate + archive;
the honest `SFU_MODE=sovereign` gate is decided only in c003, only on a real decoded frame.

## Changes

### c001 — `p29-c001-shared-demux-socket` (B2 / G1) · **agent: rust-reviewer after apply**

**Goal:** one shared `UdpSocket` owned by `StrOmTransport`, demuxed to per-session `Rtc` by
`Rtc::accepts()` — eliminate the per-session `EADDRINUSE`.

- **ADR-008** (`docs/decisions/adr-008-shared-media-socket.md`): socket ownership moves from
  per-session to the transport; single owning task holds all `Rtc`s and the socket; refines
  ADR-005/006 (not a re-open). Decision D1 recorded.
- Refactor: `StrOmTransport` binds one socket at construction (from `MediaConfig`); `negotiate` stops
  binding and takes the shared `local_addr` for the advertised candidate. Replace N `run_session`
  tasks with one demux loop: `select!` over `socket.recv_from` → `rtcs.iter_mut().find(|r|
  r.accepts(&input))` → `handle_input`; drain each `Rtc`'s `poll_output` back to the shared socket;
  keep per-session state/forward/command channels + the `RoomRouter` fan-out intact.
- **File-size:** split the demux loop into `demux.rs` (≤500 each). No library `unwrap`/`expect`;
  `tracing` spans across the port boundary; `#[non_exhaustive]` preserved.
- **Test (in-process, no browser):** two sessions created on the **one** fixed port → no
  `EADDRINUSE`; a datagram from peer-A's socket routes to peer-A's `Rtc` (accepts-demux proven);
  `two_peers_reach_dtls_connected` still green under the shared socket.
- **Exit:** multi-session bind works on one port; demux routes correctly; 29+ str0m tests green.

### c002 — `p29-c002-stun-srflx-path` (B1 / G2) · **agent: rust-reviewer + e2e check**

**Goal:** give Chrome a routable candidate str0m accepts — a STUN server-reflexive candidate.

- **compose:** add a `coturn` STUN service to `compose.sovereign.yml` (`3478/udp` published);
  `extra_hosts` as needed. Decision D2.
- **harness:** set `iceServers: [{ urls: 'stun:127.0.0.1:3478' }]` in the decode probe's
  `RTCPeerConnection` config so Chrome gathers an `srflx` candidate (a real IP that parses).
- **defensive skip:** in `driver.rs`/`ice.rs`, when `add_remote_candidate`'s parse fails (e.g. a
  `.local` mDNS host candidate), **skip that one candidate** and continue — never error the whole
  session on one unparseable candidate (this is already the observed behaviour; make it explicit +
  tested).
- **Test:** a unit test that an unparseable `.local` candidate string is skipped, not fatal; compose
  lint/boot check for the coturn service.
- **Exit:** the gateway accepts ≥1 browser candidate (`remoteCandidates>0`); ICE advances past `new`.
  (Full proof lands in c003.)

### c003 — `p29-c003-decode-run-and-flip` (G3) · **honest gate decision**

**Goal:** re-run the decode proof end-to-end; flip `SFU_MODE=sovereign` **only** on
`framesDecoded > 0`.

- Run `scripts/run-media-decode.sh`; read the browser assertion + gateway str0m logs. Record
  `docs/PHASE-29-DECODE-RESULT.md`.
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-29-SIGNOFF.md`.
- **Else:** re-affirm gated with the fresh diagnostic detail (main.rs untouched). Carry G4.
- Re-run the release gate suite; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c004 (conditional) — `p29-c004-carried-proofs-reaffirm` (G4) · **only if not folded into c003**

Re-affirm LiveKit x-node (G4.1) + admin-ui OIDC (G4.2) integration-gated in SECURITY §6 + CHANGELOG.
Likely folded into c003's SECURITY §6 edit — emit as a change only if it needs its own diff.

## Discipline (carried 16→29)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0` against a live gateway; else
  gated off with fresh rationale. No "healthy but does nothing."
- **Seed the openspec change dir at the START of every `/kbd-apply`.** Read the QA verdict **and**
  the archive output (never trust a silent archive). File-size ≤500; no library `unwrap`/`expect`;
  clippy pedantic + `deny(warnings)`.

## Change summary

| # | id | goal | blocker | risk | agent |
|---|---|---|---|---|---|
| 1 | p29-c001-shared-demux-socket | G1 | B2 | **high** (transport refactor; ADR-008) | rust-reviewer |
| 2 | p29-c002-stun-srflx-path | G2 | B1 | medium (compose + harness) | rust-reviewer + e2e |
| 3 | p29-c003-decode-run-and-flip | G3 | — | gate decision | — |
| 4 | p29-c004-carried-proofs-reaffirm (cond.) | G4 | — | low | — |

**First change to apply: `p29-c001-shared-demux-socket`.**
