# Tasks — p36-c002-decode-run-and-flip

- [ ] T1: Verify the CI decode-proof job ran (triggered by c001 T5 push); link to the run
- [ ] T2: Download and read the `gateway.log` artifact — record candidate types exchanged and why pair succeeded or failed
- [ ] T3: Read the Playwright report — record `framesDecoded`, `ice` state, `bytes`, `reason`
- [ ] T4: Make gate decision — flip `SFU_MODE=sovereign` ONLY if `framesDecoded > 0`
- [ ] T5: Write `docs/PHASE-36-DECODE-RESULT.md` with full evidence
- [ ] T6: Write `docs/PHASE-36-SIGNOFF.md` with gate decision + carried items (if any)
- [ ] T7: Update `docs/SECURITY.md` §6
- [ ] T8: Update `CHANGELOG.md` with phase-36 entry
