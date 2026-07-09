# Plan — phase-25-sovereign-sfu-decode-retry-and-flip

> Backend: **OpenSpec**. Generated 2026-07-08 from `assessment.md` + operator decision:
> **the live run uses a flint-gate-minted JWT (authenticated path)** — the proof exercises the
> full c003 authenticated ADR-007 flow, not a bypass. The flip stays strictly conditional on a
> genuine `framesDecoded > 0` (carried nine-phase discipline).
>
> Ordering: fix the runner's live path (authenticated) → run the proof → flip *only* on a real
> pass. Tight scope — the media plane is already built.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p25-c001-authenticated-decode-runner` | Fix G1 the authenticated way: the runner brings up the **sovereign stack including flint-gate** (no broken `-f` fallback), obtains a real JWT via flint-gate (host `14457`; confirm the mint mechanism — proxy-injected header vs. a mint call), seeds `(sub=dev-integration-user, view, room)`, and runs the harness with that JWT. `run-media-decode.sh` reworked; shellcheck-clean. | med |
| c002 | `p25-c002-live-decode-run` | **The proof.** Execute the authenticated runner against the live sovereign gateway; observe `getStats().framesDecoded > 0`. Record the **actual** outcome in `docs/PHASE-25-DECODE-RESULT.md` — including, if it fails, whether the blocker is ICE/DTLS/RTP (**media path**, the real unknown) vs. harness. No fabrication. | **high** (the gate-unblocker; ICE-over-UDP is untested) |
| c003 | `p25-c003-flip-or-reaffirm` | If c002 observed a genuine `framesDecoded > 0`: **flip** the `main.rs` sovereign branch (remove the warning) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-25-SIGNOFF.md`. **Else: re-affirm gated** with the fresh (now likely media-path) detail. + **G4 carried** (LiveKit `realtime`; admin-ui OIDC). | med (honest branch) |

## Dependencies

- c002 depends on c001 (the authenticated runner + seed must work first).
- **c003 depends on c002's real outcome** — flip iff `framesDecoded > 0` genuinely observed; else
  re-affirm. **c003 must never flip on a relaxed/worked-around proof** (operator-confirmed, carried).

## Notes

- **Authenticated path chosen** so the proof covers the c003 ADR-007 authz (verified JWT subject →
  Keto `view`), not a `DEV_NO_AUTH` bypass. The minimal no-auth override remains an *un-planned*
  fallback only if flint-gate minting proves impractical — and using it would be recorded as
  proving media flow, not the authenticated path.
- **The real remaining unknown is G2/c002 media reachability** — whether Chromium completes
  ICE/DTLS/RTP to the `host.docker.internal` UDP candidate. This is the first genuine test of the
  media path over a real network path; phase-24 never reached it. If it fails, the finding is
  valuable regardless (media-path vs. advertise-IP), and the gate holds.
- **Confirm at execute:** flint-gate's `mint_jwt` is a **proxy hook** (`path: /**` → gateway
  upstream, injects the JWT header), not obviously a standalone token endpoint. c001 must determine
  how the runner obtains a usable `E2E_JWT` (call through flint-gate, or read the injected header,
  or mint directly with `FLINT_GATE_JWT_SECRET`).
- Each change passes the QA gate; **read the verdict AND the archive output** before archiving.
- First change to apply: **`p25-c001-authenticated-decode-runner`**.
