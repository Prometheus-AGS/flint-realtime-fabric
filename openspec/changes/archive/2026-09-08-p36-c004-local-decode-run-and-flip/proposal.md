# p36-c004 — Decode proof and gate decision, from a LOCAL integration run

## Why this change exists

`p36-c002` was authored as "trigger the CI decode job, read `framesDecoded`,
decide the gate". The operator has since established a **permanent** policy:
CI/CD is never used to run tests; all testing is local full integration
(`CLAUDE.md` / `AGENTS.md`, this repo and the PEM and ASO repos).

Under that policy c002's remaining tasks are **unsatisfiable** — both require a
CI run. It is deferred with 6 of 8 tasks genuinely complete, and this change
supersedes it by taking the same decision from a local run.

## What this is not

It is not a re-run of the same thing somewhere else. The evidence c002 gathered
(run 29112243615) is retained and still valid: `ice=connected`,
`bytes≈1,800,000`, `framesDecoded=0`, with the gateway log showing 1,997 fan-out
events, **zero** PLI/FIR/keyframe requests, and no room-join events.

What changed since that evidence was captured is the ADR-009 child phase, which
built and shipped — unproven — the pieces most likely to move `framesDecoded`
off zero. This run is the first opportunity to see whether they do.

## The local path already exists

`scripts/run-media-decode.sh` is the decode runner, and the sovereign stack has
compose files (`compose.sovereign.yml`, `compose.host-net.yml`). The script
carries four CI-conditional branches (`GITHUB_*` / `CI=true`) that need to
become the *non-default* path rather than the assumed one.

**macOS caveat, stated up front.** Phases 28–34 established that the same-host
macOS + Colima + Docker-bridge topology could not pair ICE candidates, which is
precisely why the proof was escalated to Linux CI in phase-35. A local run on
this machine may reproduce that environmental failure rather than exercising the
media path. If it does, that is an environment result and **must not** be
reported as a media-path result — the distinction is the whole reason phases
28–35 took as long as they did.

## Gate discipline is unchanged

`SFU_MODE=sovereign` flips **only** on a genuine `framesDecoded > 0` from a real
browser receiver. A local run is a different venue, not a lower bar. If the run
does not produce a decoded frame, the gate stays OFF and this change records
another honest hold.

## Files

| File | Change |
|---|---|
| `scripts/run-media-decode.sh` | make the local path primary; CI branches become opt-in |
| `docs/PHASE-36-LOCAL-DECODE-RESULT.md` | new — local run evidence and gate decision |
| `crates/frf-gateway/src/main.rs` | `SFU_MODE` — **only if** `framesDecoded > 0` |
