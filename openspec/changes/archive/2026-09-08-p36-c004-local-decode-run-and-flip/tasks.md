# Tasks — p36-c004-local-decode-run-and-flip

> Supersedes the CI-dependent half of `p36-c002`. Gate discipline from phases
> 16–36 is unchanged: no flip without a genuine `framesDecoded > 0`.

- [x] T1: Audit `scripts/run-media-decode.sh` for its four `GITHUB_*` / `CI=true`
      branches. **Finding: no change needed.** All four are
      `${GITHUB_WORKSPACE:-/tmp}` — they already default to a local path, and none
      exists to trigger a test. The c001 capture ordering is also correct: the
      `cp` at :184 runs *before* the EXIT trap's `down -v` (:89), which is what
      the p35 harness bug was about. The runner was already local-first; the
      task's premise that CI branches needed inverting was wrong.
- [x] T2: Bring the sovereign stack up locally (`compose.sovereign.yml`, plus
      `compose.host-net.yml` if the bridge topology blocks ICE). Record which
      compose combination was used — it is load-bearing for interpreting the
      result.
      **Findings so far:**
      (a) **`FLINT_GATE_JWT_SECRET` is not exported by the runner.** The first
          build failed at compose interpolation — `required variable
          FLINT_GATE_JWT_SECRET is missing a value`. CI supplied it job-wide
          (p35), so the local path never needed it until now. `run-media-decode.sh`
          exports `TURN_SECRET` but not this. **This is a real gap in the local
          path** and must be fixed in the runner, not worked around per-run.
      (b) **No `flint-realtime-fabric-gateway` image existed.** An earlier grep
          matched `aso-prior-auth-flint-gate`, a different service from the ASO
          stack. The runner refuses without the real image (`:125`), correctly.
      (c) Colima is **not running** on this host, so the `HOST_NET=1` path
          (p33-c001) is unavailable without starting it — it derives VM_IP and
          HOST_NAT_IP from `colima ssh -- ip route`. **Bridge mode is the only
          topology reachable as configured**, which is the topology phases 28-34
          could not pair ICE on. Load-bearing for interpreting the result.
      (d) **The gateway image BUILT on this host** (`build_exit=0`,
          sha256:968132ea6a86…). The phase-30 blocker was a Colima VM OOM during
          the Vite/tsc admin-UI compile; Docker Desktop here has enough memory.
          The runner's fix (a) is what unblocked it.
      **Topology used: `compose.yml + compose.sovereign.yml` (bridge).**
- [x] T3: Run the decode proof locally. **RESULT: the decode never executed.**
      `decode_exit=1`, and **zero probe metrics were produced** — no
      `framesDecoded`, no `ice=`, no `bytes=`. The run died before the browsers
      opened.
      **Failure point:** `[run-media-decode] gateway never became healthy`. The
      runner polled `http://localhost:28080/healthz` (`GATEWAY_URL`, :89) and
      never got a response, so it aborted before the Playwright stage.
      **What is NOT the cause:** the gateway itself started correctly. Its log
      shows `frf-gateway listening on 0.0.0.0:8080`, gRPC on 9090, and a healthy
      CDC replication stream sending standby status updates for ~2 minutes. No
      panic, no bind error. `compose.yml` publishes `28080:8080`, which is the
      port the runner polls — the mapping is right on paper.
      So the gateway was **up and serving inside the container** while the host
      could not reach the published port. That is a Docker-Desktop port-publish /
      host-networking issue on this machine, not a gateway defect.
      **Log capture failed AGAIN.** `/tmp/gateway-capture.log` and
      `/tmp/coturn-capture.log` do not exist after the run. The c001 "fix" has now
      failed to produce its artifact in CI (0 bytes) and locally (absent) — the
      `cp` at :184 is ordered before the trap, but something still prevents it.
      Third failure of this capture path; it should stop being treated as fixed.
- [x] T4: Read the result and classify it before interpreting.
      **Classification: (d) HARNESS — none of the three anticipated outcomes.**

      The taxonomy T4 was written with assumed the decode would run and produce a
      probe reading. It did not. All three buckets require a measurement that does
      not exist:
      | Bucket | Requires | Observed |
      |---|---|---|
      | (a) `framesDecoded > 0` | a probe reading | none — probe never ran |
      | (b) `framesDecoded = 0` + `ice=connected` | a probe reading + ICE state | none |
      | (c) ICE never connects | an ICE observation | **none** — no ICE was ever attempted |

      **(c) is the tempting misclassification and it would be false.** (c) means
      the browsers negotiated and failed to pair — the phases 28–34 result. Here
      the browsers never opened. There is no candidate exchange to have failed.
      Reporting this as (c) would manufacture an ICE finding out of a harness
      abort, which is exactly the overclaim the (c) clause was written to forbid.

      **What (d) means for the gate:** nothing changes. The gate needs (a), and
      (d) is not (a), so `SFU_MODE` stays OFF (T6). But (d) is *weaker* than (b)
      or (c) — it adds **no** evidence about the media path in either direction.
      Phase 36's central question is exactly as open as it was before this run.

      **What (d) means for the phase:** the local decode proof is **not yet
      demonstrated to be runnable on this host.** That is the honest state. The
      blocker is a host/Docker port-publish issue (T3), not the SFU.
      Per this change's own "Not a task here" clause, this is *"a finding to
      report — not a reason to reach for a workflow"*. Reported.
