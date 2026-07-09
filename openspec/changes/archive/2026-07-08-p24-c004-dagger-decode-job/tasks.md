# Tasks — p24-c004-dagger-decode-job

- [x] 1. `scripts/run-media-decode.sh`: bring up `-f compose.yml -f compose.sovereign.yml`, wait for gateway `/healthz`, run `seed-media-view.sh`, then `SKIP_INTEGRATION=false GATEWAY_URL=... pnpm playwright test media-decode` with Chromium fake-media. Fail loudly on any step; shellcheck-clean.
- [x] 2. Execute the runner (honest live attempt) and record the ACTUAL outcome in `docs/PHASE-24-DECODE-RESULT.md`: framesDecoded>0 observed → note it; else the concrete blocker. This note is c005's flip/re-affirm input.
