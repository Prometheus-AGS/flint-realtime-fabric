# Tasks — p16-c026

- [x] Add LICENSE file (MIT)
- [x] Add CONTRIBUTING.md and SECURITY.md
- [x] Add CHANGELOG.md
- [x] Generate proto-derived API reference
- [x] Write ADR-003 for UniFFI/frb/Connect/tonic versions

## Summary (#54 / #57 / #58 / #59 — release artifacts)

Final change of phase-16. All release-hygiene artifacts a public repo needs.

- **`LICENSE`** (#54) — MIT, matching the `license = "MIT"` declared in `Cargo.toml`
  (the audit flagged the declaration with no file). Copyright Prometheus AGS 2026.
- **`CONTRIBUTING.md`** — local workflow + the exact CI quality gates (Rust: fmt +
  pedantic clippy on lib/bins/tests + no library unwrap; frontend/SDK: typecheck/lint/
  build per language), architecture rules, commit/PR conventions.
- **`SECURITY.md`** (root) — vulnerability disclosure policy (private report,
  coordinated disclosure, supported versions, scope, operator hardening checklist).
  Distinct from `docs/SECURITY.md` (the security *model*, c024) — they cross-link.
- **`CHANGELOG.md`** (#58) — Keep-a-Changelog format; the `[Unreleased]` section
  summarizes the whole phase-16 (security / deliverables / operability / docs) and its
  deferred items.
- **`docs/API-REFERENCE.md`** (#57) — proto-derived: all 6 `flint.v1` services with
  their RPCs, kinds, and request/response types, plus a server-status table marking
  Spine/Signal/Sync/Agent as live and Entity/Authz as proto-only.
- **`docs/decisions/adr-003-ffi-codegen-versions.md`** (#59) — pins tonic 0.14 /
  tonic-web 0.14 / Connect 1.6-1.7 / UniFFI 0.31.2, and records the p16-c014 decision
  to move Dart off flutter_rust_bridge onto `uniffi-bindgen-dart` (FRB cannot bridge a
  UniFFI crate) — one FFI framework across all mobile SDKs.

## Verification

- All 6 artifacts exist; `LICENSE` first line "MIT License" matches Cargo.toml.
- API reference covers all 6 proto services; ADR-003 names all four toolchains.
