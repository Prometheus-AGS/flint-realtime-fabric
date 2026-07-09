# p24-c005-flip-or-reaffirm

## Why

Phase-24 G3/G4: flip `SFU_MODE=sovereign` **only** if the c004 decode proof observed a genuine
`framesDecoded > 0`. It did **not** (`docs/PHASE-24-DECODE-RESULT.md`): the live run failed with a
compose-merge blocker in the runner's no-JWT fallback and the stack never came up — no decoded
frame. Per the operator-confirmed honest gate, this **re-affirms the gate off**.

## What Changes — RE-AFFIRM GATED (no flip)

- **`SFU_MODE=sovereign` stays OFF.** No `main.rs` flip; the honest gate-off warning stays.
- `docs/SECURITY.md` §6 media row: update to reflect phase-24 progress (bindable/advertised media
  socket, UDP compose path, authenticated-subject authz + Keto seed, decode harness+runner) while
  the in-env decode proof remains unmet — with the concrete c004 blocker + the exact next step.
- `CHANGELOG.md`: Phase 24 section — the c001–c004 real progress + the honest re-affirm.
- `docs/PHASE-24-SIGNOFF.md`: closing sign-off with the honest G1–G4 status + re-run gate results.
- **G4 carried** (re-affirmed integration-gated): LiveKit `realtime`; admin-ui OIDC.

## Impact

- `docs/SECURITY.md` §6, `CHANGELOG.md`, `docs/PHASE-24-SIGNOFF.md`.
- No production code change; no gate flip.
