# Plan — phase-21-sovereign-rtp-forwarding

> Stage: Plan · 2026-07-08 · Backend: **OpenSpec** (`change_backend: openspec`)
> Source: `assessment.md`. Operator decisions locked (below).
> Ordering: **refactor for headroom → prove the harness → ADR the load-bearing unknown →
> build + gate → close** — the phase-16–20 discipline.

## Operator decisions (locked this stage)

1. **RTP fan-out architecture:** a **central room registry in `StrOmTransport`** (session →
   co-room peers) + **per-session forwarding `mpsc`** — a driver reads `Event::MediaData` and
   pushes it to the other sessions' forwarding channels, whose drivers write via
   `rtc.writer(mid).write` (`MediaData` payload is `Arc<[u8]>`, cheap to clone). ADR first.
2. **G3 scope:** phase-21 stops at **proven 1-to-1 RTP forwarding** between two connected
   peers; **split per-room N-peer fan-out + PLI/keyframe + renegotiation into phase-22**.
3. **Live infra:** **none this phase.** Prove in-process (two-peer loopback for connected +
   1-to-1 RTP); the true browser/decode proofs + G5 (LiveKit live, admin-ui OIDC) are
   integration-gated / carried.

## Ordered changes

Each is one OpenSpec change, walked one task/turn via `/kbd-apply` from the repo root. Spec
delta before the QA gate (MUST-on-line-after-header). QA gate per change unless docs-only.

| # | Change ID | Goal | Type | QA | Summary |
|---|-----------|------|------|:--:|---------|
| c001 | `p21-c001-split-session-driver` | G0 | refactor | yes | Extract the driver loop → `driver.rs`; `session.rs` back under 500. Behavior-preserving. |
| c002 | `p21-c002-offerer-role-and-connected-proof` | G1 | feat | yes | Offerer path (`add_media().apply()`+`accept_answer`); un-`#[ignore]` the two-peer DTLS-connected loopback proof. |
| c003 | `p21-c003-rtp-fanout-adr` | G2 arch | docs | skip | ADR-006: room registry in `StrOmTransport` + per-session forwarding `mpsc`; recommend + rationale; gates the forwarding code. |
| c004 | `p21-c004-rtp-forwarding-1to1` | G2 | feat | yes | Room registry + forwarding channels + `Event::MediaData`→`writer(mid).write`; 1-to-1 forwarding proven in-process (or integration-gated). |
| c005 | `p21-c005-sovereign-gate-and-phase-22-seed` | G4/G3/G5/close | docs | skip | Enable `SFU_MODE=sovereign` **only if** 1-to-1 media flows (else re-affirm gated); seed phase-22 (N-peer + PLI); re-affirm G5; SECURITY §6 / CHANGELOG / sign-off + gate re-run. |

**5 changes.** c001 clears the file-size blocker; c002 builds the connectivity harness the
RTP proof needs; c003 ADRs the fan-out; c004 forwards media 1-to-1; c005 closes + seeds
phase-22 for N-peer/PLI.

---

## Change detail

### c001 — split session/driver (G0) · refactor · QA:yes
- **Files:** `crates/frf-media-str0m/src/{session.rs,driver.rs (new),lib.rs}`.
- **Tasks:** extract `run_session` + `state_for_event` + `SessionCommand`/`SessionMeta` into
  `driver.rs`; `session.rs` keeps the `StrOmTransport`/`MediaTransport` surface + registry.
  Behavior-preserving — **all 18 existing tests unchanged and green**. Both files ≤500.
- **Exit:** `session.rs` back under 500 lines; tests green; clean gate. **Do first.**

### c002 — offerer role + two-peer connected proof (G1) · feat · QA:yes
- **Files:** `driver.rs`/`session.rs` (an offerer/test-harness path), the `#[ignore]`d test.
- **Tasks:** add an offer-creating path (`sdp_api().add_media().apply()` → offer; accept the
  answer). Un-`#[ignore]` `two_peers_reach_dtls_connected`: two `StrOmTransport` sessions over
  loopback exchange offer/answer + host candidates and drive ICE/DTLS to `Connected`.
