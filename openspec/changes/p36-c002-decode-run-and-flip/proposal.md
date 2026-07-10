# p36-c002 — CI decode run + honest gate decision

## Summary

Trigger the CI `decode-proof` job (push to `sovereign-sfu-decode-proof` is handled by c001's T5),
wait for the result, read the gateway str0m log artifact, and make an honest gate decision:

- If `framesDecoded > 0`: flip `SFU_MODE=sovereign` in the gateway config/main; record the
  evidence; advance to c003 (PR).
- If `framesDecoded=0` but `ice=connected`: the media path stalled after ICE — diagnose from the
  gateway log and carry the fix.
- If `ice=checking`: read the gateway log for candidate exchange; adjust addresses and re-run.

Gate discipline from phases 16–35 applies: **`SFU_MODE=sovereign` flips ONLY on a genuine
`framesDecoded > 0` from a real browser receiver on CI.**

## Deliverables (regardless of gate outcome)

- `docs/PHASE-36-DECODE-RESULT.md` — full run evidence, candidate log excerpt, gate decision
- `docs/PHASE-36-SIGNOFF.md` — honest gate decision + any carried items
- `docs/SECURITY.md` — §6 update
- `CHANGELOG.md` — phase-36 entry

## Files (conditional on flip)

| File | Change | Condition |
|---|---|---|
| `crates/frf-gateway/src/main.rs` (or config) | `SFU_MODE=sovereign` | Only if `framesDecoded > 0` |
| `docs/PHASE-36-DECODE-RESULT.md` | New — run evidence | Always |
| `docs/PHASE-36-SIGNOFF.md` | New — gate decision | Always |
| `docs/SECURITY.md` | §6 update | Always |
| `CHANGELOG.md` | Phase-36 entry | Always |
