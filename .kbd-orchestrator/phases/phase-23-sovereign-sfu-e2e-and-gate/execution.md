# Execution — phase-23-sovereign-sfu-e2e-and-gate

> Backend: **OpenSpec** (detected: `openspec/` present). Dispatch contract for the 6 planned
> changes. KBD owns the loop; each task is driven one-per-turn through `/kbd-apply`, which fires
> per-task hooks + syncs `progress.json`. **Never** bare `/opsx:apply`.

## Backend selection

| Field | Value |
|-------|-------|
| Backend | `openspec` |
| Rationale | Spec-backed traceability; consistent with phases 15–22; per-change `qa-gate.sh` + `constraints.md` wired |
| Task driver | `/kbd-apply <change>` (one task per turn) |
| QA gate | `.kbd-orchestrator/bin/qa-gate.sh` after each change reaches DONE; **read the verdict before archive** |
| Verify / archive | `openspec validate <change>` → `openspec archive <change> --yes` |

## Dispatch contract (ordered)

1. **p23-c001-media-authz-adr** — doc-only ADR-007. QA: docs (fmt/validate).
2. **p23-c002-keto-view-check-on-join** — `MediaTransportBridge` + `AuthzProvider`; deny⇒no join. Rust QA (clippy/check/tests). Depends: c001.
3. **p23-c003-browser-e2e-harness** — Playwright spec + WebRTC client page; Dagger flag. TS/e2e QA. 
4. **p23-c004-decoded-media-proof** — Chromium↔headless-str0m, decode ≥1 frame. **Gate-unblocker.** Depends: c003 (+c002).
5. **p23-c005-security-doc-media-boundary** — SECURITY §1–§6 + ADR-007 ref. Doc QA.
6. **p23-c006-flip-or-reaffirm-gate** — flip `main.rs` **iff c004 proved decoded media** + c002/c005 cover boundary; else re-affirm gated. + CHANGELOG, PHASE-23-SIGNOFF, G5 re-affirm. Depends: c004 outcome + c002 + c005.

## Honesty gate (terminal)

`c006` flips `SFU_MODE=sovereign` **only** on a real c004 pass. If c004 does not prove decoded
media (browser/CI infra unavailable, or the frame never decodes), c006 re-affirms the gate off
with fresh rationale in SECURITY §6 — proven-but-not-flipped, never "healthy but does nothing."

## First change to apply

`p23-c001-media-authz-adr` (active in `progress.json`).
