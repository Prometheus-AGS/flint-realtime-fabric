# p28-c002-diagnose-and-fix

## Why

With diagnostics now surfacing (p28-c001), read what actually blocks the decode — the browser ICE
state + candidate counts and the gateway str0m lifecycle logs — and fix the revealed media-path
issue. This change is evidence-driven: the fix is dictated by the run, not pre-guessed.

## What Changes

- Re-run `scripts/run-media-decode.sh`; record the surfaced evidence + interpretation in
  `docs/PHASE-28-DIAGNOSIS.md`.
- Fix the revealed defect (candidate reachability over `host.docker.internal:40000/udp`, DTLS,
  sender-RTP-reaches-SFU, or `RoomRouter` fan-out over a real socket). Keep engine contracts intact;
  ADR any real design change; file-size ≤500; no library `unwrap`/`expect`.

## Impact

- `docs/PHASE-28-DIAGNOSIS.md` + whatever code the evidence dictates. No gate flip.
