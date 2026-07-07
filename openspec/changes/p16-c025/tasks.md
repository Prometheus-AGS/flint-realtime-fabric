# Tasks — p16-c025

- [x] Update repository structure to reflect real crates
- [x] Correct current phase/status
- [x] Fix any remaining stale references

## Summary (#42 / #43 — stale README)

### Task 1 — repository structure

The audit's #42 (README documents non-existent crates) is resolved by the crates now
EXISTING: `frf-sdk-rust` (c011) and `frf-cli` (c015) are real, and `frf-ffi` has real
transport (c014). Cross-checked every `frf-*` reference in the README against
`crates/` — all listed crates exist. No structural edits needed beyond that.

### Task 2 — current phase/status (#43)

Rewrote the stale "Current State" section (was "**Phase 12 complete** … active phase
`phase-13-live-layer3-e2e-validation`") to reflect **phase-16 production hardening**,
summarizing the security / deliverables / operability / docs work and the explicitly
deferred items (str0m SFU, federation protocol, Entity/Authz servers, Dart transport) —
so what's deferred reads as deferred, not silently missing.

### Task 3 — remaining stale references

Scanned for old phase numbers, "not built / not implemented / TODO / coming soon", and
dead crate links — none remained. Added the new operational docs to the README's doc
index for discoverability: `docs/ENVIRONMENT.md`, `docs/RUNBOOK.md`, `docs/SECURITY.md`,
`.env.example`.

## Verification

- No `Phase 12 complete` / `phase-13-live` strings remain.
- All README doc links (`ENVIRONMENT.md`, `RUNBOOK.md`, `SECURITY.md`, `.env.example`)
  resolve to existing files.
- Every `frf-*` crate named in the README exists in `crates/`.

Depends-on c011 + c015 (the crates the README references now exist) — satisfied.
