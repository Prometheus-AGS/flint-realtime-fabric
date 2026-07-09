# Plan — phase-23-sovereign-sfu-e2e-and-gate

> Backend: **OpenSpec**. Generated 2026-07-08 from `assessment.md` + two operator decisions:
> **G3 = Keto view-check at room-join** (per-participant `check(subject,"view",room)`, ADR-007 +
> code + tests), **G2 = browser↔headless-str0m** (reuse `rtc_spike`, deterministic CI).
>
> Ordering rationale: the **security boundary is documented + enforced before the flip** — the
> flip (c006) is the *last* code change and only lands if the proof (c004) passes. No change
> advertises a plane beyond what it does.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p23-c001-media-authz-adr` | **ADR-007** — media-path authz: per-participant Keto `check(subject,"view",room)` at RoomJoin, layered on the existing JWT gate + `(TenantId,room)` keying. Records the operator decision + rejected alternative. Doc-only. | low |
| c002 | `p23-c002-keto-view-check-on-join` | Enforce ADR-007: `MediaTransportBridge` gains `Arc<dyn AuthzProvider>`; on `RoomJoin`, call `check(subject,"view",room_id)` **before** `join_room` — deny ⇒ no membership, no fan-out. Subject derived from the authed session. **Authz stays in the gateway/bridge; the str0m adapter imports no authz** (dependency rule + one-port-per-adapter). Tests: unauthorized-join rejected, authorized-join admitted, cross-tenant-no-fanout. | med |
| c003 | `p23-c003-browser-e2e-harness` | **G1** — a minimal WebRTC client page (served or Playwright-injected) + a Playwright spec in `admin-ui/e2e/` that connects a real Chromium peer to a running `SFU_MODE=sovereign` gateway via the signal channel and reaches `Connected`. Reuses `playwright.config.ts`. Dagger wiring behind a browser-available flag. | med |
| c004 | `p23-c004-decoded-media-proof` | **G2** — the honestly-gated proof: Chromium offerer ↔ **headless str0m peer** (reuse `crates/frf-media-str0m/src/rtc_spike.rs`) through the sovereign gateway; the receiver **decodes ≥1 frame** — exercising real ICE + DTLS/SRTP + RTP forwarding + PLI end-to-end. Asserted in the harness (CI or documented locally-run). | **high** (the gate-unblocker) |
| c005 | `p23-c005-security-doc-media-boundary` | **G3 docs** — write the media path into `docs/SECURITY.md` §1–§5: JWT on signal/media channel (§1), Keto `view` at room-join + `(TenantId,room)` isolation (§2/§5 rows), reference ADR-007. Update §6 to reflect proven state. | low |
| c006 | `p23-c006-flip-or-reaffirm-gate` | **G4** — if c004 proved decoded media **and** c002/c005 cover the boundary: flip `main.rs` sovereign branch from the warning to a live path (remove the "unproven" warning). **Else: re-affirm gated** with fresh rationale. Update CHANGELOG + §6; PHASE-23-SIGNOFF; **G5 re-affirm** (LiveKit `realtime`, admin-ui OIDC) carried. | med (honest branch) |

## Dependencies

- c002 depends on c001 (ADR must exist before enforcing it).
- c004 depends on c003 (harness) and benefits from c002 (join is authz-gated by then).
- c006 depends on **c004 outcome** (proof pass/fail decides flip vs re-affirm) **and** c002+c005
  (boundary enforced + documented). **c006 must not flip the gate if c004 did not prove decoded
  media** — that is the phase's terminal honesty gate.

## Notes

- **Dependency rule**: G3 authz lands in `frf-gateway`/`MediaTransportBridge` (already holds the
  `KetoAuthzProvider` seam), **never** in `frf-media-str0m`. The str0m adapter stays a pure media
  transport.
- **CI browser availability** (open Q3 from assessment): if the Dagger runner lacks Chromium,
  c003/c004 run locally and the sign-off documents them as locally-proven — **not** silently
  skipped.
- Each change passes the QA gate; **read the gate verdict before archive** (carried discipline).
- First change to apply: **`p23-c001-media-authz-adr`**.
