# Plan — phase-28-sovereign-sfu-decode-harness-timing-and-proof

> Backend: **OpenSpec**. Generated 2026-07-09 from `assessment.md`. The G1 visibility fix is precise
> and small (harness timing + log capture); G2 is **evidence-driven** — its shape depends on what
> the now-surfaced diagnostics reveal, so it is a single "read + fix" change scoped by the evidence.
> The flip stays strictly conditional on a genuine `framesDecoded > 0` (carried discipline).
>
> No operator decision needed at plan time — the approach is unambiguous. If the G2 evidence reveals
> a branch (e.g. a deep str0m ICE/DTLS issue), surface it then.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p28-c001-harness-visibility` | **G1.** In `media-decode.spec.ts`: drop the redundant `connectToSovereignSfu` pre-check (frees 15s, no decode value) and add `test.setTimeout(60_000)` so the probe's 20s window + the diagnostic assertion run. In `run-media-decode.sh`: dump `docker compose logs gateway` **before** the cleanup `down -v` on harness failure, so the str0m lifecycle logs survive a decode failure. Typecheck + shellcheck. | low |
| c002 | `p28-c002-diagnose-and-fix` | **G2.** Re-run; **read** the surfaced `ice=<state>/localCandidates/remoteCandidates` + the gateway str0m logs (session negotiated / connection state / inbound MediaData / forward). Record the diagnosis in `docs/PHASE-28-DIAGNOSIS.md`, then **fix the revealed media-path issue** (candidate reachability over `host.docker.internal:40000/udp`, DTLS, sender-RTP-reaches-SFU, or `RoomRouter` fan-out over a real socket). Keep engine contracts intact / ADR any real design change. | **high** (evidence-driven; the real media-path fix) |
| c003 | `p28-c003-decode-run-and-flip` | **G3.** Re-run; observe `getStats().framesDecoded > 0`; record `docs/PHASE-28-DECODE-RESULT.md`. **If genuine pass:** flip `main.rs` sovereign branch + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-28-SIGNOFF.md`. **Else:** re-affirm gated with the fresh detail. + **G4 carried**. | med (honest branch) |

## Dependencies

- c002 depends on c001 (evidence must be readable before it can be interpreted + fixed).
- c003 depends on c002 (the media path must actually complete for a decoded frame).
- **c003 flips only on a genuine `framesDecoded > 0`** — never a relaxed/worked-around proof
  (operator-confirmed, carried).

## Notes

- **c002 is deliberately open-shaped** — it is a diagnose-then-fix change whose fix is dictated by
  the c001-surfaced evidence, not pre-guessed. Its `docs/PHASE-28-DIAGNOSIS.md` records what the
  evidence showed and what was fixed, so the reasoning is auditable.
- **Sharp hypothesis to check first (from assessment Q1):** does the sender's RTP reach the SFU and
  fan out? The gateway `inbound MediaData` log on the *sender's* session + a forward to the
  *receiver's* is the proof. If absent, the bug is fan-out/track wiring, not ICE.
- **Dependency rule / file-size / no-unwrap** apply to any Rust fix in c002.
- Each change passes the QA gate; **read the verdict AND the archive output**; **seed the openspec
  change dir at the start of every apply**.
- First change to apply: **`p28-c001-harness-visibility`**.
