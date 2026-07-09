# Reflection — phase-24-sovereign-sfu-live-decode-and-flip

> Generated 2026-07-08. The phase that built the full sovereign live-decode path, **ran the
> proof, and honestly failed it** — so `SFU_MODE=sovereign` is re-affirmed off. Genuine
> engineering progress landed; the flip did not, because a real decoded frame was not observed.

## Delta — movement against phase goals

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — live sovereign gateway env** | ◐ PARTIAL | `MediaConfig` bind/advertise seam (c001), `compose.sovereign.yml` UDP path verified via `docker compose config` (c002), Keto `view` seed (c003). Stack composes sovereign; the run's boot failed on the no-JWT fallback. |
| **G2 — live decoded-media run** | ❌ NOT PROVEN | Runner executed; failed `RUNNER_EXIT=1` on a compose-merge blocker; stack never came up; **no `framesDecoded > 0`** (`PHASE-24-DECODE-RESULT.md`). |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ RE-AFFIRMED OFF | G2 unmet → no flip; `main.rs` gate-off warning intact; production defaults hosted. |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime`; admin-ui OIDC (ADR-004 + IdP). |

**2/4 partial-or-met on infrastructure; the proof + flip did not close — attempted, not assumed.**

## Root cause — why G2/G3 did not close

Not under-execution and not (yet) a disproven media path: the live run died in **harness plumbing**
before media flowed. The runner's no-`E2E_JWT` fallback chained `compose.override.example.yml` as a
third `-f`; that dev profile *replaces* the gateway `depends_on` and references services the chain
doesn't provide → `service "gateway" depends on undefined service "keto"` → invalid project. The
sovereign-only merge is valid; the fallback path is not. A faithful run needs a real `E2E_JWT`
(flint-gate) or a purpose-built no-auth sovereign override — neither was in place.

## Delivered changes (5/5 archived)

1. **c001** — `MediaConfig` + `with_config` (bind `0.0.0.0`, advertise IP, fixed UDP port; loopback
   default). Dissolved the loopback blocker. `session.rs` split 517→314 (cleared standing R5 debt).
2. **c002** — `compose.sovereign.yml` UDP media override; hosted default untouched.
3. **c003** — media authz on the **authenticated JWT subject** (WS→envelope→bridge, `from_session`
   fallback) + `seed-media-view.sh`.
4. **c004** — `run-media-decode.sh` runner; **ran it, recorded the real failure**.
5. **c005** — re-affirm gate off + SECURITY §6 + CHANGELOG + PHASE-24-SIGNOFF.

## Artifact Quality Summary

| Metric | Value |
| ------ | ----- |
| Changes | 5/5 archived |
| Final QA verdict | 5/5 ALL PASS |
| Re-runs / archive aborts | 0 (both carried lessons held cleanly) |
| Release gate at close | fmt ✅ · clippy --workspace ✅ · check ✅ · str0m 27 · gateway 39 · domain 10 |

### Notable

- Clean phase on process: no QA BLOCK, no archive abort — the phase-23 c002 (fmt) and c005
  (archive) lessons were applied preemptively (fmt before every gate; MODIFIED headers matched the
  base spec).
- **The live proof produced a real negative result** — the highest-value artifact this phase, and
  the one that kept the gate honest.

## Technical debt introduced

- **None structural.** The runner's broken no-JWT fallback is documented debt with a precise fix
  (`PHASE-24-DECODE-RESULT.md` + SIGNOFF). The un-observed decode is a gated deferral, not hidden.

## Lessons captured

1. **Run the proof, don't reason about it.** Executing `run-media-decode.sh` surfaced a concrete
   compose-merge blocker that no amount of code review would have — and produced the honest negative
   that a described "should work" would have papered over.
2. **A no-auth fallback is not free.** Chaining a standalone dev override as a third compose `-f`
   silently breaks because `depends_on` *replaces* across files. A sovereign no-auth profile must be
   purpose-built to merge, or the run must use a real JWT.
3. **Enforcing the file-size limit on your own addition clears standing debt.** `session.rs` was
   already over 500; the split to add `MediaConfig` brought it back into compliance.
4. **Confirm directive reversals.** The flip-guard question first came back "force the flip"; a
   confirm turned it back to the honest gate before it entered the plan. A single click is not a
   reversal of an eight-phase rule.

## Recommended next phase

**`phase-25-sovereign-sfu-decode-retry-and-flip`** — the narrow finish:

- Fix the runner's live path: a **purpose-built sovereign no-auth compose override** that merges
  cleanly (define `depends_on` compatibly / use `dev-endpoints` build args without replacing it),
  **or** a flint-gate `E2E_JWT` mint step for the authenticated path (preferred — exercises c003).
- Re-run `scripts/run-media-decode.sh`; **observe `framesDecoded > 0`**.
- **Only then** flip the `main.rs` sovereign branch + SECURITY §6 (media → functional) + CHANGELOG.
- Fold in G4 (LiveKit `realtime` cross-node; admin-ui OIDC once ADR-004 + IdP exist).

If live media infra remains impractical, pivot to the carried LiveKit/OIDC proofs or SDK-parity
work and hold the flip — the plane stays honestly gated.
