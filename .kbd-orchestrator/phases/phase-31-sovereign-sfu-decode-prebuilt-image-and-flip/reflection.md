# Reflection — phase-31-sovereign-sfu-decode-prebuilt-image-and-flip

> Date: 2026-07-09. Phase goal: remove the environmental blocker (in-run gateway build OOM) so the
> decode that phases 24→30 built can finally run, and flip `SFU_MODE=sovereign` on
> `framesDecoded > 0`.
> **Outcome: G1 MET — the decouple works and the full stack now boots + starts the decode without
> OOM. But the run hit a NEW harness blocker (secure context for `getUserMedia`), so G2 is NOT met
> and the gate stays OFF. 1 of 2 goals; the objective (decoded frame) did not happen.**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — decouple the build** | ✅ MET | Exit was "the runner boots the full in-network stack (gateway + coturn + playwright) **without OOMing** and the decode harness **actually executes**." c001's presence-check + fail-fast removed the in-run build; the gateway image built **cleanly run alone** (`BUILD_EXIT=0`, no OOM — proving phase-30 was contention, not a memory ceiling); the run then booted the full stack and **started the decode spec**. The OOM that killed phase-30 is gone. Met. |
| **G2 — decoded frame + flip** | ❌ NOT MET (gate correctly held) | No `framesDecoded > 0`; no session negotiated. The decode spec ran but the in-network browser (`http://gateway:8080`, insecure origin) has `navigator.mediaDevices === undefined`, so `getUserMedia` threw before any offer. `main.rs` untouched; `PHASE-31-DECODE-RESULT.md` records it. Honest-hold branch of the exit satisfied; the objective did not occur. |
| **G3 — carried proofs** | ◐ CARRIED | LiveKit x-node + admin-ui OIDC untouched; integration-gated. |

**Honest headline:** **1 of 2 goals MET (G1).** The environmental wall that blocked phase-30 is
genuinely down — but reaching the harness only surfaced the *next* blocker (secure context), so the
decoded frame still did not happen. Real, concrete progress; not the objective.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p31-c001-prebuilt-image-runner | Presence-check the pre-built gateway image + fail-fast; no in-run build (OOM fixed) | none (harness) |
| p31-c002-decode-run-and-flip | Fixed a cascade of harness bugs (workspace mount, image v1.61, `DECODE_ONLY` webServer skip, secure-context flags) + decode run; honest gate decision (OFF) | **held OFF** |

Both archived (`openspec/changes/archive/2026-07-09-p31-c00{1,2}-*`); `media-e2e` +
`sfu-mode-consistency` specs updated.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 2/2 |
| First-pass pass rate | 2/2 (100%) |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

### Recurring Constraint Violations

None. Both changes passed R1–R5, S1, P1 on the first pass. 31/31 `frf-media-str0m` tests + admin-ui
eslint green on the host (no engine change; the Rust media-path is untouched).

## What actually moved (honest)

- **The phase-30 OOM is gone — proven.** The image builds cleanly alone; the run boots the full stack
  and starts the decode. The environment blocker G1 targeted is cleared.
- **Four real harness bugs fixed**, each surfaced only by running to the next layer: pnpm-workspace
  mount, Playwright image/CLI version, `webServer: pnpm dev` exit 127, and the secure-context flag
  attempt. The decode harness is materially closer to working end-to-end.
- **Still no decoded frame, no negotiated session.** The needle on the objective did not move.

## Technical debt introduced

- **None net-new in shipped code** (harness/compose/script + TS config only; no engine change).
- **The decode harness now carries substantial run-specific scaffolding** (self-mint JWKS, Keto seed,
  coturn, in-network Playwright service, `DECODE_ONLY`, secure-context flags). It is a proof rig, not
  production wiring — acceptable, but growing; flagged for eventual consolidation.
- **The Colima VM is unstable under load on this box** (crashed twice this phase). Not our code, but a
  real constraint on how the live proof can be driven here.

## Lessons captured

1. **A single environmental fix rarely reveals the objective — it reveals the next layer.** Removing
   the OOM (G1) let the run reach the harness, which surfaced the webServer bug, which surfaced the
   secure-context bug. The 24→31 arc is a **layered peeling**: each phase clears the current outermost
   blocker and exposes the next. The media path itself has been done since phase-30; everything since
   is *harness/environment* peeling.
2. **Know when to stop iterating live.** c002 cleared four harness layers in one change; the fifth
   (secure context) is Chromium-flag arcana with no media signal, on a VM crashing under load.
   Continuing would be thrashing. Recording the honest blocker + carrying a concrete fix (HTTPS /
   `*.localhost`) is the disciplined move — not a fifth, sixth, seventh flag guess.
3. **`getUserMedia` needs a genuine secure context; unsafe flags are unreliable in headless
   containers.** The durable fix is HTTPS or a `*.localhost` origin, not
   `--unsafely-treat-insecure-origin-as-secure-origin`. Design the harness around the browser's real
   security model, don't fight it with flags.
4. **The honest gate held a sixth time — now against "I fixed four things, surely it's basically
   working."** Four fixes and a booting stack is the most "we're basically there" the sequence has
   felt. It didn't flip: `framesDecoded == 0`, no session, so no flip. Substantial progress ≠ proof.

## Recommended next phase

**`phase-32-sovereign-sfu-decode-https-secure-context-and-flip`** — give the in-network browser a real
secure context, run the decode, flip.

- **G1 (do first):** serve the gateway over **HTTPS** in the decode stack — a self-signed cert (or a
  TLS-terminating sidecar) at `https://gateway:8443`, browser launched with
  `--ignore-certificate-errors`. This is a genuine secure context, so `navigator.mediaDevices` exists
  and `getUserMedia` works **with no unsafe flags** (drop them). Alternative: a `*.localhost` network
  alias (Chromium treats `*.localhost` as trustworthy). Harness/compose only, no engine change.
- **G2:** re-run the in-network decode; observe `ice=connected` + `framesDecoded > 0`. **Flip
  `SFU_MODE=sovereign` only on a genuine decoded frame** — this is finally the run where the whole
  media path (candidate IP, shared socket, mDNS, STUN, topology) gets exercised end-to-end.
- **G3** carried (LiveKit x-node, admin-ui OIDC) — integration-gated.
- **Operational:** `colima start`/`restart` (the VM may be down); build the gateway image once
  (c001's fail-fast enforces it); keep concurrent VM load low.

**Discipline carried (16→31):** the gate flips only on decoded media proven against a live gateway;
else it stays off with fresh rationale. Update `progress.json` to N/N before any next-stage command
(phase-29). Absence of a defect is not presence of a proof (phase-30). Stop live-iterating when a
blocker becomes flag-arcana with no media signal (phase-31).
