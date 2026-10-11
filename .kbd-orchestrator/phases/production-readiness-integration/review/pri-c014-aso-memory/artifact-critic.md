# Artifact critic review — pri-c014-aso-memory

## Verdict

**FAIL**

The delivered artifacts do not qualify for `PASS`. Acceptance A requires a same-fixture browser campaign, but the recorded measurement runs in Vitest's Node environment and samples the Node process. The OpenSpec task list also remains incomplete. I found no separate CRITICAL/HIGH correctness, security, concurrency, or durability defect in the reviewed source paths; the blocking defects are acceptance and evidence defects.

## Findings

### CRITICAL — the change is incomplete and cannot be archived

`/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/openspec/changes/pri-c014-aso-memory/tasks.md:9` leaves task 5 unchecked. That task requires the source-bound receipt, applicable local quality checks, strict OpenSpec validation, and resolution of required review findings before archive. The current artifact set therefore cannot claim completion even though tasks 1–4 are checked.

The evidence narrative says TypeScript and oxlint exited zero at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/acceptance-b.md:91`, but it does not include a command/exit receipt for those checks or for strict OpenSpec validation. During this independent review, the following commands exited zero:

- `pnpm --dir web typecheck`
- `pnpm --dir web lint`
- `openspec validate pri-c014-aso-memory --strict --no-interactive` (`Change 'pri-c014-aso-memory' is valid`)

Required action: resolve the findings below, add a source-bound final verification receipt with the exact commands and observed exits, then check task 5.

### HIGH — Acceptance A is labeled “browser” but was measured in Node

The requirement explicitly calls for a “Same-fixture browser campaign” at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/openspec/changes/pri-c014-aso-memory/specs/production-readiness-integration/spec.md:8-11`. The dependent Q7 contract likewise says the first qualification target is the browser at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c003-scope-research-contract/release-contract.md:105-110`.

The measured test instead declares `// @vitest-environment node` at `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted.integration.test.ts:1`. Its memory helper uses `process.memoryUsage()` at `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted-runtime.ts:287-289`, and the coordinator invokes Vitest directly at `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/scripts/test-ra11c-materialization.py:71-77`. The campaign receipt confirms `environment 0ms` and Node process execution at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-profile-campaign.json:59-76`. The Acceptance-A receipt itself describes the runtime as a “Vitest Node host” at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/acceptance-a.json:41-44`.

The recorded Node result is numerically below the limits—444,743,680 incremental RSS bytes and 62,831,744 heap bytes in the current-source run—but it does not establish browser-process or browser-renderer memory behavior. Node and a browser differ in WASM allocation, Blob handling, browser storage, workers, and process accounting. Calling this a browser baseline does not supply the missing browser execution.

Required action: execute the 16,203-row fixture in an actual supported browser using the production browser opening path, measure browser/renderer incremental RSS and JavaScript heap with an identified method, and bind the receipt to the exact current source/configuration hashes. Keep the gate open until both fixed limits pass there, or record the explicit operator-approved replacement contract allowed by the requirement.

### MEDIUM — Acceptance-A's designated source binding is stale

`/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/acceptance-a.json:53-54` binds the mounted test and helper to hashes `d6aa…` and `2af2…`. The current split files are bound as `273f…` and `7a2a…` in `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/source-manifest.json:27-35`. That conflicts with the stale-evidence rule at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/openspec/changes/pri-c014-aso-memory/specs/production-readiness-integration/spec.md:18-21`.

The later allocation analysis does bind the current hashes and reports a passing number at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-analysis.json:9-14` and `:77-100`, so the numeric Node result is reproducible on the current split. The authoritative Acceptance-A artifact should still be regenerated from the final campaign rather than pointing at an earlier source-adopted run.

Required action: replace or amend Acceptance A so its primary campaign hash, source hashes, measurement, and observed time all identify the final candidate.

### MEDIUM — stage samples support bounded attribution, not the full allocation claim

The stage instrumentation records process RSS and V8 heap every 10 ms and at named boundaries (`/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted-runtime.ts:305-339`). The final campaign's boundary samples reconcile to the observed peak at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-profile-campaign.json:120-177`. This is useful evidence that PGlite/open-and-schema dominates the Node process and that the cold fold plus SQL/projector stages remain bounded.

