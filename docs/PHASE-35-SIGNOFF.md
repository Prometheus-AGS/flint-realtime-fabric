# Phase-35 signoff — sovereign SFU decode: CI/Linux proof

> Date: 2026-07-09. Phase-35 escalated the decode proof to GitHub Actions `ubuntu-latest` — and it
> **worked**: the whole stack builds + boots + runs the two-browser decode end-to-end on real Linux,
> removing the six-phase same-host environment blocker. The decode is `framesDecoded=0` at
> `ice=checking`, so **`SFU_MODE=sovereign` stays gated OFF** — but the residual is now a normal
> debugging loop on a working harness, not an environmental dead-end.

## Gate decision: OFF (honest gate held) — environment unblocked

No `framesDecoded > 0` (CI run 29057452278). Per the phase-16→34 discipline: the gate does not flip
until a real receiver observes a decoded frame. `crates/frf-gateway/src/main.rs` untouched.

## Changes

| Change | Summary | Gate impact |
|---|---|---|
| p35-c001 | Linux-portable runner (172.17.0.1 host addr) + `decode-proof` GitHub Actions job (build → compose up → decode → assert framesDecoded>0) | none (CI/harness); S1-clean |
| p35-c002 | CI decode run + honest gate decision | **held OFF** |

## Evidence — the escalation worked

- ✅ **Gateway image built on CI** (~8 min, no OOM — the phase-30 OOM was Colima VM contention).
- ✅ **Sovereign stack booted + two-browser Playwright decode ran** on `ubuntu-latest`: `getUserMedia`
  works, WS signaling + offer happen, ICE reaches `checking` (`remoteCandidates=1`).
- ❌ **`framesDecoded=0`** — ICE stalls at `checking`. Gateway-side candidate detail not visible this
  run (empty `gateway.log` — the script's `down -v` EXIT trap tore the gateway down before the
  workflow's log-collect step).
- Two CI interpolation fixes en route (`FLINT_GATE_JWT_SECRET`, `TURN_SECRET` job-wide for
  `docker compose build`) — genuine, done.

## The material shift

Unlike phases 28→34, the residual is **not** environmental. The six same-host candidate-topology
confusions cannot arise on native-host-networking Linux; the proof executes end-to-end. What remains is
an ordinary debugging loop on a **working** harness.

## Carried to next phase

1. **Fix gateway-log capture in CI:** copy the in-script `/tmp/p29-gateway.log` to a workflow artifact
   path, or skip the in-script `down -v` when `CI=true` so the workflow's collect step sees a live
   container — so the Linux candidate exchange is visible.
2. **Read the gateway str0m log; fix the Linux candidate pairing** (`MEDIA_ADVERTISE_IP` / coturn
   `--external-ip` = the runner's actual reachable address) so ICE completes past `checking`.
3. Re-run the CI proof; flip `SFU_MODE=sovereign` only on a genuine `framesDecoded > 0`.

Run + artifacts: GitHub Actions run **29057452278**, branch `sovereign-sfu-decode-proof` (`main`
untouched).

## Verification

- Host `cargo test -p frf-media-str0m` (31), `cargo fmt --check` — green (no engine change).
- No-flip: `main.rs` unchanged; SECURITY §6 + CHANGELOG record the honest status + the environment
  unblock.
- No committed secrets (S1): CI JWT/TURN secrets generated in-job.
