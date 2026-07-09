# Plan — phase-19-sovereign-media-and-federation-completion

> Stage: Plan · 2026-07-07 · Backend: **OpenSpec** (`change_backend: openspec`)
> Source: `assessment.md`. Operator decisions locked (see below).
> Ordering principle (carried 16–18): **cheap correct wins first, blocked decisions as
> artifacts not fake code, big planes only as far as honest effort allows.**

## Operator decisions (locked this stage)

1. **str0m (G1):** thin **live-UDP transport-loop spike** here; **split full media fan-out
   into `phase-20-sovereign-sfu-media-loop`.** Do not attempt the full SFU this phase.
2. **OIDC (G4):** produce an **ADR** naming the IdP path (Kratos/Hydra-in-compose vs.
   flint-gate auth-code endpoint), recommend one, re-affirm the token gate as interim.
   **No frontend OIDC code** — there is no IdP to build against.
3. **LiveKit (G2):** build the realtime data-channel client + listen loop;
   **capability built + integration-gated** exit (unit-test the plumbing; gate the true
   cross-node proof behind a live-LiveKit integration test).

## Ordered changes

Each is one OpenSpec change (`/opsx:new <id>` → proposal + spec delta + tasks), driven one
at a time via `/kbd-apply`. Spec delta written **before** the QA gate (P1 runs
`openspec validate`). QA gate per change unless docs-only/verification-only.

| # | Change ID | Goal | Type | QA gate | Summary |
|---|-----------|------|------|:------:|---------|
| c001 | `p19-c001-admin-ui-lint-debt` | G5.2 | fix | yes | Fix the 2 reproducing lint errors; `pnpm lint` green. |
| c002 | `p19-c002-atproto-outbound-gateway-wiring` | G3 | feat | yes | PDS config + `with_writer` in `main.rs`; env vars documented. |
| c003 | `p19-c003-dart-async-defer-reaffirm` | G5.1 | docs | skip | Re-affirm deferral; verify no newer generator; update note. |
| c004 | `p19-c004-oidc-idp-adr` | G4 | docs | skip | ADR: IdP path decision; token gate as interim. |
| c005 | `p19-c005-livekit-inbound-relay-capability` | G2 | feat | yes | Realtime data-channel client + listen loop → `subscribe_signals`; integration-gated. |
| c006 | `p19-c006-str0m-udp-transport-loop-spike` | G1 | feat | yes | Live `UdpSocket` + `handle_input`/`poll_output` event-loop spike; findings doc. |
| c007 | `p19-c007-phase-20-seed-and-docs-close` | close | docs | skip | Seed `phase-20-sovereign-sfu-media-loop` handoff; SECURITY §6 / CHANGELOG / sign-off. |

**7 changes.** c001–c002 are the guaranteed wins; c003–c004 are honest artifacts for the
blocked items; c005–c006 push the two big planes as far as honest effort allows; c007
closes the phase and seeds the split-out phase-20.

---

## Change detail

### c001 — admin-ui lint debt (G5.2) · fix · QA:yes
- **Files:** `admin-ui/e2e/p7-smoke.spec.ts`, `admin-ui/src/features/entities/hooks/useEntitySubscription.ts`,
  possibly `admin-ui/eslint.config.*` (if the `react-hooks` rule needs registering).
- **Tasks:** (1) `p7-smoke.spec.ts:21` → `process.env["SKIP_INTEGRATION"] === "true"`.
  (2) Diagnose `react-hooks/exhaustive-deps` "rule not found" — register the
  `eslint-plugin-react-hooks` plugin in the flat config **or** remove the stale inline
  disable if the rule was intentionally dropped. Confirm which before editing.
  (3) `pnpm lint` green; `pnpm typecheck` still green.
- **Exit:** `pnpm lint` exits 0.

### c002 — ATProto outbound gateway wiring (G3) · feat · QA:yes
- **Files:** `crates/frf-gateway/src/config.rs` (+ `main.rs`), `docs/ENVIRONMENT.md`.
- **Tasks:** (1) Add PDS-writer config: `ATPROTO_PDS_URL`, `ATPROTO_PDS_IDENTIFIER`,
  `ATPROTO_PDS_APP_PASSWORD` (secret; validated together — all-or-none), optional
  `ATPROTO_WRITE_COLLECTION`. (2) In `main.rs:245-252`, when PDS config is present, call
  `AtProtoBridge::with_writer(PdsConfig, write_collection)`; update the inbound-only log.
  (3) Document the env vars in `ENVIRONMENT.md`; **never** log/commit the app-password
  (CLAUDE.md security rule). (4) Config unit test: writer present ⇒ outbound enabled;
  absent ⇒ inbound-only (existing behavior).
- **Exit:** with PDS config set, the gateway builds an outbound-capable ATProto bridge; the
  p18-c008 mocked-PDS write path is now reachable from the gateway. Clean gate.
- **Security note:** app-password is a credential — env/secret only, never in debug output.

### c003 — Dart async transport: re-affirm deferral (G5.1) · docs · QA:skip
- **Files:** `sdks/dart/GENERATED.md`, `docs/SECURITY.md` §6 (or the deferral log).
- **Tasks:** (1) Verify `uniffi-bindgen-dart` still has no async/callback fix beyond 0.1.3
  (record the check + date). (2) Re-affirm the deferral with the upstream tracking
  reference; keep the c009 patch note honest. No code change — pretending otherwise would
  be dishonest (assessment).
- **Exit:** deferral documented with a dated upstream check; no stale "coming soon" copy.

