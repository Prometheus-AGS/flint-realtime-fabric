# Goals — phase-36-sovereign-sfu-ice-linux-fix

> Seeded from: phase-35 reflection. **The harness works on real Linux — one ICE-completion
> debugging loop remains.** Phase-35 proved the sovereign SFU stack builds, boots, and runs a
> two-browser decode end-to-end on `ubuntu-latest` CI (run 29057452278). The decode reached
> `ice=checking remoteCandidates=1` but stalled at `framesDecoded=0`. The gateway-side str0m
> candidate log was empty (harness bug: EXIT trap fired before the CI log-collect step). This
> phase fixes the log capture, reads the Linux candidate exchange, fixes the ICE pairing, and
> flips `SFU_MODE=sovereign` on a genuine `framesDecoded > 0`.

## G1 — Fix gateway-log capture in CI

**Exit:** the next CI run produces a populated `gateway.log` artifact (non-zero bytes) containing
str0m candidate negotiation lines. Fix: copy `/tmp/p29-gateway.log` to `$GITHUB_WORKSPACE/gateway.log`
inside `scripts/run-media-decode.sh` **before** the EXIT trap fires `docker compose down -v`, or
skip the in-script teardown when `CI=true` so the workflow's "Collect gateway logs" step can read
a live container.

## G2 — Read the Linux str0m candidate log; diagnose ICE stall

**Exit:** a CI run with the log-capture fix produces a gateway log, and the root cause of
`ice=checking` (why no candidate pair completes on Linux) is identified. Expected candidates:
`localCandidates=5` from the gateway (host at `172.17.0.1`, possibly mDNS, TURN relay);
`remoteCandidates=1` from the Playwright browser (bridge-network container). The log should
show which types were exchanged and whether the pair was attempted.

## G3 — Fix the Linux candidate pairing

**Exit:** the ICE pair completes on `ubuntu-latest` — `ice=connected` appears in the decode
probe output and/or gateway log. Fix candidates:
- `MEDIA_ADVERTISE_IP=172.17.0.1` is the docker0 gateway — verify the bridged Playwright
  container can actually reach it (a UDP packet from the bridge subnet to the docker0 IP).
- If `172.17.0.1` is unreachable from the bridge, try the container's own bridge IP
  (`docker inspect` the gateway container for its bridge address) or restrict to `typ relay`
  only (coturn relay should always work within the Compose network).
- Alternatively, add the `--network=host` flag to the Playwright compose service so it shares
  the runner's network stack (same as the gateway).

## G4 — Re-run CI decode; flip `SFU_MODE=sovereign` on genuine `framesDecoded > 0`

**Exit:** `SFU_MODE=sovereign` is set in `crates/frf-gateway/src/main.rs` (or the gateway
config) **only after** a CI run on `ubuntu-latest` reports `framesDecoded > 0` from a real
browser receiver. Gate discipline from phases 16–35 applies: no flip on assumptions.

## G5 — ~~Open PR from `sovereign-sfu-decode-proof` to `main`~~ — RETIRED (p36-c003, 2026-09-06)

> **Original exit:** a GitHub pull request from `sovereign-sfu-decode-proof` → `main` is open,
> summarising all SFU work from phases 0–36. `main` has been untouched since phase-0; the proof
> branch holds all sovereign SFU infrastructure. Merge is gated on G4 (the flip).

**Status: MOOT — satisfied by merge, not by PR.** The premise no longer holds. `main` is *not*
untouched: all sovereign SFU work from phases 0–36 is already merged into it via PRs #3 and #4.
The last decode-proof commit `9ba04ae` is an ancestor of `main` (`git merge-base --is-ancestor`),
and the `sovereign-sfu-decode-proof` branch has since been deleted — after the merge, so nothing
was lost. A PR from that branch would be empty.

**Consequence handled by p36-c003:** `.github/workflows/decode-proof.yml` scoped its `push` trigger
to the deleted branch, leaving no way to fire the decode proof by push. That trigger is retired;
`workflow_dispatch` from `main` is now the only path:
`gh workflow run decode-proof.yml --ref main`. This supersedes c002's assumption that c001's T5
push would trigger the run.

See `docs/PHASE-36-SIGNOFF.md` for the full record.
