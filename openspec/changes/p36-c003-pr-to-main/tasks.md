# Tasks — p36-c003-pr-to-main

- [x] T1: Verify the premise — confirm sovereign SFU work is on `main` and the proof branch is gone:
      `git merge-base --is-ancestor 9ba04ae origin/main` succeeds, and
      `git branch -a --list '*sovereign*'` returns nothing. If either check fails, STOP and re-assess.
- [x] T2: Remove the dead `push:` trigger (`branches: ["sovereign-sfu-decode-proof"]`) from
      `.github/workflows/decode-proof.yml`, leaving `workflow_dispatch` as the sole trigger. Do not
      add a `push` trigger on `main` — the job runs the full heavy stack (~12 min).
- [x] T3: Confirm the workflow still dispatches after the edit: `gh workflow list` shows
      "Decode proof" active, and `workflow_dispatch` is the sole parsed trigger.
      **Amended (operator, 2026-09-06):** the original task called for actually running
      `gh workflow run decode-proof.yml --ref main`. Live dispatch is NOT exercised — the operator
      directed that CI/CD workflows are not to be used for testing, and that no testing happens
      until all code is written. Verification is static only. A live dispatch remains outstanding
      and is the first step whenever the decode proof is next run.
- [x] T4: Write `docs/PHASE-36-SIGNOFF.md` — record the gate decision from c002 (flip or hold), the
      run 29112243615 evidence (`ice=connected`, `framesDecoded=0`), G2/G3 as met, and G5 as retired
      with the reason.
- [x] T5: Amend G5 in the phase `goals.md` to state the PR is moot: the work is already on `main`,
      so the goal is satisfied by merge rather than by opening a PR.
- [ ] T6: Commit the workflow and docs changes to `main`.
