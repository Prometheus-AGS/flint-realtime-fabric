# Plan — phase-18-media-federation-and-auth-flow

> Backend: OpenSpec · Source: `assessment.md` (3 code-grounded audits) + 4 operator scope
> decisions (2026-07-07).
> Theme: **fix the cheap correctness bugs first, then complete the scoped deferred planes**
> — with the two XL items (full str0m SFU, LiveKit cross-node inbound) pushed to phase-19.

## Scope decisions (operator, 2026-07-07)

1. **G1 → token-flow hardening, not OIDC.** flint-gate has no interactive login endpoint
   and Kratos isn't deployed, so the full OIDC flow is deferred. This phase hardens the
   existing token model honestly (exp decode, expiry/401 handling, secure-storage
   guidance) — no new IdP/backend dependency.
2. **G2 → routing-bug fix + spike only.** Fix the `from_session`/`to_session` routing bug
   (unblocks signaling), then a ~1–2 day str0m spike proving one `Rtc` round-trip. The
   full per-session SFU + RTP fan-out (XL) is **deferred to phase-19**.
3. **G3 → the two self-contained directions + channel guard.** Matrix inbound (`/sync`
   over reqwest — no Tuwunel dep) and ATProto outbound (PDS `createRecord`), plus the XS
   channel-ID validate guard. LiveKit cross-node inbound (needs the realtime SDK) is
   **deferred to phase-19**.
4. **G4 → hand-written Dart shim.** `uniffi-bindgen-dart 0.1.3` is the latest (no upstream
   fix), so a thin Dart shim over the sync FFI makes the transport usable; plus doc-drift
   fix.

## Ordered change list (10 changes)

Ordering: **cheap correctness fixes first** (they de-risk everything and some unblock the
plane builds), then the scoped builds by cost, then docs/sign-off.

| # | Change | Goal | Gap | Agent | Size |
|---|--------|------|-----|-------|------|
| **p18-c001** | Fix str0m signaling **routing bug**: key delivery on `to_session` (unicast) + `room_id` (fan-out), not `from_session`; fix the masking test | G2 | B1 (HIGH) | rust-reviewer | S |
| **p18-c002** | Federation config: `validate()` requires `FEDERATION_CHANNEL_ID` when enabled (close the per-boot-random-channel gap) | G3 | B2 | rust-reviewer | XS |
| **p18-c003** | str0m consistency: stop forcing `sfu_mode=Hosted` on the wire; align `Default` vs `from_env` mode | G2 | B4 | rust-reviewer | XS |
| **p18-c004** | Dart doc drift: pubspec/GENERATED.md stop crediting flutter_rust_bridge; match the regenerated `frf.dart` + ADR-003 | G4 | B3 | doc-updater | XS |
| **p18-c005** | **G1 token-flow hardening**: decode `exp`, warn/logout on expiry, 401→re-auth, secure-storage guidance; re-scope LoginGate copy honestly | G1 | A1/A2 (re-scoped) | typescript-reviewer | M |
| **p18-c006** | **str0m spike**: bump `str0m` 0.7→0.21, prove one `Rtc` round-trip (SDP accept_offer + ICE + DTLS connect) adapter-only; gate stays off unless media flows | G2 | A3 | rust-reviewer | M |
| **p18-c007** | **Matrix inbound**: replace `stream::empty()` with a reqwest `/sync` long-poll loop (no Tuwunel dep); project events into the bridge | G3 | A5 | rust-reviewer | M |
| **p18-c008** | **ATProto outbound**: implement authenticated PDS write (`com.atproto.repo.createRecord`) replacing the `Err` stub | G3 | A6 | rust-reviewer | M |
| **p18-c009** | **Dart async-transport shim**: hand-written Dart over the sync FFI for connect/subscribe/ack; wire it through the package entry point | G4 | A8 | dart-build-resolver | M |
| **p18-c010** | **Docs + re-audit sign-off**: update SECURITY.md §6 / API-REFERENCE / CHANGELOG for what shipped; re-affirm the deferred items (full SFU, LiveKit inbound, full OIDC) with rationale; clean-checkout gate re-run | G1–G4 | — | code-reviewer | S |

**Deferred to phase-19 (documented, not dropped):** full str0m sovereign SFU (per-session
`Rtc` + UDP/ICE/DTLS loop + RTP fan-out), LiveKit cross-node inbound relay, and the full
admin-ui OIDC login flow (blocked on an IdP decision). c010 re-affirms these honestly.

## Exit criteria (from goals.md, refined by scope)

- **B1 routing bug fixed** — a signal from peer A reaches peer B (and room fan-out works);
  the masking test is corrected.
- **G1**: token expiry is handled (no silent-expired-token); LoginGate copy is honest
  about being a token gate, not OIDC.
- **G2**: str0m spike proves one real `Rtc` round-trip OR is re-affirmed with findings;
  `SFU_MODE=sovereign` still gated off unless media flows.
- **G3**: Matrix inbound streams real events; ATProto outbound writes to a PDS; channel-ID
  guarded. LiveKit inbound re-affirmed deferred.
- **G4**: Dart clients can connect/subscribe/ack via the shim; docs match reality.
- Every code change passes the QA gate (`.kbd-orchestrator/bin/qa-gate.sh`); clean-checkout
  gates green (c010).

## Notes for execute

- **c001 is the highest-value fix** — it unblocks the entire signaling substrate the media
  plane depends on, and it's small. Do it first.
- **c006 (str0m spike) is exploratory** — its deliverable is a proven round-trip *or* a
  documented finding, not a production SFU. Keep `SFU_MODE=sovereign` gated off unless
  media actually flows; do not advertise it as shipped.
- **c007/c008 are self-contained** (Matrix inbound over reqwest, ATProto PDS write) — no
  heavy external SDK; good mid-phase work.
- QA gate is ON for all code changes (c001–c003, c005–c009); c004 docs-only, c010
  verification — QA skipped per the phase-17 contract, which carries forward.