It is not an allocation profile capable of separating PGlite from archive loading/WASM/database internals, or SQL projection rows from retained graph copies. The worker figure is explicitly an upper bound over a wider continuation interval, not a worker allocation measurement (`/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-analysis.json:43-51`). The prose mostly acknowledges these limitations, but the unqualified statement that Worker/DB/graph-copy and cold-fold “allocations explain the result” is stronger than the method supports.

Required action: describe these as stage-associated net RSS/heap deltas and bounded upper limits. If allocation-specific attribution remains required, add browser-appropriate heap/allocation instrumentation or controlled A/B runs that isolate each component.

### LOW — continuation `peakStage` is demonstrably mislabeled

`trackProcessMemory.sample(label)` stores the labeled sample but does not update the active `stage`; only `enter()` does that (`/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted-runtime.ts:311-325`). The continuation scenario calls `sample()` rather than `enter()`, so its receipt reports `peakStage: "baseline"` even though baseline RSS is 99,254,272 bytes and the peak is 462,536,704 bytes (`/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-profile-campaign.json:218-251`). This does not change the budget arithmetic or the labeled endpoint values, but it makes that metadata false.

Required action: either update `stage` when a labeled sample is taken or use `enter()` at continuation boundaries, then regenerate the affected receipt.

## Acceptance and implementation assessment

| Area | Assessment | Evidence |
|---|---|---|
| Acceptance A | **Not established** | Current Node measurement is under both thresholds, but the required browser campaign was not run. |
| Acceptance B | **Partially established** | Current-source mounted behavior survives the split and all eleven recorded durability/fencing checks pass; stage deltas give a coherent Node-process explanation, with the attribution limits above. |
| SQL/checkpoint atomicity | **Preserved in reviewed evidence** | The current campaign records rollback-before-checkpoint, committed restart, and no skipped SQL at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-analysis.json:53-67`. Production writes rows and the checkpoint inside one transaction at `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-runtime.ts:348-379`. |
| Owner/authority fencing | **Preserved in reviewed evidence** | The mounted campaign records disposed-owner fencing, replacement-owner continuation, and authority-failure fencing at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-profile-campaign.json:106-117`. |
| Experimental gate | **Preserved** | The reviewed delta is limited to the mounted test and its helper; no production adoption flag is changed. |
| Critical/high production-source defect | **None found in the reviewed paths** | The blocking findings concern qualification and evidence, not a newly observed source correctness/security/concurrency/durability failure. |

## Test-helper extraction and file-size assessment

The extraction preserves the mounted campaign's material semantics. `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/source-delta.patch:1` and `:409` show that the adopted delta changes only the mounted test and adds the helper. The current-source campaign ran all three processes successfully and retained the eleven behavior assertions at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-profile-campaign.json:59-117`. The memory phase intentionally captures the authenticated cold revision before applying it so the cold-fold and SQL/projector intervals can be sampled separately (`/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted.integration.test.ts:58-81`); the initial and continuation phases still exercise the real transport, rollback, restart, deletion replacement, owner closure, and authority fence.

Both c014-modified ASO files satisfy the 500-line rule:

- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted.integration.test.ts`: 411 lines.
- `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth/web/src/shared/sync/replica-mounted-runtime.ts`: 361 lines.

Those sizes are source-bound at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/source-manifest.json:27-35`. The 524-line `replica-runtime.ts` and 659-line `replica-runtime.test.ts` remain oversized, but the c014 delta does not modify them and the evidence explicitly tracks them as untouched at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/allocation-analysis.json:89-96`.

## Checks performed by this critic

- Recomputed current SHA-256 hashes for the mounted test, extracted helper, campaign coordinator, and five named production sources; they match `source-manifest.json` and the current-source allocation analysis.
- Confirmed the unsplit baseline SHA-256 is `6aaa3a9c35b42b80cacecc11f26e3326fa05a25fa0acec76c4105ed32dea8005`, matching `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c014-aso-memory/source-manifest.json:78-81`.
- Compared the unsplit baseline with the current test/helper and inspected the complete `source-delta.patch`.
- Recounted the current ASO files; the two c014-modified files are 411 and 361 lines.
- Ran TypeScript typecheck, oxlint, and strict OpenSpec validation locally; all exited zero.
- Did not rerun the full mounted campaign. Its final receipt contains three zero-exit Vitest runs, the source-bound hashes, behavior assertions, cleanup state, and measured memory values, which were reviewed directly.
