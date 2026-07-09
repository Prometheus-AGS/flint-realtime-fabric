# Goals — phase-35-sovereign-sfu-decode-ci-linux-proof

> Seeded from: phase-34 (reflection `summaryForNext`). **The escalation, finally taken.** Across
> phases 28→34 every media-path layer was proven correct in pieces (negotiate, shared-socket demux,
> `.local` skip, secure context, str0m `typ relay` acceptance) and then a **fresh candidate-address
> confusion** blocked the decode — SIX forms: host-loopback, bridge, mDNS, srflx, host-net, and
> (phase-34) coturn `--external-ip`=the-browser's-own-loopback. That is **one environmental
> unsuitability wearing different masks**: the macOS + Colima-VM + Docker-bridge box is categorically
> wrong for a browser↔SFU media proof. See `docs/PHASE-34-DECODE-RESULT.md` + `PHASE-34-SIGNOFF.md`.
>
> This phase runs the decode where the problem does not exist: a **real Linux host with native host
> networking**, where the browser + SFU share one real network stack and candidate addresses simply
> route — then flips `SFU_MODE=sovereign` on a genuine `framesDecoded > 0`.

**⚠️ OPERATOR INPUT (surface at assess/plan): which Linux runner?** (a) **GitHub Actions
`ubuntu-latest`** — the existing `.github/workflows/ci.yml` is the natural home; uses CI minutes;
fully reproducible/shareable. (b) **A self-hosted Linux runner / box** the operator provides. The
change set differs (a CI workflow job vs. a script targeting a given host), so the target must be
chosen before planning commits.

**Hard non-goal: NO more same-host macOS/Colima network variants.** Six have failed; the environment
is the blocker, not the code.

**Discipline (carried 16→34):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (phase-29). Absence of a
defect is not presence of a proof (phase-30). Pivot when the environment is the blocker (phase-32),
toward the last working state (phase-33), and **off the wrong environment entirely once it has failed
enough (phase-34)**.

---

## G1 — Containerize the proof rig so the whole decode runs as one self-contained job

The rig currently has **macOS-host pieces** (RS256 JWT mint via host `node`, JWKS served by host
`python3 http.server`, Keto seed via host `curl localhost:4467`) reached through `host.docker.internal`
/ derived Colima IPs — none portable to CI.

- **G1.1:** move the host-side steps **into the compose stack / the job**: mint the JWT + serve the
  JWKS from a container (or a job step on the Linux runner), seed Keto in-job — so the entire proof is
  one reproducible unit with no macOS-host dependency and no `host.docker.internal`.
- **G1.2:** consolidate the run into a single entrypoint the runner invokes (a compose profile or a
  `run-media-decode` variant) that works unchanged on a Linux host. File-size ≤500; TURN credential
  stays env-sourced (S1).

**Exit:** the proof rig runs end-to-end with **no macOS-host / Colima-specific step** — reproducible
on any Linux host.

## G2 — Run the decode on a real Linux host (native host networking)

Phase-34 conclusion: on Linux, host networking is native, so the browser + SFU share one real stack
and host candidates pair with no bridge/loopback/mDNS/relay confusion.

- **G2.1 (operator target):** run the containerized proof on the chosen runner — **`ubuntu-latest`
  (GitHub Actions, `ci.yml`)** or a self-hosted Linux box. The gateway advertises its **real reachable
  IP** (`MEDIA_ADVERTISE_IP`); STUN/TURN (already wired, p29/p34) are belt-and-suspenders.
- **G2.2:** observe the gateway str0m logs + browser assertion: `ice=connected`, `state=Connected`,
  inbound `MediaData`/fan-out.

**Exit:** the decode run reaches `ice=connected` + `Connected` + `MediaData` on the Linux runner, or
the concrete blocker is diagnosed + recorded.

## G3 — Observe a real decoded frame + flip

- **G3.1** On the runner, observe `getStats().framesDecoded > 0`. Capture
  `docs/PHASE-35-DECODE-RESULT.md` (browser assertion + gateway str0m logs from the CI/host run).
- **G3.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-35-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G3.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G4 — Carried live proofs (from phase-19…34)

- **G4.1** LiveKit cross-node inbound (`realtime` feature). **G4.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**; **no committed secrets — S1**).
- `SFU_MODE=sovereign` is enabled **only** if G3 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001…008 without a new finding.
- **Any further same-host macOS/Colima network variant** (six have failed — the environment is the
  blocker).
- Net-new media features; simulcast/SVC out of scope.

## Starting point (from phase-34)

- Media path + engine proven correct in pieces (28→34); the full rig exists (JWKS mint/serve, Keto
  seed, coturn STUN+TURN, Caddy TLS, in-network Playwright, DECODE_ONLY, prebuilt-image runner) — but
  with macOS-host pieces to containerize (G1).
- Blocker to clear: **the proof environment** — run on a real Linux host (G2).
- Operator decision pending: GitHub Actions `ubuntu-latest` vs. a self-hosted Linux runner.
