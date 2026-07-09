# Reflection — phase-19-sovereign-media-and-federation-completion

> Generated: 2026-07-08
> Backend: OpenSpec, driven one change at a time via `/kbd-apply`.
> Changes: **7 / 7 DONE**, all archived. Seeded from the phase-18 reflection.

## Summary

Phase-19 took on the XL/external-dependency items phase-18 deferred. The assessment's key
act (again) was **re-grounding the seed goals against real code before planning**: it
confirmed the str0m full SFU is a ~2–4 week phase of its own, and — critically — that G4
(OIDC login) is **hard-blocked on infrastructure that does not exist** (no Kratos/Hydra in
compose; flint-gate has no `authorize` endpoint). The operator then scoped the phase to
land the genuinely-codeable wins, record the blocked items as **decisions/dated deferrals
rather than fake code**, and take the two big planes as far as honest effort allows —
splitting the full str0m media loop into a dedicated phase-20. Every plane is now either
functional end-to-end, a real capability behind an integration gate, or an honestly-dated
deferral. `SFU_MODE=sovereign` moves no media and stays gated off — no "healthy but does
nothing."

## Goal Achievement (against the operator-scoped plan)

| Goal | Intent | Delivered | Verdict | Evidence |
|------|--------|-----------|---------|----------|
| **G5.2** admin-ui lint | clean `pnpm lint` | fixed 2 errors (env-coercion skip bug + stale react-hooks disable) | **MET** | c001; `pnpm lint`/`typecheck` green |
| **G3** ATProto outbound | wire capability into gateway | PDS-writer config + `with_writer` in `main.rs`, all-or-none boot guard | **MET** | c002; 11 config tests, wired end-to-end |
| **G5.1** Dart async | revisit deferral | dated 2026-07-07 upstream check; deferral re-affirmed with removal trigger | **MET (re-affirmed deferred)** | c003; generator still 0.1.3-broken |
| **G4** OIDC login | full interactive login | **ADR-004** (recommend Kratos+Hydra); no IdP to build against | **MET as decision (impl blocked)** | c004; flint-gate has no authorize hook |
| **G2** LiveKit inbound | cross-node relay | `LiveKitDataSource` seam + relay loop, unit-tested; live source behind `realtime` feature | **MET (capability; integration-gated)** | c005; 2 tests, default build stays light |
| **G1** str0m full SFU | live media loop | **transport loop proven over a real socket**; full media split to phase-20 | **MET (thin slice; media→phase-20)** | c006; `the_event_loop_turns_over_a_real_socket` |
| **close** | seed + sign-off | phase-20 seeded; SECURITY §6 / CHANGELOG / sign-off; gates green | **MET** | c007 |

**Score: 7 / 7 delivered** against the operator-set scope. Two goals (G1 full media, G4
OIDC impl) were deliberately delivered as a *proven slice + a decision/seed* rather than a
faked full build — the honest outcome, carried to phase-20 with rationale.

## Delivered Changes (7)

- **Cheap wins:** c001 (admin-ui lint — a latent skip bug, not just a nit), c002 (ATProto
  outbound gateway wiring — makes the p18-c008 capability reachable).
- **Honest artifacts:** c003 (dated Dart deferral), c004 (ADR-004 OIDC IdP decision).
- **Big planes, honest depth:** c005 (LiveKit inbound-relay capability, integration-gated),
  c006 (str0m live-UDP transport-loop spike — the load-bearing unknown de-risked).
- **Close:** c007 (phase-20 seed + SECURITY §6 / CHANGELOG / sign-off + clean gate re-run).

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA gate run | 4 (c001, c002, c005, c006; c003/c004/c007 docs-only, skipped per contract) |
| First-pass gate pass rate | **3 / 4** |
| Changes that BLOCKED then passed | 1 (c002) |
| Changes requiring code refinement after a gate | 1 (c002) |

### Recurring patterns

- The one BLOCK (c002) hit **two** constraints at once: **R5 file-size >500** (`main.rs`
  reached 512 L and `config.rs` 580 L after the edits) and **P1 openspec validate** (the
  ADDED requirement needed a MUST on the line *immediately after* the header — the recurring
  spec-header rule). Both fixed (extracted `federation.rs` + `config/` module split; reordered
  the requirement sentence) → re-ran PASS.
- **Real issues caught at the standing Rust gates before the QA gate** (healthy): `doc_markdown`
  on `ATProto` (c002, recurring from p18-c008), `too_many_lines` on `from_env` (c002, fixed
  by extracting `federation_config_from_env` + `atproto_writer_config_from_env`),
  `large_enum_variant` on `PollStep::Event` (c006, boxed — str0m allows the same lint on its
  own `Output`).
