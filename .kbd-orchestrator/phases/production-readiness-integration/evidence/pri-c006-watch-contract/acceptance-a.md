# c006 acceptance A — watch and recovery scenario contract

Captured: `2026-09-15T14:34:31Z`

Status: **PASS** for the contract-freeze slice. Runtime delivery remains owned
by c007–c012 and is not claimed here.

## Consumer acceptance ordering

`consumer-acceptance.md` was written at `2026-09-15T14:24:06Z` while all c006
affected source paths were still clean. Its SHA-256 is
`589f79e17439ebed72fb3701e4cce03cb6ea670255cd4fff2910d91127d2b7c3`.
Only after that record existed were `proto/flint/v2/entity.proto`, Rust codegen
configuration and contract tests added. Forge and PEM implementation remains
explicitly assigned to later gates.

## Accepted scenario matrix

| Concern | Contract outcome | Deterministic fixture/assertion |
|---|---|---|
| schema/type | separate validated schema, table name and server-owned projection | three explicit entity types; no predicate or field selector |
| tenant | request tenant comes from verified context and checkpoint is scope-bound | cross-tenant checkpoint returns `checkpoint_scope_mismatch` |
| non-first key | primary-key ordinal is independent of table column order | UUID `id` is second in source order and first/only key part |
| composite key | each typed component retains boundary and ordinal | UUID `practice_id` plus signed integer `sequence` |
| non-UUID key | text keys retain type and exact UTF-8 value | `policy:cardiology:v2` fixture |
| canonical ID | field names, kinds and values cannot collide by concatenation | three exact `frfkey:v1` base64url vectors recomputed by script |
| committed transaction | no mutation publishes before COMMIT; poison blocks checkpoint | two events share LSN and have contiguous transaction indices |
| stable identity | replay retains deterministic identity | two exact `frfevent:v1` SHA-256 vectors recomputed by script |
| snapshot/live race | barrier is fixed before rows and live begins after completion | ordered accepted → rows → complete → post-barrier mutation frames |
| restart/replay | valid retained checkpoint resumes strictly after its boundary | `valid-restart` case returns accepted-after-checkpoint |
| old cursor | expired retention never silently seeks oldest | terminal `history_expired` resnapshot case |
| source recreation | epoch mismatch never resumes a different history | terminal `source_epoch_changed` resnapshot case |
| unauthorized history | no typed metadata or payload escapes | zero data frames; opaque checkpoint-only progress is permitted |
| projection | source-column order and allowlist are fixed by server | allowed fields ordered; tenant/internal note fields excluded |
| delete | authorization uses canonical key and no old row crosses boundary | record and previous record are absent; checkpoint follows auth |
| lag | bounded overflow is explicit and terminal | lag frame; silent drop false |
| cancellation | server stops production and releases subscription | zero produced frames after observed cancellation |

## Executed checks

- `node scripts/check-watch-contract.mjs`: PASS — 6 frozen v1 files, 3 key
  shapes, 4 resume cases, plus transaction, barrier, projection, delete,
  unauthorized-history, lag and cancellation assertions.
- `buf lint proto`: PASS.
- `buf breaking proto --against '.git#ref=HEAD,subdir=proto'`: PASS; v2 is
  additive relative to the Fabric base revision.
- `cargo test --offline -p frf-proto --all-targets --locked`: PASS — 5 tests.
  Generated oneofs, typed records, separate source/broker/checkpoint positions,
  key-only delete and typed resnapshot control round-trip through prost.

## Artifact identity

- v2 proto: `02ea2bf4db1a86a21e2cae25c4f2e6a2cd3c566fb158d3391f637f518d0df528`
- fixture vectors: `b9bdf9e37ff9ca18b4a610ce338f5443aa6df8336b787fe782a8f2abfd50ed29`
- ADR-010: `bfe7555ddee03ca312cd4fded85a8cb7c0049635c3f1e3e37963d8e039de9cf0`
- Rust generated-contract tests: `019ced5515effc934e7a792cb1d90d1575ad0cb59fe531683a090312c9f4a834`
- deterministic checker: `871eb326801a052931c7bc4ab73279733cad92950fc5ccdff260f771017746f2`
