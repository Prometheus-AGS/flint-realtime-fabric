# Reflection — phase-18-media-federation-and-auth-flow

> Generated: 2026-07-07
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **10 / 10 DONE**, all archived. Seeded from the phase-17 reflection.

## Summary

Phase-18 took on the deferred "big planes" phases 16–17 gated off. The assessment's most
important act was catching that the seed goals' **ordering and scope were wrong** — G1
(admin-ui login) was seeded as "cheapest, no backend dep" but is actually blocked on a
missing IdP + flint-gate login endpoint; G2 (str0m) hid a HIGH routing bug that broke even
signaling. The operator then re-scoped all four goals (verify + pure/self-contained work;
XL items deferred to phase-19), and the phase delivered exactly that scope: cheap
correctness fixes first, then the achievable planes. Every plane is now either functional
end-to-end or honestly deferred and gated off — no "healthy but does nothing."

## Goal Achievement (against the operator-chosen scope)

| Goal | Original intent | Re-scoped (operator) | Verdict | Evidence |
|------|-----------------|----------------------|---------|----------|
| **G1** | admin-ui interactive OIDC login | **token-flow hardening** (no IdP) | **MET (re-scoped)** | c005: `exp` decode, expiry watchdog auto-logout, 401→clear-token, honest LoginGate copy. Full OIDC re-affirmed deferred (no IdP/endpoint). |
| **G2** | str0m sovereign SFU real WebRTC | **routing fix + spike only** | **MET (re-scoped)** | c001: signaling routing bug fixed (unicast/room fan-out). c006: str0m 0.7→0.21, negotiation round-trip proven (2 tests). Full media loop deferred (`SPIKE-FINDINGS.md`). |
| **G3** | all 3 broken federation dirs | **Matrix inbound + ATProto outbound + channel guard** | **MET (re-scoped)** | c007: Matrix `/sync` loop (no Tuwunel dep). c008: ATProto PDS write (mocked-PDS test). c002: channel-ID guard. LiveKit inbound re-affirmed deferred. |
| **G4** | Dart async-transport bindings | **hand-written shim** | **MET (re-scoped)** | c009: `FrfTransport` shim + working `FrfCrdt`; generated file patched (documented) so the package compiles. Async bindings deferred (upstream generator bug). |

**Score: 4 / 4 MET** against the scope the operator set. No goal NOT-MET. The deferred XL
items (full SFU, LiveKit cross-node inbound) were explicitly carried to phase-19 with
rationale, not dropped.

## Delivered Changes (10)

- **Correctness (cheap-first):** c001 (str0m routing bug — HIGH), c002 (federation
  channel-ID guard), c003 (sfu_mode wire consistency), c004 (Dart doc drift).
- **Planes:** c005 (admin-ui token hardening), c006 (str0m spike), c007 (Matrix inbound
  `/sync`), c008 (ATProto outbound PDS write), c009 (Dart transport shim).
- **Close:** c010 (docs — SECURITY §6 / API-REF / CHANGELOG / README / sign-off + clean
  gate re-run).

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA gate run | 8 (c001–c003, c005–c009; c004 docs-only + c010 verification skipped per contract) |
| First-pass gate pass rate | **7 / 8** |
| Changes that BLOCKED then passed | 1 (c005 — blocked on a missing spec delta, added, re-ran green) |
| Changes requiring code refinement after a gate | 0 |

### Recurring patterns

- No BLOCKING *code* constraint failed. The one BLOCK (c005) was a **process** miss — the
  spec delta must exist before the gate runs its P1 (`openspec validate`) check. Once
  caught, I wrote the delta first for c006–c009, so they passed on the first gate run.
- Real issues were caught at the **standing Rust gates** before the QA gate: cast-wrap
  cleanups, `needless_pass_by_value` (c007), `assigning_clones` (c001), `doc_markdown`
  backticks (c008). Healthy — the gates caught them, they were fixed, QA then passed.

## Technical Debt Introduced / Carried

1. **Dart generated file is hand-patched.** `uniffi-bindgen-dart 0.1.3` emits a
   non-compiling `frf.dart`; c009 replaced two methods with documented compile-valid
   throws so the package builds. This is post-generation patching (normally forbidden) —
   documented in `GENERATED.md`, and it evaporates when a fixed generator regenerates the
   file. Follow-up: track the upstream fix.
2. **ATProto outbound capability not wired into the gateway.** c008 implemented + tested
   the bridge write path, but `main.rs` still builds the bridge inbound-only. Enabling
   outbound needs PDS identity config — a small follow-up, flagged in SECURITY §6.
3. **Pre-existing admin-ui lint debt.** The full `pnpm lint` fails on 2 errors in files
   c005 didn't touch (`e2e/p7-smoke.spec.ts`, `entities/useEntitySubscription.ts`). I did
   not silently fix unrelated files; flagged for a future cleanup.
4. **str0m full media loop + LiveKit cross-node inbound + full OIDC** remain deferred (by
   design) to phase-19.

## Lessons Captured

- **Re-audit the seed goals, don't trust them.** The phase-17 reflection seeded G1 as
  "cheapest, no backend dep" — the code-grounded assessment proved that false (no IdP, no
  flint-gate login endpoint). Grounding each goal against real code *before* planning is
  what turned a wrong plan into an achievable one.
- **A spike's deliverable can be a proof OR a documented finding — both are honest wins.**
  c006 proved the str0m negotiation round-trip and documented the remaining media-loop
  risk in `SPIKE-FINDINGS.md`; the gate stayed off. That's better than either faking a
  full SFU or skipping the de-risking.
- **"Never edit generated code" has a real exception: when it doesn't compile.** c009's
  generated file blocked the whole package from building; the honest fix was a *documented*
  minimal patch, not pretending the shim alone sufficed.
- **Spec delta before the QA gate.** The gate's P1 runs `openspec validate`; writing the
  delta first (learned at c005) avoided spurious BLOCKs for the rest of the phase.
- **Run the apply driver from the repo root.** c004's `end-task` silently no-op'd because
  I'd `cd`'d into `sdks/dart` (the driver detects the backend from cwd). Caught via
  `progress`, re-marked from root.
- **`git grep` for a forbidden token matches comments/docs too** (recurring from phase-17)
  — distinguish active settings from documentation before calling a regression.

## Recommended Next Phase

**phase-19-sovereign-media-and-federation-completion** — the deferred XL/external items,
now that the cheap correctness and self-contained planes are done:

1. **str0m full sovereign SFU** — the live UDP/ICE/DTLS event loop + RTP fan-out, building
   on the c006 negotiation spike and its findings. Largest, highest-risk; likely its own
   focus. (`str0m` is already at 0.21 and the API is mapped.)
2. **LiveKit cross-node inbound relay** — subscribe to the LiveKit server data channel so
   remote-node signals surface locally.
3. **Wire ATProto outbound into the gateway** — small: add PDS identity config + call
   `AtProtoBridge::with_writer` in `main.rs` (capability already built + tested).
4. **Full admin-ui OIDC login** — requires standing up Kratos/Hydra (or a flint-gate
   auth-code endpoint) first; the token gate is the hardened interim until then.
5. **Dart async transport** — revisit when `uniffi-bindgen-dart` fixes async/callback
   codegen, or land the full hand-lowering; remove the c009 generated-file patch when the
   generator is fixed.
6. **Admin-ui lint debt cleanup** — fix the 2 pre-existing lint errors.

Do not advertise any of these as shipped until each functions end-to-end or is re-affirmed
deferred — the discipline that carried phases 16–18.
