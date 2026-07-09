# Execution — phase-28-sovereign-sfu-decode-harness-timing-and-proof

> Backend: **OpenSpec**. Dispatch contract for the 3 planned changes. KBD owns the loop; one task
> per turn via `/kbd-apply`. Never bare `/opsx:apply`.
>
> **Process reminder:** seed the openspec change dir (`openspec/changes/<id>/{proposal,tasks,specs}`)
> at the START of each `/kbd-apply`, before `begin-task` (phase-26 c002 lesson).

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–27; per-change `qa-gate.sh` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict AND the archive output** |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p28-c001-harness-visibility** — `media-decode.spec.ts`: drop the redundant
   `connectToSovereignSfu` pre-check + `test.setTimeout(60_000)`; `run-media-decode.sh`: dump gateway
   logs before the cleanup `down -v` on harness failure. Typecheck + shellcheck.
2. **p28-c002-diagnose-and-fix** — re-run; read `ice`/`remoteCandidates` + gateway str0m logs;
   record `docs/PHASE-28-DIAGNOSIS.md`; **fix the revealed media-path issue** (evidence-driven).
   Depends: c001.
3. **p28-c003-decode-run-and-flip** — re-run; observe `framesDecoded > 0`; record
   `PHASE-28-DECODE-RESULT.md`; **flip iff genuine pass** (main.rs + SECURITY §6 + CHANGELOG +
   PHASE-28-SIGNOFF), else re-affirm. + G4 carried. Depends: c002.

## Honesty gate (terminal, operator-confirmed)

`c003` flips `SFU_MODE=sovereign` **only** on a genuine `framesDecoded > 0`. A relaxed or
worked-around proof does **not** qualify. If the run can't pass cleanly, land c001–c002 + the
diagnosis and re-affirm the gate off. No forced flip.

## First change to apply

`p28-c001-harness-visibility` (active in `progress.json`).
