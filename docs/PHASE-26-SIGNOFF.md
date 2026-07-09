# Phase-26 Sign-off — gateway Docker admin-ui embed, live media path & the gate decision

> Date: 2026-07-09 · Closing verification for phase-26 (p26-c003). Fixed the gateway image build
> and drove the authenticated decode run all the way to the media exchange. The honest decision
> holds: `SFU_MODE=sovereign` **stays off** — the WebRTC decode does not complete yet, but the
> failure is finally at the media transport, not plumbing.

## Gates (re-run at phase close — actual results)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ (exit 0) |
| `cargo check --workspace` | ✅ (exit 0) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ (exit 0) |
| `cargo test -p frf-media-str0m --lib` | ✅ 27 passed |
| `cargo test -p frf-gateway --lib` | ✅ 39 passed |

> Phase-26 changed the Dockerfile + harness scripts + docs (no new Rust), so host test counts are
> unchanged from the phase-25 close — the workspace stays green.

## Goal status (honest)

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — fix the Dockerfile admin-ui embed** | ✅ MET | A Node 24 stage builds the admin UI (pnpm workspace + SDK siblings; `frf-wasm` stubbed) and `COPY`s `admin-ui/dist` into the Rust context before `cargo build`. **The gateway image builds** (`docker compose build gateway` → `Built`; `rust-embed` resolves). p26-c001. |
| **G2 — real decoded frame** | ❌ NOT PROVEN | The authenticated run reached the media exchange: build → boot → **gateway healthy** → **Keto `view` granted (201)** → **browser harness runs** — then the WebRTC decode **timed out (30s)**; no `framesDecoded` (`docs/PHASE-26-DECODE-RESULT.md`). p26-c002. |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet → no flip; `main.rs` gate-off warning intact; production defaults hosted. |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC (ADR-004 + IdP). |

## The blocker arc — converged to the media transport

Ten phases of honest, convergent failures; phase-26 cleared **eight** in sequence and reached the
media exchange itself:

`P24 compose-merge → P25a RS256/HS256 verifier → P25b Dockerfile embed → P26 { JWKS-port leak →
JWT_ISSUER boot → flint-gate --build stall → Keto /admin write path → getUserMedia secure-context
} → **P26f the WebRTC decode does not complete (media transport)**.`

Everything *around* the media now works. The remaining unknown is genuine WebRTC: whether Chromium
completes ICE/DTLS/RTP to the `host.docker.internal:40000/udp` candidate and the SFU relays the
sender's RTP to the receiver over a real network.

## The gate decision — stated plainly

**`SFU_MODE=sovereign` stays off.** The proof reaches the media exchange but observes no decoded
frame. Enabling the gate now would advertise a plane that has never moved one — the failure this
project has refused for eleven phases.

**To make it pass (next phase):** a focused media-transport investigation — instrument ICE
connection state + gathered/received candidates + DTLS on both browser peers, capture gateway media
logs, confirm the sender↔receiver share the room and the `RoomRouter` fan-out delivers the sender's
RTP to the receiver over the real UDP path; then re-run and observe `framesDecoded > 0`. Only then
does the gate flip.

## Process note (honest)

- **c001 fixed a real Dockerfile defect** and, iterating on the actual build, cleared two build
  sub-issues (nonexistent root `package.json`; SDK siblings needing a build before admin-ui's tsc).
- **c002 ran the proof for real, repeatedly**, clearing five more concrete blockers to reach the
  media path, and recorded the media-transport timeout plainly — not dressed up.
- **A process error was caught by the verify gate:** c002's openspec change dir was never seeded
  (I went straight to begin-task). `verify` reported "not found"; I seeded it retroactively
  (proposal/tasks/spec matching the real work), validated, and archived cleanly — rather than
  skipping. c003 was seeded up front.

## Sign-off

Phase-26 made the gateway image build and drove the authenticated proof to the media exchange for
the first time — then honestly recorded that the WebRTC decode does not yet complete, so
**`SFU_MODE=sovereign` is re-affirmed off** with a precise media-transport next step. No new
CRITICAL/HIGH; all gates green. Hosted (LiveKit) remains the media path. No plane was advertised
beyond what it does.