### c004 — OIDC IdP ADR (G4) · docs · QA:skip
- **Files:** `docs/adr/ADR-004-admin-ui-oidc-idp.md` (new), `docs/SECURITY.md` §6 xref.
- **Tasks:** (1) State the constraint: no IdP in `compose.yml`; flint-gate is a
  token-metering proxy with **no `authorize` hook**. (2) Present both paths —
  **A) Kratos/Hydra in compose**, **B) flint-gate authorization-code endpoint** — with
  effort/ownership/tradeoffs. (3) **Recommend one**, with rationale. (4) Re-affirm the
  p18-c005 token gate as the hardened interim; note the router change G4.2 will need.
  (5) Mark G4.2 (frontend flow) blocked-on-ADR-acceptance.
- **Exit:** an operator can pick the IdP path from the ADR; no OIDC code shipped against a
  non-existent IdP.

### c005 — LiveKit inbound-relay capability (G2) · feat · QA:yes
- **Files:** `crates/frf-media-livekit/Cargo.toml` (add the realtime `livekit` client dep),
  `crates/frf-media-livekit/src/adapter.rs` (+ possibly a new `inbound.rs` module to keep
  ≤500 L / one concern).
- **Tasks:** (1) Add the LiveKit **realtime** SDK (WebRTC data-channel client) — confirm the
  current crate name/version (docfork/Tavily) before pinning. (2) A listen loop that
  connects to the room, receives server-originated data packets, and feeds them into the
  `subscribe_signals` broadcast — replacing the "in-process only" limitation documented at
  `adapter.rs:26-38`. (3) Unit-test the plumbing (decode → forward) with a mocked/injected
  data source. (4) Gate the true **cross-node** proof behind a `#[ignore]` / feature-flagged
  integration test that needs a live LiveKit server; document it as integration-gated.
- **Exit (integration-gated):** the realtime client + forward path compile and unit-test;
  cross-node end-to-end proof is an integration test, honestly gated. Clean gate.
- **Risk:** the realtime SDK may pull heavy transitive deps / need tokio features — watch
  the link budget (phase-18 hit this with str0m + httpmock).

### c006 — str0m live-UDP transport-loop spike (G1 thin slice) · feat · QA:yes
- **Files:** `crates/frf-media-str0m/src/` (new `transport_spike.rs` or extend the spike;
  keep files ≤500 L), `crates/frf-media-str0m/SPIKE-FINDINGS.md` (update).
- **Tasks:** (1) Bind a real `UdpSocket`; drive one `Rtc` through the sans-I/O loop:
  `handle_input(Input::Receive/Timeout)` + `poll_output()` → `Transmit/Timeout/Event` on a
  timer; write outbound datagrams. (2) Prove the **load-bearing unknown** — a local
  UDP round-trip through the event loop (e.g. two sockets / a loopback drive), NOT full
  browser interop. (3) Document exactly what the spike proves vs. what the full media loop
  (phase-20) still needs; keep `SFU_MODE=sovereign` **gated off**. (4) No `unwrap()`/`expect()`
  in the lib crate; anyhow only at edges (CLAUDE.md).
- **Exit:** the transport loop runs a real UDP datagram through str0m's I/O in a test, OR
  the specific blocker is documented honestly. `SFU_MODE` stays gated. Clean gate.
- **Discipline:** this is a *spike*, deliberately not the full SFU. Do not flip the gate.

### c007 — phase-20 seed + docs close · docs · QA:skip
- **Files:** `.kbd-orchestrator/phases/phase-20-sovereign-sfu-media-loop/` (goals seed +
  handoff), `docs/SECURITY.md` §6, `CHANGELOG.md`, `docs/PHASE-19-SIGNOFF.md` (new),
  `README.md` (if a status table changed).
- **Tasks:** (1) Seed the phase-20 handoff: full str0m media loop (UDP event loop → trickle
  ICE → DTLS/SRTP RTP fan-out → per-session architecture) building on the c006 spike, plus
  carry-forwards (LiveKit cross-node live proof if still gated; G4.2 pending ADR
  acceptance). (2) Update SECURITY §6 with the new posture of each item (ATProto outbound
  now wired; sovereign SFU spike-only; OIDC ADR pending). (3) CHANGELOG + sign-off; re-run
  the release gate suite (fmt / clippy pedantic + unwrap_used / check / test) and record it.
- **Exit:** phase-19 signed off; phase-20 seeded; no plane advertised beyond what functions.

---

## Sequencing & rationale

- **c001 → c002 first:** the two guaranteed end-to-end wins; get the phase to real value fast.
- **c003, c004 next:** honest artifacts for the toolchain-/infra-blocked items — cheap,
  and they unblock the operator's decisions rather than faking implementation.
- **c005, c006:** the two big planes taken as far as *honest* effort allows (capability +
  integration-gate; transport-loop spike). Ordered after the sure wins so a spill here
  doesn't strand the phase with nothing shipped.
- **c007 last:** seeds the split-out phase-20 and closes with a clean gate re-run.

## Phase exit criteria (from goals.md)

- Every goal **functions end-to-end** OR is **re-affirmed deferred** with fresh rationale in
  `docs/SECURITY.md` §6 + CHANGELOG.
- Release gate suite green; each code change (c001, c002, c005, c006) passes the QA gate.
- **No "healthy but does nothing":** `SFU_MODE=sovereign` stays gated off (spike only);
  LiveKit cross-node proof is honestly integration-gated; no OIDC path shipped against a
  non-existent IdP; ATProto outbound only enabled when PDS config is actually present.

## First change to apply

`p19-c001-admin-ui-lint-debt` — smallest, self-contained, both errors already reproduce.
