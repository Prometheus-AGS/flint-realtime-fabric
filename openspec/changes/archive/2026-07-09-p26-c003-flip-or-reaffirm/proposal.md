# p26-c003-flip-or-reaffirm

## Why

Phase-26 G3: flip `SFU_MODE=sovereign` **only** if the c002 decode run observed a genuine
`framesDecoded > 0`. It did **not** (`docs/PHASE-26-DECODE-RESULT.md`): the run reached the media
exchange for the first time (build→boot→healthy→Keto view grant→browser harness all pass) but the
WebRTC decode timed out — a genuine media-transport symptom, not plumbing. Per the
operator-confirmed honest gate, this **re-affirms the gate off**.

## What Changes — RE-AFFIRM GATED (no flip)

- **`SFU_MODE=sovereign` stays OFF.** No `main.rs` flip; the honest gate-off warning stays.
- `docs/SECURITY.md` §6 media row: record phase-26 progress (image builds + full pipeline reaches
  the media path; five blockers cleared) + the current media-transport blocker (decode timeout) +
  next step.
- `CHANGELOG.md`: Phase 26 section — the Dockerfile fix + the media-path frontier + the honest
  re-affirm.
- `docs/PHASE-26-SIGNOFF.md`: closing sign-off with the honest G1–G4 status + re-run gate results.
- **G4 carried** (re-affirmed integration-gated): LiveKit `realtime`; admin-ui OIDC.

## Impact

- `docs/SECURITY.md` §6, `CHANGELOG.md`, `docs/PHASE-26-SIGNOFF.md`. No production code; no flip.
