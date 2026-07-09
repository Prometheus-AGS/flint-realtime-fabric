# Goals — phase-24-sovereign-sfu-live-decode-and-flip

> Seeded from: phase-23 (reflection). The sovereign media plane is composed, authz-gated
> (ADR-007), browser-drivable (`/ws/v1/signal` inbound path), and documented (SECURITY §1–§5).
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-assess`.

Phase-23 reached the honest terminus: `SFU_MODE=sovereign` stays **off** because the one
un-fakeable proof — a real receiver observing `getStats().framesDecoded > 0` for media relayed
by a live sovereign gateway — could not run in a headless CI shell (str0m is sans-codec; the
decode is browser-side and needs a live gateway + Chromium fake-media). The decode harness is
**authored and correct** (`admin-ui/e2e/media-decode.spec.ts`, `decode-probe.ts`). This phase
brings up that environment, runs the proof, and — **only if it passes** — flips the gate.

**Discipline (carried 16–23):** `SFU_MODE=sovereign` flips on **only** once decoded media
provably flows to a real receiver against a live gateway; else it stays gated off with fresh
rationale. No "healthy but does nothing." Update SECURITY §6 + CHANGELOG as each lands; re-run
the gate suite.

---

## G1 — Live sovereign gateway environment

- **G1.1** A runnable `SFU_MODE=sovereign` gateway (compose service or Dagger step) with a real
  UDP/ICE path reachable from a browser, JWT wired (or `DEV_NO_AUTH` for the harness tenant).
- **G1.2** A Keto relation seeded so the harness subject has `view` on the test room (ADR-007) —
  the join must pass the real fail-closed check, not a bypass.

**Exit:** the gateway boots in `sovereign` mode and `/healthz` + `/ws/v1/signal` are reachable.

## G2 — Live decoded-media run (the proof)

- **G2.1** Run `media-decode.spec.ts` in Dagger (Node + Chromium, `--use-fake-device-for-media-stream`)
  against the live gateway with `SKIP_INTEGRATION=false` + `GATEWAY_URL`; observe
  `inbound-rtp.framesDecoded > 0` on the receiver.
- **G2.2** Capture the run as CI evidence (or document it as locally-run with the transcript if CI
  browser infra is unavailable — honest, not silently skipped).

**Exit:** a real receiver decodes media relayed by the sovereign SFU, proven by the harness.

## G3 — Flip `SFU_MODE=sovereign` (only on a real G2 pass)

- **G3.1** Flip the `main.rs` sovereign branch from the "unproven end-to-end" warning to a live
  production path; update SECURITY §6 (media row → functional) + CHANGELOG.
- **G3.2** If G2 does **not** pass against real infra, **do not flip** — re-affirm gated with the
  fresh failure detail. The gate flip is conditional on G2, always.

**Exit:** `SFU_MODE=sovereign` moves real decoded media in production, or stays gated with rationale.

## G4 — Carried live proofs (from phase-19/20/21/22/23)

- **G4.1** LiveKit cross-node inbound: enable the `realtime` feature `LiveKitDataSource`; prove
  cross-node relay vs a live LiveKit server (the env now exists).
- **G4.2** admin-ui OIDC authorization-code flow — once ADR-004 is Accepted + an IdP (Kratos+Hydra)
  is deployed.

**Exit:** each live proof passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate suite green; each change passes the QA gate (**verdict read before archive**; and
  **archive output read**, not just verify — phase-23 c005 lesson).
- `SFU_MODE=sovereign` is enabled **only** if G2 proves decoded media against a live gateway.

## Non-goals

- Re-opening ADR-001/003/004/005/006/007 without a new finding.
- Net-new media features beyond what the live proof + flip require.

## Starting point (proven in phase-23)

- `admin-ui/e2e/{media-webrtc,media-decode}.spec.ts` + `e2e/support/{webrtc-client,decode-probe}.ts`
  — the signaling + decode harnesses (authored, integration-gated).
- `crates/frf-gateway/src/routes/signal.rs` — the `/ws/v1/signal` inbound→bridge path.
- `crates/frf-gateway/src/media_bridge.rs` — ADR-007 fail-closed authz.
- `crates/frf-gateway/src/main.rs` — the shared sovereign bridge (gate off, warning intact).
- ADR-005/006/007; `docs/SECURITY.md` §1/§2 Layer C/§5/§6.
