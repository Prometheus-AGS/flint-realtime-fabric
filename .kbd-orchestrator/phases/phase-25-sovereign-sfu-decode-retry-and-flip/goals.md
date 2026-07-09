# Goals — phase-25-sovereign-sfu-decode-retry-and-flip

> Seeded from: phase-24 (reflection). The full sovereign live-decode path is built — bindable +
> advertised media socket (`MediaConfig`), sovereign UDP compose override, authenticated-subject
> ADR-007 authz + Keto seed, and a decode runner. Phase-24 **ran the proof and it failed** on a
> harness-plumbing blocker (no `framesDecoded` observed) — see `docs/PHASE-24-DECODE-RESULT.md`.
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-assess`.

This phase is the **narrow finish**: make the decode run actually reach the browser, observe a
real `framesDecoded > 0`, and — **only then** — flip `SFU_MODE=sovereign` on. Nothing new is
designed; the media plane is done. What remains is a runnable proof and the trigger.

**Discipline (carried 16–24):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite.

---

## G1 — Fix the runner's live path

The phase-24 blocker: the no-`E2E_JWT` fallback chained `compose.override.example.yml` as a third
`-f`, which *replaces* the gateway `depends_on` → `service "gateway" depends on undefined service
"keto"` → invalid project. The sovereign-only merge is valid.

- **G1.1** Either a **purpose-built sovereign no-auth override** that merges cleanly with
  `compose.yml + compose.sovereign.yml` (define `depends_on` compatibly, or use `dev-endpoints`
  build args without replacing it), **or** a **flint-gate `E2E_JWT` mint** step so the runner uses
  the real authenticated path (preferred — it exercises the c003 authenticated-subject authz).
- **G1.2** `scripts/run-media-decode.sh` boots the stack cleanly, seeds the `view` grant, and the
  gateway reaches `/healthz` in `SFU_MODE=sovereign`.

**Exit:** the runner brings the sovereign stack up and the harness connects (reaches `Connected`).

## G2 — Observe a real decoded frame

- **G2.1** Run `media-decode.spec.ts` (Chromium fake-media) against the live sovereign gateway and
  observe `getStats().inbound-rtp.framesDecoded > 0` on the receiver. Capture the run output as
  evidence (`docs/PHASE-25-DECODE-RESULT.md`).
- **G2.2** If it still fails, record the concrete blocker honestly and **do not flip** — the gate
  holds. Diagnose whether the remaining issue is ICE/DTLS/RTP (media path) vs. harness.

**Exit:** a real receiver decodes media relayed by the sovereign SFU, proven by the harness.

## G3 — Flip `SFU_MODE=sovereign` (only on a real G2 pass)

- **G3.1** Flip the `main.rs` sovereign branch from the gate-off warning to a live path; update
  `docs/SECURITY.md` §6 (media row → functional, boundary already documented) + CHANGELOG +
  `docs/PHASE-25-SIGNOFF.md`.
- **G3.2** If G2 does not pass, re-affirm gated with the fresh detail. The flip is conditional on
  G2, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with rationale.

## G4 — Carried live proofs (from phase-19…24)

- **G4.1** LiveKit cross-node inbound (`realtime` feature `LiveKitDataSource`) vs a live server.
- **G4.2** admin-ui OIDC authorization-code flow — once ADR-004 is Accepted + an IdP exists.

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**).
- `SFU_MODE=sovereign` is enabled **only** if G2 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001/003/004/005/006/007 without a new finding.
- Net-new media features beyond what the live proof + flip require.

## Starting point (built in phase-24)

- `crates/frf-media-str0m/src/config.rs` + `session.rs` — the `MediaConfig` bind/advertise seam.
- `compose.sovereign.yml` — the sovereign UDP media override.
- `crates/frf-gateway/src/{routes/signal.rs,media_bridge.rs}` — authenticated-subject authz.
- `scripts/{seed-media-view,run-media-decode}.sh` — the seed + runner.
- `admin-ui/e2e/{media-decode.spec.ts,support/decode-probe.ts}` — the decode harness.
- `docs/PHASE-24-DECODE-RESULT.md` — the recorded failure + the precise next step.
