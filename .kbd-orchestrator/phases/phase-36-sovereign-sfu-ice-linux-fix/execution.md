# Execution — phase-36-sovereign-sfu-ice-linux-fix

> Backend: OpenSpec. Driver: /kbd-apply (one change at a time).

## Dispatch Contract

| Change | Status | Notes |
|---|---|---|
| p36-c001-ice-fix-and-log-capture | PENDING | First — fixes both root causes; push triggers CI |
| p36-c002-decode-run-and-flip | PENDING | Blocked on CI run from c001 push |
| p36-c003-pr-to-main | PENDING | Gated on c002 flip confirmation |

## Backend

OpenSpec (`openspec/` present, `change_backend: openspec` in project.json).

## First change to apply

`/kbd-apply p36-c001-ice-fix-and-log-capture`
