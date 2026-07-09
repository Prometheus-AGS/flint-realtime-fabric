# Reflection — phase-30-sovereign-sfu-decode-topology-and-flip

> Date: 2026-07-09. Phase goal: fix the same-host candidate topology (browser-in-Docker) so a
> real decoded frame flows, then flip `SFU_MODE=sovereign` on `framesDecoded > 0`.
> **Outcome: the topology fix + harness are implemented, but the decode NEVER RAN — the Colima VM
> crashed building the gateway image. No decoded frame was observed; the gate stays OFF. G1's fix is
> implemented but UNVERIFIED; G2 NOT met.**

## Goal achievement

| Goal | Verdict | Evidence (against the goal's own exit criterion) |
|---|---|---|
| **G1 — same-host candidate topology** | ◐ **IMPLEMENTED but UNVERIFIED** | Exit was "the decode run reaches `ice=connected` and ≥1 session logs `state=Connected` + inbound `MediaData`." The fix (browser-in-Docker: Playwright service on the compose network, in-network `gateway:8080`, secure-context flag, `stun:coturn:3478`) is **built and archived** — but the run **crashed before any browser connected** (gateway image build OOM'd the Colima VM). So `ice=connected` was **never observed**. The code is correct in design; its exit criterion is **not** demonstrated. Honest verdict: implemented, unproven. |
| **G2 — decoded frame + flip** | ❌ **NOT MET (gate correctly held)** | No receiver observed `framesDecoded > 0` — the decode never executed. `main.rs` untouched; `PHASE-30-DECODE-RESULT.md` records the environmental blocker. G2's exit ("moves media **or** stays gated with fresh rationale") is satisfied only by the honest-hold branch. |
| **G3 — carried live proofs** | ◐ **CARRIED** | LiveKit x-node + admin-ui OIDC untouched; integration-gated. |

**Honest headline:** **0 goals fully MET this phase.** G1's fix is implemented but its exit
(`ice=connected`) was never demonstrated because the decode run died at the gateway image build — an
**environment/resource failure (Colima VM OOM)**, not a media-path defect. G2 did not happen. This is
the least-progress phase of the sequence in terms of *proven* outcome, and I will not dress it up: the
media-path engineering is complete, but phase-30 did **not** produce the decoded frame it set out to,
nor did it verify its own topology fix.

## Delivered changes

| Change | Summary | Gate impact |
|---|---|---|
| p30-c001-browser-in-docker-harness | Playwright service on the compose net; in-network gateway; secure-context; coturn service STUN | none (harness); topology fix implemented |
| p30-c002-decode-run-and-flip | Harness plumbing fix (repo-root mount, reuse host install, image v1.61.0) + decode run; honest gate decision (OFF) | **held OFF** |

Both archived (`openspec/changes/archive/2026-07-09-p30-c00{1,2}-*`); `media-e2e` +
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
eslint green on the host (no engine change this phase; Docker being down did not affect the Rust/TS
gate).

## What actually moved (honest, limited)

- The **topology fix is written and archived** — the browser now runs in-network, which is the
  correct fix for the phase-29 host-vs-container split (validated by construction, not by a live run).
- Two real harness-plumbing bugs were found and fixed: the pnpm-workspace mount (`workspace:*` deps
  only resolve from the repo root) and the Playwright image/CLI version mismatch (declared `^1.52`
  resolved to `1.61`).
- **No decoded frame. No verified `ice=connected`.** The needle on the actual objective did not move.

## Technical debt introduced

- **None net-new in shipped code** (no engine change; harness/compose only).
- **A latent process/environment problem is now explicit:** the decode runner does an **in-run
  gateway image build** (heavy admin-ui Vite/`tsc` compile) that OOMs the Colima VM when combined
  with the full stack + Playwright. This coupling must be broken (pre-build the image) before the
  decode proof can run at all here.

## Lessons captured

1. **The environment is a first-class dependency of a live proof — and it just failed.** Phases 24–29
   converged the *media-path* residual to zero, but phase-30 hit a wall that has nothing to do with
   the media path: the local VM can't build + run the whole stack in one pass. A "live proof" is only
   as reachable as the box it runs on. **Do not conflate "code is correct" with "proof observed."**
2. **An in-run image build is the wrong coupling for a proof runner.** Building the gateway image
   (which compiles the admin-ui) *inside* the decode run makes every proof pay a heavy, OOM-prone
   build. The image should be built once, out-of-band, and the runner should `up --no-build`.
3. **`workspace:*` + a partial mount is a trap.** Mounting only `admin-ui` into a container looks
   right but breaks pnpm workspace resolution — the lockfile and store live at the repo root. Mount
   the workspace root or nothing.
4. **The honest gate held a fifth time — this time against a *tempting infra excuse*.** "The code is
   done, it only failed on OOM" is a real and seductive reason to soft-pass. It didn't: no
   `framesDecoded > 0` was observed, so no flip, and G1 is recorded as *unverified*, not *met*.
   Absence of a defect is not presence of a proof.
5. **Restart hygiene:** the Colima daemon is now down; the next phase must `colima start` (with more
   memory) before any Docker work.

## Recommended next phase

**`phase-31-sovereign-sfu-decode-prebuilt-image-and-flip`** — remove the environmental blocker, then
run the decode that phase-30 couldn't.

- **G1 (do first):** **decouple the gateway image build from the decode run.** Build the gateway
  image **once** (out-of-band locally, or in CI) and change `scripts/run-media-decode.sh` to
  `up -d --no-build gateway` against the pre-built image — no in-run `build gateway`. Optionally raise
  the Colima VM memory (`colima start --memory 8`+) as a belt-and-suspenders. Restart the daemon
  first.
- **G2:** re-run the in-network decode (all media-path + harness fixes already in place from
  phases 24–30); observe `ice=connected` + `framesDecoded > 0`. **Flip `SFU_MODE=sovereign` only on
  a genuine decoded frame.**
- **G3** carried (LiveKit x-node, admin-ui OIDC) — integration-gated.

**Discipline carried (16→30):** the gate flips only on decoded media proven against a live gateway;
else it stays off with fresh rationale. No "healthy but does nothing." Update `progress.json` to N/N
before any command mentioning the next stage (phase-29 lesson).
