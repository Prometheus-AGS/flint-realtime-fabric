# Assessment — phase-19-sovereign-media-and-federation-completion

> Stage: Assess · 2026-07-07 · Backend: OpenSpec
> Grounds the six seeded goals (`goals.md`) against the real tree — the phase-18 lesson
> was "re-audit the seed goals, don't trust them." Each verdict below is code-grounded.

## Method

Seeded goals came from the phase-18 reflection's "Recommended Next Phase." I inspected the
actual state of each target (crate layout, `main.rs` wiring, compose infra, lint output,
generator pins) before scoping. Findings drive the plan's ordering and — critically —
which goals are **codeable this phase** vs. **blocked on infra/toolchain decisions**.

## Goal-by-goal grounding

### G1 — str0m full sovereign SFU · **XL / OWN-PHASE-SIZED · not fully codeable here**

- `crates/frf-media-str0m/`: `sfu.rs` (320 L, signaling relay — routing fixed p18-c001),
  `rtc_spike.rs` (107 L, negotiation proof p18-c006), `SPIKE-FINDINGS.md`. str0m pinned
  **0.21.0** in `Cargo.lock`. The negotiation half is proven; the media half is not built.
- `SPIKE-FINDINGS.md` scopes the remainder itself as **"~2–4 week full-build, its own
  phase"**: live UDP `handle_input`/`poll_output` event loop, trickle ICE (candidate
  gathering + connectivity), DTLS/SRTP RTP fan-out, and a per-session `Rtc`+socket+task
  architecture replacing the channel-only `StrOmSignaler`.
- **Biggest risk (unchanged):** the sans-I/O UDP/ICE/DTLS transport loop is **untestable
  without a real browser** — where WebRTC interop actually breaks. It cannot be honestly
  "proven end-to-end" inside a normal KBD change with a unit test.
- **Gate today:** `SFU_MODE` config default is `Sovereign` in `config.rs:135` but the
  gateway boots `hosted` by validation/warning; sovereign moves **no media** (documented).
- **Verdict:** This is a full phase on its own. Attempting it as one change here would
  either be faked or spill weeks. **Recommend: carve a thin, honest slice this phase (the
  live UDP echo/transport-loop spike — the load-bearing unknown) and split the full media
  fan-out into a dedicated `phase-20-sovereign-sfu-media-loop`.** The plan must make this
  split explicit; do not pretend a full SFU lands here.

### G2 — LiveKit cross-node inbound relay · **REAL BUILD (external SDK) · codeable, medium**

