# p23-c006-flip-or-reaffirm-gate

## Why

Phase-23 G4: flip `SFU_MODE=sovereign` on — **only** if decoded media provably flows end-to-end
(G2) and the boundary is documented + enforced (G3). G3 is met (ADR-007 + p23-c002 enforcement +
p23-c005 docs). **G2 is NOT met in this environment**: str0m is sans-codec, so the decode proof
is browser-side and requires a live sovereign gateway + Chromium fake-media, which this headless
CI env lacks. The harness is authored + correct (p23-c004) but `test.skip`-gated and not run
here — proven-*capable*, not proven-*here*.

## What Changes — RE-AFFIRM GATED (no flip)

- **`SFU_MODE=sovereign` stays OFF.** The gateway sovereign branch keeps its honest gate-off
  warning; no code flip. The one gate on the flip (an observed decoded frame) has not been
  satisfied in-environment.
- `CHANGELOG.md`: Phase 23 section — ADR-007 media authz, Keto view-check enforcement, WS
  inbound path, browser E2E + decode harnesses, SECURITY media boundary; gate re-affirmed off.
- `docs/PHASE-23-SIGNOFF.md`: closing sign-off with the honest G1–G5 status + the re-run gate
  results.
- **G5 carried** (re-affirmed integration-gated): LiveKit cross-node `realtime` feature;
  admin-ui OIDC (ADR-004 + IdP).

## Impact

- `CHANGELOG.md`, `docs/PHASE-23-SIGNOFF.md`, `docs/SECURITY.md` §6 (already current from c005).
- No production code change; no gate flip. Doc-only close.
