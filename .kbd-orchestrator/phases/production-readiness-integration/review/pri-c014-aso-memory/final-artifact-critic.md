# Final artifact critic — pri-c014-aso-memory

## Verdict

**FAIL**

The final runtime result is credible and internally consistent, but the mandatory exact source binding is incomplete. The certification gate must remain open under the stale-or-missing fingerprint scenario in the change spec.

## Findings

### CRITICAL

None.

### HIGH — the campaign's executed untracked inputs are not content-bound

`scripts/test-ra11c-materialization.py` directly imports `scripts/test-ra11a-sync-conformance.py`. That coordinator imports `scripts/ra06c_campaign_config.py`, dynamically loads `scripts/test-authorized-shape-composition.py`, and invokes `scripts/ra05-stack.sh`. These four files are all untracked in the ASO checkout. They control fixture creation, service topology, readiness, authentication, cleanup, and the final pass/fail report, but none appears in `source-manifest.json`, `acceptance-a.json`, `allocation-analysis.json`, or `receipt.json` with a content hash.

The manifest's `porcelainSha256` and `untrackedPathsSha256` match the current checkout, but Git porcelain and `git ls-files --others -z` hash only untracked path names. Their values do not change when an untracked file's contents change. The tracked-diff hash cannot cover these files either. Runtime image/config hashes bind the containers that ran; they do not bind the Python and shell evidence producers.

This leaves the exact final versions of the fixture generator and campaign coordinator unprovable. It violates the design requirement to capture source/configuration fingerprints and triggers the spec's rule that a missing matching fingerprint keeps certification open.

Required resolution: content-hash every directly executed/imported campaign input, including the four files above and any required untracked composition/config inputs, capture the hashes before and after the run, rerun the authoritative final campaign, and regenerate the primary artifacts and receipt from that run. Keep task 5 unchecked until that source-bound receipt passes review.

### MEDIUM

None.

### LOW

None.

## Verified final-candidate results

- The authoritative `browser-campaign.json` is an actual Google Chrome 152 run. Chrome was launched as an isolated process tree, Playwright connected over CDP, renderer RSS was sampled from `ps`, and heap came from `performance.memory` with precise-memory reporting.
- The final renderer grew by **513,245,184 bytes**, below the **536,870,912-byte** limit by **23,625,728 bytes**. Heap grew by **197,966,575 bytes**, below the **268,435,456-byte** limit by **70,468,881 bytes**. The receipt arithmetic and stage samples reconcile.
- The 16,203-row fixture arithmetic reconciles: 360 cases, 4,320 case-evidence rows, 2,880 documents, 8,640 citations, and 3 evidence states. Browser SQL and graph counts match for every replica table, with all seven checkpoints present.
- The mounted durability passes preserve initial SQL/graph parity, SQL/checkpoint rollback, restart recovery, delete replacement, disposed-owner fencing, replacement-owner continuation, and authority-failure fencing. All eleven recorded checks are true.
- Cleanup is complete in the final receipt: synthetic relational counts are zero, the isolated PGlite directory is absent, the synthetic identity was deleted, the database role state matches its pre-run state, and pre-existing services returned to their pre-run existence/running sets.
- All six files in `source-delta.patch` are at most 500 lines. The reverse patch applies cleanly to the current ASO checkout. The untouched 524-line `replica-runtime.ts` and 659-line test remain explicitly tracked.
- All eight primary artifact hashes and sizes in `receipt.json` recompute exactly. All 18 listed source hashes, the complete tracked diff hash, the porcelain hash, the untracked-path hash, ASO HEAD, Fabric HEAD, spec hash, and task hash match the current checkouts.
- `openspec validate pri-c014-aso-memory --strict --no-interactive` independently prints `Change 'pri-c014-aso-memory' is valid`. The recorded typecheck, lint, build, architecture audit, and Python compile commands exited zero on the same listed source.

## Earlier findings

The prior actual-browser, stale-primary-hash, allocation-wording, and continuation-stage findings are resolved in the final artifacts. Allocation claims are now stage-associated bounds rather than allocator-specific claims, and continuation `peakStage` is labeled `restart-update`. Older failed and pre-final campaigns were treated only as historical evidence.

The generic Rust/semver findings in `final-findings.json` are not findings against this candidate: `source-delta.patch` changes six ASO TypeScript/Python/browser-harness files and contains no Fabric Rust or public-port edit. The remaining open issue is the incomplete content binding above.
