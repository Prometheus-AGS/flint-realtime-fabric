# p28-c001-harness-visibility

## Why

Phase-27's decode run couldn't report whether the ICE/fan-out fixes worked: the receiver runs a 15s
connect pre-check + a 20s probe = 35s > Playwright's 30s default test timeout, so the test aborts
before the diagnostic assertion prints. And the runner destroys the gateway (cleanup `down -v`)
before its str0m logs can be read on a decode failure. Make the diagnostics survive.

## What Changes

- `admin-ui/e2e/media-decode.spec.ts`: **drop the redundant `connectToSovereignSfu` pre-check** (a
  separate recvonly PC with no media — 15s, no decode value; the probe self-connects + RoomJoins),
  and add **`test.setTimeout(60_000)`** so the probe's 20s window + the diagnostic `expect(...)` run.
- `scripts/run-media-decode.sh`: on harness failure, dump `docker compose logs gateway` **before**
  the cleanup `down -v`, so the str0m lifecycle logs (session negotiated / connection state / inbound
  MediaData / forward) survive for diagnosis.

## Impact

- `admin-ui/e2e/media-decode.spec.ts`, `scripts/run-media-decode.sh`. No production code; no flip.
- A failed run now prints `ice=<state> localCandidates=N remoteCandidates=M` + the gateway logs.