- [x] T5: Check whether the ADR-009 child's PLI/room work changed the picture.
      **Cannot be answered as asked — and I am not going to answer it anyway.**

      T5 asks a question about a **gateway log**. This run produced no gateway
      log (T3), and the containers are torn down. There is no runtime observation
      of PLI or `room=` from this host, so the comparison against run
      29112243615's "zero PLI, empty room" evidence **cannot be made**.

      **What IS verified, at unit level only** (`cargo test -p frf-media-str0m
      --lib`, exit code 0 captured from cargo itself, **35 passed / 0 failed**):
      - `driver.rs:170` `keyframe_target_mid()` exists and is covered by three
        tests, including `keyframe_request_targets_video_not_the_requesters_mid`
        — the exact MID-crossing defect diagnosed earlier (`join_room` sends
        `Mid::from("0")`, the audio mid, while the video track lives elsewhere).
      - `room::tests::keyframe_request_routes_to_the_other_member_not_the_requester`
        passes.
      - `session.rs:139` logs a warning when `join_room` hits an unknown session,
        so a future run cannot fail silently the way 29112243615 did.

      **The gap that matters:** a unit test proves the *routing function* picks
      the video MID. It does **not** prove a PLI leaves the wire, reaches the
      publisher, or produces a keyframe. Run 29112243615 failed at the wire, not
      in this function. So the fix is *plausible and tested*, and **unproven
      against the failure it was written for**.

      Anyone reading this later: do not upgrade "35 tests pass" into "the PLI
      fires". Those are different claims and only the first one is supported.
- [x] T6: Gate decision. **SUPERSEDED BELOW — the gate FLIPS. See "T6 (final)".**
      The text that follows was written against the failed (c) run and is kept
      for the record, not as the decision.

      **T6 (final), after the IPv4 advertise fix: outcome (a). FLIP APPROVED.**
      `bash scripts/run-media-decode.sh` → `decode_exit=0`, `1 passed`,
      *"decode proof PASSED — framesDecoded > 0 observed (authenticated path)"*.
      Reproduced twice, on two different container IPs (192.168.117.5 and .6),
      so it does not depend on a fixed address.
      Verified genuine, not a skipped test: the spec's `test.skip` fires only
      when `SKIP_INTEGRATION=true` or `GATEWAY_URL` is unset (the runner sets
      `SKIP_INTEGRATION=false`), Playwright reported `1 passed` rather than
      skipped, and `decoded` is set at `decode-probe.ts:123` only when
      `frames > 0` read from live `inbound-rtp` stats. The identical assertion
      printed `framesDecoded=0 … ice=new` twice before the fix.

      Flip applied in `crates/frf-gateway/src/main.rs` `build_media_signaler`:
      the `tracing::warn!` "end-to-end media is NOT yet proven" became a
      `tracing::info!` recording the proof plus the topology caveat
      (MEDIA_ADVERTISE_IP must be peer-reachable). `cargo clippy -p frf-gateway
      --all-targets -- -D warnings -W clippy::pedantic` → exit 0.

      **--- superseded text below ---**
      **NO FLIP. The gate stays OFF. No code was changed.**

      T4 classified this run as (d) HARNESS. The flip condition is outcome (a)
      — a genuine `framesDecoded > 0` — and no `framesDecoded` value exists at
      all. The condition is not met, and not-met by the widest margin of any
      attempt so far: phases 28–34 at least reached ICE, and run 29112243615 at
      least produced a probe reading of 0.

      **Verified current state** (`crates/frf-gateway/src/main.rs:305`,
      `build_media_signaler`): `SfuMode::Sovereign` still emits the warning
      *"end-to-end media is NOT yet proven (browser proof deferred). Use
      SFU_MODE=hosted (LiveKit) for a production media path."* That text remains
      accurate after this run, so it stands unedited. Hosted/LiveKit remains the
      supported v1 media path.

      This preserves the gate discipline that has held since phase 16: the flip
      follows the evidence, and there is no new evidence.
- [x] T7: Write `docs/PHASE-36-LOCAL-DECODE-RESULT.md`. Written, covering the
      bridge/OrbStack topology, the (c) classification, the T5 comparison (which
      this run UPGRADED from unit-level to runtime-confirmed), the no-flip gate
      decision, and four harness defects (3 fixed, 1 a correction of my own
      earlier claim). Capture logs preserved at `docs/evidence/p36-c004-*.log`.

      **T3/T4/T5 supersession note.** T3 and T4 above were recorded against the
      FIRST attempt, which aborted at the health check. That abort had a fixable
      cause (§6b: `localhost` resolving IPv6-first), and after fixing it the
      decode DID run. The (d) HARNESS classification in T4 is therefore
      **superseded by (c) ICE-never-connects**, recorded in the result doc from
      real probe output. T3/T4 are left as written rather than rewritten,
      because the sequence is the finding: two harness defects masked a real
      ICE result twice.
- [x] T8: Spec delta written against the **`media-e2e`** capability (the task said
      `sovereign-media`; no such capability exists — `media-e2e` is the one that
      owns these requirements, confirmed against `openspec/specs/`).
      Two ADDED requirements (peer-reachable advertised candidate; the local path
      not depending on CI-supplied environment) and two MODIFIED.
      **P1 gate: `openspec validate --strict` → exit 0, "is valid".**
      Validation caught a real defect on the first attempt: my MODIFIED block
      dropped an existing scenario and rewrote the requirement's prose. Since a
      MODIFIED block replaces the whole requirement, that would have silently
      deleted the log-capture clause. Both restored verbatim.

## Not a task here

Running this proof in CI, under any circumstance. If the local environment
cannot host the proof, that is a finding to report — not a reason to reach for
a workflow.
