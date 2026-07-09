# Goals — phase-26-gateway-docker-adminui-embed-and-decode

> Seeded from: phase-25 (reflection). The authenticated live-decode path is built and works up to
> the Docker gateway build. Phase-25 ran the proof and it failed on a **Dockerfile defect**:
> `frf-gateway` `rust-embed`s `admin-ui/dist`, which the Dockerfile never builds/copies →
> the gateway image won't compile → the stack never boots (`docs/PHASE-25-DECODE-RESULT.md`).
> **Not yet active** — `/kbd-next-phase` flips the waypoint here after `/kbd-assess`.

This is the **narrowest remaining finish**: fix that one Dockerfile defect so the gateway image
builds, re-run the proof — at which point, **for the first time**, the stack boots and the harness
reaches the SFU — and observe a real `framesDecoded > 0`, then flip. The media-path itself
(ICE/DTLS/RTP over the UDP candidate) is still untested; this phase finally tests it.

**Discipline (carried 16–25):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite.

---

## G1 — Fix the Dockerfile admin-ui embed

The defect: `Dockerfile` copies only `Cargo.*`/`crates/`/`proto/`; `admin-ui/dist` is absent in the
build context, so `#[derive(RustEmbed)]` on it fails (`E0599 no get for AdminUiAssets`, folder does
not exist). Host builds pass only because a prior `vite build` left `dist/` on disk.

- **G1.1** Add a Node build stage (e.g. `FROM node … RUN corepack enable && pnpm --dir admin-ui
  install --frozen-lockfile && pnpm --dir admin-ui build`) and `COPY --from=<node> …/admin-ui/dist
  ./admin-ui/dist` into the Rust build context **before** the `cargo build`. (Or build admin-ui on
  the host and COPY the artifact — pick the approach that keeps the image reproducible.)
- **G1.2** The gateway image builds clean (`docker compose -f compose.yml -f compose.sovereign.yml
  build gateway` succeeds).

**Exit:** the sovereign gateway image builds in Docker with the admin UI embedded.

## G2 — Live decode run (the first real media-path test)

- **G2.1** Re-run `scripts/run-media-decode.sh` (authenticated path from p25). The stack boots, the
  harness authenticates + joins, and — for the first time — the run reaches ICE/DTLS/RTP.
- **G2.2** Observe `getStats().inbound-rtp.framesDecoded > 0` on the receiver; capture the run as
  `docs/PHASE-26-DECODE-RESULT.md`. If it fails, this is now most likely a **genuine media-path**
  issue (advertise-IP / ICE / UDP reachability) — diagnose it concretely; the gate holds.

**Exit:** a real receiver decodes media relayed by the sovereign SFU, proven by the harness.

## G3 — Flip `SFU_MODE=sovereign` (only on a real G2 pass)

- **G3.1** Flip the `main.rs` sovereign branch from the gate-off warning to a live path; update
  `docs/SECURITY.md` §6 (media → functional) + CHANGELOG + `docs/PHASE-26-SIGNOFF.md`.
- **G3.2** If G2 does not pass, re-affirm gated with the fresh (media-path) detail. Conditional on
  G2, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with rationale.

## G4 — Carried live proofs (from phase-19…25)

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
- Reworking the admin UI itself — this phase only fixes how it is built + embedded in the image.

## Starting point (from phase-24/25)

- `Dockerfile` — the gateway image build (needs the admin-ui embed fix).
- `scripts/{mint-e2e-jwt.mjs,run-media-decode.sh,seed-media-view.sh}` — the authenticated runner.
- `compose.sovereign.yml` — sovereign UDP media override + `GATEWAY_JWKS_URL` param.
- `admin-ui/e2e/{media-decode.spec.ts,support/{decode-probe,webrtc-client}.ts}` — the harness.
- `crates/frf-media-str0m/src/config.rs` — `MediaConfig` bind/advertise seam.
- `docs/PHASE-25-DECODE-RESULT.md` — the recorded Dockerfile blocker + the precise fix.
