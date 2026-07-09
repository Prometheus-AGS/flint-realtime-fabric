# Execution — phase-25-sovereign-sfu-decode-retry-and-flip

> Backend: **OpenSpec**. Dispatch contract for the 3 planned changes. KBD owns the loop; one task
> per turn via `/kbd-apply`. Never bare `/opsx:apply`.

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–24; per-change `qa-gate.sh` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict AND the archive output before archive** |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p25-c001-authenticated-decode-runner** — rework `run-media-decode.sh` for the authenticated
   path: bring the sovereign stack up **with flint-gate** (no broken `-f` fallback), obtain a real
   JWT (host `14457`; confirm the mint mechanism — proxy hook vs. mint call vs. direct HS256 with
   `FLINT_GATE_JWT_SECRET`), seed `(sub=dev-integration-user, view, room)`, run the harness with the
   JWT. shellcheck-clean.
2. **p25-c002-live-decode-run** — **the proof.** Execute the authenticated runner; observe
   `framesDecoded > 0`. Record the real outcome in `docs/PHASE-25-DECODE-RESULT.md`, incl. an
   ICE/DTLS/RTP (media-path) vs. harness diagnosis on failure. Depends: c001.
3. **p25-c003-flip-or-reaffirm** — flip `main.rs` **iff** c002 truly observed `framesDecoded > 0`;
   else re-affirm gated. + SECURITY §6, CHANGELOG, PHASE-25-SIGNOFF, G4 carried. Depends: c002.

## Honesty gate (terminal, operator-confirmed)

`c003` flips `SFU_MODE=sovereign` **only** on a genuine `framesDecoded > 0` from c002. A relaxed or
worked-around proof does **not** qualify. If the live run can't pass cleanly, land c001–c002 as
honest progress (with the concrete blocker) and re-affirm the gate off. No forced flip.

## First change to apply

`p25-c001-authenticated-decode-runner` (active in `progress.json`).
