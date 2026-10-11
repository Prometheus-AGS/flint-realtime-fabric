# Baseline — pri-c014-aso-memory

Date: 2026-09-17  
Change: `pri-c014-aso-memory` task 1  
Owner repository: `/Users/gqadonis/Projects/TribeHealth/kevin/prior-auth`

## Scope and repository identity

- ASO branch: `main`
- ASO `HEAD`: `d95242542060d1e19a6efe9143f57c015d6162f4`
- Affected paths:
  - `web/src/shared/sync/`
  - `scripts/test-ra11c-materialization.py`
- PGlite declaration: `^0.5.8`
- PGlite lock identity: `0.5.8`
- `AGENTS.md` SHA-256: `67ce1bb8e415fbc523a77639ab690033627de2fca4564577094297973b722e41`
- `CLAUDE.md` SHA-256: `67ce1bb8e415fbc523a77639ab690033627de2fca4564577094297973b722e41`

The ASO working tree contains extensive pre-existing tracked and untracked work.
No ASO source was changed while producing this baseline. The c014 implementation
must be isolated from that work and must not stage or commit it.

## Dirty-state binding

Hashes were recomputed immediately before this receipt because concurrent ASO
work changed the earlier snapshot.

| Surface | SHA-256 |
|---|---|
| Complete unstaged diff | `a7dd3a3b27f93f414015c42263d11c906830ad4e9554d1a4546de17ab666be49` |
| Complete staged diff | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Sorted untracked-path list | `996c3fbdee40668e60069236796a48efb5bc89144685ee4bf73e02dbce1ba5e8` |
| Porcelain status | `c3dd2ecc58d66bbc9df54454fb9824dd4c6a84f61688d49184c37af0f7c4b9ec` |
| Affected-path tracked diff | `58221d2dae34378bb9baf6331a4d150a0923b68d060ef44e791438c641a4026b` |
| Sorted affected-file content manifest | `45d23082bb6688224f17de895a1d72e7db057e3999cba3a31fd8a0e6a6c8497c` |

Twenty-one files in the affected surface are currently untracked, including the
campaign coordinator, PGlite bootstrap, mounted integration campaign, worker
owner, continuation, and materializer tests. A clean implementation candidate
therefore has to be constructed from the exact current snapshot before a c014
delta can be reviewed.

## File-size boundary

The owning repository requires every modified file to remain at most 500 lines.
The current affected surface includes these pre-existing oversized files:

| File | Lines | Consequence |
|---|---:|---|
| `replica-runtime.ts` | 524 | Must be split before or as part of any modification. |
| `replica-runtime.test.ts` | 659 | May remain untouched; must be split if modified. |
| `replica-mounted.integration.test.ts` | 695 | May remain untouched; must be split if modified. |

`replica-worker-owner.ts` is 494 lines, leaving only six lines under the same
limit. Other affected implementation files are below 500 lines at baseline.

## Dependency receipts

| Dependency | Result | Receipt SHA-256 |
|---|---|---|
| `pri-c002-local-fixtures` | PASS: isolated local integration runner, real Iggy delivery, disabled-delivery negative control, cleanup, workspace checks | `4409fe7fa6ca1db3b01091ea8160fb4317a8102b8fd1e5074912177185716622` |
| `pri-c003-scope-research-contract` | COMPLETE: conservative release contract and dependency qualification | `84583fc30b6b5ef13bd41ace5d02b2447f9133da40bb4ee3cd2ed9c09778c366` |
| c003 release contract | Q7 fixes the ASO ceiling at 512 MiB incremental RSS and 256 MiB heap | `29752a3109a4602a1edcf14e72be70b6f5b43270ba7ea165b3de9ab148c857bd` |

Both required dependencies are complete and permit c014 implementation to
start. No operator decision is outstanding for the fixed memory contract.

## Existing mounted evidence

The current-source mounted campaign already proves the functional behaviors
that c014 must preserve: lifecycle, SQL/graph parity, SQL/checkpoint rollback,
restart after committed SQL, owner fencing, and authority failure before
checkpoint commit. It does not satisfy the memory gate.

| Measurement | Existing result | Contract |
|---|---:|---:|
| Rows | 16,203 | Same fixture required |
| Baseline RSS | 92,389,376 bytes | informational |
| Peak RSS | 1,109,082,112 bytes | informational |
| Incremental RSS | 1,016,692,736 bytes | <= 536,870,912 bytes |
| Heap | 77,579,792 bytes | <= 268,435,456 bytes |
| Peak stage | `memory-initial-sync` | must be explained |

The incremental RSS exceeds the fixed ceiling by 479,821,824 bytes. Heap passes,
so the first profile must distinguish PGlite/WASM/database allocation from the
worker, JavaScript graph copy, and cold-fold allocation surfaces.

Evidence bindings:

| Existing ASO artifact | SHA-256 |
|---|---|
| `task-4-memory-block.md` | `06afe33aa6c18645295d1f750f517fc296ecefc24506c4b03f934532daf56ed2` |
| `task-4-mounted-acceptance.json` | `4cdc18dd32e7bcf3db358bcb5e41acf793ed8ed46aa32d39f607bcaca9c0c7c4` |
| `task-4-mounted-materializer.json.memory.json` | `95cc8971d5bf4dd7b5026b43c8ec0c0bbe66fb9f527eaf0c4f7a7f370895fcbd` |
| `task-4-mounted-materializer.json` | `14134c66c20e04711c67501e9cfa7d632ae5ae32c09d301d9db9b286aa191b91` |
| `task-9-current-source-mounted-replay.md` | `de069d74ce740635fe73f1db82f6c5529e18302373bed3c9a0e71853d149976d` |

## Task-2 entry decision

Task 2 may begin. Reproduce the current 16,203-row campaign before editing,
capture stage-level process and PGlite measurements, then create a clean isolated
candidate from this exact affected-file snapshot. Preserve the experimental
adoption gate, SQL/checkpoint atomicity, and owner fencing. Do not modify the
dirty ASO checkout until the measured correction and its bounded delta are known.