- **File-size discipline forced two architecturally-sound splits** rather than slipping:
  `config.rs`→`config/{mod,tests}.rs`, `main.rs`→`federation.rs`. The 500-line limit did its
  job as a design pressure, not a formality.

## Technical Debt Introduced / Carried

1. **str0m full sovereign SFU media loop** — deferred to phase-20 (by design). The transport
   loop is proven; trickle ICE, DTLS/SRTP, RTP fan-out, and per-session async tasks remain.
   `SFU_MODE=sovereign` stays gated off.
2. **LiveKit live cross-node proof** — the relay capability + seam exist; the libwebrtc-backed
   `LiveKitDataSource` is behind the off-by-default `realtime` feature and its cross-node proof
   is integration-gated on a live server. Deliberate, to keep the default gateway build light.
3. **admin-ui OIDC** — blocked on ADR-004 acceptance + standing up an IdP (Kratos/Hydra). The
   token gate is the interim; no frontend OIDC code exists.
4. **Dart async transport** — still upstream-blocked (`uniffi-bindgen-dart 0.1.3`); the c009
   patch + shim remain, with a dated removal trigger.
5. **Local full-workspace test link budget** — `cargo test --workspace` exceeds the local link
   budget (str0m + httpmock); per-crate lib suites + `check`/`clippy` workspace gates run
   locally, full suite in CI. Recurring since phase-18; flagged in the sign-off, not hidden.

## Lessons Captured

- **Re-audit the seed goals against real infra, not just real code.** The seed listed OIDC
  as codeable; grounding it revealed there is no authorization server in the stack at all —
  turning a would-be fake login into ADR-004. The most valuable assess output was "this is
  blocked, here's the decision," not a task list.
- **A blocked goal's honest deliverable is a decision or a dated deferral.** c004 (ADR) and
  c003 (dated upstream check) are real progress: they unblock the operator and stop the
  deferral note from rotting — better than faking implementation against missing infra.
- **A trait seam turns an "XL heavy-dependency" goal into a codeable capability.** c005:
  abstracting the data-channel behind `LiveKitDataSource` let the relay logic be built +
  unit-tested now, with the heavy libwebrtc source gated behind a feature — capability
  without bloating every gateway build.
- **A spike's job is to kill the load-bearing unknown, cheaply and provably.** c006 proved
  the sans-I/O UDP loop turns over a real socket (reading the vendored str0m 0.21 source to
  get the exact `Input`/`Output`/`net` API) — de-risking phase-20's start without pretending
  media flows.
- **The 500-line limit is a design tool.** Twice this phase it forced a module split
  (`config/`, `federation.rs`) exactly where cohesion justified it. Treat the file-size gate
  as architectural pressure, not paperwork.
- **Bump progress.json to N/N *before* any command mentioning `/kbd-reflect`** (recurring
  16–19): the pipeline-enforce hook reads progress and blocks reflect-mentioning commands
  while a change is pending. Split the progress bump from the waypoint edit.
- **Write the spec delta with a MUST on the line after the header, before the QA gate**
  (recurring): P1 `openspec validate` enforces it; c002 re-learned it.

## Recommended Next Phase

**phase-20-sovereign-sfu-media-loop** (already seeded in c007) — turn the proven transport
loop into a working sovereign SFU that actually moves media:

1. **Per-session async transport tasks** — promote the c006 `TransportLoop` (single `Rtc`,
   blocking socket) into per-session tokio tasks (`recv_from` + `poll_output`-deadline timer).
2. **Trickle ICE over the signaling channel** — inbound `IceCandidate` → `add_remote_candidate`;
   emit local candidates; real host/srflx gathering + connectivity checks.
3. **DTLS/SRTP + RTP forwarding** — DTLS to `Connected`, forward `Event::MediaData` between
   peers with per-room fan-out + PLI handling.
4. **Enable the sovereign gate only when media flows** — flip `SFU_MODE=sovereign` to live
   *only* after G1–G3 move media end-to-end; cover its boundary in SECURITY §1–§5 first.
5. **Live cross-node proofs (carry-forward)** — implement the libwebrtc `LiveKitDataSource`
   behind `realtime` and prove cross-node relay; admin-ui OIDC once ADR-004 is Accepted + an
   IdP is stood up.

Do not advertise any of these as shipped until each functions end-to-end or is re-affirmed
deferred — the discipline that carried phases 16–19.