- `crates/frf-media-livekit/adapter.rs:26-38` documents the gap in-code: `subscribe_signals`
  is **in-process only** (local broadcast of this process's own outbound); it does **not**
  subscribe to the LiveKit server data channel. The dep is `livekit-api` (server REST SDK,
  `RoomClient`) — **not** the realtime `livekit` client (WebRTC data-channel).
- **Finding:** true cross-node inbound needs adding the LiveKit **realtime** SDK (a WebRTC
  data-channel client) and a listen loop feeding `subscribe_signals`. That's a real new
  external dependency + an async subscription task — medium effort, testable only against a
  live LiveKit server (like G1, hard to unit-test end-to-end).
- **Verdict:** codeable, but the "prove cross-node end-to-end" bar needs a live LiveKit.
  Plan for the client + wiring; the exit may be "capability built + integration-gated" (the
  honest phase-18 pattern), not a green unit test.

### G3 — Wire ATProto outbound into the gateway · **SMALL / SELF-CONTAINED · fully codeable** ✅

- `crates/frf-bridge-atproto/src/lib.rs`: `with_writer(PdsConfig, Option<String>)` exists
  and is **tested** (mocked-PDS `send_writes_record_to_pds`, p18-c008). The capability is done.
- `crates/frf-gateway/src/main.rs:245-252`: builds `AtProtoBridge::new(...)` **inbound-only**
  and logs "OUTBOUND send is unsupported for v1."
- `docs/ENVIRONMENT.md:67-78`: has `ATPROTO_JETSTREAM_URL` / `ATPROTO_COLLECTIONS` (inbound);
  **no** PDS-writer env vars yet.
- **Verdict:** the cleanest win of the phase. Add PDS config (service URL, identifier,
  app-password via secret) + call `with_writer` in `main.rs` when configured; document the
  new env vars; never commit the app-password. Small, testable, end-to-end honest.

### G4 — Full admin-ui OIDC login · **HARD-BLOCKED on infra · NOT codeable this phase**

- `compose.yml` services: `gateway, iggy-server, keto-migrate, keto, flint-gate, surrealdb,
  postgres`. **No Kratos, no Hydra — there is no IdP / authorization server in the stack.**
- flint-gate is a **token-metering enforcement proxy**, explicitly **"no `authorize`
  hook"** (its own README/config) — it does RFC-8693 token-exchange + client_credentials,
  **not** interactive OIDC authorization-code. It cannot serve a browser login redirect.
- admin-ui has **no** `/callback`, `authorize`, or PKCE code today (only the p18-c005
  token gate + `handleUnauthorized`).
- **Verdict:** G4.2 (the frontend flow) is **strictly downstream of G4.1** — an IdP
  decision the operator must make (stand up Kratos/Hydra in compose, OR add an auth-code
  endpoint to flint-gate). **Recommend: G4 is a decision/ADR this phase, not an
  implementation.** Produce the ADR that names the IdP path + effort; re-affirm the token
  gate as the hardened interim. Do not build a frontend OIDC flow against a non-existent IdP.

### G5.1 — Dart async transport · **TOOLCHAIN-BLOCKED · re-affirm deferred**

- `sdks/dart/build_dart.sh` + `analysis_options.yaml` pin the constraint: `uniffi-bindgen-dart
  0.1.3` (latest) emits non-compiling async/callback code; the committed `frf.dart` is the
  c009 documented patch. No newer generator is available.
- **Verdict:** nothing changed upstream; **re-affirm deferred** with the tracking reference.
  A code change here would be pretending. (Cheap: verify no newer release, update the note.)

### G5.2 — Admin-ui lint debt · **TRIVIAL · fully codeable** ✅

- `pnpm lint` reproduces exactly 2 errors:
  1. `e2e/p7-smoke.spec.ts:21` — `local/no-boolean-env-coercion`: `!!process.env[…]` →
     use `process.env["SKIP_INTEGRATION"] === "true"`.
  2. `src/features/entities/hooks/useEntitySubscription.ts:41` — `react-hooks/exhaustive-deps`
     rule **"Definition not found"** → the eslint-plugin-react-hooks rule isn't registered
     in the flat config; fix is a config wiring issue (register the plugin) or remove the
     stale inline disable. Confirm which before editing.
- **Verdict:** small, self-contained cleanup; `pnpm lint` must go green.

## Gap summary & phase-shape recommendation

| Goal | Grounded verdict | Codeable this phase? |
|------|------------------|----------------------|
| **G3** ATProto outbound wiring | small, capability already built + tested | ✅ yes — cleanest win |
| **G5.2** admin-ui lint | 2 trivial errors, both reproduce | ✅ yes |
| **G5.1** Dart async | upstream generator still broken | ⚠️ re-affirm deferred (cheap) |
| **G4** OIDC login | no IdP in stack; flint-gate has no auth hook | ⚠️ **ADR/decision only** — G4.2 blocked on G4.1 |
| **G2** LiveKit inbound | needs realtime SDK + live server to prove | ◐ build capability; integration-gated |
| **G1** str0m full SFU | XL, ~2–4wk, transport loop untestable sans browser | ◐ **thin transport-loop slice; split full media to phase-20** |

**Recommended phase shape (for `/kbd-plan`):** land the genuinely-codeable wins honestly
(G3 wiring, G5.2 lint), record the two blocked decisions as artifacts not fake code (G4
ADR, G5.1 deferral re-affirmation), then take the two big planes as far as *honest* effort
allows — a live str0m **transport-loop spike** (G1's load-bearing unknown) and a LiveKit
**inbound-relay capability** — and **explicitly split the full str0m media fan-out into its
own phase-20**. This preserves the 16–18 discipline: nothing advertised shipped until it
functions end-to-end or is re-affirmed deferred; no "healthy but does nothing."

## Open questions for `/kbd-plan`

1. **str0m split:** confirm carving G1 into a thin transport-loop slice here + a dedicated
   `phase-20-sovereign-sfu-media-loop`. (Assessment recommends yes.)
2. **G4 IdP path:** does the operator want the ADR to recommend Kratos/Hydra-in-compose,
   or a flint-gate auth-code endpoint? (Assessment: ADR names both, recommends one, defers
   implementation.)
3. **G2 exit bar:** accept "capability built + integration-gated" (no live-LiveKit unit
   proof), matching the phase-18 honest-defer pattern?
