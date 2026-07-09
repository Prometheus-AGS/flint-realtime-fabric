# Plan — phase-31-sovereign-sfu-decode-prebuilt-image-and-flip

> Date: 2026-07-09. Ordered change list from `assessment.md`. Backend: **OpenSpec**. The assessment
> settled G1's approach unambiguously (presence-check + fail-fast, low-risk, no operator decision) —
> no clarifying question needed.

## Ordering rationale

Serial: **G1 → G2**. G1 decouples the OOM-prone in-run gateway build so a decode run can even
execute; G2 is the run + honest gate decision. The `SFU_MODE=sovereign` gate is decided only in c002,
only on a real `framesDecoded > 0`. **No `frf-*` engine change** — the engine is proven; this is a
runner/script + docs change plus an operational prerequisite (daemon restart + one-time image build).

## Operational prerequisite (manual, operator runs via `!`)

Before c002's decode run — the daemon is DOWN and the image must exist:

```
! colima start --memory 8            # restart the crashed VM (belt-and-suspenders memory)
! docker compose -f compose.yml -f compose.sovereign.yml build gateway   # build the image ONCE
```

The c001 runner change makes this a hard, fail-fast requirement rather than a silent in-run rebuild.

## Changes

### c001 — `p31-c001-prebuilt-image-runner` (G1) · **agent: rust-build-resolver / e2e-runner on scripts**

**Goal:** stop the decode runner from rebuilding the heavy gateway image in-run (the phase-30 OOM);
use a **pre-built** image and fail fast if it's absent.

- **`scripts/run-media-decode.sh`:** replace the unconditional `docker compose build gateway`
  (line ~70) with a **presence check** on the implicit image `flint-realtime-fabric-gateway`
  (`docker image inspect`): present → skip build, go to `up -d --no-build`; absent → **exit non-zero
  with a clear message** naming the one-time out-of-band build command. Honor a `PREBUILD_GATEWAY=1`
  escape hatch that builds-then-runs for a fresh checkout.
- **Runner header + docs:** document the one-time build + `colima start --memory 8` prerequisite.
- No compose change (the gateway already builds to the stable implicit image name); no engine change.
  File-size ≤500; no library `unwrap`/`expect` (no Rust change).

**Exit:** with a pre-built image present, `run-media-decode.sh` boots the full in-network stack
(gateway + coturn + playwright) **without an in-run build** and the decode harness executes (reaches
the browser connect + gateway str0m logs). With the image absent, it fails fast with the build hint.

### c002 — `p31-c002-decode-run-and-flip` (G2) · **honest gate decision**

**Goal:** run the in-network decode against the pre-built image on a restarted daemon; flip
`SFU_MODE=sovereign` **only** on `framesDecoded > 0`. First real verification of the phase-30
topology fix (`ice=connected`).

- Run the in-network `scripts/run-media-decode.sh`; read the browser assertion + gateway str0m logs.
  Record `docs/PHASE-31-DECODE-RESULT.md` (including whether `ice=connected` / `state=Connected` /
  `MediaData` — the phase-30 UNVERIFIED G1 exit).
- **If `framesDecoded > 0`:** flip `crates/frf-gateway/src/main.rs` sovereign branch (remove the
  gate-off warning → live path) + SECURITY §6 (media → functional) + CHANGELOG +
  `docs/PHASE-31-SIGNOFF.md`.
- **Else:** re-affirm gated with the fresh diagnostic detail (main.rs untouched). Carry G3.
- Re-run the release gate suite; QA gate (read verdict AND archive output).
- **Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

### c003 (conditional) — `p31-c003-carried-proofs-reaffirm` (G3) · **only if not folded into c002**

Re-affirm LiveKit x-node (G3.1) + admin-ui OIDC (G3.2) integration-gated in SECURITY §6 + CHANGELOG.
Likely folded into c002's SECURITY §6 edit.

## Discipline (carried 16→30)

- `SFU_MODE=sovereign` flips **only** on a real `framesDecoded > 0`; else gated with fresh rationale.
- Seed the openspec change dir up-front; read the QA verdict **and** the archive output; **update
  `progress.json` to N/N before any command mentioning the next stage** (phase-29 lesson). ≤500
  lines; no library `unwrap`/`expect`.
- **G1 (phase-30 topology) stays UNVERIFIED until this phase's live run observes `ice=connected`** —
  absence of a defect is not presence of a proof (phase-30 lesson).

## Change summary

| # | id | goal | risk | agent |
|---|---|---|---|---|
| 1 | p31-c001-prebuilt-image-runner | G1 | low (runner/docs) | e2e-runner |
| 2 | p31-c002-decode-run-and-flip | G2 | gate decision (needs live daemon + built image) | — |
| 3 | p31-c003-carried-proofs-reaffirm (cond.) | G3 | low | — |

**First change to apply: `p31-c001-prebuilt-image-runner`.**
