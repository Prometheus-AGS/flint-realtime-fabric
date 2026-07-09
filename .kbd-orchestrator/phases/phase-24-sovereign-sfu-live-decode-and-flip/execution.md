# Execution — phase-24-sovereign-sfu-live-decode-and-flip

> Backend: **OpenSpec**. Dispatch contract for the 5 planned changes. KBD owns the loop; one task
> per turn via `/kbd-apply`. Never bare `/opsx:apply`.

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–23; per-change `qa-gate.sh` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict AND the archive output before archive** (phase-23 c002 + c005 lessons) |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p24-c001-str0m-bind-config** — `MediaConfig` + `with_config`, threaded to `negotiate`; bind
   `0.0.0.0:<udp_port>`, advertise `advertise_ip`; loopback default kept. Rust QA.
2. **p24-c002-compose-sovereign-media** — `SFU_MODE=sovereign` compose path + UDP mapping +
   `MEDIA_ADVERTISE_IP`; prod hosted default untouched. Depends: c001.
3. **p24-c003-keto-view-seed** — Keto `view` tuple for the harness subject/room (no bypass).
4. **p24-c004-dagger-decode-job** — boot sovereign stack + run `media-decode.spec.ts` w/ Chromium
   fake-media; assert `framesDecoded>0`; capture evidence. **The proof.** Depends: c001+c002+c003.
5. **p24-c005-flip-or-reaffirm** — flip `main.rs` **iff** c004 truly observed `framesDecoded>0`;
   else re-affirm gated. + SECURITY §6, CHANGELOG, PHASE-24-SIGNOFF, G4 carried. Depends: c004.

## Honesty gate (terminal, operator-confirmed)

`c005` flips `SFU_MODE=sovereign` **only** on a genuine `framesDecoded>0` from c004. A relaxed or
worked-around proof does **not** qualify — if the live run can't pass cleanly this phase, land
c001–c004 as honest progress and re-affirm the gate off with fresh detail. No forced flip (the
8-phase discipline, re-confirmed at plan time).

## First change to apply

`p24-c001-str0m-bind-config` (active in `progress.json`).
