# Phase-24 Sign-off — sovereign SFU live-decode path & the gate decision

> Date: 2026-07-08 · Closing verification for phase-24 (p24-c005). This phase built the full
> infrastructure for a real browser↔gateway decoded-media proof, **ran it**, and reached the
> honest decision: `SFU_MODE=sovereign` **stays off** — the live decode did not pass.

## Gates (re-run at phase close — actual results)

| Gate | Result |
|------|--------|
| `cargo fmt --check --all` | ✅ (exit 0) |
| `cargo check --workspace` | ✅ (exit 0) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ (exit 0) |
| `cargo test -p frf-media-str0m --lib` | ✅ 27 passed (24 → 27: `MediaConfig` + advertise-IP tests) |
| `cargo test -p frf-gateway --lib` | ✅ 39 passed (38 → 39: authenticated-subject authz test) |
| `cargo test -p frf-domain` | ✅ 10 passed (serde roundtrip carries `subject`) |

## Goal status (honest)

| Goal | Status | Evidence |
|------|--------|----------|
| **G1 — live sovereign gateway env** | ◐ PARTIAL | `MediaConfig` bind/advertise seam (p24-c001) + `compose.sovereign.yml` UDP path (p24-c002, verified via `docker compose config`) + Keto `view` seed (p24-c003). The stack *can* be composed sovereign; the run's boot failed on the no-JWT fallback (below). |
| **G2 — live decoded-media run** | ❌ **NOT PROVEN** | `scripts/run-media-decode.sh` was **executed**; the run failed (`RUNNER_EXIT=1`) with `service "gateway" depends on undefined service "keto"` — the no-JWT fallback compose chain is invalid; the stack never came up; **no `framesDecoded > 0` observed** (`docs/PHASE-24-DECODE-RESULT.md`). |
| **G3 — flip `SFU_MODE=sovereign`** | ⛔ **RE-AFFIRMED OFF** | G2 unmet → no flip. `main.rs` keeps the honest gate-off warning; production `from_env` defaults to hosted. No relaxed proof, no forced flip (operator-confirmed). |
| **G4 — carried live proofs** | ⏳ RE-AFFIRMED gated | LiveKit `realtime` cross-node; admin-ui OIDC (ADR-004 + IdP). |

## What now functions (phase-24)

- **The str0m loopback blocker is gone** — `MediaConfig` binds `0.0.0.0` + advertises a
  host-reachable candidate IP on a fixed UDP port; loopback stays the test default (p24-c001).
  (`session.rs` split 517→314 to hold the 500-line limit — cleared pre-existing debt.)
- **A sovereign media compose path exists** — `compose.sovereign.yml` maps the UDP port +
  advertise IP; the hosted `compose.yml` default is untouched (p24-c002).
- **Media authz is correct for a live run** — the ADR-007 `view` check authorizes the
  **authenticated JWT subject** (stable/seedable), not the ephemeral session id; seeded by
  `scripts/seed-media-view.sh` (p24-c003).
- **A decode runner exists and was exercised** — it produced a real, recorded failure, not a
  fabricated pass (p24-c004).

## The gate decision — stated plainly

**`SFU_MODE=sovereign` stays off.** The un-fakeable proof — a real receiver observing
`framesDecoded > 0` — was **attempted and did not pass**. The blocker was harness plumbing (an
invalid no-JWT compose fallback), so the media path itself is not yet disproven either; it is
simply **unproven**. Enabling the gate now would advertise a plane that has never been observed to
move a decoded frame — the failure this project has refused for nine phases.

**To flip in a real environment (next phase):** fix the runner's no-JWT path (a purpose-built
sovereign no-auth override that merges cleanly) **or** mint a real `E2E_JWT` via flint-gate and run
the authenticated path; re-run `scripts/run-media-decode.sh`; confirm `framesDecoded > 0`. Then flip
the `main.rs` sovereign branch + update SECURITY §6.

## Process note (honest)

- **c003 scope grew on a real finding** — the seeded-tuple plan couldn't work (the check used a
  random per-connection session id); surfaced to the operator and fixed by authorizing on the
  authenticated subject.
- **c004 ran the proof for real and it failed** — recorded plainly in `PHASE-24-DECODE-RESULT.md`
  with a diagnosed root cause, not dressed up.
- **The flip-guard was confirmed, not assumed** — a mid-plan question first came back "push to
  flip / accept relaxed proof"; it was re-confirmed to "keep the honest gate" before planning, so
  no forced flip ever entered the plan.
- Both carried QA lessons applied every change: read the gate **verdict** and the **archive
  output** (c003's MODIFIED delta archived cleanly this phase).

## Sign-off

Phase-24 built every piece of the sovereign live-decode path and honestly ran the proof, which did
not pass — so **`SFU_MODE=sovereign` is re-affirmed off** with a concrete, diagnosed blocker and a
precise next step. No new CRITICAL/HIGH; all gates green. Hosted (LiveKit) remains the media path.
No plane was advertised beyond what it does.
