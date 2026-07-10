# Plan — phase-36-sovereign-sfu-ice-linux-fix

> Backend: OpenSpec. Changes are in `openspec/changes/p36-c00[123]-*/`.
> Phase goal: fix the two root causes that blocked ICE completion on Linux CI, re-run the decode,
> flip `SFU_MODE=sovereign` on `framesDecoded > 0`, open the PR.

## Ordered Change List

### p36-c001 — ICE candidate fix + gateway-log capture

**Priority: FIRST — unblocks everything.**

Two root causes found in assessment, fixed in parallel in one change:

1. **Gateway-log capture**: `run-media-decode.sh` writes `/tmp/p29-gateway.log` on failure but the
   CI workflow reads `./gateway.log` (workspace root). Fix: `cp` to `$GITHUB_WORKSPACE/gateway.log`
   on the failure path, before the EXIT trap fires.

2. **`MEDIA_ADVERTISE_IP=gateway`**: current value `172.17.0.1` (docker0 host bridge) is
   unreachable from the Compose bridge. Setting the service name `gateway` lets
   `config.rs:resolve_advertised_ip()` resolve it via DNS to the container's bridge IP at negotiate
   time. No code changes — compose env var only.

3. **coturn `--external-ip` shell expansion**: same wrong IP. Override coturn `entrypoint` to
   `/bin/sh -c` so `$(hostname -i | cut -d' ' -f1)` expands the bridge IP at startup.

**Files:** `scripts/run-media-decode.sh` (+1 line), `compose.sovereign.yml` (gateway env + coturn
entrypoint).

**Agent:** general-purpose (shell + YAML edits, no complex logic).

---

### p36-c002 — CI decode run + honest gate decision

**Priority: SECOND — depends on c001 T5 (push triggering CI).**

Wait for the `decode-proof` CI job; download `gateway.log` + Playwright report; record candidate
exchange; make the gate decision. Flip `SFU_MODE=sovereign` ONLY on `framesDecoded > 0`.
Write `PHASE-36-DECODE-RESULT.md`, `PHASE-36-SIGNOFF.md`, update `SECURITY.md` + `CHANGELOG.md`.

If ICE still stalls: the gateway log now has content — read it and carry the next fix to phase-37.

**Agent:** general-purpose (CI observation, docs, conditional code edit).

---

### p36-c003 — Open PR sovereign-sfu-decode-proof → main

**Priority: THIRD — gated on c002 flip.**

Only execute if c002 confirms `framesDecoded > 0`. Open PR via `gh pr create` with full summary
of phases 0–36. **Skip / defer to phase-37 if gate is held OFF.**

**Agent:** general-purpose (gh CLI).

---

## Ordering Rationale

c001 must land first (fixes both root causes + triggers CI). c002 is observation-only until the CI
run completes (no code to write until the result is known). c003 is gated on the flip and may be
deferred. The critical path is c001 → push → CI run → c002.

## Exit Criteria

- **Phase exit (minimal):** c001 applied + CI run triggered + c002 result doc written.
- **Phase exit (full):** `framesDecoded > 0` confirmed in CI, `SFU_MODE=sovereign` flipped, PR open.
- **Phase exit (carried):** gate held OFF with fresh evidence from gateway log; phase-37 seeds from
  the log diagnostic.
