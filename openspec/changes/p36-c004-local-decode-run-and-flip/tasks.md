# Tasks — p36-c004-local-decode-run-and-flip

> Supersedes the CI-dependent half of `p36-c002`. Gate discipline from phases
> 16–36 is unchanged: no flip without a genuine `framesDecoded > 0`.

- [ ] T1: Audit `scripts/run-media-decode.sh` for its four `GITHUB_*` / `CI=true`
      branches. Make the local path the default; keep CI branches only where they
      are genuinely CI-specific (artifact upload paths), and delete any that
      exist solely to trigger a test.
- [ ] T2: Bring the sovereign stack up locally (`compose.sovereign.yml`, plus
      `compose.host-net.yml` if the bridge topology blocks ICE). Record which
      compose combination was used — it is load-bearing for interpreting the
      result.
- [ ] T3: Run the decode proof locally. Capture the gateway str0m log to a file
      that is **not** cleared by the script's EXIT trap (the c001 defect: the
      intended `gateway.log` uploads empty; only `gateway-capture.log` carried
      content).
- [ ] T4: Read the result and classify it before interpreting:
      **(a)** `framesDecoded > 0` — the media path works;
      **(b)** `framesDecoded = 0` with `ice=connected` — a media-path failure,
      compare against run 29112243615's fan-out/PLI evidence;
      **(c)** ICE never connects — an ENVIRONMENT result (the phases 28–34
      macOS/Colima topology), **not** a media-path result, and must not be
      reported as one.
- [ ] T5: Check whether the ADR-009 child's PLI/room work changed the picture:
      does the gateway log now show PLI/FIR requests and non-empty `room=`,
      where run 29112243615 showed zero and empty?
- [ ] T6: Gate decision. Flip `SFU_MODE=sovereign` in `crates/frf-gateway/src/main.rs`
      **only** on outcome (a). Any other outcome leaves it OFF.
- [ ] T7: Write `docs/PHASE-36-LOCAL-DECODE-RESULT.md` — compose topology used,
      the classification from T4, the T5 comparison, and the gate decision.
- [ ] T8: Spec delta — `specs/sovereign-media/spec.md`, `## MODIFIED Requirements`.
      **P1 gate:** `openspec validate p36-c004-local-decode-run-and-flip` passes.

## Not a task here

Running this proof in CI, under any circumstance. If the local environment
cannot host the proof, that is a finding to report — not a reason to reach for
a workflow.
