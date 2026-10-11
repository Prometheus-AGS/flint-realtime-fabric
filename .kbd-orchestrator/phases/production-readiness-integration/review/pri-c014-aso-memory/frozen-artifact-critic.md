# Frozen artifact critic — pri-c014-aso-memory

## Verdict

**PASS — closure ready.**

The frozen seven-file ASO delta and its source-bound evidence substantively satisfy acceptance A, acceptance B, and task 5. Task 5 may be checked when `kbd-apply` performs its end-task transition. Its currently open checkbox is not a defect in this review.

## Findings

### BLOCKING

None.

### HIGH

None.

### MEDIUM

None.

### WARNING — browser launcher is host-specific

`scripts/test-ra11c-browser-memory.py` fixes Chrome to `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`. This is valid for the recorded local qualification host and both final campaigns ran with that exact executable, but rerunning on another valid host requires editing the script. Accepting a CLI or environment override would improve portability. This does not invalidate the frozen local evidence or block this change.

### RELEASE RISK — measured margin is finite

The authoritative run has 149,323,776 bytes of target-renderer RSS headroom and 126,027,350 bytes of heap headroom. It passes the frozen limits; later capacity qualification should retain this as an explicit risk.

## Acceptance assessment

- **Acceptance A: Passed.** `browser-campaign.json` records a real Google Chrome 152 campaign over the 16,203-row fixture. Target-renderer RSS grew 387,547,136 bytes, aggregate isolated Chrome-tree RSS grew 371,605,504 bytes, and JS heap grew 142,408,106 bytes. The limits are 536,870,912 RSS bytes and 268,435,456 heap bytes. SQL counts, graph counts, seven shape checkpoints, and heap availability all passed.
- **Strict repeat: Passed.** `browser-campaign-aggregate-gate-pass-1.json` uses the identical 31-file input aggregate and records 406,781,952 target-renderer RSS bytes, 172,949,504 aggregate RSS bytes, and 185,001,736 heap bytes. Both renderer and aggregate RSS gates and the heap gate are true in both final receipts.
- **Acceptance B: Passed.** The stage measurements describe net boundaries and upper bounds without claiming allocator-specific attribution. PGlite/WASM dominates the opening boundary; cold fold adds 4,205,841 net heap bytes over the empty projector boundary; the final post-fold renderer interval is bounded at 7,946,240 bytes. The explanation matches the recorded samples.
- **Durability and fencing: Passed.** All 13 derived checks are true: initial parity, complete checkpoint, gate-summary publication, evidence-state identities, cold rollback, row/checkpoint rollback, restart without skip, delete replacement, disposed-owner fencing, replacement-owner continuation, authority-failure fencing, final SQL/graph counts, and generation/checkpoint parity. Both mounted Vitest phases exited 0 in each strict campaign.
- **Cleanup: Passed.** All synthetic relational counts are zero; the isolated PGlite store and callback are gone; the synthetic identity was deleted; service existence/running state and the database role state were restored.

## Source and scope binding

ASO HEAD is `d95242542060d1e19a6efe9143f57c015d6162f4`; Fabric HEAD is `ae26f45d8afa037677004cd118b0571b8b552131`. Both match the receipt and current files.

The seven current ASO file hashes match `source-manifest.json` and the reverse patch check succeeds:

| File | SHA-256 | Lines |
|---|---|---:|
| `web/src/shared/sync/replica-mounted.integration.test.ts` | `5e394e3e505b7f6cb9daf32ba8dd8c61a4398e94457325f18f42c52470c93849` | 449 |
| `web/src/shared/sync/pglite-bootstrap.ts` | `5d7cf6b33bfae8c0c6e74287950575c10741c2aa3e07dddca2b69045a799b0c6` | 45 |
| `web/src/shared/sync/replica-mounted-runtime.ts` | `7caaaa829f37e7f393bd3f56f9de015ef6503acdd5536cbca4e9f2f9d37756c0` | 355 |
| `web/ra11c-browser-memory.html` | `4a26bd7963b6efb023a4a5f152170a9e350e9737a18f982b97b449e17662dd72` | 12 |
| `web/src/shared/sync/replica-browser-memory.ts` | `a8d17a9a591b08cce218f843621b35cf9b9788320b241a0eb98bb4f6e56a2208` | 341 |
| `scripts/test-ra11c-browser-memory.py` | `10e3c049d5c4299e34bb1024377d06912a4fb5ff05b5f4de3ea4860fc1b24205` | 346 |
| `scripts/test-ra11c-materialization.py` | `be8af39283fe56e1fac1c384a755986df5e10712770f96ca02962730f2b9bdf1` | 259 |

`source-delta.patch` contains exactly these seven paths and has SHA-256 `fc7caadd22b9c03a9e248a0e99cd99e2ab0285a1dd37d35bb358c652916be41e`. It excludes the unrelated dirty ASO workspace and concurrent Fabric Rust work. Every touched hand-authored file is below 500 lines. The untouched oversized files remain explicitly recorded as `replica-runtime.ts` (524) and `replica-runtime.test.ts` (659).

The recursive campaign closure contains 31 current inputs. Every current file hash matches the manifest, and the canonical before/after aggregate is identical: `b71fc53df00686138ea1472cbff2ed01fa91d592b0d1761b6ae8e272dd70c3dd`. The manifest SHA-256 is `696ee918fc4305d6734fbd4c7d5e7fde09408e7bca73c61b7b09e3d818416f5f`.

Primary artifact integrity also recomputes exactly from `receipt.json`: authoritative campaign `fff597119fce523e633b7030138159f758fa622e4e520776693c0d658e8f31ac`, strict repeat `4c3666b2beeef3a46ce7f32a76d01fca1341ad9cacd75efd2d9ce30207a907a7`, acceptance A `ab280a0fb15f1badc630e5370cc07e5505fcadf8a133afd8b07dcca9be30181a`, allocation analysis `73ba118a76c7ff7d04c5f546936f7a69117f795b488eeead0050e52a44e9bfdc`, receipt `9e4587429e79372c948eb14b56e3efe83461359b9e244637ae1d400c792891e8`, quality checks `3551ed7fb9286c01299df079b3b993012f7f3273e16bebaba9520c2c86b61f88`, final packet `8daa966edcc7826413b2f2d4f2062b27fbd85d76cee43e8953d921f1ac7992c7`, and final findings `fb4fb72369affdfbc34771e2279ffac8119850187024573630803d3ed8396103`.

## Commands independently run

```text
pnpm --dir web typecheck
  exit 0; $ tsc --noEmit

pnpm --dir web lint
  exit 0; $ oxlint

bash scripts/audit.sh
  exit 0; audit: PASS (6/6 checks)

openspec validate pri-c014-aso-memory --strict --no-interactive
  exit 0; Change 'pri-c014-aso-memory' is valid

git apply --reverse --check <absolute-path>/source-delta.patch
  exit 0

shasum -a 256 <seven ASO files and frozen artifacts>
  all values match the manifest and receipt

jq -cS -j '.executedInputFingerprints.before.files' source-manifest.json | shasum -a 256
  b71fc53df00686138ea1472cbff2ed01fa91d592b0d1761b6ae8e272dd70c3dd
```

The recorded production build and isolated Python compile also exited 0. I did not rerun the full mounted campaign; I verified both complete final receipts, their source closure, current hashes, derived verdicts, cleanup, and negative historical receipts.
