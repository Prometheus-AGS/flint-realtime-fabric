# Phase-24 decoded-media proof — live run result (p24-c004)

> Date: 2026-07-08. The actual, un-fabricated outcome of running the decode proof. This is the
> input that decides the `SFU_MODE=sovereign` flip (p24-c005).

## Result: ❌ NOT PROVEN — `framesDecoded > 0` was NOT observed

The live run **failed before any media flowed**. `RUNNER_EXIT=1`. No decoded frame was observed,
so **G2 is not met** and the gate must stay off (p24-c005 re-affirms gated).

## What actually happened

```
[run-media-decode] E2E_JWT unset — adding the dev override (DEV_NO_AUTH)…
[run-media-decode] bringing up the sovereign stack…
service "gateway" depends on undefined service "keto": invalid compose project
RUNNER_EXIT=1
```

The stack never came up. The concrete blocker is a **compose-merge failure in the runner's
no-JWT fallback path**, not the media plane itself.

## Root cause (diagnosed)

- `docker compose -f compose.yml -f compose.sovereign.yml config` → **VALID** (the sovereign
  media override merges cleanly on its own).
- Adding `compose.override.example.yml` (the runner's `DEV_NO_AUTH` fallback when `E2E_JWT` is
  unset) → **INVALID**: `service "gateway" depends on undefined service "keto"/"postgres"`.
- Why: `compose.override.example.yml` **redefines the gateway `depends_on`**, and Compose
  *replaces* (not merges) `depends_on` across `-f` files. That example override is a standalone
  local-dev profile (auto-merged as `compose.override.yml`), not designed to be chained as a
  third `-f` alongside the sovereign override. So the fallback path is structurally broken.

## Honest assessment

- **The decode was not observed.** Whatever else is true, no `framesDecoded > 0` was captured —
  the proof does not pass.
- The failure is in the **harness plumbing** (the no-auth fallback compose chain), not (yet)
  demonstrated to be in the media path. The sovereign-only stack merges; the seam that broke is
  the DEV_NO_AUTH fallback the runner reaches for when no real JWT is supplied.
- A faithful run still requires **either** a real `E2E_JWT` minted by flint-gate (the authenticated
  path c003 built for) **or** a correctly-composed no-auth profile. Neither was in place, so the
  run could not proceed to ICE/DTLS/RTP/decode.

## Consequence for the flip (p24-c005)

**`SFU_MODE=sovereign` stays gated OFF.** G2 is unproven. Per the operator-confirmed honest gate,
c005 re-affirms gated with this concrete blocker — no forced flip, no relaxed proof.

## To make the proof pass (next attempt)

1. Fix the runner's no-JWT path: build a **purpose-made** sovereign+no-auth compose override that
   merges cleanly (define `depends_on` compatibly, or use `dev-endpoints` build args without
   replacing `depends_on`), **or** mint a real `E2E_JWT` via flint-gate and run the authenticated
   path (preferred — it exercises the c003 authz).
2. Re-run `scripts/run-media-decode.sh`; capture `framesDecoded`.
3. Only if `framesDecoded > 0` is observed does the gate flip (a later change/phase).
