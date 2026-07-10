# p36-c003 — Open PR sovereign-sfu-decode-proof → main

## Summary

Open a GitHub pull request from `sovereign-sfu-decode-proof` to `main`, summarising the full
sovereign SFU implementation (phases 0–36). This change is gated on c002 — `SFU_MODE=sovereign`
must be flipped (i.e. `framesDecoded > 0` confirmed) before the PR is opened.

`main` has been untouched since phase-0. The proof branch holds all infrastructure:
- Proto v1 freeze + workspace scaffold (phases 0–7)
- str0m adapter, shared demux socket, CRDT, auth (phases 8–27)
- Six-phase SFU media-path proof (phases 28–36)

## Gate

**Blocked on c002 T4 (flip confirmed).** If c002 holds the gate OFF, this change is deferred
to a future phase. Do not open the PR until `framesDecoded > 0` is confirmed in CI.

## Files

| File | Change |
|---|---|
| GitHub PR (gh CLI) | Title: `feat: sovereign SFU media plane — phases 0–36`; body: summary of all SFU work |
