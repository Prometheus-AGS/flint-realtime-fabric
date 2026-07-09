# Execution — phase-26-gateway-docker-adminui-embed-and-decode

> Backend: **OpenSpec**. Dispatch contract for the 3 planned changes. KBD owns the loop; one task
> per turn via `/kbd-apply`. Never bare `/opsx:apply`.

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–25; per-change `qa-gate.sh` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict AND the archive output before archive** |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p26-c001-dockerfile-adminui-embed** — add a Node 24 build stage to `Dockerfile` (pnpm
   workspace install + `vite build`, using the `frf-wasm` stub — no Rust→wasm step) and
   `COPY --from=<node> …/admin-ui/dist ./admin-ui/dist` into the Rust builder **before**
   `cargo build`. Verify `docker compose -f compose.yml -f compose.sovereign.yml build gateway`
   succeeds.
2. **p26-c002-live-decode-run** — **the proof, first run to reach the SFU.** Re-run
   `scripts/run-media-decode.sh` (authenticated); observe `framesDecoded > 0`. Record the actual
   outcome in `docs/PHASE-26-DECODE-RESULT.md`, incl. a media-path diagnosis on failure. Depends: c001.
3. **p26-c003-flip-or-reaffirm** — flip `main.rs` **iff** c002 truly observed `framesDecoded > 0`;
   else re-affirm gated. + SECURITY §6, CHANGELOG, PHASE-26-SIGNOFF, G4 carried. Depends: c002.

## Honesty gate (terminal, operator-confirmed)

`c003` flips `SFU_MODE=sovereign` **only** on a genuine `framesDecoded > 0` from c002. A relaxed or
worked-around proof does **not** qualify. If the live run can't pass cleanly, land c001–c002 as
honest progress (with the concrete blocker) and re-affirm the gate off. No forced flip.

## First change to apply

`p26-c001-dockerfile-adminui-embed` (active in `progress.json`).
