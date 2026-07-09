# p24-c004-dagger-decode-job

## Why

Phase-24 G2 — the un-fakeable proof: a real receiver observing `getStats().framesDecoded > 0`
for media relayed by a live `SFU_MODE=sovereign` gateway. c001–c003 built the full path (bindable
media socket + advertised IP, UDP compose mapping, authenticated-subject authz + Keto seed). This
change adds the runner that exercises it and records the **actual** result — which decides the
gate flip (c005).

## What Changes

- `scripts/run-media-decode.sh`: boot `compose.yml + compose.sovereign.yml`, wait for
  `/healthz`, seed the `view` grant (`seed-media-view.sh`), then run `media-decode.spec.ts`
  (`SKIP_INTEGRATION=false`, `GATEWAY_URL`, Chromium fake-media) and report pass/fail.
- Run it and **record the real outcome** in the change (a `RESULTS.md` / signoff note): either
  `framesDecoded > 0` observed (→ c005 flips), or the concrete failure (→ c005 re-affirms gated
  with that detail). No fabricated pass; no forced flip.

## Impact

- New: `scripts/run-media-decode.sh` + a recorded results note.
- No production code; no gate flip here. The recorded outcome is c005's input.