- **Exit:** two in-process sessions reach `ConnectionState::Connected`, proven by a test — or,
  if loopback ICE is CI-flaky, `#[ignore]` in CI with the local-pass documented (honest bar).
  This harness is the prerequisite for the c004 RTP proof.

### c003 — RTP fan-out ADR (G2 arch) · docs · QA:skip
- **Files:** `docs/decisions/adr-006-rtp-fanout.md` (new).
- **Tasks:** state the constraint (per-session `Rtc` in isolated driver tasks → forwarding is
  cross-task); present the options (central room registry + per-session `mpsc` [chosen] vs
  direct peer channels vs shared-Rtc actor); **recommend the central registry + per-session
  forwarding `mpsc`**; sketch the `MediaTransport` RTP surface (or internal); note
  `SFU_MODE=sovereign` stays off until media flows.
- **Exit:** the fan-out architecture is decided; c004 implements it.

### c004 — RTP forwarding, 1-to-1 (G2) · feat · QA:yes
- **Files:** `session.rs`/`driver.rs` (room registry + forwarding channels),
  possibly a `room.rs` if it nears 500 L.
- **Tasks:** (1) a room registry (`DashMap<(TenantId, room), HashSet<SessionId>>`) + a
  per-session forwarding `mpsc<ForwardedMedia>`. (2) driver: on `Event::MediaData`, look up
  the room's *other* session(s) and push the media to their forwarding channels; on a
  forwarding-channel item, `rtc.writer(mid).write(pt, wallclock, rtp_time, data)`. (3) join a
  session to a room at `create_session`; deregister at `remove_session`. (4) **1-to-1** proof:
  two connected peers (c002 harness), one sends media, the other's session forwards it —
  proven in-process where feasible, else integration-gated. **N-peer + PLI = phase-22.**
- **Exit:** a media packet from peer A is forwarded to peer B through the SFU (proven or
  integration-gated). No lib `unwrap`/`expect`. `SFU_MODE=sovereign` still off.

### c005 — sovereign gate + phase-22 seed + close · docs · QA:skip
- **Files:** `crates/frf-gateway/src/main.rs` (only if flipping the gate),
  `.kbd-orchestrator/phases/phase-22-sovereign-sfu-npeer-pli/` (seed), `docs/SECURITY.md` §6,
  `CHANGELOG.md`, `docs/PHASE-21-SIGNOFF.md` (new).
- **Tasks:** (1) **enable `SFU_MODE=sovereign`** to compose `StrOmSignaler` (signaling) +
  `StrOmTransport` (media) **only if** 1-to-1 media flows end-to-end and its boundary is
  covered in SECURITY §1–§5; **else re-affirm gated-off** with rationale (the honest default
  given browser-gated proofs). (2) Seed phase-22 (N-peer fan-out + PLI/keyframe +
  renegotiation; carry G5). (3) SECURITY §6 / CHANGELOG / sign-off; re-run the gate suite.
- **Exit:** phase-21 signed off; phase-22 seeded; the gate is flipped **only** if media flows,
  else honestly gated.

---

## Sequencing & rationale

- **c001 first** — the 498/500 file-size blocker; nothing else can add engine code until
  `session.rs` has headroom. Behavior-preserving, so low-risk.
- **c002 before c004** — the RTP forwarding proof needs two connected peers; the offerer role
  + connected harness is that prerequisite.
- **c003 (ADR) before c004** — the cross-task fan-out is the load-bearing unknown; decide the
  architecture before writing forwarding code (ADR-004/005 discipline).
- **c004** — 1-to-1 RTP against the decided design.
- **c005 last** — flips the gate *only* if media flows, seeds phase-22 for N-peer/PLI, closes.

## Phase exit criteria

- `session.rs` under 500 (c001); two peers reach `Connected` (c002); the RTP fan-out is
  decided (c003) and 1-to-1 forwarding works or is integration-gated (c004); the gate is
  enabled only if media flows, else re-affirmed off (c005).
- Release gate suite green; each code change passes the QA gate.
- **No "healthy but does nothing":** `SFU_MODE=sovereign` is flipped on **only** when 1-to-1
  media actually flows end-to-end; N-peer/PLI honestly deferred to phase-22; G5 carried.

## First change to apply

`p21-c001-split-session-driver` — clears the file-size blocker for all the engine work.
