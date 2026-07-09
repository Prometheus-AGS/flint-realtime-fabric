# Execution — phase-27-sovereign-sfu-media-transport-debug

> Backend: **OpenSpec**. Dispatch contract for the 4 planned changes. KBD owns the loop; one task
> per turn via `/kbd-apply`. Never bare `/opsx:apply`.
>
> **Process reminder (phase-26 c002 lesson):** seed the openspec change dir
> (`openspec/changes/<id>/{proposal,tasks,specs}`) at the START of each `/kbd-apply`, before
> `begin-task`.

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–26; per-change `qa-gate.sh` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict AND the archive output** |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p27-c001-media-instrumentation** — str0m driver lifecycle `info!` logs (offer/host-candidate/
   ICE-state/first-MediaData/forward) + harness ICE-state + candidate logging + last-state-on-
   timeout. Rust + TS.
2. **p27-c002-trickle-ice-over-ws** — FIND-1/2: gateway `/ws/v1/signal` relays
   `MediaTransport::local_signals` (trickle + state) out as `ice-candidate` frames; harness
   `onicecandidate`→send + inbound `ice-candidate`→`addIceCandidate`. Rust tests. Depends: c001.
3. **p27-c003-room-join-fanout** — FIND-3: harness sends `RoomJoin` so sender+receiver share
   `e2e-decode-room` → `RoomRouter` fans out. Depends: c001/c002.
4. **p27-c004-decode-run-and-flip** — re-run; observe `framesDecoded > 0`; record
   `PHASE-27-DECODE-RESULT.md`; **flip iff genuine pass** (main.rs + SECURITY §6 + CHANGELOG +
   PHASE-27-SIGNOFF), else re-affirm gated. + G4 carried. Depends: c002 + c003.

## Honesty gate (terminal, operator-confirmed)

`c004` flips `SFU_MODE=sovereign` **only** on a genuine `framesDecoded > 0`. A relaxed or
worked-around proof does **not** qualify. If the live run can't pass cleanly, land c001–c003 +
the diagnosis and re-affirm the gate off. No forced flip.

## First change to apply

`p27-c001-media-instrumentation` (active in `progress.json`).
