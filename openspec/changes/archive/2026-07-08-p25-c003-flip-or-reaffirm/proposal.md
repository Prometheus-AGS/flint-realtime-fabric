# p25-c003-flip-or-reaffirm

## Why

Phase-25 G3: flip `SFU_MODE=sovereign` **only** if the c002 authenticated decode run observed a
genuine `framesDecoded > 0`. It did **not** (`docs/PHASE-25-DECODE-RESULT.md`): the run cleared
two prior blockers (compose merge, HS256/RS256 mismatch) but failed on a **Dockerfile defect** —
`frf-gateway`'s `rust-embed` of `admin-ui/dist` fails because the Dockerfile never builds/copies
it — so the gateway image won't compile and the stack never came up. No decoded frame. Per the
operator-confirmed honest gate, this **re-affirms the gate off**.

## What Changes — RE-AFFIRM GATED (no flip)

- **`SFU_MODE=sovereign` stays OFF.** No `main.rs` flip; the honest gate-off warning stays.
- `docs/SECURITY.md` §6 media row: record phase-25 progress (authenticated RS256 mint + JWKS +
  clean sovereign stack, two blockers cleared) + the current Dockerfile-embed blocker + next step.
- `CHANGELOG.md`: Phase 25 section — authenticated runner + the honest re-affirm.
- `docs/PHASE-25-SIGNOFF.md`: closing sign-off with the honest G1–G4 status + re-run gate results.
- **G4 carried** (re-affirmed integration-gated): LiveKit `realtime`; admin-ui OIDC.

## Impact

- `docs/SECURITY.md` §6, `CHANGELOG.md`, `docs/PHASE-25-SIGNOFF.md`. No production code; no flip.
