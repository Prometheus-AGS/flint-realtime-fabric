# Tasks — p36-c002-decode-run-and-flip

> **Status (2026-09-06): partially applied.** The documentation tasks are complete against the
> most recent existing run, 29112243615. The tasks that require a *new* CI run are open: the
> operator directed that CI/CD workflows are not used for testing until all code is written.
> Resume T1 and T4 when testing is authorized — the trigger is now
> `gh workflow run decode-proof.yml --ref main` (see p36-c003; the old push trigger was dead).

- [ ] T1: Verify the CI decode-proof job ran (triggered by c001 T5 push); link to the run
      **BLOCKED — needs a live CI run.** Also note the premise changed: c001's T5 push cannot
      trigger anything, as the `sovereign-sfu-decode-proof` branch and its push trigger are gone.
      Use `gh workflow run decode-proof.yml --ref main` instead.
- [x] T2: Download and read the `gateway.log` artifact — record candidate types exchanged and why pair succeeded or failed
      Done against run 29112243615. `gateway.log` is 0 bytes (the c001 fix is only partially
      effective); read `gateway-capture.log` (642 KB) instead. Findings: 1,997 fan-out events,
      zero PLI/FIR/keyframe, `room=` empty throughout, no room-join events.
- [x] T3: Read the Playwright report — record `framesDecoded`, `ice` state, `bytes`, `reason`
      Done against run 29112243615: `framesDecoded=0`, `ice=connected`, `bytes≈1,800,000`,
      `reason=timeout`, `localCandidates=1 remoteCandidates=1`, consistent across 3 attempts.
- [ ] T4: Make gate decision — flip `SFU_MODE=sovereign` ONLY if `framesDecoded > 0`
      **BLOCKED — needs a live CI run.** On the existing evidence the gate is correctly OFF
      (`framesDecoded=0`); no flip is warranted. Re-evaluate after the next run.
- [x] T5: Write `docs/PHASE-36-DECODE-RESULT.md` with full evidence
- [x] T6: Write `docs/PHASE-36-SIGNOFF.md` with gate decision + carried items (if any)
      Satisfied by p36-c003 T4, which wrote this file. Not duplicated.
- [x] T7: Update `docs/SECURITY.md` §6
- [x] T8: Update `CHANGELOG.md` with phase-36 entry
