# Tasks — p36-c002-decode-run-and-flip

> **Status (2026-09-07): DEFERRED — cannot be completed as written.**
>
> The operator has established a **permanent** policy: CI/CD is never used to run tests; all
> testing is local full integration. It is recorded in `CLAUDE.md` and `AGENTS.md` here and in
> the PEM and ASO repos.
>
> T1 and T4 are the only remaining tasks and **both require a CI run**, so under that policy
> this change is unsatisfiable in its current form. 6 of 8 tasks are genuinely complete —
> the evidence reading and all documentation, against run 29112243615.
>
> **This is not "done".** It is deferred with two tasks permanently blocked, and it is
> superseded by a successor change that verifies the decode path with a **local** integration
> run. Do not archive it as complete.

- [ ] T1: ~~Verify the CI decode-proof job ran~~ — **PERMANENTLY BLOCKED.**
      Requires a CI run, which policy forbids. The premise was already void (c001's T5 push
      branch is gone). Superseded by a local-integration decode run in the successor change.
- [x] T2: Download and read the `gateway.log` artifact — record candidate types exchanged and why pair succeeded or failed
      Done against run 29112243615. `gateway.log` is 0 bytes (the c001 fix is only partially
      effective); read `gateway-capture.log` (642 KB) instead. Findings: 1,997 fan-out events,
      zero PLI/FIR/keyframe, `room=` empty throughout, no room-join events.
- [x] T3: Read the Playwright report — record `framesDecoded`, `ice` state, `bytes`, `reason`
      Done against run 29112243615: `framesDecoded=0`, `ice=connected`, `bytes≈1,800,000`,
      `reason=timeout`, `localCandidates=1 remoteCandidates=1`, consistent across 3 attempts.
- [ ] T4: ~~Make gate decision from a fresh CI run~~ — **PERMANENTLY BLOCKED.**
      Requires a CI run, which policy forbids. On the last evidence (`framesDecoded=0`) the gate
      is correctly OFF and stays OFF; no flip is warranted and none was made. The gate decision
      moves to the successor change, made from a local integration run.
- [x] T5: Write `docs/PHASE-36-DECODE-RESULT.md` with full evidence
- [x] T6: Write `docs/PHASE-36-SIGNOFF.md` with gate decision + carried items (if any)
      Satisfied by p36-c003 T4, which wrote this file. Not duplicated.
- [x] T7: Update `docs/SECURITY.md` §6
- [x] T8: Update `CHANGELOG.md` with phase-36 entry
